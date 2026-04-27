use crate::error::SlmpError;
use crate::model::{
    SlmpCompatibilityMode, SlmpDeviceAddress, SlmpDeviceCode, SlmpNamedTarget, SlmpPlcFamily,
    SlmpQualifiedDeviceAddress, SlmpTargetAddress,
};

pub struct SlmpAddress;

impl SlmpAddress {
    pub fn parse(text: &str) -> Result<SlmpDeviceAddress, SlmpError> {
        parse_device(text)
    }

    pub fn parse_for_plc_family(
        text: &str,
        family: SlmpPlcFamily,
    ) -> Result<SlmpDeviceAddress, SlmpError> {
        parse_device_for_plc_family(text, family)
    }

    pub fn try_parse(text: &str) -> Option<SlmpDeviceAddress> {
        parse_device(text).ok()
    }

    pub fn try_parse_for_plc_family(
        text: &str,
        family: SlmpPlcFamily,
    ) -> Option<SlmpDeviceAddress> {
        parse_device_for_plc_family(text, family).ok()
    }

    pub fn format(address: SlmpDeviceAddress) -> String {
        let number = format_number(address, None);
        format!("{}{}", address.code.prefix(), number)
    }

    pub fn format_for_plc_family(address: SlmpDeviceAddress, family: SlmpPlcFamily) -> String {
        let number = format_number(address, Some(family));
        format!("{}{}", address.code.prefix(), number)
    }

    pub fn normalize(text: &str) -> Result<String, SlmpError> {
        Ok(Self::format(Self::parse(text)?))
    }

    pub fn normalize_for_plc_family(
        text: &str,
        family: SlmpPlcFamily,
    ) -> Result<String, SlmpError> {
        Ok(Self::format_for_plc_family(
            Self::parse_for_plc_family(text, family)?,
            family,
        ))
    }
}

