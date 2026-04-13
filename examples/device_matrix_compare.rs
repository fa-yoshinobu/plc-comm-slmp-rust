mod common;

use common::{env_csv, options_from_env, print_connection_banner};
use plc_comm_slmp::{
    NamedAddress, SlmpAddress, SlmpClient, SlmpCommand, SlmpCompatibilityMode, SlmpDeviceAddress,
    SlmpDeviceCode, SlmpExtensionSpec, SlmpQualifiedDeviceAddress, SlmpValue, encode_device_spec,
    parse_qualified_device, read_named, read_typed, write_named, write_typed,
};
use std::error::Error;

const BIT_DEVICES: &[&str] = &[
    "STS10", "STC10", "TS10", "TC10", "CS10", "CC10", "SB10", "DX10", "DY10", "X10", "Y10", "M10",
    "L10", "F100", "V10", "B10", "SM10", "LTS10", "LTC10", "LSTS10", "LSTC10", "LCS10", "LCC10",
];

const WORD_DEVICES: &[&str] = &[
    "STN10", "TN10", "CN10", "SW10", "ZR10", "D10", "W10", "Z10", "R10", "SD10", "RD10",
];

const DWORD_DEVICES: &[(&str, &str)] = &[
    ("LTN10", "LTN10:D"),
    ("LSTN10", "LSTN10:D"),
    ("LCN10", "LCN10:D"),
    ("LZ0", "LZ0:D"),
    ("LZ1", "LZ1:D"),
];

const EXT_BIT_DEVICES: &[&str] = &["J1\\X10", "J1\\Y10", "J1\\B10", "J1\\SB10"];
const EXT_WORD_DEVICES: &[&str] = &["J1\\W10", "J1\\SW10"];

fn make_error(message: impl Into<String>) -> Box<dyn Error> {
    Box::new(std::io::Error::other(message.into()))
}

fn subcommand(mode: SlmpCompatibilityMode, bit_unit: bool) -> u16 {
    match (mode, bit_unit) {
        (SlmpCompatibilityMode::Legacy, false) => 0x0000,
        (SlmpCompatibilityMode::Legacy, true) => 0x0001,
        (SlmpCompatibilityMode::Iqr, false) => 0x0002,
        (SlmpCompatibilityMode::Iqr, true) => 0x0003,
    }
}

fn ext_subcommand(
    mode: SlmpCompatibilityMode,
    bit_unit: bool,
    extension: SlmpExtensionSpec,
) -> u16 {
    if extension.direct_memory_specification == 0xF9 {
        if bit_unit { 0x0081 } else { 0x0080 }
    } else if matches!(mode, SlmpCompatibilityMode::Legacy) {
        if bit_unit { 0x0081 } else { 0x0080 }
    } else if bit_unit {
        0x0083
    } else {
        0x0082
    }
}

fn pack_bits(values: &[bool]) -> Vec<u8> {
    let mut result = Vec::with_capacity(values.len().div_ceil(2));
    let mut index = 0usize;
    while index < values.len() {
        let high = if values[index] { 0x10 } else { 0x00 };
        index += 1;
        let low = if index < values.len() && values[index] {
            0x01
        } else {
            0x00
        };
        if index < values.len() {
            index += 1;
        }
        result.push(high | low);
    }
    result
}

fn unpack_bits(payload: &[u8], points: usize) -> Result<Vec<bool>, Box<dyn Error>> {
    let need = points.div_ceil(2);
    if payload.len() < need {
        return Err(make_error("bit payload too short"));
    }
    let mut result = Vec::with_capacity(points);
    for byte in payload.iter().take(need) {
        if result.len() < points {
            result.push(((byte >> 4) & 0x01) != 0);
        }
        if result.len() < points {
            result.push((byte & 0x01) != 0);
        }
    }
    Ok(result)
}

fn words_to_payload(words: &[u16]) -> Vec<u8> {
    let mut payload = Vec::with_capacity(words.len() * 2);
    for word in words {
        payload.extend_from_slice(&word.to_le_bytes());
    }
    payload
}

fn payload_to_words(payload: &[u8]) -> Result<Vec<u16>, Box<dyn Error>> {
    if payload.len() % 2 != 0 {
        return Err(make_error("word payload length must be even"));
    }
    Ok(payload
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect())
}

fn dword_to_words(value: u32) -> [u16; 2] {
    [(value & 0xFFFF) as u16, (value >> 16) as u16]
}

