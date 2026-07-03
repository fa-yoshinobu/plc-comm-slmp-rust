use crate::capability_profiles::{self, SlmpProfileLimit};
use crate::error::SlmpError;
use crate::model::{
    SlmpBlockRead, SlmpBlockWrite, SlmpCompatibilityMode, SlmpCpuOperationState,
    SlmpCpuOperationStatus, SlmpDeviceAddress, SlmpDeviceCode, SlmpLongTimerResult, SlmpPlcProfile,
};

const DIRECT_WORD_POINT_LIMIT: usize = 960;
const DIRECT_BIT_POINT_LIMIT: usize = 7168;
const DIRECT_IQF_BIT_POINT_LIMIT: usize = 3584;
const MEMORY_WORD_LIMIT: usize = 480;
const EXTEND_UNIT_BYTE_LIMIT: usize = 1920;

pub(crate) fn validate_non_empty_u16_count(count: usize, name: &str) -> Result<(), SlmpError> {
    if count == 0 {
        return Err(SlmpError::new(format!("{name} must not be empty")));
    }
    validate_u16_count(count, name)
}

pub(crate) fn validate_u16_count(count: usize, name: &str) -> Result<(), SlmpError> {
    if count > u16::MAX as usize {
        return Err(SlmpError::new(format!("{name} must be <= 65535")));
    }
    Ok(())
}

pub(crate) fn validate_direct_access_points(
    points: usize,
    bit_unit: bool,
    write: bool,
    name: &str,
    plc_profile: SlmpPlcProfile,
) -> Result<(), SlmpError> {
    let limit_key = match (bit_unit, write) {
        (false, false) => SlmpProfileLimit::DirectWordRead,
        (false, true) => SlmpProfileLimit::DirectWordWrite,
        (true, false) => SlmpProfileLimit::DirectBitRead,
        (true, true) => SlmpProfileLimit::DirectBitWrite,
    };
    let limit = capability_profiles::profile_limit(plc_profile, limit_key)
        .map(|profile_limit| profile_limit.max)
        .unwrap_or_else(|| {
            if bit_unit {
                if matches!(plc_profile, SlmpPlcProfile::IqF) {
                    DIRECT_IQF_BIT_POINT_LIMIT
                } else {
                    DIRECT_BIT_POINT_LIMIT
                }
            } else {
                DIRECT_WORD_POINT_LIMIT
            }
        });
    let unit = if bit_unit { "bit" } else { "word" };
    if points < 1 || points > limit {
        return Err(SlmpError::new(format!(
            "{name} {unit} access points out of range (1..{limit}): {points}"
        )));
    }
    Ok(())
}

pub(crate) fn validate_random_read_like_counts(
    word_points: usize,
    dword_points: usize,
    compatibility_mode: SlmpCompatibilityMode,
    plc_profile: SlmpPlcProfile,
    limit_key: SlmpProfileLimit,
    name: &str,
) -> Result<(), SlmpError> {
    let total = word_points + dword_points;
    let limit = capability_profiles::profile_limit(plc_profile, limit_key)
        .map(|profile_limit| profile_limit.max)
        .unwrap_or_else(|| {
            if matches!(compatibility_mode, SlmpCompatibilityMode::Legacy) {
                192
            } else {
                96
            }
        });
    if total < 1 || total > limit {
        return Err(SlmpError::new(format!(
            "{name} total access points out of range (1..{limit}): word={word_points}, dword={dword_points}"
        )));
    }
    Ok(())
}