pub fn parse_named_address(address: &str) -> Result<NamedAddressParts, SlmpError> {
    let trimmed = address.trim();
    if let Some((base, dtype)) = trimmed.split_once(':') {
        return Ok(NamedAddressParts {
            base: base.trim().to_string(),
            dtype: dtype.trim().to_uppercase(),
            bit_index: None,
        });
    }

    if let Some((base, bit)) = trimmed.split_once('.')
        && let Ok(bit_index) = u8::from_str_radix(bit.trim(), 16)
    {
        return Ok(NamedAddressParts {
            base: base.trim().to_string(),
            dtype: "BIT_IN_WORD".to_string(),
            bit_index: Some(bit_index),
        });
    }

    Ok(NamedAddressParts {
        base: trimmed.to_string(),
        dtype: "U".to_string(),
        bit_index: None,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedAddressParts {
    pub base: String,
    pub dtype: String,
    pub bit_index: Option<u8>,
}

pub fn normalize_named_address(address: &str) -> Result<String, SlmpError> {
    let parts = parse_named_address(address)?;
    let canonical_base = SlmpAddress::normalize(&parts.base)?;
    if let Some(bit_index) = parts.bit_index {
        return Ok(format!("{canonical_base}.{bit_index:X}"));
    }
    if address.contains(':') {
        return Ok(format!("{canonical_base}:{}", parts.dtype));
    }
    Ok(canonical_base)
}

pub fn parse_device(text: &str) -> Result<SlmpDeviceAddress, SlmpError> {
    parse_device_internal(text, None)
}

pub fn parse_device_for_plc_family(
    text: &str,
    family: SlmpPlcFamily,
) -> Result<SlmpDeviceAddress, SlmpError> {
    parse_device_internal(text, Some(family))
}

pub fn parse_device_for_family_hint(
    text: &str,
    family: Option<SlmpPlcFamily>,
) -> Result<SlmpDeviceAddress, SlmpError> {
    let device = match family {
        Some(family) => parse_device_for_plc_family(text, family)?,
        None => parse_device(text)?,
    };
    if family.is_none() && matches!(device.code, SlmpDeviceCode::X | SlmpDeviceCode::Y) {
        return Err(SlmpError::new(
            "X/Y string addresses require explicit plc_family. Use IqF for FX/iQ-F targets, choose an explicit non-iQ-F family, or pass a numeric SlmpDeviceAddress.",
        ));
    }
    Ok(device)
}

fn parse_device_internal(
    text: &str,
    family: Option<SlmpPlcFamily>,
) -> Result<SlmpDeviceAddress, SlmpError> {
    let token = text.trim().to_uppercase();
    if token.is_empty() {
        return Err(SlmpError::new("Device text is required."));
    }

    for prefix in [
        "LSTS", "LSTC", "LSTN", "LTS", "LTC", "LTN", "STS", "STC", "STN", "SM", "SD", "TS", "TC",
        "TN", "CS", "CC", "CN", "SB", "SW", "DX", "DY", "LCS", "LCC", "LCN", "LZ", "ZR", "RD",
        "HG", "X", "Y", "M", "L", "F", "V", "B", "D", "W", "Z", "R", "G",
    ] {
        if token.starts_with(prefix)
            && let Some(code) = SlmpDeviceCode::parse_prefix(prefix)
        {
            let number_text = &token[prefix.len()..];
            let radix = device_radix(code, family);
            let number = parse_u32_with_radix(number_text, radix).map_err(|_| {
                SlmpError::new(format!(
                    "Invalid SLMP device number '{number_text}' for device code '{prefix}' in '{text}'."
                ))
            })?;
            return Ok(SlmpDeviceAddress::new(code, number));
        }
    }

    Err(SlmpError::new(format!(
        "Invalid SLMP device string '{text}'."
    )))
}

fn device_radix(code: SlmpDeviceCode, family: Option<SlmpPlcFamily>) -> u32 {
    if matches!(code, SlmpDeviceCode::X | SlmpDeviceCode::Y)
        && family.is_some_and(SlmpPlcFamily::uses_iqf_xy_octal)
    {
        return 8;
    }
    if code.is_hex_addressed() { 16 } else { 10 }
}

fn parse_u32_with_radix(text: &str, radix: u32) -> Result<u32, std::num::ParseIntError> {
    match radix {
        8 => u32::from_str_radix(text, 8),
        16 => u32::from_str_radix(text, 16),
        _ => text.parse::<u32>(),
    }
}

fn format_number(address: SlmpDeviceAddress, family: Option<SlmpPlcFamily>) -> String {
    match device_radix(address.code, family) {
        8 => format!("{:o}", address.number).to_ascii_uppercase(),
        16 => format!("{:X}", address.number),
        _ => address.number.to_string(),
    }
}

pub fn parse_qualified_device(text: &str) -> Result<SlmpQualifiedDeviceAddress, SlmpError> {
    let token = text.trim().to_uppercase();
    if token.is_empty() {
        return Err(SlmpError::new("Device text is required."));
    }

    if let Some(rest) = token.strip_prefix('J')
        && let Some((network, device_text)) = split_slash(rest)
    {
        let network: u16 = network
            .parse()
            .map_err(|_| SlmpError::new("Invalid J-direct network."))?;
        return Ok(SlmpQualifiedDeviceAddress {
            device: parse_device(device_text)?,
            extension_specification: Some(network),
            direct_memory_specification: Some(0xF9),
        });
    }

    if let Some(rest) = token.strip_prefix('U')
        && let Some((extension, device_text)) = split_slash(rest)
    {
        let extension_specification = u16::from_str_radix(extension, 16)
            .map_err(|_| SlmpError::new("Invalid extension specification."))?;
        let device = parse_device(device_text)?;
        let direct_memory_specification = match device.code {
            SlmpDeviceCode::G => Some(0xF8),
            SlmpDeviceCode::HG => Some(0xFA),
            _ => None,
        };
        return Ok(SlmpQualifiedDeviceAddress {
            device,
            extension_specification: Some(extension_specification),
            direct_memory_specification,
        });
    }

    Ok(SlmpQualifiedDeviceAddress {
        device: parse_device(&token)?,
        extension_specification: None,
        direct_memory_specification: None,
    })
}

fn split_slash(text: &str) -> Option<(&str, &str)> {
    text.split_once('\\').or_else(|| text.split_once('/'))
}

pub fn parse_target_auto_number(text: &str) -> Result<u32, SlmpError> {
    if let Some(hex) = text.strip_prefix("0X").or_else(|| text.strip_prefix("0x")) {
        return u32::from_str_radix(hex, 16).map_err(|_| SlmpError::new("Invalid numeric text."));
    }
    text.parse::<u32>()
        .map_err(|_| SlmpError::new("Invalid numeric text."))
}

pub fn parse_named_target(text: &str) -> Result<SlmpNamedTarget, SlmpError> {
    let token = text.trim();
    if token.eq_ignore_ascii_case("SELF") {
        return Ok(SlmpNamedTarget {
            name: "SELF".to_string(),
            target: SlmpTargetAddress::default(),
        });
    }
    if let Some(cpu) = token.strip_prefix("SELF-CPU") {
        let index: u16 = cpu
            .parse()
            .map_err(|_| SlmpError::new("Invalid SELF-CPU target."))?;
        if !(1..=4).contains(&index) {
            return Err(SlmpError::new("SELF-CPU must be 1..4."));
        }
        return Ok(SlmpNamedTarget {
            name: format!("SELF-CPU{index}"),
            target: SlmpTargetAddress {
                module_io: 0x03E0 + (index - 1),
                ..SlmpTargetAddress::default()
            },
        });
    }
    if let Some(rest) = token.strip_prefix("NW")
        && let Some((network, station)) = rest.split_once("-ST")
    {
        return Ok(SlmpNamedTarget {
            name: format!("NW{}-ST{}", network, station),
            target: SlmpTargetAddress {
                network: network
                    .parse()
                    .map_err(|_| SlmpError::new("Invalid network."))?,
                station: station
                    .parse()
                    .map_err(|_| SlmpError::new("Invalid station."))?,
                ..SlmpTargetAddress::default()
            },
        });
    }
    let parts: Vec<_> = token.split(',').map(str::trim).collect();
    if parts.len() == 5 {
        return Ok(SlmpNamedTarget {
            name: parts[0].to_string(),
            target: SlmpTargetAddress {
                network: parse_target_auto_number(parts[1])? as u8,
                station: parse_target_auto_number(parts[2])? as u8,
                module_io: parse_target_auto_number(parts[3])? as u16,
                multidrop: parse_target_auto_number(parts[4])? as u8,
            },
        });
    }
    Err(SlmpError::new(
        "target must be SELF, SELF-CPU1..4, NWx-STy, or NAME,NETWORK,STATION,MODULE_IO,MULTIDROP",
    ))
}

pub fn device_spec_size(mode: SlmpCompatibilityMode) -> usize {
    match mode {
        SlmpCompatibilityMode::Legacy => 4,
        SlmpCompatibilityMode::Iqr => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::{SlmpAddress, parse_device, parse_device_for_family_hint, parse_device_for_plc_family};
    use crate::model::{SlmpDeviceAddress, SlmpDeviceCode, SlmpPlcFamily};

    #[test]
    fn iq_f_xy_strings_are_parsed_as_octal() {
        assert_eq!(
            parse_device_for_plc_family("X100", SlmpPlcFamily::IqF).unwrap(),
            SlmpDeviceAddress::new(SlmpDeviceCode::X, 0o100)
        );
        assert_eq!(
            SlmpAddress::format_for_plc_family(
                SlmpDeviceAddress::new(SlmpDeviceCode::Y, 0o217),
                SlmpPlcFamily::IqF,
            ),
            "Y217"
        );
    }

    #[test]
    fn non_iq_f_xy_strings_remain_hex() {
        assert_eq!(
            parse_device_for_plc_family("X100", SlmpPlcFamily::IqR).unwrap(),
            SlmpDeviceAddress::new(SlmpDeviceCode::X, 0x100)
        );
        assert_eq!(
            SlmpAddress::format_for_plc_family(
                SlmpDeviceAddress::new(SlmpDeviceCode::X, 0x1A),
                SlmpPlcFamily::IqR,
            ),
            "X1A"
        );
    }

    #[test]
    fn hex_number_can_be_all_letters() {
        assert_eq!(
            parse_device("XFF").unwrap(),
            SlmpDeviceAddress::new(SlmpDeviceCode::X, 0xff)
        );
    }

    #[test]
    fn known_code_with_invalid_number_does_not_fallback() {
        let error = parse_device("DFFFF").unwrap_err();
        assert!(
            error.message.contains("device code 'D'"),
            "unexpected error: {}",
            error.message
        );
    }

    #[test]
    fn ambiguous_xy_strings_require_explicit_family_for_high_level_parse() {
        let error = parse_device_for_family_hint("Y217", None).unwrap_err();
        assert!(
            error.message.contains("explicit plc_family"),
            "unexpected error: {}",
            error.message
        );
    }
}
