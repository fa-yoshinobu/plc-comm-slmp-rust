use crate::address::{parse_device, parse_named_address};
use crate::client::{PreparedRandomRead, SlmpClient};
use crate::error::SlmpError;
use crate::model::{
    SlmpDeviceAddress, SlmpDeviceCode, SlmpLongTimerResult, SlmpPlcProfile,
    SlmpQualifiedDeviceAddress,
};
use async_stream::try_stream;
use futures_core::stream::Stream;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::Duration;

const RANDOM_READ_BATCH_LIMIT: usize = 96;

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(untagged)]
pub enum SlmpValue {
    Bool(bool),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    F32(f32),
}

impl SlmpValue {
    pub fn as_bool(&self) -> Result<bool, SlmpError> {
        match self {
            Self::Bool(value) => Ok(*value),
            _ => Err(SlmpError::new("Expected bool value.")),
        }
    }
}

pub type NamedAddress = BTreeMap<String, SlmpValue>;

#[derive(Debug, Clone)]
struct LongTimerReadSpec {
    base_code: SlmpDeviceCode,
    kind: LongTimerReadKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LongTimerReadKind {
    Current,
    Contact,
    Coil,
}

#[derive(Debug, Clone)]
struct NamedReadEntry {
    address: String,
    device: SlmpDeviceAddress,
    dtype: String,
    bit_word_read: Option<BitWordRead>,
    source: Option<NamedReadSource>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NamedReadSource {
    Word(usize),
    DWord(usize),
    Bit { word_index: usize, bit_index: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BitWordRead {
    device: SlmpDeviceAddress,
    bit_index: u8,
}

#[derive(Debug, Clone)]
struct NamedReadPlan {
    entries: Vec<NamedReadEntry>,
    word_devices: Vec<SlmpDeviceAddress>,
    dword_devices: Vec<SlmpDeviceAddress>,
}

pub async fn read_typed(
    client: &SlmpClient,
    device: SlmpDeviceAddress,
    dtype: &str,
) -> Result<SlmpValue, SlmpError> {
    let normalized_dtype = require_dtype(dtype)?;
    let client_profile = client.plc_profile().await;
    if device.plc_profile() != client_profile {
        return Err(SlmpError::new(format!(
            "device plc_profile '{}' does not match client plc_profile '{}'",
            device.plc_profile().canonical_name(),
            client_profile.canonical_name()
        )));
    }
    validate_dword_only_entry(&device.to_string(), device, &normalized_dtype)?;
    validate_typed_device_dtype(device, &normalized_dtype)?;
    if matches!(device.code(), SlmpDeviceCode::LZ) && matches!(normalized_dtype.as_str(), "D" | "L")
    {
        let raw = read_random_dword_scalar(client, device).await?;
        return Ok(if normalized_dtype == "L" {
            SlmpValue::I32(raw as i32)
        } else {
            SlmpValue::U32(raw)
        });
    }
    if let Some(spec) = long_timer_read_spec(device.code()) {
        validate_long_timer_entry(&device.to_string(), device, &normalized_dtype)?;
        if matches!(spec.base_code, SlmpDeviceCode::LCN)
            && matches!(spec.kind, LongTimerReadKind::Current)
        {
            let raw = read_random_dword_scalar(client, device).await?;
            return Ok(if normalized_dtype == "L" {
                SlmpValue::I32(raw as i32)
            } else {
                SlmpValue::U32(raw)
            });
        }
        if is_long_counter_state_device(device.code()) {
            return Ok(SlmpValue::Bool(client.read_bits(device, 1).await?[0]));
        }

        let timer = read_long_like_point(client, spec.base_code, device.number()).await?;
        return decode_long_like_value(&normalized_dtype, &spec, &timer);
    }

    match normalized_dtype.as_str() {
        "BIT" => Ok(SlmpValue::Bool(client.read_bits(device, 1).await?[0])),
        "F" => Ok(SlmpValue::F32(f32::from_bits(
            client.read_dwords_raw(device, 1).await?[0],
        ))),
        "D" => Ok(SlmpValue::U32(client.read_dwords_raw(device, 1).await?[0])),
        "L" => Ok(SlmpValue::I32(
            client.read_dwords_raw(device, 1).await?[0] as i32,
        )),
        "S" => Ok(SlmpValue::I16(
            client.read_words_raw(device, 1).await?[0] as i16,
        )),
        "U" => Ok(SlmpValue::U16(client.read_words_raw(device, 1).await?[0])),
        other => Err(SlmpError::new(format!("Unsupported dtype '{other}'."))),
    }
}

pub async fn write_typed(
    client: &SlmpClient,
    device: SlmpDeviceAddress,
    dtype: &str,
    value: &SlmpValue,
) -> Result<(), SlmpError> {
    let normalized_dtype = require_dtype(dtype)?;
    if long_timer_read_spec(device.code()).is_some() {
        validate_long_timer_entry(&device.to_string(), device, &normalized_dtype)?;
    }
    validate_dword_only_entry(&device.to_string(), device, &normalized_dtype)?;
    validate_typed_device_dtype(device, &normalized_dtype)?;
    let route = resolve_write_route(device, &normalized_dtype);
    match route {
        NamedWriteRoute::RandomBits => {
            client
                .write_random_bits(&[(device, scalar_to_bool(value)?)])
                .await
        }
        NamedWriteRoute::ContiguousBits => {
            client.write_bits(device, &[scalar_to_bool(value)?]).await
        }
        NamedWriteRoute::RandomDWords | NamedWriteRoute::ContiguousDWords => {
            let raw = match normalized_dtype.as_str() {
                "F" => scalar_to_f32(value)?.to_bits(),
                "L" => scalar_to_i32(value)? as u32,
                _ => scalar_to_u32(value)?,
            };
            if matches!(route, NamedWriteRoute::RandomDWords) {
                client.write_random_words(&[], &[(device, raw)]).await
            } else {
                client.write_dwords(device, &[raw]).await
            }
        }
        NamedWriteRoute::ContiguousWords => {
            let raw = if normalized_dtype == "S" {
                scalar_to_i16(value)? as u16
            } else {
                scalar_to_u16(value)?
            };
            client.write_words(device, &[raw]).await
        }
    }
}

/// Set or clear one bit through one immutable direct word route.
///
/// The mandatory read and write occupy one FIFO turn and one absolute
/// post-admission deadline. A successful read is always followed by the write,
/// even when the bit is unchanged. The pair is not PLC-atomic, never retries,
/// and an unconfirmed possibly transmitted write is outcome unknown.
pub async fn write_bit_in_word(
    client: &SlmpClient,
    device: SlmpDeviceAddress,
    bit_index: u8,
    value: bool,
) -> Result<(), SlmpError> {
    if bit_index > 15 {
        return Err(SlmpError::new("bit_index must be 0-15."));
    }
    if !device.code().is_word_device() {
        return Err(SlmpError::new("write_bit_in_word requires a word device"));
    }
    client
        .write_bit_in_word_turn(device, bit_index, value)
        .await
}

/// Set or clear one bit through one immutable qualified Extended Device route.
///
/// U-qualified module-buffer and J-qualified link-direct addresses retain the
/// same route for the mandatory read and write. The pair owns one client FIFO
/// turn and one absolute post-admission deadline, but is not atomic at the PLC.
/// A successful read is always followed by the write; the pair never retries,
/// and an unconfirmed possibly transmitted write is outcome unknown.
pub async fn write_bit_in_word_extended(
    client: &SlmpClient,
    device: SlmpQualifiedDeviceAddress,
    bit_index: u8,
    value: bool,
) -> Result<(), SlmpError> {
    if bit_index > 15 {
        return Err(SlmpError::new("bit_index must be 0-15."));
    }
    if !device.device().code().is_word_device() {
        return Err(SlmpError::new(
            "write_bit_in_word_extended requires a word device",
        ));
    }
    client
        .write_bit_in_word_extended_turn(device, bit_index, value)
        .await
}

pub async fn read_words_single_request(
    client: &SlmpClient,
    start: SlmpDeviceAddress,
    count: usize,
) -> Result<Vec<u16>, SlmpError> {
    validate_single_request_count(count, 960)?;
    client.read_words_raw(start, count as u16).await
}

/// Reads one contiguous bit-device range with exactly one SLMP request.
///
/// The complete address/profile/count admission is performed before the
/// low-level client sends the request. This helper never splits or retries.
pub async fn read_bits_single_request(
    client: &SlmpClient,
    start: SlmpDeviceAddress,
    count: usize,
) -> Result<Vec<bool>, SlmpError> {
    validate_single_request_count(count, usize::from(u16::MAX))?;
    client.read_bits(start, count as u16).await
}

pub async fn read_dwords_single_request(
    client: &SlmpClient,
    start: SlmpDeviceAddress,
    count: usize,
) -> Result<Vec<u32>, SlmpError> {
    if matches!(start.code(), SlmpDeviceCode::LZ) {
        validate_single_request_count(count, RANDOM_READ_BATCH_LIMIT)?;
        client
            .validate_random_native_dword_sequence(start, count, "read_dwords_single_request")
            .await?;
        let devices = (0..count)
            .map(|offset| {
                let number = u32::try_from(u64::from(start.number()) + offset as u64)
                    .expect("selected wire span was validated");
                SlmpDeviceAddress::new(start.code(), number, start.plc_profile())
            })
            .collect::<Vec<_>>();
        return Ok(client.read_random(&[], &devices).await?.dword_values);
    }
    validate_single_request_count(count, 480)?;
    client.read_dwords_raw(start, count as u16).await
}

pub async fn write_words_single_request(
    client: &SlmpClient,
    start: SlmpDeviceAddress,
    values: &[u16],
) -> Result<(), SlmpError> {
    validate_single_request_values(values.len(), 960)?;
    client.write_words(start, values).await
}

/// Writes one contiguous bit-device range with exactly one SLMP request.
///
/// The complete address/profile/value admission is performed before the
/// low-level client sends the request. This helper never splits or retries.
pub async fn write_bits_single_request(
    client: &SlmpClient,
    start: SlmpDeviceAddress,
    values: &[bool],
) -> Result<(), SlmpError> {
    validate_single_request_values(values.len(), usize::from(u16::MAX))?;
    client.write_bits(start, values).await
}

pub async fn write_dwords_single_request(
    client: &SlmpClient,
    start: SlmpDeviceAddress,
    values: &[u32],
) -> Result<(), SlmpError> {
    validate_single_request_values(values.len(), 480)?;
    client.write_dwords(start, values).await
}

pub async fn read_named(
    client: &SlmpClient,
    addresses: &[String],
) -> Result<NamedAddress, SlmpError> {
    let plan = compile_read_plan(addresses, client.plc_profile().await)?;
    let prepared = prepare_named_read(client, &plan).await?;
    read_named_compiled(client, &plan, prepared.as_ref()).await
}

pub async fn write_named(client: &SlmpClient, updates: &NamedAddress) -> Result<(), SlmpError> {
    if updates.is_empty() {
        return Err(SlmpError::new("write_named requires at least one update."));
    }
    let plc_profile = client.plc_profile().await;
    let mut bit_entries = Vec::new();
    let mut word_entries = Vec::new();
    let mut dword_entries = Vec::new();
    for (address, value) in updates {
        let parts = parse_named_address(address)?;
        let device = parse_device(&parts.base, plc_profile)?;
        if parts.dtype == "BIT_IN_WORD" {
            return Err(SlmpError::new(format!(
                "Address '{address}' requires a read-modify-write sequence. Use write_bit_in_word explicitly."
            )));
        }
        let resolved_dtype =
            resolve_dtype_for_address(address, device, &parts.dtype, parts.bit_index)?;
        validate_named_device_dtype(address, device, &resolved_dtype)?;
        validate_long_timer_entry(address, device, &resolved_dtype)?;
        validate_dword_only_entry(address, device, &resolved_dtype)?;
        match resolve_write_route(device, &resolved_dtype) {
            NamedWriteRoute::RandomBits | NamedWriteRoute::ContiguousBits => {
                bit_entries.push((device, scalar_to_bool(value)?));
            }
            NamedWriteRoute::RandomDWords | NamedWriteRoute::ContiguousDWords => {
                let raw = match resolved_dtype.as_str() {
                    "F" => scalar_to_f32(value)?.to_bits(),
                    "L" => scalar_to_i32(value)? as u32,
                    _ => scalar_to_u32(value)?,
                };
                dword_entries.push((device, raw));
            }
            NamedWriteRoute::ContiguousWords => {
                let raw = if resolved_dtype == "S" {
                    scalar_to_i16(value)? as u16
                } else {
                    scalar_to_u16(value)?
                };
                word_entries.push((device, raw));
            }
        }
    }
    if !bit_entries.is_empty() && (!word_entries.is_empty() || !dword_entries.is_empty()) {
        return Err(SlmpError::new(
            "write_named cannot combine bit and word/dword write families in one request. Split them explicitly.",
        ));
    }
    if !bit_entries.is_empty() {
        client.write_random_bits(&bit_entries).await
    } else {
        client
            .write_random_words(&word_entries, &dword_entries)
            .await
    }
}

pub fn poll_named<'a>(
    client: &'a SlmpClient,
    addresses: &'a [String],
    interval: Duration,
) -> impl Stream<Item = Result<NamedAddress, SlmpError>> + 'a {
    try_stream! {
        let plan = compile_read_plan(addresses, client.plc_profile().await)?;
        let prepared = prepare_named_read(client, &plan).await?;
        loop {
            yield read_named_compiled(client, &plan, prepared.as_ref()).await?;
            tokio::time::sleep(interval).await;
        }
    }
}

fn validate_single_request_count(count: usize, max: usize) -> Result<(), SlmpError> {
    if count == 0 || count > max {
        return Err(SlmpError::new(format!(
            "count must be in the range 1-{max}."
        )));
    }
    Ok(())
}

fn validate_single_request_values(count: usize, max: usize) -> Result<(), SlmpError> {
    if count == 0 || count > max {
        return Err(SlmpError::new(format!(
            "values.len() must be in the range 1-{max}."
        )));
    }
    Ok(())
}

fn compile_read_plan(
    addresses: &[String],
    plc_profile: SlmpPlcProfile,
) -> Result<NamedReadPlan, SlmpError> {
    let mut entries = Vec::with_capacity(addresses.len());
    let mut word_devices = Vec::with_capacity(addresses.len());
    let mut dword_devices = Vec::with_capacity(addresses.len());
    let mut seen_word_devices = HashSet::with_capacity(addresses.len());
    let mut seen_dword_devices = HashSet::with_capacity(addresses.len());
    for address in addresses {
        let parts = parse_named_address(address)?;
        let device = parse_device(&parts.base, plc_profile)?;

        let (dtype, bit_word_read) = if parts.dtype == "BIT_IN_WORD" {
            validate_bit_in_word_target(address, device)?;
            let bit_index = require_bit_in_word_index(address, parts.bit_index)?;
            if device.code().is_word_batchable() && seen_word_devices.insert(device) {
                word_devices.push(device);
            }
            (
                "BIT_IN_WORD".to_string(),
                Some(BitWordRead { device, bit_index }),
            )
        } else {
            let dtype = resolve_dtype_for_address(address, device, &parts.dtype, parts.bit_index)?;
            validate_named_device_dtype(address, device, &dtype)?;
            validate_long_timer_entry(address, device, &dtype)?;
            validate_dword_only_entry(address, device, &dtype)?;
            if long_timer_read_spec(device.code()).is_some()
                && !matches!(device.code(), SlmpDeviceCode::LCN)
            {
                return Err(SlmpError::new(format!(
                    "read_named accepts only addresses that fit one random-read request; use read_typed or an explicit long-timer helper for '{address}'."
                )));
            }
            let mut bit_word_read = None;
            if dtype == "BIT" {
                bit_word_read = plain_bit_word_read(device);
                if let Some(read) = bit_word_read {
                    if seen_word_devices.insert(read.device) {
                        word_devices.push(read.device);
                    }
                }
            } else if matches!(dtype.as_str(), "U" | "S") && device.code().is_word_batchable() {
                if seen_word_devices.insert(device) {
                    word_devices.push(device);
                }
            } else if matches!(dtype.as_str(), "D" | "L" | "F")
                && device.code().is_word_batchable()
                && seen_dword_devices.insert(device)
            {
                dword_devices.push(device);
            }
            (dtype, bit_word_read)
        };

        entries.push(NamedReadEntry {
            address: address.clone(),
            device,
            dtype,
            bit_word_read,
            source: None,
        });
    }

    let unsupported = entries
        .iter()
        .filter(|entry| {
            let supported = if let Some(read) = entry.bit_word_read {
                word_devices.contains(&read.device)
            } else {
                match entry.dtype.as_str() {
                    "U" | "S" => word_devices.contains(&entry.device),
                    "D" | "L" | "F" => dword_devices.contains(&entry.device),
                    _ => false,
                }
            };
            !supported
        })
        .map(|entry| entry.address.as_str())
        .collect::<Vec<_>>();
    if !unsupported.is_empty() {
        return Err(SlmpError::new(format!(
            "read_named accepts only addresses that fit one random-read request; use explicit read calls for {unsupported:?}."
        )));
    }

    let word_indexes = word_devices
        .iter()
        .copied()
        .enumerate()
        .map(|(index, device)| (device, index))
        .collect::<HashMap<_, _>>();
    let dword_indexes = dword_devices
        .iter()
        .copied()
        .enumerate()
        .map(|(index, device)| (device, index))
        .collect::<HashMap<_, _>>();
    for entry in &mut entries {
        entry.source = if let Some(read) = entry.bit_word_read {
            word_indexes
                .get(&read.device)
                .copied()
                .map(|word_index| NamedReadSource::Bit {
                    word_index,
                    bit_index: read.bit_index,
                })
        } else if matches!(entry.dtype.as_str(), "U" | "S") {
            word_indexes
                .get(&entry.device)
                .copied()
                .map(NamedReadSource::Word)
        } else {
            dword_indexes
                .get(&entry.device)
                .copied()
                .map(NamedReadSource::DWord)
        };
    }

    Ok(NamedReadPlan {
        entries,
        word_devices,
        dword_devices,
    })
}

async fn read_named_compiled(
    client: &SlmpClient,
    plan: &NamedReadPlan,
    prepared: Option<&PreparedRandomRead>,
) -> Result<NamedAddress, SlmpError> {
    let mut result = NamedAddress::new();
    let random = match prepared {
        Some(prepared) => client.execute_prepared_random_read(prepared).await?,
        None => Default::default(),
    };

    for entry in &plan.entries {
        let value = match entry.source {
            Some(NamedReadSource::Bit {
                word_index,
                bit_index,
            }) => {
                let word = *random.word_values.get(word_index).ok_or_else(|| {
                    SlmpError::new(format!(
                        "read_named random-read response omitted required word device {}",
                        entry.device
                    ))
                })?;
                SlmpValue::Bool(((word >> bit_index) & 1) != 0)
            }
            Some(NamedReadSource::Word(index)) if entry.dtype == "S" => {
                SlmpValue::I16(*random.word_values.get(index).ok_or_else(|| {
                    SlmpError::new(format!(
                        "read_named random-read response omitted required word device {}",
                        entry.device
                    ))
                })? as i16)
            }
            Some(NamedReadSource::Word(index)) if entry.dtype == "U" => {
                SlmpValue::U16(*random.word_values.get(index).ok_or_else(|| {
                    SlmpError::new(format!(
                        "read_named random-read response omitted required word device {}",
                        entry.device
                    ))
                })?)
            }
            Some(NamedReadSource::DWord(index)) if entry.dtype == "F" => SlmpValue::F32(
                f32::from_bits(*random.dword_values.get(index).ok_or_else(|| {
                    SlmpError::new(format!(
                        "read_named random-read response omitted required dword device {}",
                        entry.device
                    ))
                })?),
            ),
            Some(NamedReadSource::DWord(index)) if entry.dtype == "L" => {
                SlmpValue::I32(*random.dword_values.get(index).ok_or_else(|| {
                    SlmpError::new(format!(
                        "read_named random-read response omitted required dword device {}",
                        entry.device
                    ))
                })? as i32)
            }
            Some(NamedReadSource::DWord(index)) if entry.dtype == "D" => {
                SlmpValue::U32(*random.dword_values.get(index).ok_or_else(|| {
                    SlmpError::new(format!(
                        "read_named random-read response omitted required dword device {}",
                        entry.device
                    ))
                })?)
            }
            _ => {
                return Err(SlmpError::new(format!(
                    "read_named plan contains unsupported dtype '{}' for '{}'",
                    entry.dtype, entry.address
                )));
            }
        };
        result.insert(entry.address.clone(), value);
    }

    Ok(result)
}

async fn prepare_named_read(
    client: &SlmpClient,
    plan: &NamedReadPlan,
) -> Result<Option<PreparedRandomRead>, SlmpError> {
    if plan.word_devices.is_empty() && plan.dword_devices.is_empty() {
        return Ok(None);
    }
    client
        .prepare_random_read(&plan.word_devices, &plan.dword_devices)
        .await
        .map(Some)
}

fn plain_bit_word_read(device: SlmpDeviceAddress) -> Option<BitWordRead> {
    if !is_plain_bit_word_batchable(device.code()) {
        return None;
    }
    let bit_index = (device.number() % 16) as u8;
    Some(BitWordRead {
        device: SlmpDeviceAddress::new(
            device.code(),
            device.number() - u32::from(bit_index),
            device.plc_profile(),
        ),
        bit_index,
    })
}

fn is_plain_bit_word_batchable(code: SlmpDeviceCode) -> bool {
    // Do not add TS/TC/STS/STC/CS/CC/DX/DY here just because they are bit
    // devices. R120PCPU live checks accept their direct bit reads but reject
    // the 0x0403 random-word route used by this batching path with end code
    // 0x4032. Keep named-bit batching limited to families validated on both
    // mock and real PLC paths.
    matches!(
        code,
        SlmpDeviceCode::SM
            | SlmpDeviceCode::X
            | SlmpDeviceCode::Y
            | SlmpDeviceCode::M
            | SlmpDeviceCode::L
            | SlmpDeviceCode::F
            | SlmpDeviceCode::V
            | SlmpDeviceCode::B
            | SlmpDeviceCode::SB
    )
}

async fn read_random_dword_scalar(
    client: &SlmpClient,
    device: SlmpDeviceAddress,
) -> Result<u32, SlmpError> {
    let result = client.read_random(&[], &[device]).await?;
    result
        .dword_values
        .first()
        .copied()
        .ok_or_else(|| SlmpError::new("Read Random dword response did not contain a value."))
}

async fn read_long_like_point(
    client: &SlmpClient,
    base_code: SlmpDeviceCode,
    number: u32,
) -> Result<SlmpLongTimerResult, SlmpError> {
    match base_code {
        SlmpDeviceCode::LTN => Ok(client.read_long_timer(number, 1).await?.remove(0)),
        SlmpDeviceCode::LSTN => Ok(client.read_long_retentive_timer(number, 1).await?.remove(0)),
        SlmpDeviceCode::LCN => Err(SlmpError::new(
            "LCN current values use random dword read; LCS/LCC state reads use direct bit read.",
        )),
        _ => Err(SlmpError::new("Unsupported long-family base code.")),
    }
}

fn decode_long_like_value(
    dtype: &str,
    spec: &LongTimerReadSpec,
    timer: &SlmpLongTimerResult,
) -> Result<SlmpValue, SlmpError> {
    Ok(match spec.kind {
        LongTimerReadKind::Current => {
            if dtype.eq_ignore_ascii_case("L") {
                SlmpValue::I32(timer.current_value as i32)
            } else {
                SlmpValue::U32(timer.current_value)
            }
        }
        LongTimerReadKind::Contact => SlmpValue::Bool(timer.contact),
        LongTimerReadKind::Coil => SlmpValue::Bool(timer.coil),
    })
}

#[cfg(test)]
mod optimization_tests {
    use super::*;
    use crate::SlmpProfileLimitKey;
    use crate::model::{
        SlmpConnectionOptions, SlmpFrameType, SlmpTargetAddress, SlmpTransportMode,
    };
    use futures_util::StreamExt;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn serve_bit_read_responses(listener: TcpListener, expected_points: &[usize]) {
        let (mut stream, _) = listener.accept().await.unwrap();
        for &points in expected_points {
            let mut header = [0u8; 9];
            stream.read_exact(&mut header).await.unwrap();
            let body_length = u16::from_le_bytes([header[7], header[8]]) as usize;
            let mut body = vec![0; body_length];
            stream.read_exact(&mut body).await.unwrap();

            assert!(body.len() >= 8);
            assert_eq!(&body[2..4], &0x0401u16.to_le_bytes());
            assert_eq!(
                u16::from_le_bytes([body[body.len() - 2], body[body.len() - 1]]),
                u16::try_from(points).unwrap()
            );

            let data = vec![0; points.div_ceil(2)];
            let response_length = u16::try_from(data.len() + 2).unwrap().to_le_bytes();
            let mut response = vec![
                0xD0,
                0x00,
                header[2],
                header[3],
                header[4],
                header[5],
                header[6],
                response_length[0],
                response_length[1],
                0x00,
                0x00,
            ];
            response.extend_from_slice(&data);
            stream.write_all(&response).await.unwrap();
        }
    }