pub(crate) fn validate_random_write_word_counts(
    word_points: usize,
    dword_points: usize,
    compatibility_mode: SlmpCompatibilityMode,
    plc_profile: SlmpPlcProfile,
    name: &str,
) -> Result<(), SlmpError> {
    let total = word_points + dword_points;
    if total < 1 {
        return Err(SlmpError::new(format!(
            "{name} word/dword access points out of range: word={word_points}, dword={dword_points}"
        )));
    }
    if let Some(limit) =
        capability_profiles::profile_limit(plc_profile, SlmpProfileLimit::RandomWriteWord)
    {
        if total > limit.max {
            return Err(SlmpError::new(format!(
                "{name} word/dword access points out of range (1..{}): word={word_points}, dword={dword_points}",
                limit.max
            )));
        }
        if let Some(weighted_max) = limit.weighted_max {
            let weighted = (word_points * 12) + (dword_points * 14);
            if weighted > weighted_max {
                return Err(SlmpError::new(format!(
                    "{name} word/dword access points out of range: word={word_points}, dword={dword_points}, weighted={weighted}, limit={weighted_max}"
                )));
            }
        }
    } else {
        let weighted = (word_points * 12) + (dword_points * 14);
        let limit = if matches!(compatibility_mode, SlmpCompatibilityMode::Legacy) {
            1920
        } else {
            960
        };
        if weighted > limit {
            return Err(SlmpError::new(format!(
                "{name} word/dword access points out of range: word={word_points}, dword={dword_points}, weighted={weighted}, limit={limit}"
            )));
        }
    }
    Ok(())
}

pub(crate) fn validate_random_bit_write_count(
    points: usize,
    compatibility_mode: SlmpCompatibilityMode,
    plc_profile: SlmpPlcProfile,
    name: &str,
) -> Result<(), SlmpError> {
    let limit = capability_profiles::profile_limit(plc_profile, SlmpProfileLimit::RandomWriteBit)
        .map(|profile_limit| profile_limit.max)
        .unwrap_or_else(|| {
            if matches!(compatibility_mode, SlmpCompatibilityMode::Legacy) {
                188
            } else {
                94
            }
        });
    if points < 1 || points > limit {
        return Err(SlmpError::new(format!(
            "{name} bit access points out of range (1..{limit}): {points}"
        )));
    }
    Ok(())
}

pub(crate) fn validate_block_read_limits(
    word_blocks: &[SlmpBlockRead],
    bit_blocks: &[SlmpBlockRead],
    compatibility_mode: SlmpCompatibilityMode,
) -> Result<(), SlmpError> {
    let total_blocks = word_blocks.len() + bit_blocks.len();
    validate_block_count(total_blocks, compatibility_mode, "read_block")?;
    let total_points: usize = word_blocks
        .iter()
        .map(|block| validate_block_points(block.points as usize, "read_block word"))
        .sum::<Result<usize, SlmpError>>()?
        + bit_blocks
            .iter()
            .map(|block| validate_block_points(block.points as usize, "read_block bit"))
            .sum::<Result<usize, SlmpError>>()?;
    if total_points > DIRECT_WORD_POINT_LIMIT {
        return Err(SlmpError::new(format!(
            "read_block total device points out of range (<=960): total_points={total_points}"
        )));
    }
    Ok(())
}

pub(crate) fn validate_block_write_limits(
    word_blocks: &[SlmpBlockWrite],
    bit_blocks: &[SlmpBlockWrite],
    compatibility_mode: SlmpCompatibilityMode,
) -> Result<(), SlmpError> {
    let total_blocks = word_blocks.len() + bit_blocks.len();
    validate_block_count(total_blocks, compatibility_mode, "write_block")?;
    let total_points: usize = word_blocks
        .iter()
        .map(|block| validate_block_points(block.values.len(), "write_block word"))
        .sum::<Result<usize, SlmpError>>()?
        + bit_blocks
            .iter()
            .map(|block| validate_block_points(block.values.len(), "write_block bit"))
            .sum::<Result<usize, SlmpError>>()?;
    let per_block_overhead = if matches!(compatibility_mode, SlmpCompatibilityMode::Legacy) {
        4
    } else {
        9
    };
    let weighted = total_points + (total_blocks * per_block_overhead);
    if weighted > DIRECT_WORD_POINT_LIMIT {
        return Err(SlmpError::new(format!(
            "write_block total device points out of range (<=960): weighted={weighted}, total_points={total_points}"
        )));
    }
    Ok(())
}

pub(crate) fn validate_memory_word_length(word_length: usize, name: &str) -> Result<(), SlmpError> {
    if !(1..=MEMORY_WORD_LIMIT).contains(&word_length) {
        return Err(SlmpError::new(format!(
            "{name} word length out of range (1..480): {word_length}"
        )));
    }
    Ok(())
}

