use crate::address::SlmpAddress;
use crate::client::SlmpClient;
use crate::device_ranges::{SlmpDeviceRangeEntry, SlmpDeviceRangeFamily};
use crate::error::SlmpError;
use crate::helpers::{SlmpValue, read_typed, write_typed};
use crate::model::{SlmpBlockRead, SlmpDeviceAddress, SlmpDeviceCode};
use std::collections::BTreeSet;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlmpDeviceRangeSampleOptions {
    #[serde(default = "default_sample_points")]
    pub sample_points: usize,
    #[serde(default)]
    pub only: Vec<String>,
}

impl Default for SlmpDeviceRangeSampleOptions {
    fn default() -> Self {
        Self {
            sample_points: default_sample_points(),
            only: Vec::new(),
        }
    }
}

impl SlmpDeviceRangeSampleOptions {
    pub fn normalized(mut self) -> Self {
        if self.sample_points == 0 {
            self.sample_points = default_sample_points();
        }
        self.only = self
            .only
            .into_iter()
            .map(|device| device.trim().to_ascii_uppercase())
            .filter(|device| !device.is_empty())
            .collect();
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum SlmpDeviceRangeSampleValueKind {
    Bit,
    Word,
    Dword,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlmpDeviceRangeSampleSummary {
    pub passed: usize,
    pub read_failed: usize,
    pub write_failed: usize,
    #[serde(default)]
    pub readback_failed: usize,
    pub restore_failed: usize,
    pub skipped: usize,
    pub unsupported: usize,
    pub bit_blocks_passed: usize,
    pub bit_blocks_failed: usize,
}

impl SlmpDeviceRangeSampleSummary {
    pub fn is_success(&self) -> bool {
        self.read_failed == 0
            && self.write_failed == 0
            && self.readback_failed == 0
            && self.restore_failed == 0
            && self.bit_blocks_failed == 0
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlmpDeviceRangeSampleFailure {
    pub address: String,
    pub phase: String,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlmpDeviceRangeSampleDeviceReport {
    pub device: String,
    pub address_range: Option<String>,
    pub value_kind: Option<SlmpDeviceRangeSampleValueKind>,
    pub sample_addresses: Vec<String>,
    pub bit_block_addresses: Vec<String>,
    pub untested_reason: Option<String>,
    pub passed: usize,
    pub read_failed: usize,
    pub write_failed: usize,
    #[serde(default)]
    pub readback_failed: usize,
    pub restore_failed: usize,
    pub skipped: usize,
    pub unsupported: usize,
    pub bit_blocks_passed: usize,
    pub bit_blocks_failed: usize,
    pub failures: Vec<SlmpDeviceRangeSampleFailure>,
}

impl SlmpDeviceRangeSampleDeviceReport {
    fn new(entry: &SlmpDeviceRangeEntry) -> Self {
        Self {
            device: entry.device.clone(),
            address_range: entry.address_range.clone(),
            value_kind: None,
            sample_addresses: Vec::new(),
            bit_block_addresses: Vec::new(),
            untested_reason: None,
            passed: 0,
            read_failed: 0,
            write_failed: 0,
            readback_failed: 0,
            restore_failed: 0,
            skipped: 0,
            unsupported: 0,
            bit_blocks_passed: 0,
            bit_blocks_failed: 0,
            failures: Vec::new(),
        }
    }

    fn fail(&mut self, address: String, phase: &str, message: String) {
        self.failures.push(SlmpDeviceRangeSampleFailure {
            address,
            phase: phase.to_string(),
            message,
        });
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlmpDeviceRangeSampleReport {
    pub model: String,
    pub family: SlmpDeviceRangeFamily,
    pub sample_points: usize,
    pub only: Vec<String>,
    pub summary: SlmpDeviceRangeSampleSummary,
    pub devices: Vec<SlmpDeviceRangeSampleDeviceReport>,
}

impl SlmpDeviceRangeSampleReport {
    pub fn is_success(&self) -> bool {
        self.summary.is_success()
    }
}

pub async fn run_device_range_sample_compare(
    client: &SlmpClient,
    options: SlmpDeviceRangeSampleOptions,
) -> Result<SlmpDeviceRangeSampleReport, SlmpError> {
    let options = options.normalized();
    let only = options.only.iter().cloned().collect::<BTreeSet<_>>();
    let plc_family = client.plc_family().await;
    let catalog = client.read_device_range_catalog().await?;
    let range_family = catalog.family;
    let mut report = SlmpDeviceRangeSampleReport {
        model: catalog.model,
        family: range_family,
        sample_points: options.sample_points,
        only: options.only,
        summary: SlmpDeviceRangeSampleSummary::default(),
        devices: Vec::new(),
    };

    for entry in catalog.entries {
        if !only.is_empty() && !only.contains(&entry.device.to_ascii_uppercase()) {
            continue;
        }

        let mut device_report = SlmpDeviceRangeSampleDeviceReport::new(&entry);
        if !entry.supported {
            report.summary.skipped += 1;
            device_report.skipped += 1;
            device_report.untested_reason = Some("unsupported by catalog".to_string());
            report.devices.push(device_report);
            continue;
        }

        let Some(upper_bound) = entry.upper_bound else {
            report.summary.skipped += 1;
            device_report.skipped += 1;
            device_report.untested_reason = Some("open-ended range".to_string());
            report.devices.push(device_report);
            continue;
        };

        let Some(code) = SlmpDeviceCode::parse_prefix(&entry.device) else {
            report.summary.unsupported += 1;
            device_report.unsupported += 1;
            device_report.untested_reason = Some("parser/client has no device code".to_string());
            report.devices.push(device_report);
            continue;
        };

        let kind = kind_for(&entry, code);
        device_report.value_kind = Some(kind);
        for number in sample_numbers(upper_bound, options.sample_points) {
            let device = SlmpDeviceAddress::new(code, number);
            let address = SlmpAddress::format_for_plc_family(device, plc_family);
            device_report.sample_addresses.push(address.clone());

            match exercise_point(client, device, &address, kind).await {
                Ok(()) => {
                    report.summary.passed += 1;
                    device_report.passed += 1;
                }
                Err((phase, message)) if phase == "read" => {
                    report.summary.read_failed += 1;
                    device_report.read_failed += 1;
                    device_report.fail(address.clone(), phase, message);
                }
                Err((phase, message)) if phase == "restore" => {
                    report.summary.restore_failed += 1;
                    device_report.restore_failed += 1;
                    device_report.fail(address.clone(), phase, message);
                }
                Err((phase, message)) if phase == "readback" => {
                    report.summary.readback_failed += 1;
                    device_report.readback_failed += 1;
                    device_report.fail(address.clone(), phase, message);
                }
                Err((phase, message)) => {
                    report.summary.write_failed += 1;
                    device_report.write_failed += 1;
                    device_report.fail(address.clone(), phase, message);
                }
            }

            if kind == SlmpDeviceRangeSampleValueKind::Bit
                && supports_bit_block_route(range_family)
                && supports_direct_bit_block(code)
            {
                let available_bits = upper_bound.saturating_sub(number) + 1;
                let word_points = (available_bits / 16).min(1) as u16;
                if word_points == 0 {
                    continue;
                }

                let write_block = supports_direct_bit_block_write(code);
                match exercise_bit_block(client, device, &address, word_points, write_block).await {
                    Ok(()) => {
                        report.summary.bit_blocks_passed += 1;
                        device_report.bit_blocks_passed += 1;
                        device_report.bit_block_addresses.push(address.clone());
                    }
                    Err((phase, message)) if phase == "restore" => {
                        report.summary.restore_failed += 1;
                        report.summary.bit_blocks_failed += 1;
                        device_report.restore_failed += 1;
                        device_report.bit_blocks_failed += 1;
                        device_report.fail(address.clone(), "bit-block-restore", message);
                    }
                    Err((phase, message)) => {
                        report.summary.bit_blocks_failed += 1;
                        device_report.bit_blocks_failed += 1;
                        device_report.fail(address.clone(), &format!("bit-block-{phase}"), message);
                    }
                }
            }
        }

        report.devices.push(device_report);
    }

    Ok(report)
}

fn default_sample_points() -> usize {
    10
}

fn sample_numbers(upper_bound: u32, count: usize) -> Vec<u32> {
    if count == 0 {
        return Vec::new();
    }

    if (upper_bound as u64) < count as u64 {
        return (0..=upper_bound).collect();
    }

    let upper = upper_bound as u64;
    let mut selected = BTreeSet::new();
    for candidate in [
        0,
        upper,
        upper / 2,
        upper / 4,
        (upper * 3) / 4,
        upper / 8,
        (upper * 3) / 8,
        (upper * 5) / 8,
        (upper * 7) / 8,
        1,
        upper.saturating_sub(1),
    ] {
        if selected.len() < count {
            selected.insert(candidate as u32);
        }
    }

    for index in 0..count {
        if selected.len() >= count {
            break;
        }
        let denominator = count.saturating_sub(1) as u64;
        let candidate = if denominator == 0 {
            0
        } else {
            ((index as u64 * upper) / denominator) as u32
        };
        selected.insert(candidate);
    }

    let mut cursor = 0u32;
    while selected.len() < count {
        selected.insert(cursor);
        cursor = cursor.saturating_add(1);
    }

    selected.into_iter().collect()
}

fn kind_for(entry: &SlmpDeviceRangeEntry, code: SlmpDeviceCode) -> SlmpDeviceRangeSampleValueKind {
    if entry.is_bit_device {
        SlmpDeviceRangeSampleValueKind::Bit
    } else if matches!(
        code,
        SlmpDeviceCode::LTN | SlmpDeviceCode::LSTN | SlmpDeviceCode::LCN | SlmpDeviceCode::LZ
    ) {
        SlmpDeviceRangeSampleValueKind::Dword
    } else {
        SlmpDeviceRangeSampleValueKind::Word
    }
}

fn dtype_for(kind: SlmpDeviceRangeSampleValueKind) -> &'static str {
    match kind {
        SlmpDeviceRangeSampleValueKind::Bit => "BIT",
        SlmpDeviceRangeSampleValueKind::Word => "U",
        SlmpDeviceRangeSampleValueKind::Dword => "D",
    }
}

fn supports_direct_bit_block(code: SlmpDeviceCode) -> bool {
    !matches!(
        code,
        SlmpDeviceCode::LTS
            | SlmpDeviceCode::LTC
            | SlmpDeviceCode::LSTS
            | SlmpDeviceCode::LSTC
            | SlmpDeviceCode::LCS
            | SlmpDeviceCode::LCC
    )
}

fn supports_direct_bit_block_write(code: SlmpDeviceCode) -> bool {
    supports_direct_bit_block(code) && !matches!(code, SlmpDeviceCode::SM)
}

fn supports_bit_block_route(family: SlmpDeviceRangeFamily) -> bool {
    !matches!(
        family,
        SlmpDeviceRangeFamily::QCpu
            | SlmpDeviceRangeFamily::LCpu
            | SlmpDeviceRangeFamily::QnU
            | SlmpDeviceRangeFamily::QnUDV
    )
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

fn test_values(
    address: &str,
    original: &SlmpValue,
    kind: SlmpDeviceRangeSampleValueKind,
) -> (SlmpValue, SlmpValue) {
    match (kind, original) {
        (SlmpDeviceRangeSampleValueKind::Bit, SlmpValue::Bool(value)) => {
            (SlmpValue::Bool(!*value), SlmpValue::Bool(*value))
        }
        (SlmpDeviceRangeSampleValueKind::Word, SlmpValue::U16(original)) => {
            let mut a = seeded_u16(address, 0x1111);
            let mut b = seeded_u16(address, 0x2222);
            if a == *original {
                a ^= 0x00FF;
            }
            if b == a {
                b ^= 0xFF00;
            }
            (SlmpValue::U16(a), SlmpValue::U16(b))
        }
        (SlmpDeviceRangeSampleValueKind::Dword, SlmpValue::U32(original)) => {
            let mut a = seeded_u32(address, 0x3333);
            let mut b = seeded_u32(address, 0x4444);
            if a == *original {
                a ^= 0x0000_FFFF;
            }
            if b == a {
                b ^= 0xFFFF_0000;
            }
            (SlmpValue::U32(a), SlmpValue::U32(b))
        }
        _ => {
            let a = seeded_u16(address, 0x5555);
            let b = seeded_u16(address, 0x6666);
            (SlmpValue::U16(a), SlmpValue::U16(b))
        }
    }
}

async fn read_value(
    client: &SlmpClient,
    device: SlmpDeviceAddress,
    kind: SlmpDeviceRangeSampleValueKind,
) -> Result<SlmpValue, SlmpError> {
    read_typed(client, device, dtype_for(kind)).await
}

async fn write_value(
    client: &SlmpClient,
    device: SlmpDeviceAddress,
    kind: SlmpDeviceRangeSampleValueKind,
    value: &SlmpValue,
) -> Result<(), SlmpError> {
    write_typed(client, device, dtype_for(kind), value).await
}

async fn assert_value(
    client: &SlmpClient,
    device: SlmpDeviceAddress,
    kind: SlmpDeviceRangeSampleValueKind,
    expected: &SlmpValue,
) -> Result<(), (&'static str, String)> {
    let observed = read_value(client, device, kind)
        .await
        .map_err(|error| ("readback", error.to_string()))?;
    if &observed != expected {
        return Err((
            "readback",
            format!("readback mismatch: expected={expected:?} observed={observed:?}"),
        ));
    }
    Ok(())
}

async fn exercise_point(
    client: &SlmpClient,
    device: SlmpDeviceAddress,
    address: &str,
    kind: SlmpDeviceRangeSampleValueKind,
) -> Result<(), (&'static str, String)> {
    let original = read_value(client, device, kind)
        .await
        .map_err(|error| ("read", error.to_string()))?;
    let (value_a, value_b) = test_values(address, &original, kind);

    let test_result: Result<(), (&'static str, String)> = async {
        write_value(client, device, kind, &value_a)
            .await
            .map_err(|error| ("write", error.to_string()))?;
        assert_value(client, device, kind, &value_a).await?;
        write_value(client, device, kind, &value_b)
            .await
            .map_err(|error| ("write", error.to_string()))?;
        assert_value(client, device, kind, &value_b).await?;
        Ok(())
    }
    .await;

    let restore_result = write_value(client, device, kind, &original).await;
    match (test_result, restore_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Ok(()), Err(error)) => Err(("restore", error.to_string())),
        (Err((phase, message)), Ok(())) => Err((phase, message)),
        (Err((_phase, test_error)), Err(restore_error)) => Err((
            "restore",
            format!("{test_error}; restore also failed: {restore_error}"),
        )),
    }
}

async fn read_bit_block_checked(
    client: &SlmpClient,
    device: SlmpDeviceAddress,
    word_points: u16,
) -> Result<Vec<bool>, SlmpError> {
    let bit_points = word_points * 16;
    let direct = client.read_bits(device, bit_points).await?;
    let block = client
        .read_block(
            &[],
            &[SlmpBlockRead {
                device,
                points: word_points,
            }],
        )
        .await?;
    let packed = pack_bit_words(&direct);
    if packed != block.bit_values {
        return Err(SlmpError::new(format!(
            "bit block mismatch: read_bits_packed={packed:?} read_block={:?}",
            block.bit_values
        )));
    }
    Ok(direct)
}

fn pack_bit_words(values: &[bool]) -> Vec<u16> {
    values
        .chunks(16)
        .map(|chunk| {
            let mut word = 0u16;
            for (index, value) in chunk.iter().enumerate() {
                if *value {
                    word |= 1 << index;
                }
            }
            word
        })
        .collect()
}

async fn exercise_bit_block(
    client: &SlmpClient,
    device: SlmpDeviceAddress,
    address: &str,
    word_points: u16,
    write: bool,
) -> Result<(), (&'static str, String)> {
    let original = read_bit_block_checked(client, device, word_points)
        .await
        .map_err(|error| ("read", error.to_string()))?;
    if !write {
        return Ok(());
    }
    let value_a = original.iter().map(|value| !*value).collect::<Vec<_>>();
    let value_b = (0..original.len())
        .map(|index| ((device.number as usize + index) % 2) == 0)
        .collect::<Vec<_>>();

    let test_result: Result<(), SlmpError> = async {
        client.write_bits(device, &value_a).await?;
        let observed = read_bit_block_checked(client, device, word_points).await?;
        if observed != value_a {
            return Err(SlmpError::new(format!(
                "{address} bit block write A mismatch: expected={value_a:?} observed={observed:?}"
            )));
        }
        client.write_bits(device, &value_b).await?;
        let observed = read_bit_block_checked(client, device, word_points).await?;
        if observed != value_b {
            return Err(SlmpError::new(format!(
                "{address} bit block write B mismatch: expected={value_b:?} observed={observed:?}"
            )));
        }
        Ok(())
    }
    .await;

    let restore_result = client.write_bits(device, &original).await;
    match (test_result, restore_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Ok(()), Err(error)) => Err(("restore", error.to_string())),
        (Err(error), Ok(())) => Err(("write", error.to_string())),
        (Err(test_error), Err(restore_error)) => Err((
            "restore",
            format!("{test_error}; restore also failed: {restore_error}"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{SlmpDeviceRangeSampleOptions, sample_numbers};

    #[test]
    fn sample_numbers_include_edges_and_middle() {
        let samples = sample_numbers(100, 10);
        assert_eq!(samples.len(), 10);
        assert!(samples.contains(&0));
        assert!(samples.contains(&50));
        assert!(samples.contains(&100));
    }

    #[test]
    fn options_normalize_zero_sample_count_to_default() {
        let options = SlmpDeviceRangeSampleOptions {
            sample_points: 0,
            only: vec![" d ".to_string(), "".to_string(), "x".to_string()],
        }
        .normalized();

        assert_eq!(options.sample_points, 10);
        assert_eq!(options.only, vec!["D", "X"]);
    }
}