    #[tokio::test]
    async fn read_bits_single_request_enforces_every_profile_boundary() {
        for &profile in SlmpPlcProfile::available_connection_profiles() {
            let maximum = profile
                .profile_limit(SlmpProfileLimitKey::DirectBitRead)
                .unwrap()
                .max_points;
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let port = listener.local_addr().unwrap().port();
            let server = tokio::spawn(async move {
                serve_bit_read_responses(listener, &[1, maximum]).await;
            });
            let mut options = SlmpConnectionOptions::new(
                "127.0.0.1",
                port,
                SlmpTransportMode::Tcp,
                SlmpTargetAddress::default(),
                profile,
            )
            .unwrap();
            options.frame_type = SlmpFrameType::Frame3E;
            let client = SlmpClient::connect(options).await.unwrap();
            let bit = SlmpDeviceAddress::new(SlmpDeviceCode::M, 0, profile);

            let zero_error = read_bits_single_request(&client, bit, 0).await.unwrap_err();
            assert!(zero_error.to_string().contains("range 1-65535"));
            assert_eq!(client.traffic_stats().await.request_count, 0);

            assert_eq!(
                read_bits_single_request(&client, bit, 1).await.unwrap(),
                [false]
            );
            let maximum_values = read_bits_single_request(&client, bit, maximum)
                .await
                .unwrap();
            assert_eq!(maximum_values.len(), maximum);
            assert!(maximum_values.iter().all(|value| !value));

            let profile_error = read_bits_single_request(&client, bit, maximum + 1)
                .await
                .unwrap_err();
            assert!(profile_error.to_string().contains(&format!(
                "read_bits bit access points out of range (1..{maximum}): {}",
                maximum + 1
            )));

            let u16_error = read_bits_single_request(&client, bit, usize::from(u16::MAX) + 1)
                .await
                .unwrap_err();
            assert!(u16_error.to_string().contains("range 1-65535"));
            assert_eq!(client.traffic_stats().await.request_count, 2);
            server.await.unwrap();
        }
    }