pub(crate) fn validate_extend_unit_byte_length(
    byte_length: usize,
    name: &str,
) -> Result<(), SlmpError> {
    if !(2..=EXTEND_UNIT_BYTE_LIMIT).contains(&byte_length) {
        return Err(SlmpError::new(format!(
            "{name} byte length out of range (2..1920): {byte_length}"
        )));
    }
    Ok(())
}

pub(crate) fn validate_extend_unit_word_length(
    word_length: usize,
    name: &str,
) -> Result<(), SlmpError> {
    if !(1..=DIRECT_WORD_POINT_LIMIT).contains(&word_length) {
        return Err(SlmpError::new(format!(
            "{name} word length out of range (1..960): {word_length}"
        )));
    }
    Ok(())
}

pub(crate) fn validate_block_route_for_profile(
    plc_profile: SlmpPlcProfile,
    command_label: &str,
) -> Result<(), SlmpError> {
    let _ = (plc_profile, command_label);
    Ok(())
}

fn validate_block_count(
    total_blocks: usize,
    compatibility_mode: SlmpCompatibilityMode,
    name: &str,
) -> Result<(), SlmpError> {
    let limit = if matches!(compatibility_mode, SlmpCompatibilityMode::Legacy) {
        120
    } else {
        60
    };
    if total_blocks < 1 || total_blocks > limit {
        return Err(SlmpError::new(format!(
            "{name} total block count out of range (1..{limit}): {total_blocks}"
        )));
    }
    Ok(())
}

fn validate_block_points(points: usize, name: &str) -> Result<usize, SlmpError> {
    if points < 1 || points > u16::MAX as usize {
        return Err(SlmpError::new(format!(
            "{name} block points out of range (1..65535): {points}"
        )));
    }
    Ok(points)
}

pub(crate) fn validate_direct_bit_read(device: SlmpDeviceAddress) -> Result<(), SlmpError> {
    if is_qualified_only_device(device.code) {
        return Err(SlmpError::new(
            "Direct device access does not support standalone G/HG. Use U-qualified extended access.",
        ));
    }
    // Long timer state bits are decoded from the LTN/LSTN 4-word status block.
    // Do not send direct bit read (0x0401) for these devices.
    if is_long_timer_state_device(device.code) {
        return Err(SlmpError::new(
            "Direct bit read is not supported for long timer state devices. Use read_typed/read_named or a 4-word current-value block read.",
        ));
    }
    Ok(())
}

pub(crate) fn validate_direct_bit_write(
    device: SlmpDeviceAddress,
    plc_profile: SlmpPlcProfile,
) -> Result<(), SlmpError> {
    if is_qualified_only_device(device.code) {
        return Err(SlmpError::new(
            "Direct device access does not support standalone G/HG. Use U-qualified extended access.",
        ));
    }
    if is_read_only_device(device.code, plc_profile) {
        return Err(read_only_write_error(device.code, plc_profile));
    }
    // PLCs reject direct bit write (0x1401) for these state bits. The
    // supported write path is write_typed/write_named, which selects 0x1402.
    if requires_random_bit_write(device.code) {
        return Err(SlmpError::new(
            "Direct bit write is not supported for long-family state devices. Use write_typed/write_named so random bit write (0x1402) is selected.",
        ));
    }
    Ok(())
}

pub(crate) fn validate_direct_word_read(
    device: SlmpDeviceAddress,
    points: u16,
) -> Result<(), SlmpError> {
    if is_qualified_only_device(device.code) {
        return Err(SlmpError::new(
            "Direct device access does not support standalone G/HG. Use U-qualified extended access.",
        ));
    }
    match device.code {
        code if is_random_dword_only_read_device(code) => Err(SlmpError::new(
            "Direct word read is not supported for LCN/LZ. Use read_typed/read_named for 32-bit access.",
        )),
        code if matches!(code, SlmpDeviceCode::LTN | SlmpDeviceCode::LSTN)
            && (points == 0 || points % 4 != 0) =>
        {
            Err(SlmpError::new(
                "Long timer and long retentive timer current values must be read as 4-word blocks.",
            ))
        }
        _ => Ok(()),
    }
}

