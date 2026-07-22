use crate::address::{parse_device, parse_named_address};
use crate::client::SlmpClient;
use crate::client_rules::checked_span_end;
use crate::error::SlmpError;
use crate::model::{SlmpDeviceAddress, SlmpDeviceCode, SlmpLongTimerResult, SlmpPlcProfile};
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
    long_timer_read: Option<LongTimerReadSpec>,
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

pub async fn write_bit_in_word(
    client: &SlmpClient,
    device: SlmpDeviceAddress,
    bit_index: u8,
    value: bool,
) -> Result<(), SlmpError> {
    if bit_index > 15 {
        return Err(SlmpError::new("bit_index must be 0-15."));
    }
    let mut current = client.read_words_raw(device, 1).await?[0];
    if value {
        current |= 1 << bit_index;
    } else {
        current &= !(1 << bit_index);
    }
    client.write_words(device, &[current]).await
}

pub async fn read_words_single_request(
    client: &SlmpClient,
    start: SlmpDeviceAddress,
    count: usize,
) -> Result<Vec<u16>, SlmpError> {
    validate_single_request_count(count, 960)?;
    client.read_words_raw(start, count as u16).await
}

pub async fn read_dwords_single_request(
    client: &SlmpClient,
    start: SlmpDeviceAddress,
    count: usize,
) -> Result<Vec<u32>, SlmpError> {
    if matches!(start.code(), SlmpDeviceCode::LZ) {
        validate_single_request_count(count, RANDOM_READ_BATCH_LIMIT)?;
        let end = checked_span_end(start.number(), count, "read_dwords_single_request")?;
        let devices = (start.number()..=end)
            .map(|number| SlmpDeviceAddress::new(start.code(), number, start.plc_profile()))
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
    read_named_compiled(client, &plan).await
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
        loop {
            yield read_named_compiled(client, &plan).await?;
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
            let mut bit_word_read = None;
            if dtype == "BIT" {
                bit_word_read = plain_bit_word_read(device);
                if let Some(read) = bit_word_read
                    && seen_word_devices.insert(read.device)
                {
                    word_devices.push(read.device);
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
            long_timer_read: long_timer_read_spec(device.code()),
        });
    }

    let unsupported = entries
        .iter()
        .filter(|entry| {
            let supported = if let Some(spec) = &entry.long_timer_read {
                matches!(entry.device.code(), SlmpDeviceCode::LCN)
                    && matches!(spec.kind, LongTimerReadKind::Current)
                    && dword_devices.contains(&entry.device)
            } else if let Some(read) = entry.bit_word_read {
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

    Ok(NamedReadPlan {
        entries,
        word_devices,
        dword_devices,
    })
}

async fn read_named_compiled(
    client: &SlmpClient,
    plan: &NamedReadPlan,
) -> Result<NamedAddress, SlmpError> {
    let mut result = NamedAddress::new();
    let (word_values, dword_values) =
        read_random_maps(client, &plan.word_devices, &plan.dword_devices).await?;
    let mut long_timer_cache: HashMap<(SlmpDeviceCode, u32), SlmpLongTimerResult> =
        HashMap::with_capacity(plan.entries.len());

    for entry in &plan.entries {
        let value = if let Some(spec) = &entry.long_timer_read {
            if matches!(spec.base_code, SlmpDeviceCode::LCN)
                && matches!(spec.kind, LongTimerReadKind::Current)
            {
                let raw = if let Some(value) = dword_values.get(&entry.device) {
                    *value
                } else {
                    read_random_dword_scalar(client, entry.device).await?
                };
                if entry.dtype == "L" {
                    SlmpValue::I32(raw as i32)
                } else {
                    SlmpValue::U32(raw)
                }
            } else if is_long_counter_state_device(entry.device.code()) {
                SlmpValue::Bool(client.read_bits(entry.device, 1).await?[0])
            } else {
                let key = (spec.base_code, entry.device.number());
                if let std::collections::hash_map::Entry::Vacant(vacant) =
                    long_timer_cache.entry(key)
                {
                    let timer =
                        read_long_like_point(client, spec.base_code, entry.device.number()).await?;
                    vacant.insert(timer);
                }
                decode_long_like_value(&entry.dtype, spec, long_timer_cache.get(&key).unwrap())?
            }
        } else if entry.dtype == "BIT_IN_WORD" {
            let read = entry
                .bit_word_read
                .ok_or_else(|| missing_bit_in_word_index_error(&entry.address))?;
            let word = if let Some(word) = word_values.get(&read.device) {
                *word
            } else {
                client.read_words_raw(read.device, 1).await?[0]
            };
            SlmpValue::Bool(((word >> read.bit_index) & 1) != 0)
        } else if entry.dtype == "BIT" {
            if let Some(read) = entry.bit_word_read {
                let word = if let Some(word) = word_values.get(&read.device) {
                    *word
                } else {
                    client.read_words_raw(read.device, 1).await?[0]
                };
                SlmpValue::Bool(((word >> read.bit_index) & 1) != 0)
            } else {
                read_typed(client, entry.device, &entry.dtype).await?
            }
        } else if entry.dtype == "S" {
            if let Some(value) = word_values.get(&entry.device) {
                SlmpValue::I16(*value as i16)
            } else {
                read_typed(client, entry.device, &entry.dtype).await?
            }
        } else if entry.dtype == "U" {
            if let Some(value) = word_values.get(&entry.device) {
                SlmpValue::U16(*value)
            } else {
                read_typed(client, entry.device, &entry.dtype).await?
            }
        } else if entry.dtype == "F" {
            if let Some(value) = dword_values.get(&entry.device) {
                SlmpValue::F32(f32::from_bits(*value))
            } else {
                read_typed(client, entry.device, &entry.dtype).await?
            }
        } else if entry.dtype == "L" {
            if let Some(value) = dword_values.get(&entry.device) {
                SlmpValue::I32(*value as i32)
            } else {
                read_typed(client, entry.device, &entry.dtype).await?
            }
        } else if entry.dtype == "D" {
            if let Some(value) = dword_values.get(&entry.device) {
                SlmpValue::U32(*value)
            } else {
                read_typed(client, entry.device, &entry.dtype).await?
            }
        } else {
            read_typed(client, entry.device, &entry.dtype).await?
        };
        result.insert(entry.address.clone(), value);
    }

    Ok(result)
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

async fn read_random_maps(
    client: &SlmpClient,
    word_devices: &[SlmpDeviceAddress],
    dword_devices: &[SlmpDeviceAddress],
) -> Result<
    (
        HashMap<SlmpDeviceAddress, u16>,
        HashMap<SlmpDeviceAddress, u32>,
    ),
    SlmpError,
> {
    let mut words = HashMap::with_capacity(word_devices.len());
    let mut dwords = HashMap::with_capacity(dword_devices.len());
    if word_devices.is_empty() && dword_devices.is_empty() {
        return Ok((words, dwords));
    }
    let random = client.read_random(word_devices, dword_devices).await?;
    for (device, value) in word_devices.iter().copied().zip(random.word_values) {
        words.insert(device, value);
    }
    for (device, value) in dword_devices.iter().copied().zip(random.dword_values) {
        dwords.insert(device, value);
    }

    Ok((words, dwords))
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