    #[tokio::test]
    async fn canonical_contiguous_helpers_send_one_request_each_and_reject_before_send() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            for data in [vec![0x34, 0x12], Vec::new(), vec![0x10], Vec::new()] {
                let mut header = [0u8; 9];
                stream.read_exact(&mut header).await.unwrap();
                let body_length = u16::from_le_bytes([header[7], header[8]]) as usize;
                let mut body = vec![0; body_length];
                stream.read_exact(&mut body).await.unwrap();
                let response_length = u16::try_from(data.len() + 2).unwrap().to_le_bytes();
                let mut response = vec![
                    0xD0,
                    0x00,
                    header[2],
                    header[3],
                    header[4],
                    header[5],
                    header[6],
                    response_length[0],
                    response_length[1],
                    0x00,
                    0x00,
                ];
                response.extend_from_slice(&data);
                stream.write_all(&response).await.unwrap();
            }
        });
        let mut options = SlmpConnectionOptions::new(
            "127.0.0.1",
            port,
            SlmpTransportMode::Tcp,
            SlmpTargetAddress::default(),
            SlmpPlcProfile::IqR,
        )
        .unwrap();
        options.frame_type = SlmpFrameType::Frame3E;
        let client = SlmpClient::connect(options).await.unwrap();
        let word = SlmpDeviceAddress::new(SlmpDeviceCode::D, 0, SlmpPlcProfile::IqR);
        let bit = SlmpDeviceAddress::new(SlmpDeviceCode::M, 0, SlmpPlcProfile::IqR);