pub(crate) fn validate_direct_word_write(
    device: SlmpDeviceAddress,
    plc_profile: SlmpPlcProfile,
) -> Result<(), SlmpError> {
    if is_qualified_only_device(device.code) {
        return Err(SlmpError::new(
            "Direct device access does not support standalone G/HG. Use U-qualified extended access.",
        ));
    }
    if is_read_only_device(device.code, plc_profile) {
        return Err(read_only_write_error(device.code, plc_profile));
    }
    if is_long_current_value_device(device.code) || is_dword_only_scalar_device(device.code) {
        return Err(SlmpError::new(
            "Direct word write is not supported for LTN/LSTN/LCN/LZ. Use write_typed/write_named with ':D' or ':L' instead.",
        ));
    }
    Ok(())
}

pub(crate) fn validate_direct_dword_read(device: SlmpDeviceAddress) -> Result<(), SlmpError> {
    if is_qualified_only_device(device.code) {
        return Err(SlmpError::new(
            "Direct device access does not support standalone G/HG. Use U-qualified extended access.",
        ));
    }
    if is_long_current_value_device(device.code) || is_dword_only_scalar_device(device.code) {
        return Err(SlmpError::new(
            "Direct dword read is not supported for LTN/LSTN/LCN/LZ. Use read_typed/read_named or the supported long-family helper route.",
        ));
    }
    Ok(())
}

pub(crate) fn validate_direct_dword_write(
    device: SlmpDeviceAddress,
    plc_profile: SlmpPlcProfile,
) -> Result<(), SlmpError> {
    if is_qualified_only_device(device.code) {
        return Err(SlmpError::new(
            "Direct device access does not support standalone G/HG. Use U-qualified extended access.",
        ));
    }
    if is_read_only_device(device.code, plc_profile) {
        return Err(read_only_write_error(device.code, plc_profile));
    }
    if is_long_current_value_device(device.code) || is_dword_only_scalar_device(device.code) {
        return Err(SlmpError::new(
            "Direct dword write is not supported for LTN/LSTN/LCN/LZ. Use write_typed/write_named so random dword write (0x1402) is selected.",
        ));
    }
    Ok(())
}

