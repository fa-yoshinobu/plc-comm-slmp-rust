use crate::error::SlmpError;
use crate::model::{
    SlmpCompatibilityMode, SlmpDeviceAddress, SlmpDeviceCode, SlmpNamedTarget, SlmpPlcProfile,
    SlmpQualifiedDeviceAddress, SlmpTargetAddress,
};

pub struct SlmpAddress;

impl SlmpAddress {
    pub fn parse(text: &str) -> Result<SlmpDeviceAddress, SlmpError> {
        parse_device(text)
    }

    pub fn parse_for_plc_profile(
        text: &str,
        family: SlmpPlcProfile,
    ) -> Result<SlmpDeviceAddress, SlmpError> {
        parse_device_for_plc_profile(text, family)
    }

    pub fn try_parse(text: &str) -> Option<SlmpDeviceAddress> {
        parse_device(text).ok()
    }

    pub fn try_parse_for_plc_profile(
        text: &str,
        family: SlmpPlcProfile,
    ) -> Option<SlmpDeviceAddress> {
        parse_device_for_plc_profile(text, family).ok()
    }

    pub fn format(address: SlmpDeviceAddress) -> String {
        let number = format_number(address, None);
        format!("{}{}", address.code.prefix(), number)
    }

    pub fn format_for_plc_profile(address: SlmpDeviceAddress, family: SlmpPlcProfile) -> String {
        let number = format_number(address, Some(family));
        format!("{}{}", address.code.prefix(), number)
    }

    pub fn normalize(text: &str) -> Result<String, SlmpError> {
        Ok(Self::format(Self::parse(text)?))
    }

    pub fn normalize_for_plc_profile(
        text: &str,
        family: SlmpPlcProfile,
    ) -> Result<String, SlmpError> {
        Ok(Self::format_for_plc_profile(
            Self::parse_for_plc_profile(text, family)?,
            family,
        ))
    }
}

pub fn parse_named_address(address: &str) -> Result<NamedAddressParts, SlmpError> {
    let trimmed = address.trim();
    if let Some((base, dtype)) = trimmed.split_once(':') {
        let dtype = require_named_dtype(dtype)?;
        if dtype == "BIT_IN_WORD" {
            return Err(SlmpError::new(
                "BIT_IN_WORD requires an explicit bit index. Use '.0' through '.F' notation.",
            ));
        }

        return Ok(NamedAddressParts {
            base: base.trim().to_string(),
            dtype,
            bit_index: None,
        });
    }

    if let Some((base, bit)) = trimmed.split_once('.') {
        let bit_text = bit.trim();
        if bit_text.len() == 1 && bit_text.chars().all(|ch| ch.is_ascii_hexdigit()) {
            let bit_index = u8::from_str_radix(bit_text, 16).unwrap();
            return Ok(NamedAddressParts {
                base: base.trim().to_string(),
                dtype: "BIT_IN_WORD".to_string(),
                bit_index: Some(bit_index),
            });
        }
        return Err(SlmpError::new(
            "Invalid bit-in-word index. Use one hex digit 0-F or ':' for data type.",
        ));
    }

    Err(SlmpError::new(format!(
        "Address '{trimmed}' requires an explicit dtype such as ':U', ':D', or ':BIT'."
    )))
}

fn require_named_dtype(dtype: &str) -> Result<String, SlmpError> {
    let normalized = dtype.trim().to_uppercase();
    if normalized.is_empty() {
        return Err(SlmpError::new(
            "dtype is required; specify BIT/U/S/D/L/F explicitly.",
        ));
    }
    if normalized != "BIT_IN_WORD"
        && !matches!(normalized.as_str(), "BIT" | "U" | "S" | "D" | "L" | "F")
    {
        return Err(SlmpError::new(format!(
            "Unsupported dtype '{normalized}'; expected BIT/U/S/D/L/F."
        )));
    }
    Ok(normalized)
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
    Ok(format!("{canonical_base}:{}", parts.dtype))
}