        assert_eq!(
            read_words_single_request(&client, word, 1).await.unwrap(),
            [0x1234]
        );
        assert_eq!(client.traffic_stats().await.request_count, 1);
        write_words_single_request(&client, word, &[1])
            .await
            .unwrap();
        assert_eq!(client.traffic_stats().await.request_count, 2);
        assert_eq!(
            read_bits_single_request(&client, bit, 2).await.unwrap(),
            [true, false]
        );
        assert_eq!(client.traffic_stats().await.request_count, 3);
        write_bits_single_request(&client, bit, &[true, false])
            .await
            .unwrap();
        assert_eq!(client.traffic_stats().await.request_count, 4);

        assert!(read_bits_single_request(&client, word, 1).await.is_err());
        assert!(
            write_bits_single_request(&client, bit, &vec![false; 7169])
                .await
                .is_err()
        );
        assert_eq!(client.traffic_stats().await.request_count, 4);
        server.await.unwrap();
    }

    #[tokio::test]
    async fn poll_prepares_random_payload_once_and_decodes_by_compact_index() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            for value in [0x1234u16, 0x5678u16] {
                let mut header = [0u8; 9];
                stream.read_exact(&mut header).await.unwrap();
                let body_length = u16::from_le_bytes([header[7], header[8]]) as usize;
                let mut body = vec![0; body_length];
                stream.read_exact(&mut body).await.unwrap();
                let mut response = vec![
                    0xD0, 0x00, header[2], header[3], header[4], header[5], header[6], 0x04, 0x00,
                    0x00, 0x00,
                ];
                response.extend_from_slice(&value.to_le_bytes());
                stream.write_all(&response).await.unwrap();
            }
        });
        let mut options = SlmpConnectionOptions::new(
            "127.0.0.1",
            port,
            SlmpTransportMode::Tcp,
            SlmpTargetAddress::default(),
            SlmpPlcProfile::IqR,
        )
        .unwrap();
        options.frame_type = SlmpFrameType::Frame3E;
        let client = SlmpClient::connect(options).await.unwrap();
        let addresses = vec!["D100:U".to_string()];
        let mut stream = Box::pin(poll_named(&client, &addresses, Duration::ZERO));

        assert_eq!(
            stream.next().await.unwrap().unwrap()["D100:U"],
            SlmpValue::U16(0x1234)
        );
        assert_eq!(
            stream.next().await.unwrap().unwrap()["D100:U"],
            SlmpValue::U16(0x5678)
        );
        assert_eq!(client.optimization_test_counters().await.0, 1);

        server.await.unwrap();
    }
}

