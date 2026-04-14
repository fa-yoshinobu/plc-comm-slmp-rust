use crate::address::{parse_device_for_family_hint, parse_named_address};
use crate::client::SlmpClient;
use crate::error::SlmpError;
use crate::model::{SlmpDeviceAddress, SlmpDeviceCode, SlmpLongTimerResult, SlmpPlcFamily};
use async_stream::try_stream;
use futures_core::stream::Stream;
use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

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
    bit_index: Option<u8>,
    long_timer_read: Option<LongTimerReadSpec>,
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
    let normalized_dtype = dtype.to_uppercase();
    if let Some(spec) = long_timer_read_spec(device.code) {
        let timer = read_long_like_point(client, spec.base_code, device.number).await?;
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
        _ => Ok(SlmpValue::U16(client.read_words_raw(device, 1).await?[0])),
    }
}

pub async fn write_typed(
    client: &SlmpClient,
    device: SlmpDeviceAddress,
    dtype: &str,
    value: &SlmpValue,
) -> Result<(), SlmpError> {
    match resolve_write_route(device, dtype) {
        NamedWriteRoute::RandomBits => {
            client
                .write_random_bits(&[(device, scalar_to_bool(value)?)])
                .await
        }
        NamedWriteRoute::ContiguousBits => {
            client.write_bits(device, &[scalar_to_bool(value)?]).await
        }
        NamedWriteRoute::RandomDWords | NamedWriteRoute::ContiguousDWords => {
            let raw = match dtype.to_uppercase().as_str() {
                "F" => scalar_to_f32(value)?.to_bits(),
                "L" => scalar_to_i32(value)? as u32,
                _ => scalar_to_u32(value)?,
            };
            if matches!(
                resolve_write_route(device, dtype),
                NamedWriteRoute::RandomDWords
            ) {
                client.write_random_words(&[], &[(device, raw)]).await
            } else {
                client.write_dwords(device, &[raw]).await
            }
        }
        NamedWriteRoute::ContiguousWords => {
            client.write_words(device, &[scalar_to_u16(value)?]).await
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

pub async fn read_words_chunked(
    client: &SlmpClient,
    start: SlmpDeviceAddress,
    count: usize,
    max_words_per_request: usize,
) -> Result<Vec<u16>, SlmpError> {
    let chunk = (max_words_per_request / 2) * 2;
    if chunk == 0 {
        return Err(SlmpError::new("max_words_per_request must be at least 2."));
    }
    let mut remaining = count;
    let mut offset = 0u32;
    let mut result = Vec::with_capacity(count);
    while remaining > 0 {
        let next = remaining.min(chunk);
        result.extend(
            client
                .read_words_raw(
                    SlmpDeviceAddress::new(start.code, start.number + offset),
                    next as u16,
                )
                .await?,
        );
        remaining -= next;
        offset += next as u32;
    }
    Ok(result)
}

pub async fn read_dwords_chunked(
    client: &SlmpClient,
    start: SlmpDeviceAddress,
    count: usize,
    max_dwords_per_request: usize,
) -> Result<Vec<u32>, SlmpError> {
    let mut remaining = count;
    let mut offset = 0u32;
    let mut result = Vec::with_capacity(count);
    while remaining > 0 {
        let next = remaining.min(max_dwords_per_request);
        result.extend(
            client
                .read_dwords_raw(
                    SlmpDeviceAddress::new(start.code, start.number + offset * 2),
                    next as u16,
                )
                .await?,
        );
        remaining -= next;
        offset += next as u32;
    }
    Ok(result)
}

pub async fn write_words_chunked(
    client: &SlmpClient,
    start: SlmpDeviceAddress,
    values: &[u16],
    max_words_per_request: usize,
) -> Result<(), SlmpError> {
    if max_words_per_request == 0 {
        return Err(SlmpError::new("chunk size must be positive."));
    }
    let mut offset = 0usize;
    while offset < values.len() {
        let end = (offset + max_words_per_request).min(values.len());
        client
            .write_words(
                SlmpDeviceAddress::new(start.code, start.number + offset as u32),
                &values[offset..end],
            )
            .await?;
        offset = end;
    }
    Ok(())
}

pub async fn write_dwords_chunked(
    client: &SlmpClient,
    start: SlmpDeviceAddress,
    values: &[u32],
    max_dwords_per_request: usize,
) -> Result<(), SlmpError> {
    if max_dwords_per_request == 0 {
        return Err(SlmpError::new("chunk size must be positive."));
    }
    let mut offset = 0usize;
    while offset < values.len() {
        let end = (offset + max_dwords_per_request).min(values.len());
        client
            .write_dwords(
                SlmpDeviceAddress::new(start.code, start.number + (offset * 2) as u32),
                &values[offset..end],
            )
            .await?;
        offset = end;
    }
    Ok(())
}

pub async fn read_named(
    client: &SlmpClient,
    addresses: &[String],
) -> Result<NamedAddress, SlmpError> {
    let plan = compile_read_plan(addresses, client.plc_family().await)?;
    read_named_compiled(client, &plan).await
}

pub async fn write_named(client: &SlmpClient, updates: &NamedAddress) -> Result<(), SlmpError> {
    let plc_family = client.plc_family().await;
    for (address, value) in updates {
        let parts = parse_named_address(address)?;
        let device = parse_device_for_family_hint(&parts.base, Some(plc_family))?;
        let resolved_dtype =
            resolve_dtype_for_address(address, device, &parts.dtype, parts.bit_index);
        validate_long_timer_entry(address, device, &resolved_dtype)?;
        if parts.dtype == "BIT_IN_WORD" {
            validate_bit_in_word_target(address, device)?;
            write_bit_in_word(
                client,
                device,
                parts.bit_index.unwrap_or(0),
                scalar_to_bool(value)?,
            )
            .await?;
            continue;
        }
        write_typed(client, device, &resolved_dtype, value).await?;
    }
    Ok(())
}

pub fn poll_named<'a>(
    client: &'a SlmpClient,
    addresses: &'a [String],
    interval: Duration,
) -> impl Stream<Item = Result<NamedAddress, SlmpError>> + 'a {
    try_stream! {
        loop {
            yield read_named(client, addresses).await?;
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
    plc_family: SlmpPlcFamily,
) -> Result<NamedReadPlan, SlmpError> {
    let mut entries = Vec::new();
    let mut word_devices = Vec::new();
    let mut dword_devices = Vec::new();
    for address in addresses {
        let parts = parse_named_address(address)?;
        let device = parse_device_for_family_hint(&parts.base, Some(plc_family))?;
        let dtype = resolve_dtype_for_address(address, device, &parts.dtype, parts.bit_index);
        validate_long_timer_entry(address, device, &dtype)?;

        if parts.dtype == "BIT_IN_WORD" {
            validate_bit_in_word_target(address, device)?;
            if device.code.is_word_batchable() && !word_devices.contains(&device) {
                word_devices.push(device);
            }
        } else if matches!(dtype.as_str(), "U" | "S") && device.code.is_word_batchable() {
            if !word_devices.contains(&device) {
                word_devices.push(device);
            }
        } else if matches!(dtype.as_str(), "D" | "L" | "F") && device.code.is_word_batchable() {
            if !dword_devices.contains(&device) {
                dword_devices.push(device);
            }
        }

        entries.push(NamedReadEntry {
            address: address.clone(),
            device,
            dtype,
            bit_index: parts.bit_index,
            long_timer_read: long_timer_read_spec(device.code),
        });
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
    let mut long_timer_cache: HashMap<(SlmpDeviceCode, u32), SlmpLongTimerResult> = HashMap::new();

    for entry in &plan.entries {
        let value = if let Some(spec) = &entry.long_timer_read {
            let key = (spec.base_code, entry.device.number);
            if !long_timer_cache.contains_key(&key) {
                let timer =
                    read_long_like_point(client, spec.base_code, entry.device.number).await?;
                long_timer_cache.insert(key, timer);
            }
            decode_long_like_value(&entry.dtype, spec, long_timer_cache.get(&key).unwrap())?
        } else if entry.dtype == "BIT_IN_WORD" {
            let word = if let Some(word) = word_values.get(&entry.device) {
                *word
            } else {
                client.read_words_raw(entry.device, 1).await?[0]
            };
            SlmpValue::Bool(((word >> entry.bit_index.unwrap_or(0)) & 1) != 0)
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
    let mut words = HashMap::new();
    let mut dwords = HashMap::new();
    let mut word_index = 0usize;
    let mut dword_index = 0usize;

    while word_index < word_devices.len() || dword_index < dword_devices.len() {
        let word_end = (word_index + 0xFF).min(word_devices.len());
        let dword_end = (dword_index + 0xFF).min(dword_devices.len());
        let random = client
            .read_random(
                &word_devices[word_index..word_end],
                &dword_devices[dword_index..dword_end],
            )
            .await?;
        for (device, value) in word_devices[word_index..word_end]
            .iter()
            .copied()
            .zip(random.word_values.into_iter())
        {
            words.insert(device, value);
        }
        for (device, value) in dword_devices[dword_index..dword_end]
            .iter()
            .copied()
            .zip(random.dword_values.into_iter())
        {
            dwords.insert(device, value);
        }
        word_index = word_end;
        dword_index = dword_end;
    }

    Ok((words, dwords))
}

async fn read_long_like_point(
    client: &SlmpClient,
    base_code: SlmpDeviceCode,
    number: u32,
) -> Result<SlmpLongTimerResult, SlmpError> {
    match base_code {
        SlmpDeviceCode::LTN => Ok(client.read_long_timer(number, 1).await?.remove(0)),
        SlmpDeviceCode::LSTN => Ok(client.read_long_retentive_timer(number, 1).await?.remove(0)),
        SlmpDeviceCode::LCN => {
            let raw_words = client
                .read_words_raw(SlmpDeviceAddress::new(SlmpDeviceCode::LCN, number), 4)
                .await?;
            Ok(SlmpLongTimerResult {
                index: number,
                device: format!("LCN{number}"),
                current_value: raw_words[0] as u32 | ((raw_words[1] as u32) << 16),
                contact: (raw_words[2] & 0x0002) != 0,
                coil: (raw_words[2] & 0x0001) != 0,
                status_word: raw_words[2],
                raw_words,
            })
        }
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
    if !device.code.is_word_device() {
        return Err(SlmpError::new(format!(
            "Address '{address}' uses '.bit' notation, which is only valid for word devices."
        )));
    }
    Ok(())
}

fn resolve_dtype_for_address(
    address: &str,
    device: SlmpDeviceAddress,
    dtype: &str,
    bit_index: Option<u8>,
) -> String {
    let normalized = if dtype == "U" && device.code.is_bit_device() {
        "BIT".to_string()
    } else {
        dtype.to_uppercase()
    };
    if !address.contains(':')
        && bit_index.is_none()
        && matches!(
            device.code,
            SlmpDeviceCode::LTN | SlmpDeviceCode::LSTN | SlmpDeviceCode::LCN | SlmpDeviceCode::LZ
        )
    {
        "D".to_string()
    } else {
        normalized
    }
}

fn resolve_write_route(device: SlmpDeviceAddress, dtype: &str) -> NamedWriteRoute {
    let normalized = if dtype.eq_ignore_ascii_case("U") && device.code.is_bit_device() {
        "BIT".to_string()
    } else {
        dtype.to_uppercase()
    };
    match normalized.as_str() {
        "BIT"
            if matches!(
                device.code,
                SlmpDeviceCode::LTS
                    | SlmpDeviceCode::LTC
                    | SlmpDeviceCode::LSTS
                    | SlmpDeviceCode::LSTC
            ) =>
        {
            NamedWriteRoute::RandomBits
        }
        "BIT" => NamedWriteRoute::ContiguousBits,
        "D" | "L"
            if matches!(
                device.code,
                SlmpDeviceCode::LTN | SlmpDeviceCode::LSTN | SlmpDeviceCode::LZ
            ) =>
        {
            NamedWriteRoute::RandomDWords
        }
        "D" | "L" | "F" => NamedWriteRoute::ContiguousDWords,
        _ => NamedWriteRoute::ContiguousWords,
    }
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
        SlmpDeviceCode::LCS => (SlmpDeviceCode::LCN, LongTimerReadKind::Contact),
        SlmpDeviceCode::LCC => (SlmpDeviceCode::LCN, LongTimerReadKind::Coil),
        _ => return None,
    };
    Some(LongTimerReadSpec { base_code, kind })
}

fn validate_long_timer_entry(
    address: &str,
    device: SlmpDeviceAddress,
    dtype: &str,
) -> Result<(), SlmpError> {
    let Some(spec) = long_timer_read_spec(device.code) else {
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

fn scalar_to_bool(value: &SlmpValue) -> Result<bool, SlmpError> {
    match value {
        SlmpValue::Bool(v) => Ok(*v),
        SlmpValue::U16(v) => Ok(*v != 0),
        SlmpValue::I16(v) => Ok(*v != 0),
        SlmpValue::U32(v) => Ok(*v != 0),
        SlmpValue::I32(v) => Ok(*v != 0),
        SlmpValue::F32(v) => Ok(*v != 0.0),
    }
}

fn scalar_to_u16(value: &SlmpValue) -> Result<u16, SlmpError> {
    match value {
        SlmpValue::U16(v) => Ok(*v),
        SlmpValue::I16(v) => Ok(*v as u16),
        SlmpValue::Bool(v) => Ok(u16::from(*v)),
        SlmpValue::U32(v) => Ok(*v as u16),
        SlmpValue::I32(v) => Ok(*v as u16),
        SlmpValue::F32(v) => Ok(*v as u16),
    }
}

fn scalar_to_u32(value: &SlmpValue) -> Result<u32, SlmpError> {
    match value {
        SlmpValue::U32(v) => Ok(*v),
        SlmpValue::I32(v) => Ok(*v as u32),
        SlmpValue::U16(v) => Ok(*v as u32),
        SlmpValue::I16(v) => Ok(*v as u32),
        SlmpValue::Bool(v) => Ok(u32::from(*v)),
        SlmpValue::F32(v) => Ok(*v as u32),
    }
}

fn scalar_to_i32(value: &SlmpValue) -> Result<i32, SlmpError> {
    match value {
        SlmpValue::I32(v) => Ok(*v),
        SlmpValue::U32(v) => Ok(*v as i32),
        SlmpValue::U16(v) => Ok(*v as i32),
        SlmpValue::I16(v) => Ok(*v as i32),
        SlmpValue::Bool(v) => Ok(i32::from(*v)),
        SlmpValue::F32(v) => Ok(*v as i32),
    }
}

fn scalar_to_f32(value: &SlmpValue) -> Result<f32, SlmpError> {
    match value {
        SlmpValue::F32(v) => Ok(*v),
        SlmpValue::U32(v) => Ok(*v as f32),
        SlmpValue::I32(v) => Ok(*v as f32),
        SlmpValue::U16(v) => Ok(*v as f32),
        SlmpValue::I16(v) => Ok(*v as f32),
        SlmpValue::Bool(v) => Ok(if *v { 1.0 } else { 0.0 }),
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

pub fn parse_scalar_for_named(address: &str, value: &str) -> Result<SlmpValue, SlmpError> {
    let parts = parse_named_address(address)?;
    let device = parse_device_for_family_hint(&parts.base, None)?;
    if parts.bit_index.is_some() || device.code.is_bit_device() {
        return Ok(SlmpValue::Bool(matches!(
            value,
            "1" | "true" | "TRUE" | "True"
        )));
    }
    if parts.dtype.eq_ignore_ascii_case("F") {
        return value
            .parse::<f32>()
            .map(SlmpValue::F32)
            .map_err(|_| SlmpError::new("Invalid float value."));
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
    Ok(
        match resolve_dtype_for_address(address, device, &parts.dtype, parts.bit_index).as_str() {
            "L" => SlmpValue::I32(parsed as i32),
            "D" => SlmpValue::U32(parsed as u32),
            "S" => SlmpValue::I16(parsed as i16),
            _ => SlmpValue::U16(parsed as u16),
        },
    )
}