fn words_to_dword(words: &[u16]) -> Result<u32, Box<dyn Error>> {
    if words.len() != 2 {
        return Err(make_error(
            "expected exactly two words for dword conversion",
        ));
    }
    Ok(words[0] as u32 | ((words[1] as u32) << 16))
}

fn seeded_u16(label: &str, salt: u32) -> u16 {
    let mut hash = 0x811C9DC5u32 ^ salt;
    for byte in label.as_bytes() {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    let value = ((hash & 0xFFFF) as u16) | 1;
    if value == 0 { 1 } else { value }
}

fn seeded_u32(label: &str, salt: u32) -> u32 {
    let high = seeded_u16(label, salt) as u32;
    let low = seeded_u16(label, salt ^ 0xA5A5_5A5A) as u32;
    (high << 16) | low
}

fn value_from_named_bool(values: &NamedAddress, key: &str) -> Result<bool, Box<dyn Error>> {
    values
        .get(key)
        .ok_or_else(|| make_error(format!("missing named value for {key}")))?
        .as_bool()
        .map_err(Into::into)
}

fn value_from_named_u16(values: &NamedAddress, key: &str) -> Result<u16, Box<dyn Error>> {
    match values
        .get(key)
        .ok_or_else(|| make_error(format!("missing named value for {key}")))?
    {
        SlmpValue::U16(value) => Ok(*value),
        other => Err(make_error(format!("expected U16 for {key}, got {other:?}"))),
    }
}

fn value_from_named_u32(values: &NamedAddress, key: &str) -> Result<u32, Box<dyn Error>> {
    match values
        .get(key)
        .ok_or_else(|| make_error(format!("missing named value for {key}")))?
    {
        SlmpValue::U32(value) => Ok(*value),
        other => Err(make_error(format!("expected U32 for {key}, got {other:?}"))),
    }
}

fn effective_extension(
    qualified: SlmpQualifiedDeviceAddress,
    extension: SlmpExtensionSpec,
) -> SlmpExtensionSpec {
    let mut result = extension;
    if let Some(value) = qualified.extension_specification {
        result.extension_specification = value;
    }
    if let Some(value) = qualified.direct_memory_specification {
        result.direct_memory_specification = value;
    }
    result
}

fn encode_extended_device_spec(
    mode: SlmpCompatibilityMode,
    device: SlmpDeviceAddress,
    extension: SlmpExtensionSpec,
) -> Vec<u8> {
    if extension.direct_memory_specification == 0xF9 {
        return vec![
            0x00,
            0x00,
            (device.number & 0xFF) as u8,
            ((device.number >> 8) & 0xFF) as u8,
            ((device.number >> 16) & 0xFF) as u8,
            device.code.as_u8(),
            0x00,
            0x00,
            (extension.extension_specification & 0xFF) as u8,
            0x00,
            0xF9,
        ];
    }

    let capture_aligned = matches!(device.code, SlmpDeviceCode::G | SlmpDeviceCode::HG)
        && matches!(extension.direct_memory_specification, 0xF8 | 0xFA);
    let device_spec = encode_device_spec(mode, device);

    if capture_aligned {
        let mut payload = Vec::with_capacity(2 + device_spec.len() + 1 + 1 + 2 + 1);
        payload.push(extension.extension_specification_modification);
        payload.push(extension.device_modification_index);
        payload.extend_from_slice(&device_spec);
        payload.push(extension.device_modification_flags);
        payload.push(0x00);
        payload.extend_from_slice(&extension.extension_specification.to_le_bytes());
        payload.push(extension.direct_memory_specification);
        return payload;
    }

    let mut payload = Vec::with_capacity(2 + 1 + 1 + 1 + device_spec.len() + 1);
    payload.extend_from_slice(&extension.extension_specification.to_le_bytes());
    payload.push(extension.extension_specification_modification);
    payload.push(extension.device_modification_index);
    payload.push(extension.device_modification_flags);
    payload.extend_from_slice(&device_spec);
    payload.push(extension.direct_memory_specification);
    payload
}

async fn request_plain_read_bits(
    client: &SlmpClient,
    mode: SlmpCompatibilityMode,
    device: SlmpDeviceAddress,
    points: u16,
) -> Result<Vec<bool>, Box<dyn Error>> {
    let mut payload = encode_device_spec(mode, device);
    payload.extend_from_slice(&points.to_le_bytes());
    let raw = client
        .request(
            SlmpCommand::DeviceRead,
            subcommand(mode, true),
            &payload,
            true,
        )
        .await?;
    unpack_bits(&raw, points as usize)
}

async fn request_plain_write_bits(
    client: &SlmpClient,
    mode: SlmpCompatibilityMode,
    device: SlmpDeviceAddress,
    values: &[bool],
) -> Result<(), Box<dyn Error>> {
    let mut payload = encode_device_spec(mode, device);
    payload.extend_from_slice(&(values.len() as u16).to_le_bytes());
    payload.extend_from_slice(&pack_bits(values));
    let _ = client
        .request(
            SlmpCommand::DeviceWrite,
            subcommand(mode, true),
            &payload,
            true,
        )
        .await?;
    Ok(())
}

async fn request_plain_read_words(
    client: &SlmpClient,
    mode: SlmpCompatibilityMode,
    device: SlmpDeviceAddress,
    points: u16,
) -> Result<Vec<u16>, Box<dyn Error>> {
    let mut payload = encode_device_spec(mode, device);
    payload.extend_from_slice(&points.to_le_bytes());
    let raw = client
        .request(
            SlmpCommand::DeviceRead,
            subcommand(mode, false),
            &payload,
            true,
        )
        .await?;
    payload_to_words(&raw)
}

async fn request_plain_write_words(
    client: &SlmpClient,
    mode: SlmpCompatibilityMode,
    device: SlmpDeviceAddress,
    values: &[u16],
) -> Result<(), Box<dyn Error>> {
    let mut payload = encode_device_spec(mode, device);
    payload.extend_from_slice(&(values.len() as u16).to_le_bytes());
    payload.extend_from_slice(&words_to_payload(values));
    let _ = client
        .request(
            SlmpCommand::DeviceWrite,
            subcommand(mode, false),
            &payload,
            true,
        )
        .await?;
    Ok(())
}

async fn request_ext_read_bits(
    client: &SlmpClient,
    mode: SlmpCompatibilityMode,
    qualified: SlmpQualifiedDeviceAddress,
    points: u16,
    extension: SlmpExtensionSpec,
) -> Result<Vec<bool>, Box<dyn Error>> {
    let extension = effective_extension(qualified, extension);
    let mut payload = encode_extended_device_spec(mode, qualified.device, extension);
    payload.extend_from_slice(&points.to_le_bytes());
    let raw = client
        .request(
            SlmpCommand::DeviceRead,
            ext_subcommand(mode, true, extension),
            &payload,
            true,
        )
        .await?;
    unpack_bits(&raw, points as usize)
}

async fn request_ext_write_bits(
    client: &SlmpClient,
    mode: SlmpCompatibilityMode,
    qualified: SlmpQualifiedDeviceAddress,
    values: &[bool],
    extension: SlmpExtensionSpec,
) -> Result<(), Box<dyn Error>> {
    let extension = effective_extension(qualified, extension);
    let mut payload = encode_extended_device_spec(mode, qualified.device, extension);
    payload.extend_from_slice(&(values.len() as u16).to_le_bytes());
    payload.extend_from_slice(&pack_bits(values));
    let _ = client
        .request(
            SlmpCommand::DeviceWrite,
            ext_subcommand(mode, true, extension),
            &payload,
            true,
        )
        .await?;
    Ok(())
}

async fn request_ext_read_words(
    client: &SlmpClient,
    mode: SlmpCompatibilityMode,
    qualified: SlmpQualifiedDeviceAddress,
    points: u16,
    extension: SlmpExtensionSpec,
) -> Result<Vec<u16>, Box<dyn Error>> {
    let extension = effective_extension(qualified, extension);
    let mut payload = encode_extended_device_spec(mode, qualified.device, extension);
    payload.extend_from_slice(&points.to_le_bytes());
    let raw = client
        .request(
            SlmpCommand::DeviceRead,
            ext_subcommand(mode, false, extension),
            &payload,
            true,
        )
        .await?;
    payload_to_words(&raw)
}

async fn request_ext_write_words(
    client: &SlmpClient,
    mode: SlmpCompatibilityMode,
    qualified: SlmpQualifiedDeviceAddress,
    values: &[u16],
    extension: SlmpExtensionSpec,
) -> Result<(), Box<dyn Error>> {
    let extension = effective_extension(qualified, extension);
    let mut payload = encode_extended_device_spec(mode, qualified.device, extension);
    payload.extend_from_slice(&(values.len() as u16).to_le_bytes());
    payload.extend_from_slice(&words_to_payload(values));
    let _ = client
        .request(
            SlmpCommand::DeviceWrite,
            ext_subcommand(mode, false, extension),
            &payload,
            true,
        )
        .await?;
    Ok(())
}

fn ensure_all_equal<T: PartialEq + std::fmt::Debug>(
    label: &str,
    observed: &[(&str, T)],
) -> Result<(), Box<dyn Error>> {
    let Some((_, expected)) = observed.first() else {
        return Ok(());
    };
    for (name, value) in observed.iter().skip(1) {
        if value != expected {
            let details = observed
                .iter()
                .map(|(source, value)| format!("{source}={value:?}"))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(make_error(format!(
                "{label} mismatch: {details}; diverged at {name}"
            )));
        }
    }
    Ok(())
}

fn long_timer_state_base(device: SlmpDeviceAddress) -> Option<(SlmpDeviceAddress, bool)> {
    match device.code {
        SlmpDeviceCode::LTS => Some((
            SlmpDeviceAddress::new(SlmpDeviceCode::LTN, device.number),
            true,
        )),
        SlmpDeviceCode::LTC => Some((
            SlmpDeviceAddress::new(SlmpDeviceCode::LTN, device.number),
            false,
        )),
        SlmpDeviceCode::LSTS => Some((
            SlmpDeviceAddress::new(SlmpDeviceCode::LSTN, device.number),
            true,
        )),
        SlmpDeviceCode::LSTC => Some((
            SlmpDeviceAddress::new(SlmpDeviceCode::LSTN, device.number),
            false,
        )),
        _ => None,
    }
}

fn decode_long_state(words: &[u16], contact: bool) -> Result<bool, Box<dyn Error>> {
    if words.len() != 4 {
        return Err(make_error("expected 4-word long timer block"));
    }
    let status = words[2];
    Ok(if contact {
        (status & 0x0002) != 0
    } else {
        (status & 0x0001) != 0
    })
}

fn decode_long_current(words: &[u16]) -> Result<u32, Box<dyn Error>> {
    if words.len() != 4 {
        return Err(make_error("expected 4-word long timer block"));
    }
    Ok(words[0] as u32 | ((words[1] as u32) << 16))
}

async fn assert_long_timer_state_reads(
    client: &SlmpClient,
    mode: SlmpCompatibilityMode,
    address: &str,
    expected: bool,
) -> Result<(), Box<dyn Error>> {
    let device = SlmpAddress::parse(address)?;
    let (base_device, contact) = long_timer_state_base(device)
        .ok_or_else(|| make_error(format!("{address} is not a long timer state device")))?;
    let named = read_named(client, &[address.to_string()]).await?;
    let typed = read_typed(client, device, "BIT").await?.as_bool()?;
    let block_words = client.read_words_raw(base_device, 4).await?;
    let request_words = request_plain_read_words(client, mode, base_device, 4).await?;
    let dedicated = match base_device.code {
        SlmpDeviceCode::LTN => {
            let result = client.read_long_timer(base_device.number, 1).await?;
            if contact {
                result[0].contact
            } else {
                result[0].coil
            }
        }
        SlmpDeviceCode::LSTN => {
            let result = client
                .read_long_retentive_timer(base_device.number, 1)
                .await?;
            if contact {
                result[0].contact
            } else {
                result[0].coil
            }
        }
        _ => return Err(make_error("unsupported long timer base code")),
    };
    let observed = [
        ("read_typed", typed),
        ("read_named", value_from_named_bool(&named, address)?),
        (
            "read_words_raw(4)",
            decode_long_state(&block_words, contact)?,
        ),
        (
            "request_block(4)",
            decode_long_state(&request_words, contact)?,
        ),
        ("dedicated", dedicated),
        ("expected", expected),
    ];
    ensure_all_equal(&format!("{address} expected={expected}"), &observed)
}

async fn assert_long_timer_current_reads(
    client: &SlmpClient,
    mode: SlmpCompatibilityMode,
    address: &str,
    named_address: &str,
    expected: u32,
) -> Result<(), Box<dyn Error>> {
    let device = SlmpAddress::parse(address)?;
    let named = read_named(client, &[named_address.to_string()]).await?;
    let typed = match read_typed(client, device, "D").await? {
        SlmpValue::U32(value) => value,
        other => {
            return Err(make_error(format!(
                "expected U32 for {address}, got {other:?}"
            )));
        }
    };
    let block_words = client.read_words_raw(device, 4).await?;
    let request_words = request_plain_read_words(client, mode, device, 4).await?;
    let dedicated = match device.code {
        SlmpDeviceCode::LTN => client.read_long_timer(device.number, 1).await?[0].current_value,
        SlmpDeviceCode::LSTN => {
            client.read_long_retentive_timer(device.number, 1).await?[0].current_value
        }
        _ => {
            return Err(make_error(format!(
                "{address} is not a long timer current device"
            )));
        }
    };
    let observed = [
        ("read_typed", typed),
        ("read_named", value_from_named_u32(&named, named_address)?),
        ("read_words_raw(4)", decode_long_current(&block_words)?),
        ("request_block(4)", decode_long_current(&request_words)?),
        ("dedicated", dedicated),
        ("expected", expected),
    ];
    ensure_all_equal(&format!("{address} expected={expected}"), &observed)
}

async fn assert_bit_reads(
    client: &SlmpClient,
    mode: SlmpCompatibilityMode,
    address: &str,
    expected: bool,
) -> Result<(), Box<dyn Error>> {
    let device = SlmpAddress::parse(address)?;
    let named = read_named(client, &[address.to_string()]).await?;
    let observed = [
        ("read_bits", client.read_bits(device, 1).await?[0]),
        (
            "read_typed",
            read_typed(client, device, "BIT").await?.as_bool()?,
        ),
        ("read_named", value_from_named_bool(&named, address)?),
        (
            "request",
            request_plain_read_bits(client, mode, device, 1).await?[0],
        ),
    ];
    ensure_all_equal(
        &format!("{address} expected={expected}"),
        &[
            ("read_bits", observed[0].1),
            ("read_typed", observed[1].1),
            ("read_named", observed[2].1),
            ("request", observed[3].1),
            ("expected", expected),
        ],
    )
}

async fn assert_word_reads(
    client: &SlmpClient,
    mode: SlmpCompatibilityMode,
    address: &str,
    expected: u16,
) -> Result<(), Box<dyn Error>> {
    let device = SlmpAddress::parse(address)?;
    let named = read_named(client, &[address.to_string()]).await?;
    let random = client.read_random(&[device], &[]).await?;
    let typed = match read_typed(client, device, "U").await? {
        SlmpValue::U16(value) => value,
        other => {
            return Err(make_error(format!(
                "expected U16 for {address}, got {other:?}"
            )));
        }
    };
    let observed = [
        ("read_words_raw", client.read_words_raw(device, 1).await?[0]),
        ("read_typed", typed),
        ("read_named", value_from_named_u16(&named, address)?),
        (
            "read_random",
            *random
                .word_values
                .first()
                .ok_or_else(|| make_error("missing random word value"))?,
        ),
        (
            "request",
            request_plain_read_words(client, mode, device, 1).await?[0],
        ),
        ("expected", expected),
    ];
    ensure_all_equal(&format!("{address} expected={expected}"), &observed)
}

async fn assert_dword_reads(
    client: &SlmpClient,
    mode: SlmpCompatibilityMode,
    address: &str,
    named_address: &str,
    expected: u32,
) -> Result<(), Box<dyn Error>> {
    let device = SlmpAddress::parse(address)?;
    let named = read_named(client, &[named_address.to_string()]).await?;
    let random = client.read_random(&[], &[device]).await?;
    let typed = match read_typed(client, device, "D").await? {
        SlmpValue::U32(value) => value,
        other => {
            return Err(make_error(format!(
                "expected U32 for {address}, got {other:?}"
            )));
        }
    };
    let request_words = request_plain_read_words(client, mode, device, 2).await?;
    let observed = [
        (
            "read_dwords_raw",
            client.read_dwords_raw(device, 1).await?[0],
        ),
        ("read_typed", typed),
        ("read_named", value_from_named_u32(&named, named_address)?),
        (
            "read_random",
            *random
                .dword_values
                .first()
                .ok_or_else(|| make_error("missing random dword value"))?,
        ),
        ("request", words_to_dword(&request_words)?),
        ("expected", expected),
    ];
    ensure_all_equal(&format!("{address} expected={expected}"), &observed)
}

async fn compare_bit_device(
    client: &SlmpClient,
    mode: SlmpCompatibilityMode,
    address: &str,
) -> Result<(), Box<dyn Error>> {
    let device = SlmpAddress::parse(address)?;
    if long_timer_state_base(device).is_some() {
        let original =
            value_from_named_bool(&read_named(client, &[address.to_string()]).await?, address)?;
        let result: Result<(), Box<dyn Error>> = async {
            assert_long_timer_state_reads(client, mode, address, original).await?;
            for (writer, value) in [
                ("write_random_bits:on", true),
                ("write_random_bits:off", false),
                ("write_typed:on", true),
                ("write_typed:off", false),
                ("write_named:on", true),
                ("write_named:off", false),
            ] {
                match writer {
                    "write_random_bits:on" | "write_random_bits:off" => {
                        client.write_random_bits(&[(device, value)]).await?
                    }
                    "write_typed:on" | "write_typed:off" => {
                        write_typed(client, device, "BIT", &SlmpValue::Bool(value)).await?
                    }
                    _ => {
                        let mut updates = NamedAddress::new();
                        updates.insert(address.to_string(), SlmpValue::Bool(value));
                        write_named(client, &updates).await?;
                    }
                }
                assert_long_timer_state_reads(client, mode, address, value).await?;
            }
            Ok(())
        }
        .await;
        let mut restore = NamedAddress::new();
        restore.insert(address.to_string(), SlmpValue::Bool(original));
        write_named(client, &restore).await?;
        return result;
    }

    let original = client.read_bits(device, 1).await?[0];
    let result: Result<(), Box<dyn Error>> = async {
        assert_bit_reads(client, mode, address, original).await?;
        for (writer, value) in [
            ("write_bits:on", true),
            ("write_bits:off", false),
            ("write_random_bits:on", true),
            ("write_random_bits:off", false),
            ("write_typed:on", true),
            ("write_typed:off", false),
            ("write_named:on", true),
            ("write_named:off", false),
            ("request:on", true),
            ("request:off", false),
        ] {
            match writer {
                "write_bits:on" | "write_bits:off" => client.write_bits(device, &[value]).await?,
                "write_random_bits:on" | "write_random_bits:off" => {
                    client.write_random_bits(&[(device, value)]).await?
                }
                "write_typed:on" | "write_typed:off" => {
                    write_typed(client, device, "BIT", &SlmpValue::Bool(value)).await?
                }
                "write_named:on" | "write_named:off" => {
                    let mut updates = NamedAddress::new();
                    updates.insert(address.to_string(), SlmpValue::Bool(value));
                    write_named(client, &updates).await?;
                }
                _ => request_plain_write_bits(client, mode, device, &[value]).await?,
            }
            assert_bit_reads(client, mode, address, value).await?;
        }
        Ok(())
    }
    .await;
    client.write_bits(device, &[original]).await?;
    result
}

async fn compare_word_device(
    client: &SlmpClient,
    mode: SlmpCompatibilityMode,
    address: &str,
) -> Result<(), Box<dyn Error>> {
    let device = SlmpAddress::parse(address)?;
    let original = client.read_words_raw(device, 1).await?[0];
    let value_a = seeded_u16(address, 0x11);
    let value_b = seeded_u16(address, 0x22);
    let result: Result<(), Box<dyn Error>> = async {
        assert_word_reads(client, mode, address, original).await?;
        for (writer, value) in [
            ("write_words:a", value_a),
            ("write_words:b", value_b),
            ("write_random_words:a", value_a),
            ("write_random_words:b", value_b),
            ("write_typed:a", value_a),
            ("write_typed:b", value_b),
            ("write_named:a", value_a),
            ("write_named:b", value_b),
            ("request:a", value_a),
            ("request:b", value_b),
        ] {
            match writer {
                "write_words:a" | "write_words:b" => client.write_words(device, &[value]).await?,
                "write_random_words:a" | "write_random_words:b" => {
                    client.write_random_words(&[(device, value)], &[]).await?
                }
                "write_typed:a" | "write_typed:b" => {
                    write_typed(client, device, "U", &SlmpValue::U16(value)).await?
                }
                "write_named:a" | "write_named:b" => {
                    let mut updates = NamedAddress::new();
                    updates.insert(address.to_string(), SlmpValue::U16(value));
                    write_named(client, &updates).await?;
                }
                _ => request_plain_write_words(client, mode, device, &[value]).await?,
            }
            assert_word_reads(client, mode, address, value).await?;
        }
        Ok(())
    }
    .await;
    client.write_words(device, &[original]).await?;
    result
}

async fn compare_dword_device(
    client: &SlmpClient,
    mode: SlmpCompatibilityMode,
    address: &str,
    named_address: &str,
) -> Result<(), Box<dyn Error>> {
    let device = SlmpAddress::parse(address)?;
    if matches!(device.code, SlmpDeviceCode::LTN | SlmpDeviceCode::LSTN) {
        let original = value_from_named_u32(
            &read_named(client, &[named_address.to_string()]).await?,
            named_address,
        )?;
        let value_a = seeded_u32(address, 0x33);
        let value_b = seeded_u32(address, 0x44);
        let result: Result<(), Box<dyn Error>> = async {
            assert_long_timer_current_reads(client, mode, address, named_address, original).await?;
            for (writer, value) in [
                ("write_random_words:a", value_a),
                ("write_random_words:b", value_b),
                ("write_typed:a", value_a),
                ("write_typed:b", value_b),
                ("write_named:a", value_a),
                ("write_named:b", value_b),
            ] {
                match writer {
                    "write_random_words:a" | "write_random_words:b" => {
                        client.write_random_words(&[], &[(device, value)]).await?
                    }
                    "write_typed:a" | "write_typed:b" => {
                        write_typed(client, device, "D", &SlmpValue::U32(value)).await?
                    }
                    _ => {
                        let mut updates = NamedAddress::new();
                        updates.insert(named_address.to_string(), SlmpValue::U32(value));
                        write_named(client, &updates).await?;
                    }
                }
                assert_long_timer_current_reads(client, mode, address, named_address, value)
                    .await?;
            }
            Ok(())
        }
        .await;
        let mut restore = NamedAddress::new();
        restore.insert(named_address.to_string(), SlmpValue::U32(original));
        write_named(client, &restore).await?;
        return result;
    }

    let original = client.read_dwords_raw(device, 1).await?[0];
    let value_a = seeded_u32(address, 0x33);
    let value_b = seeded_u32(address, 0x44);
    let result: Result<(), Box<dyn Error>> = async {
        assert_dword_reads(client, mode, address, named_address, original).await?;
        for (writer, value) in [
            ("write_dwords:a", value_a),
            ("write_dwords:b", value_b),
            ("write_random_words:a", value_a),
            ("write_random_words:b", value_b),
            ("write_typed:a", value_a),
            ("write_typed:b", value_b),
            ("write_named:a", value_a),
            ("write_named:b", value_b),
            ("request:a", value_a),
            ("request:b", value_b),
        ] {
            match writer {
                "write_dwords:a" | "write_dwords:b" => {
                    client.write_dwords(device, &[value]).await?
                }
                "write_random_words:a" | "write_random_words:b" => {
                    client.write_random_words(&[], &[(device, value)]).await?
                }
                "write_typed:a" | "write_typed:b" => {
                    write_typed(client, device, "D", &SlmpValue::U32(value)).await?
                }
                "write_named:a" | "write_named:b" => {
                    let mut updates = NamedAddress::new();
                    updates.insert(named_address.to_string(), SlmpValue::U32(value));
                    write_named(client, &updates).await?;
                }
                _ => {
                    request_plain_write_words(client, mode, device, &dword_to_words(value)).await?
                }
            }
            assert_dword_reads(client, mode, address, named_address, value).await?;
        }
        Ok(())
    }
    .await;
    client.write_dwords(device, &[original]).await?;
    result
}

async fn compare_ext_bit_device(
    client: &SlmpClient,
    mode: SlmpCompatibilityMode,
    address: &str,
) -> Result<(), Box<dyn Error>> {
    let qualified = parse_qualified_device(address)?;
    let extension = effective_extension(qualified, SlmpExtensionSpec::default());
    let original = client.read_bits_extended(qualified, 1, extension).await?[0];
    let result: Result<(), Box<dyn Error>> = async {
        for (writer, value) in [
            ("write_bits_extended:on", true),
            ("write_bits_extended:off", false),
            ("request:on", true),
            ("request:off", false),
        ] {
            match writer {
                "write_bits_extended:on" | "write_bits_extended:off" => {
                    client
                        .write_bits_extended(qualified, &[value], extension)
                        .await?
                }
                _ => request_ext_write_bits(client, mode, qualified, &[value], extension).await?,
            }
            let observed = [
                (
                    "read_bits_extended",
                    client.read_bits_extended(qualified, 1, extension).await?[0],
                ),
                (
                    "request",
                    request_ext_read_bits(client, mode, qualified, 1, extension).await?[0],
                ),
                ("expected", value),
            ];
            ensure_all_equal(&format!("{address} expected={value}"), &observed)?;
        }
        Ok(())
    }
    .await;
    client
        .write_bits_extended(qualified, &[original], extension)
        .await?;
    result
}

async fn compare_ext_word_device(
    client: &SlmpClient,
    mode: SlmpCompatibilityMode,
    address: &str,
) -> Result<(), Box<dyn Error>> {
    let qualified = parse_qualified_device(address)?;
    let extension = effective_extension(qualified, SlmpExtensionSpec::default());
    let original = client.read_words_extended(qualified, 1, extension).await?[0];
    let value_a = seeded_u16(address, 0x55);
    let value_b = seeded_u16(address, 0x66);
    let result: Result<(), Box<dyn Error>> = async {
        for (writer, value) in [
            ("write_words_extended:a", value_a),
            ("write_words_extended:b", value_b),
            ("request:a", value_a),
            ("request:b", value_b),
        ] {
            match writer {
                "write_words_extended:a" | "write_words_extended:b" => {
                    client
                        .write_words_extended(qualified, &[value], extension)
                        .await?
                }
                _ => request_ext_write_words(client, mode, qualified, &[value], extension).await?,
            }
            let observed = [
                (
                    "read_words_extended",
                    client.read_words_extended(qualified, 1, extension).await?[0],
                ),
                (
                    "request",
                    request_ext_read_words(client, mode, qualified, 1, extension).await?[0],
                ),
                ("expected", value),
            ];
            ensure_all_equal(&format!("{address} expected={value}"), &observed)?;
        }
        Ok(())
    }
    .await;
    client
        .write_words_extended(qualified, &[original], extension)
        .await?;
    result
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    print_connection_banner("device_matrix_compare");
    let options = options_from_env()?;
    let mode = options.compatibility_mode;
    let client = SlmpClient::connect(options).await?;
    let selected = env_csv("SLMP_COMPARE_ONLY", "");
    let run_all = selected.is_empty();
    let wants = |address: &str| run_all || selected.iter().any(|item| item == address);
    let mut failures = Vec::new();
    let mut passed = 0usize;

    for address in BIT_DEVICES {
        if !wants(address) {
            continue;
        }
        match compare_bit_device(&client, mode, address).await {
            Ok(()) => {
                passed += 1;
                println!("PASS bit   {address}");
            }
            Err(error) => failures.push(format!("{address}: {error}")),
        }
    }

    for address in WORD_DEVICES {
        if !wants(address) {
            continue;
        }
        match compare_word_device(&client, mode, address).await {
            Ok(()) => {
                passed += 1;
                println!("PASS word  {address}");
            }
            Err(error) => failures.push(format!("{address}: {error}")),
        }
    }

    for (address, named_address) in DWORD_DEVICES {
        if !wants(address) {
            continue;
        }
        match compare_dword_device(&client, mode, address, named_address).await {
            Ok(()) => {
                passed += 1;
                println!("PASS dword {address}");
            }
            Err(error) => failures.push(format!("{address}: {error}")),
        }
    }

    for address in EXT_BIT_DEVICES {
        if !wants(address) {
            continue;
        }
        match compare_ext_bit_device(&client, mode, address).await {
            Ok(()) => {
                passed += 1;
                println!("PASS ext-bit  {address}");
            }
            Err(error) => failures.push(format!("{address}: {error}")),
        }
    }

    for address in EXT_WORD_DEVICES {
        if !wants(address) {
            continue;
        }
        match compare_ext_word_device(&client, mode, address).await {
            Ok(()) => {
                passed += 1;
                println!("PASS ext-word {address}");
            }
            Err(error) => failures.push(format!("{address}: {error}")),
        }
    }

    let total = if run_all {
        BIT_DEVICES.len()
            + WORD_DEVICES.len()
            + DWORD_DEVICES.len()
            + EXT_BIT_DEVICES.len()
            + EXT_WORD_DEVICES.len()
    } else {
        selected.len()
    };
    println!(
        "summary: passed={passed} failed={} total={total}",
        failures.len()
    );

    if failures.is_empty() {
        return Ok(());
    }

    println!("failures:");
    for failure in &failures {
        println!("  - {failure}");
    }
    Err(make_error(format!(
        "{} device checks failed",
        failures.len()
    )))
}