pub fn parse_device(text: &str) -> Result<SlmpDeviceAddress, SlmpError> {
    parse_device_internal(text, None)
}

pub fn parse_device_for_plc_profile(
    text: &str,
    family: SlmpPlcProfile,
) -> Result<SlmpDeviceAddress, SlmpError> {
    parse_device_internal(text, Some(family))
}

pub fn parse_device_for_family_hint(
    text: &str,
    family: Option<SlmpPlcProfile>,
) -> Result<SlmpDeviceAddress, SlmpError> {
    let device = match family {
        Some(family) => parse_device_for_plc_profile(text, family)?,
        None => parse_device(text)?,
    };
    if family.is_none() && matches!(device.code, SlmpDeviceCode::X | SlmpDeviceCode::Y) {
        return Err(SlmpError::new(
            "X/Y string addresses require explicit plc_profile. Use IqF for FX/iQ-F targets, choose an explicit non-iQ-F family, or pass a numeric SlmpDeviceAddress.",
        ));
    }
    Ok(device)
}

fn parse_device_internal(
    text: &str,
    family: Option<SlmpPlcProfile>,
) -> Result<SlmpDeviceAddress, SlmpError> {
    let token = text.trim().to_uppercase();
    if token.is_empty() {
        return Err(SlmpError::new("Device text is required."));
    }

    for prefix in [
        "LSTS", "LSTC", "LSTN", "LTS", "LTC", "LTN", "STS", "STC", "STN", "SM", "SD", "TS", "TC",
        "TN", "CS", "CC", "CN", "SB", "SW", "DX", "DY", "LCS", "LCC", "LCN", "LZ", "ZR", "RD",
        "HG", "X", "Y", "M", "L", "F", "V", "B", "S", "D", "W", "Z", "R", "G",
    ] {
        if token.starts_with(prefix)
            && let Some(code) = SlmpDeviceCode::parse_prefix(prefix)
        {
            ensure_device_supported_for_family(prefix, code, family)?;
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

fn ensure_device_supported_for_family(
    prefix: &str,
    code: SlmpDeviceCode,
    family: Option<SlmpPlcProfile>,
) -> Result<(), SlmpError> {
    if let Some(family) = family {
        if device_is_unsupported_for_family(code, family) {
            let profile = family.canonical_name();
            return Err(SlmpError::new(format!(
                "SLMP device code '{prefix}' is not supported for plc_profile '{profile}'."
            )));
        }
    }
    Ok(())
}

fn device_is_unsupported_for_family(code: SlmpDeviceCode, family: SlmpPlcProfile) -> bool {
    match family.address_profile() {
        SlmpPlcProfile::IqF => matches!(
            code,
            SlmpDeviceCode::DX
                | SlmpDeviceCode::DY
                | SlmpDeviceCode::V
                | SlmpDeviceCode::LTS
                | SlmpDeviceCode::LTC
                | SlmpDeviceCode::LTN
                | SlmpDeviceCode::LSTS
                | SlmpDeviceCode::LSTC
                | SlmpDeviceCode::LSTN
                | SlmpDeviceCode::ZR
                | SlmpDeviceCode::RD
        ),
        SlmpPlcProfile::QCpu
        | SlmpPlcProfile::LCpu
        | SlmpPlcProfile::QnU
        | SlmpPlcProfile::QnUDV => matches!(
            code,
            SlmpDeviceCode::LTS
                | SlmpDeviceCode::LTC
                | SlmpDeviceCode::LTN
                | SlmpDeviceCode::LSTS
                | SlmpDeviceCode::LSTC
                | SlmpDeviceCode::LSTN
                | SlmpDeviceCode::LCS
                | SlmpDeviceCode::LCC
                | SlmpDeviceCode::LCN
                | SlmpDeviceCode::LZ
                | SlmpDeviceCode::RD
        ),
        SlmpPlcProfile::IqR
        | SlmpPlcProfile::IqRRj71En71
        | SlmpPlcProfile::IqL
        | SlmpPlcProfile::MxR
        | SlmpPlcProfile::MxF => false,
        SlmpPlcProfile::QCpuQj71E71100
        | SlmpPlcProfile::LCpuLj71E71100
        | SlmpPlcProfile::QnUQj71E71100
        | SlmpPlcProfile::QnUDVQj71E71100 => {
            unreachable!("unit profiles are mapped to their address profile")
        }
    }
}

fn device_radix(code: SlmpDeviceCode, family: Option<SlmpPlcProfile>) -> u32 {
    if matches!(code, SlmpDeviceCode::X | SlmpDeviceCode::Y)
        && family.is_some_and(SlmpPlcProfile::uses_iqf_xy_octal)
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

fn format_number(address: SlmpDeviceAddress, family: Option<SlmpPlcProfile>) -> String {
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
            SlmpDeviceCode::HG => {
                if !matches!(extension_specification, 0x03E0..=0x03E3) {
                    return Err(SlmpError::new(
                        "HG Extended Device access is valid only for U3E0\\HG through U3E3\\HG.",
                    ));
                }
                Some(0xFA)
            }
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

fn parse_target_u8(text: &str, label: &str) -> Result<u8, SlmpError> {
    let value = parse_target_auto_number(text)?;
    u8::try_from(value).map_err(|_| SlmpError::new(format!("{label} must be 0..255.")))
}

fn parse_target_u16(text: &str, label: &str) -> Result<u16, SlmpError> {
    let value = parse_target_auto_number(text)?;
    u16::try_from(value).map_err(|_| SlmpError::new(format!("{label} must be 0..65535.")))
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
                module_io: crate::model::SlmpModuleIo::CPU1 + (index - 1),
                ..SlmpTargetAddress::default()
            },
        });
    }
    let parts: Vec<_> = token.split(',').map(str::trim).collect();
    if parts.len() == 5 {
        return Ok(SlmpNamedTarget {
            name: parts[0].to_string(),
            target: SlmpTargetAddress {
                network: parse_target_u8(parts[1], "network")?,
                station: parse_target_u8(parts[2], "station")?,
                module_io: parse_target_u16(parts[3], "module_io")?,
                multidrop: parse_target_u8(parts[4], "multidrop")?,
            },
        });
    }
    Err(SlmpError::new(
        "target must be SELF, SELF-CPU1..4, or NAME,NETWORK,STATION,MODULE_IO,MULTIDROP",
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
    use super::{
        SlmpAddress, parse_device, parse_device_for_family_hint, parse_device_for_plc_profile,
        parse_named_address, parse_named_target,
    };
    use crate::model::{SlmpDeviceAddress, SlmpDeviceCode, SlmpPlcProfile};

    #[test]
    fn iq_f_xy_strings_are_parsed_as_octal() {
        assert_eq!(
            parse_device_for_plc_profile("X100", SlmpPlcProfile::IqF).unwrap(),
            SlmpDeviceAddress::new(SlmpDeviceCode::X, 0o100)
        );
        assert_eq!(
            SlmpAddress::format_for_plc_profile(
                SlmpDeviceAddress::new(SlmpDeviceCode::Y, 0o217),
                SlmpPlcProfile::IqF,
            ),
            "Y217"
        );
    }

    #[test]
    fn non_iq_f_xy_strings_remain_hex() {
        assert_eq!(
            parse_device_for_plc_profile("X100", SlmpPlcProfile::IqR).unwrap(),
            SlmpDeviceAddress::new(SlmpDeviceCode::X, 0x100)
        );
        assert_eq!(
            SlmpAddress::format_for_plc_profile(
                SlmpDeviceAddress::new(SlmpDeviceCode::X, 0x1A),
                SlmpPlcProfile::IqR,
            ),
            "X1A"
        );
    }

    #[test]
    fn iq_f_direct_io_devices_are_rejected() {
        for address in ["DX10", "DY10", "V10", "LTS10", "ZR10", "RD10"] {
            let error = parse_device_for_plc_profile(address, SlmpPlcProfile::IqF).unwrap_err();
            assert!(
                error.message.contains("not supported"),
                "unexpected error: {}",
                error.message
            );
        }
    }

    #[test]
    fn measured_legacy_profiles_reject_unsupported_device_families() {
        for (address, profile) in [
            ("LCS10", SlmpPlcProfile::QnUDV),
            ("LZ0", SlmpPlcProfile::QnU),
            ("RD0", SlmpPlcProfile::LCpu),
            ("LTN0", SlmpPlcProfile::QCpu),
        ] {
            let error = parse_device_for_plc_profile(address, profile).unwrap_err();
            assert!(
                error.message.contains("not supported"),
                "unexpected error for {address}: {}",
                error.message
            );
        }
    }

    #[test]
    fn profile_unsupported_device_codes_match_canonical_fixture() {
        let payload: serde_json::Value = serde_json::from_str(include_str!(
            "../tests/fixtures/slmp_device_range_rules.json"
        ))
        .unwrap();
        let rows = payload["rows"].as_object().unwrap();
        let profiles = payload["profiles"].as_object().unwrap();

        for (profile_name, profile_payload) in profiles {
            let plc_profile = SlmpPlcProfile::parse_label(profile_name).unwrap();
            for (item, rule) in profile_payload["rules"].as_object().unwrap() {
                let expected_supported = rule["kind"].as_str().unwrap() != "unsupported";
                for device in rows[item]["devices"].as_array().unwrap() {
                    let device_name = device["device"].as_str().unwrap();
                    let address = format!("{device_name}10");
                    let parsed = parse_device_for_plc_profile(&address, plc_profile);
                    assert_eq!(
                        parsed.is_ok(),
                        expected_supported,
                        "{profile_name} {device_name}"
                    );
                }
            }
        }

        assert!(parse_device_for_plc_profile("DX10", SlmpPlcProfile::IqF).is_err());
        assert!(parse_device_for_plc_profile("DY10", SlmpPlcProfile::IqF).is_err());
    }

    #[test]
    fn hex_number_can_be_all_letters() {
        assert_eq!(
            parse_device("XFF").unwrap(),
            SlmpDeviceAddress::new(SlmpDeviceCode::X, 0xff)
        );
    }

    #[test]
    fn step_relay_is_parsed_as_decimal_bit_device() {
        assert_eq!(
            parse_device("S10").unwrap(),
            SlmpDeviceAddress::new(SlmpDeviceCode::S, 10)
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
            error.message.contains("explicit plc_profile"),
            "unexpected error: {}",
            error.message
        );
    }

    #[test]
    fn named_target_accepts_numeric_boundaries() {
        let parsed = parse_named_target("PLC,0xFF,255,0xFFFF,0").unwrap();

        assert_eq!(parsed.name, "PLC");
        assert_eq!(parsed.target.network, 0xFF);
        assert_eq!(parsed.target.station, 255);
        assert_eq!(parsed.target.module_io, 0xFFFF);
        assert_eq!(parsed.target.multidrop, 0);
    }

    #[test]
    fn named_target_rejects_out_of_range_fields_before_casting() {
        for (target, label) in [
            ("PLC,256,0,0,0", "network"),
            ("PLC,0,256,0,0", "station"),
            ("PLC,0,0,65536,0", "module_io"),
            ("PLC,0,0,0,256", "multidrop"),
        ] {
            let error = parse_named_target(target).unwrap_err();
            assert!(
                error.message.contains(label),
                "unexpected error for {target}: {}",
                error.message
            );
        }
    }

    #[test]
    fn named_address_dot_suffix_is_one_hex_bit() {
        let parsed = parse_named_address("D100.D").unwrap();
        assert_eq!(parsed.dtype, "BIT_IN_WORD");
        assert_eq!(parsed.bit_index, Some(13));

        assert!(parse_named_address("D100.10").is_err());
    }

    #[test]
    fn named_address_bit_in_word_requires_explicit_bit_index() {
        let error = parse_named_address("D100:BIT_IN_WORD").unwrap_err();
        assert!(
            error.message.contains("explicit bit index"),
            "unexpected error: {}",
            error.message
        );
    }
}