fn validate_bit_in_word_target(address: &str, device: SlmpDeviceAddress) -> Result<(), SlmpError> {
    if !device.code().is_word_device() {
        return Err(SlmpError::new(format!(
            "Address '{address}' uses '.bit' notation, which is only valid for word devices."
        )));
    }
    Ok(())
}

fn require_bit_in_word_index(address: &str, bit_index: Option<u8>) -> Result<u8, SlmpError> {
    bit_index.ok_or_else(|| missing_bit_in_word_index_error(address))
}

fn missing_bit_in_word_index_error(address: &str) -> SlmpError {
    SlmpError::new(format!(
        "Address '{address}' uses BIT_IN_WORD but no bit index was specified. Use '.0' through '.F' notation."
    ))
}

fn require_dtype(dtype: &str) -> Result<String, SlmpError> {
    let normalized = dtype.trim().to_uppercase();
    if normalized.is_empty() {
        return Err(SlmpError::new(
            "dtype is required; specify BIT/U/S/D/L/F explicitly.",
        ));
    }
    if !matches!(normalized.as_str(), "BIT" | "U" | "S" | "D" | "L" | "F") {
        return Err(SlmpError::new(format!(
            "Unsupported dtype '{normalized}'; expected BIT/U/S/D/L/F."
        )));
    }
    Ok(normalized)
}