pub(crate) fn validate_random_read_devices(
    word_devices: &[SlmpDeviceAddress],
    dword_devices: &[SlmpDeviceAddress],
) -> Result<(), SlmpError> {
    for device in word_devices.iter().chain(dword_devices.iter()) {
        if is_qualified_only_device(device.code) {
            return Err(SlmpError::new(
                "Read Random (0x0403) does not support standalone G/HG. Use U-qualified extended access.",
            ));
        }
        // LTS/LTC/LSTS/LSTC can be written by random bit write, but they are
        // not readable by Read Random (0x0403); use status-block reads.
        if is_long_timer_state_device(device.code) {
            return Err(SlmpError::new(
                "Read Random (0x0403) does not support LTS/LTC/LSTS/LSTC. Use read_typed/read_named or a 4-word current-value block read.",
            ));
        }

        if matches!(device.code, SlmpDeviceCode::LCS | SlmpDeviceCode::LCC) {
            return Err(SlmpError::new(
                "Read Random (0x0403) does not support LCS/LCC. Use read_typed/read_named so direct bit read is selected.",
            ));
        }
    }
    for device in word_devices {
        if is_long_current_value_device(device.code) || is_dword_only_scalar_device(device.code) {
            return Err(SlmpError::new(
                "Read Random (0x0403) does not support LTN/LSTN/LCN/LZ as word entries. Use dword entries or read_typed/read_named with ':D' or ':L' instead.",
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_random_write_word_devices(
    word_entries: &[(SlmpDeviceAddress, u16)],
    dword_entries: &[(SlmpDeviceAddress, u32)],
    plc_profile: SlmpPlcProfile,
) -> Result<(), SlmpError> {
    for (device, _) in word_entries {
        if is_read_only_device(device.code, plc_profile) {
            return Err(read_only_random_write_error(plc_profile));
        }
        if is_qualified_only_device(device.code) {
            return Err(SlmpError::new(
                "Write Random (0x1402) does not support standalone G/HG. Use U-qualified extended access.",
            ));
        }
    }
    for (device, _) in dword_entries {
        if is_read_only_device(device.code, plc_profile) {
            return Err(read_only_random_write_error(plc_profile));
        }
        if is_qualified_only_device(device.code) {
            return Err(SlmpError::new(
                "Write Random (0x1402) does not support standalone G/HG. Use U-qualified extended access.",
            ));
        }
    }
    for (device, _) in word_entries {
        if is_long_current_value_device(device.code) || is_dword_only_scalar_device(device.code) {
            return Err(SlmpError::new(
                "Write Random (0x1402) does not support LTN/LSTN/LCN/LZ as word entries. Use dword entries or write_typed/write_named with ':D' or ':L' instead.",
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_random_bit_write_devices(
    bit_entries: &[(SlmpDeviceAddress, bool)],
    plc_profile: SlmpPlcProfile,
) -> Result<(), SlmpError> {
    for (device, _) in bit_entries {
        if is_read_only_device(device.code, plc_profile) {
            return Err(read_only_random_write_error(plc_profile));
        }
        if is_qualified_only_device(device.code) {
            return Err(SlmpError::new(
                "Write Random (0x1402) does not support standalone G/HG bit entries. Use U-qualified word access.",
            ));
        }
    }
    Ok(())
}

pub(crate) fn is_long_timer_state_device(code: SlmpDeviceCode) -> bool {
    matches!(
        code,
        SlmpDeviceCode::LTS | SlmpDeviceCode::LTC | SlmpDeviceCode::LSTS | SlmpDeviceCode::LSTC
    )
}

pub(crate) fn is_qualified_only_device(code: SlmpDeviceCode) -> bool {
    matches!(code, SlmpDeviceCode::G | SlmpDeviceCode::HG)
}

pub(crate) fn is_read_only_device(code: SlmpDeviceCode, plc_profile: SlmpPlcProfile) -> bool {
    matches!(code, SlmpDeviceCode::S)
        || capability_profiles::is_profile_read_only_device(plc_profile, code)
}

pub(crate) fn requires_random_bit_write(code: SlmpDeviceCode) -> bool {
    is_long_timer_state_device(code) || matches!(code, SlmpDeviceCode::LCS | SlmpDeviceCode::LCC)
}

pub(crate) fn is_long_current_value_device(code: SlmpDeviceCode) -> bool {
    matches!(
        code,
        SlmpDeviceCode::LTN | SlmpDeviceCode::LSTN | SlmpDeviceCode::LCN
    )
}

pub(crate) fn is_dword_only_scalar_device(code: SlmpDeviceCode) -> bool {
    matches!(code, SlmpDeviceCode::LZ)
}

pub(crate) fn is_random_dword_only_read_device(code: SlmpDeviceCode) -> bool {
    matches!(code, SlmpDeviceCode::LCN | SlmpDeviceCode::LZ)
}

pub(crate) fn validate_no_lcs_lcc_block_read(
    word_blocks: &[SlmpBlockRead],
    bit_blocks: &[SlmpBlockRead],
) -> Result<(), SlmpError> {
    for block in word_blocks {
        if is_qualified_only_device(block.device.code) {
            return Err(SlmpError::new(
                "Read Block (0x0406) does not support standalone G/HG. Use U-qualified extended access.",
            ));
        }
        if matches!(
            block.device.code,
            SlmpDeviceCode::LTN | SlmpDeviceCode::LSTN
        ) && block.points % 4 != 0
        {
            return Err(SlmpError::new(
                "Read Block (0x0406) direct long timer current reads require 4-word blocks.",
            ));
        }
    }
    for block in word_blocks.iter().chain(bit_blocks.iter()) {
        if is_qualified_only_device(block.device.code) {
            return Err(SlmpError::new(
                "Read Block (0x0406) does not support standalone G/HG. Use U-qualified extended access.",
            ));
        }
        if is_random_dword_only_read_device(block.device.code) {
            return Err(SlmpError::new(
                "Read Block (0x0406) does not support LCN/LZ as word or bit blocks. Use read_typed/read_named so random dword read is selected.",
            ));
        }
    }
    for block in word_blocks.iter().chain(bit_blocks.iter()) {
        if matches!(block.device.code, SlmpDeviceCode::LCS | SlmpDeviceCode::LCC) {
            return Err(SlmpError::new(
                "Read Block (0x0406) does not support LCS/LCC. Use read_typed/read_named so direct bit read is selected.",
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_no_lcs_lcc_block_write(
    word_blocks: &[SlmpBlockWrite],
    bit_blocks: &[SlmpBlockWrite],
    plc_profile: SlmpPlcProfile,
) -> Result<(), SlmpError> {
    for block in word_blocks.iter().chain(bit_blocks.iter()) {
        if is_read_only_device(block.device.code, plc_profile) {
            return Err(SlmpError::new(format!(
                "Write Block (0x1406) does not support read-only devices for plc_profile '{}'.",
                plc_profile.canonical_name()
            )));
        }
        if is_qualified_only_device(block.device.code) {
            return Err(SlmpError::new(
                "Write Block (0x1406) does not support standalone G/HG. Use U-qualified extended access.",
            ));
        }
        if is_long_current_value_device(block.device.code)
            || is_dword_only_scalar_device(block.device.code)
        {
            return Err(SlmpError::new(
                "Write Block (0x1406) does not support LTN/LSTN/LCN/LZ as word or bit blocks. Use write_typed/write_named with ':D' or ':L' instead.",
            ));
        }
    }
    for block in word_blocks.iter().chain(bit_blocks.iter()) {
        if matches!(block.device.code, SlmpDeviceCode::LCS | SlmpDeviceCode::LCC) {
            return Err(SlmpError::new(
                "Write Block (0x1406) does not support LCS/LCC. Use write_typed/write_named or other supported write routes.",
            ));
        }
    }
    Ok(())
}

fn read_only_write_error(code: SlmpDeviceCode, plc_profile: SlmpPlcProfile) -> SlmpError {
    if capability_profiles::is_profile_read_only_device(plc_profile, code) {
        SlmpError::new(format!(
            "{} is read-only for plc_profile '{}' and cannot be written.",
            code.prefix(),
            plc_profile.canonical_name()
        ))
    } else {
        SlmpError::new("S is read-only and cannot be written.")
    }
}

fn read_only_random_write_error(plc_profile: SlmpPlcProfile) -> SlmpError {
    SlmpError::new(format!(
        "Write Random (0x1402) does not support read-only devices such as S or profile read-only families for plc_profile '{}'.",
        plc_profile.canonical_name()
    ))
}

pub(crate) fn unpack_bit_values(data: &[u8], points: usize) -> Result<Vec<bool>, SlmpError> {
    let need = points.div_ceil(2);
    if data.len() < need {
        return Err(SlmpError::new("read_bits payload size mismatch"));
    }
    let mut result = Vec::with_capacity(points);
    for byte in data.iter().take(need) {
        if result.len() < points {
            result.push(((byte >> 4) & 0x01) != 0);
        }
        if result.len() < points {
            result.push((byte & 0x01) != 0);
        }
    }
    Ok(result)
}

pub(crate) fn parse_long_timer_words(
    words: &[u16],
    head_no: u32,
    prefix: &str,
) -> Vec<SlmpLongTimerResult> {
    let mut result = Vec::with_capacity(words.len() / 4);
    for (index, chunk) in words.chunks_exact(4).enumerate() {
        let status_word = chunk[2];
        let current_value = chunk[0] as u32 | ((chunk[1] as u32) << 16);
        result.push(SlmpLongTimerResult {
            index: head_no + index as u32,
            device: format!("{prefix}{}", head_no + index as u32),
            current_value,
            contact: (status_word & 0x0002) != 0,
            coil: (status_word & 0x0001) != 0,
            status_word,
            raw_words: chunk.to_vec(),
        });
    }
    result
}

pub(crate) fn decode_cpu_operation_state(status_word: u16) -> SlmpCpuOperationState {
    let raw_code = (status_word & 0x000F) as u8;
    let status = match raw_code {
        0x00 => SlmpCpuOperationStatus::Run,
        0x02 => SlmpCpuOperationStatus::Stop,
        0x03 => SlmpCpuOperationStatus::Pause,
        _ => SlmpCpuOperationStatus::Unknown,
    };
    SlmpCpuOperationState {
        status,
        raw_status_word: status_word,
        raw_code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_u16_sized_counts() {
        assert!(validate_u16_count(u16::MAX as usize, "labels").is_ok());
        assert_eq!(
            validate_u16_count(u16::MAX as usize + 1, "labels")
                .unwrap_err()
                .to_string(),
            "labels must be <= 65535"
        );
        assert_eq!(
            validate_non_empty_u16_count(0, "labels")
                .unwrap_err()
                .to_string(),
            "labels must not be empty"
        );
    }

    #[test]
    fn classifies_long_family_device_rules() {
        assert!(is_long_timer_state_device(SlmpDeviceCode::LTS));
        assert!(is_long_timer_state_device(SlmpDeviceCode::LSTC));
        assert!(!is_long_timer_state_device(SlmpDeviceCode::LCS));

        assert!(requires_random_bit_write(SlmpDeviceCode::LTC));
        assert!(requires_random_bit_write(SlmpDeviceCode::LCC));
        assert!(!requires_random_bit_write(SlmpDeviceCode::M));

        assert!(is_long_current_value_device(SlmpDeviceCode::LTN));
        assert!(is_long_current_value_device(SlmpDeviceCode::LCN));
        assert!(!is_long_current_value_device(SlmpDeviceCode::LZ));

        assert!(is_dword_only_scalar_device(SlmpDeviceCode::LZ));
        assert!(is_random_dword_only_read_device(SlmpDeviceCode::LCN));
        assert!(is_random_dword_only_read_device(SlmpDeviceCode::LZ));
        assert!(!is_random_dword_only_read_device(SlmpDeviceCode::LTN));
    }

    #[test]
    fn unpacks_bit_values_high_nibble_then_low_nibble() {
        let values = unpack_bit_values(&[0x10, 0x01, 0x11], 5).unwrap();
        assert_eq!(values, vec![true, false, false, true, true]);

        assert_eq!(
            unpack_bit_values(&[0x00], 3).unwrap_err().to_string(),
            "read_bits payload size mismatch"
        );
    }

    #[test]
    fn parses_long_timer_words_as_four_word_blocks() {
        let values = parse_long_timer_words(
            &[
                0x5678, 0x1234, 0x0003, 0xAAAA, 0x0001, 0x0000, 0x0002, 0xBBBB,
            ],
            10,
            "LTN",
        );

        assert_eq!(values.len(), 2);
        assert_eq!(values[0].index, 10);
        assert_eq!(values[0].device, "LTN10");
        assert_eq!(values[0].current_value, 0x1234_5678);
        assert!(values[0].contact);
        assert!(values[0].coil);
        assert_eq!(values[0].status_word, 0x0003);
        assert_eq!(values[0].raw_words, vec![0x5678, 0x1234, 0x0003, 0xAAAA]);

        assert_eq!(values[1].index, 11);
        assert_eq!(values[1].device, "LTN11");
        assert_eq!(values[1].current_value, 1);
        assert!(values[1].contact);
        assert!(!values[1].coil);
        assert_eq!(values[1].status_word, 0x0002);
    }

    #[test]
    fn decodes_cpu_operation_state_from_low_nibble() {
        let state = decode_cpu_operation_state(0x00A2);
        assert_eq!(state.status, SlmpCpuOperationStatus::Stop);
        assert_eq!(state.raw_status_word, 0x00A2);
        assert_eq!(state.raw_code, 0x02);

        assert_eq!(
            decode_cpu_operation_state(0x00F5).status,
            SlmpCpuOperationStatus::Unknown
        );
    }
}