fn validate_named_device_dtype(
    address: &str,
    device: SlmpDeviceAddress,
    dtype: &str,
) -> Result<(), SlmpError> {
    if device.code().is_bit_device() && dtype != "BIT" {
        return Err(SlmpError::new(format!(
            "Address '{address}' is a bit device and requires ':BIT'."
        )));
    }
    if !device.code().is_bit_device() && dtype == "BIT" {
        return Err(SlmpError::new(format!(
            "Address '{address}' uses ':BIT', which is only valid for bit devices. Use '.bit' notation for a bit inside a word device."
        )));
    }
    Ok(())
}

fn validate_typed_device_dtype(device: SlmpDeviceAddress, dtype: &str) -> Result<(), SlmpError> {
    if device.code().is_bit_device() && dtype != "BIT" {
        return Err(SlmpError::new(format!(
            "{} is a bit device and requires dtype 'BIT'; use an explicit word API for packed bit-device access.",
            device.code()
        )));
    }
    if !device.code().is_bit_device() && dtype == "BIT" {
        return Err(SlmpError::new(format!(
            "{} is a word device and cannot use dtype 'BIT'; use explicit bit-in-word access for a bit inside a word device.",
            device.code()
        )));
    }
    Ok(())
}

fn resolve_dtype_for_address(
    address: &str,
    device: SlmpDeviceAddress,
    dtype: &str,
    bit_index: Option<u8>,
) -> Result<String, SlmpError> {
    if bit_index.is_some() {
        return Ok("BIT_IN_WORD".to_string());
    }
    let _ = address;
    let _ = device;
    require_dtype(dtype)
}

fn resolve_write_route(device: SlmpDeviceAddress, dtype: &str) -> NamedWriteRoute {
    let normalized = dtype.to_uppercase();
    match normalized.as_str() {
        // Long-family state writes must use Device Write Random (0x1402).
        // Direct bit write (0x1401) is guarded in the low-level client.
        "BIT"
            if matches!(
                device.code(),
                SlmpDeviceCode::LTS
                    | SlmpDeviceCode::LTC
                    | SlmpDeviceCode::LSTS
                    | SlmpDeviceCode::LSTC
                    | SlmpDeviceCode::LCS
                    | SlmpDeviceCode::LCC
            ) =>
        {
            NamedWriteRoute::RandomBits
        }
        "BIT" => NamedWriteRoute::ContiguousBits,
        "D" | "L"
            if matches!(
                device.code(),
                SlmpDeviceCode::LTN
                    | SlmpDeviceCode::LSTN
                    | SlmpDeviceCode::LCN
                    | SlmpDeviceCode::LZ
            ) =>
        {
            NamedWriteRoute::RandomDWords
        }
        "D" | "L" | "F" => NamedWriteRoute::ContiguousDWords,
        _ => NamedWriteRoute::ContiguousWords,
    }
}

fn is_long_counter_state_device(code: SlmpDeviceCode) -> bool {
    matches!(code, SlmpDeviceCode::LCS | SlmpDeviceCode::LCC)
}

fn long_timer_read_spec(code: SlmpDeviceCode) -> Option<LongTimerReadSpec> {
    let (base_code, kind) = match code {
        SlmpDeviceCode::LTN => (SlmpDeviceCode::LTN, LongTimerReadKind::Current),
        SlmpDeviceCode::LTS => (SlmpDeviceCode::LTN, LongTimerReadKind::Contact),
        SlmpDeviceCode::LTC => (SlmpDeviceCode::LTN, LongTimerReadKind::Coil),
        SlmpDeviceCode::LSTN => (SlmpDeviceCode::LSTN, LongTimerReadKind::Current),
        SlmpDeviceCode::LSTS => (SlmpDeviceCode::LSTN, LongTimerReadKind::Contact),
        SlmpDeviceCode::LSTC => (SlmpDeviceCode::LSTN, LongTimerReadKind::Coil),
        SlmpDeviceCode::LCN => (SlmpDeviceCode::LCN, LongTimerReadKind::Current),
        SlmpDeviceCode::LCS => (SlmpDeviceCode::LCS, LongTimerReadKind::Contact),
        SlmpDeviceCode::LCC => (SlmpDeviceCode::LCC, LongTimerReadKind::Coil),
        _ => return None,
    };
    Some(LongTimerReadSpec { base_code, kind })
}

fn validate_long_timer_entry(
    address: &str,
    device: SlmpDeviceAddress,
    dtype: &str,
) -> Result<(), SlmpError> {
    let Some(spec) = long_timer_read_spec(device.code()) else {
        return Ok(());
    };
    if matches!(spec.kind, LongTimerReadKind::Current) {
        if dtype != "D" && dtype != "L" {
            return Err(SlmpError::new(format!(
                "Address '{address}' uses a 32-bit long current value. Use the plain form or ':D' / ':L'."
            )));
        }
        return Ok(());
    }
    if !dtype.eq_ignore_ascii_case("BIT") {
        return Err(SlmpError::new(format!(
            "Address '{address}' is a long timer state device. Use the plain device form without a dtype override."
        )));
    }
    Ok(())
}

fn validate_dword_only_entry(
    address: &str,
    device: SlmpDeviceAddress,
    dtype: &str,
) -> Result<(), SlmpError> {
    if !matches!(device.code(), SlmpDeviceCode::LZ) {
        return Ok(());
    }
    if dtype != "D" && dtype != "L" {
        return Err(SlmpError::new(format!(
            "Address '{address}' is a 32-bit device. Use ':D' or ':L'."
        )));
    }
    Ok(())
}

fn scalar_to_bool(value: &SlmpValue) -> Result<bool, SlmpError> {
    match value {
        SlmpValue::Bool(v) => Ok(*v),
        _ => Err(SlmpError::new("BIT value must be SlmpValue::Bool.")),
    }
}

fn scalar_to_u16(value: &SlmpValue) -> Result<u16, SlmpError> {
    match value {
        SlmpValue::U16(v) => Ok(*v),
        _ => Err(SlmpError::new("U value must be SlmpValue::U16.")),
    }
}

fn scalar_to_i16(value: &SlmpValue) -> Result<i16, SlmpError> {
    match value {
        SlmpValue::I16(v) => Ok(*v),
        _ => Err(SlmpError::new("S value must be SlmpValue::I16.")),
    }
}

fn scalar_to_u32(value: &SlmpValue) -> Result<u32, SlmpError> {
    match value {
        SlmpValue::U32(v) => Ok(*v),
        _ => Err(SlmpError::new("D value must be SlmpValue::U32.")),
    }
}

fn scalar_to_i32(value: &SlmpValue) -> Result<i32, SlmpError> {
    match value {
        SlmpValue::I32(v) => Ok(*v),
        _ => Err(SlmpError::new("L value must be SlmpValue::I32.")),
    }
}

fn scalar_to_f32(value: &SlmpValue) -> Result<f32, SlmpError> {
    match value {
        SlmpValue::F32(v) if v.is_finite() => Ok(*v),
        SlmpValue::F32(_) => Err(SlmpError::new("F value must be finite.")),
        _ => Err(SlmpError::new("F value must be SlmpValue::F32.")),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NamedWriteRoute {
    ContiguousBits,
    ContiguousWords,
    ContiguousDWords,
    RandomBits,
    RandomDWords,
}

pub fn parse_scalar_for_named(
    address: &str,
    value: &str,
    plc_profile: SlmpPlcProfile,
) -> Result<SlmpValue, SlmpError> {
    let parts = parse_named_address(address)?;
    let device = parse_device(&parts.base, plc_profile)?;
    if parts.dtype == "BIT_IN_WORD" {
        require_bit_in_word_index(address, parts.bit_index)?;
        return parse_bool_scalar(value);
    }
    let resolved_dtype = resolve_dtype_for_address(address, device, &parts.dtype, parts.bit_index)?;
    validate_named_device_dtype(address, device, &resolved_dtype)?;
    validate_long_timer_entry(address, device, &resolved_dtype)?;
    validate_dword_only_entry(address, device, &resolved_dtype)?;
    if resolved_dtype == "BIT" {
        return parse_bool_scalar(value);
    }
    if resolved_dtype == "F" {
        let parsed = value
            .parse::<f32>()
            .map_err(|_| SlmpError::new("Invalid float value."));
        return match parsed {
            Ok(number) if number.is_finite() => Ok(SlmpValue::F32(number)),
            Ok(_) => Err(SlmpError::new("Float value must be finite.")),
            Err(error) => Err(error),
        };
    }
    let parsed = if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        i64::from_str_radix(hex, 16).map_err(|_| SlmpError::new("Invalid integer value."))?
    } else {
        value
            .parse::<i64>()
            .map_err(|_| SlmpError::new("Invalid integer value."))?
    };
    match resolved_dtype.as_str() {
        "L" => i32::try_from(parsed)
            .map(SlmpValue::I32)
            .map_err(|_| SlmpError::new("L value must be in range -2147483648..=2147483647.")),
        "D" => u32::try_from(parsed)
            .map(SlmpValue::U32)
            .map_err(|_| SlmpError::new("D value must be in range 0..=4294967295.")),
        "S" => i16::try_from(parsed)
            .map(SlmpValue::I16)
            .map_err(|_| SlmpError::new("S value must be in range -32768..=32767.")),
        _ => u16::try_from(parsed)
            .map(SlmpValue::U16)
            .map_err(|_| SlmpError::new("U value must be in range 0..=65535.")),
    }
}

fn parse_bool_scalar(value: &str) -> Result<SlmpValue, SlmpError> {
    match value {
        "1" | "true" | "TRUE" | "True" => Ok(SlmpValue::Bool(true)),
        "0" | "false" | "FALSE" | "False" => Ok(SlmpValue::Bool(false)),
        _ => Err(SlmpError::new(
            "Boolean value must be 0, 1, false, or true.",
        )),
    }
}
