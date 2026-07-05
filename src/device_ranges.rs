use crate::client::SlmpClient;
use crate::error::SlmpError;
use crate::model::{SlmpDeviceAddress, SlmpDeviceCode, SlmpPlcProfile};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SlmpDeviceRangeCategory {
    Bit,
    Word,
    TimerCounter,
    Index,
    FileRegister,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SlmpDeviceRangeNotation {
    Decimal,
    Octal,
    Hexadecimal,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SlmpDeviceRangeEntry {
    pub device: String,
    pub category: SlmpDeviceRangeCategory,
    pub is_bit_device: bool,
    pub supported: bool,
    pub lower_bound: u32,
    pub upper_bound: Option<u32>,
    pub point_count: Option<u32>,
    pub address_range: Option<String>,
    pub notation: SlmpDeviceRangeNotation,
    pub source: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SlmpDeviceRangeCatalog {
    /// Synthetic label for the explicitly selected PLC profile.
    pub model: String,
    /// Always zero because device-range catalogs do not infer profiles from type-name responses.
    pub model_code: u16,
    /// Always false because profile selection is explicit.
    pub has_model_code: bool,
    pub plc_profile: SlmpPlcProfile,
    pub entries: Vec<SlmpDeviceRangeEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SlmpRangeValueKind {
    Unsupported,
    Undefined,
    Fixed,
    WordRegister,
    DWordRegister,
    WordRegisterClipped,
    DWordRegisterClipped,
}

#[derive(Debug, Clone)]
struct SlmpRangeValueSpec {
    kind: SlmpRangeValueKind,
    register: u16,
    fixed_value: u32,
    clip_value: u32,
    source: &'static str,
    notes: Option<&'static str>,
}

#[derive(Debug, Clone, Copy)]
struct SlmpDeviceRangeRow {
    category: SlmpDeviceRangeCategory,
    devices: &'static [(&'static str, bool)],
    notation: SlmpDeviceRangeNotation,
}

#[derive(Debug, Clone)]
pub(crate) struct SlmpDeviceRangeProfile {
    pub(crate) plc_profile: SlmpPlcProfile,
    pub(crate) register_start: u16,
    pub(crate) register_count: u16,
    rules: BTreeMap<&'static str, SlmpRangeValueSpec>,
}

const ORDERED_ITEMS: &[&str] = &[
    "X", "Y", "M", "B", "SB", "F", "V", "L", "S", "D", "W", "SW", "R", "T", "ST", "C", "LT", "LST",
    "LC", "Z", "LZ", "ZR", "RD", "SM", "SD",
];

const T_DEVICES: &[(&str, bool)] = &[("TS", true), ("TC", true), ("TN", false)];
const ST_DEVICES: &[(&str, bool)] = &[("STS", true), ("STC", true), ("STN", false)];
const C_DEVICES: &[(&str, bool)] = &[("CS", true), ("CC", true), ("CN", false)];
const LT_DEVICES: &[(&str, bool)] = &[("LTS", true), ("LTC", true), ("LTN", false)];
const LST_DEVICES: &[(&str, bool)] = &[("LSTS", true), ("LSTC", true), ("LSTN", false)];
const LC_DEVICES: &[(&str, bool)] = &[("LCS", true), ("LCC", true), ("LCN", false)];

pub(crate) fn resolve_profile_for_plc_profile(
    plc_profile: SlmpPlcProfile,
) -> SlmpDeviceRangeProfile {
    match plc_profile.address_profile() {
        SlmpPlcProfile::IqR => create_iqr_profile(),
        SlmpPlcProfile::IqRRj71En71 => create_iqr_profile(),
        SlmpPlcProfile::IqL => create_iql_profile(),
        SlmpPlcProfile::MxF => create_mxf_profile(),
        SlmpPlcProfile::MxR => create_mxr_profile(),
        SlmpPlcProfile::IqF => create_iqf_profile(),
        SlmpPlcProfile::QCpu => create_qcpu_profile(),
        SlmpPlcProfile::LCpu => create_lcpu_profile(),
        SlmpPlcProfile::QnU => create_qnu_profile(),
        SlmpPlcProfile::QnUDV => create_qnudv_profile(),
        SlmpPlcProfile::QCpuQj71E71100
        | SlmpPlcProfile::LCpuLj71E71100
        | SlmpPlcProfile::QnUQj71E71100
        | SlmpPlcProfile::QnUDVQj71E71100 => {
            unreachable!("unit profiles are mapped to their address profile")
        }
    }
}

pub(crate) async fn read_registers(
    client: &SlmpClient,
    profile: &SlmpDeviceRangeProfile,
) -> Result<BTreeMap<u16, u16>, SlmpError> {
    if profile.register_count == 0 {
        return Ok(BTreeMap::new());
    }

    let values = client
        .read_words_raw(
            SlmpDeviceAddress::new(SlmpDeviceCode::SD, u32::from(profile.register_start)),
            profile.register_count,
        )
        .await?;

    let mut map = BTreeMap::new();
    for (index, value) in values.into_iter().enumerate() {
        map.insert(profile.register_start + index as u16, value);
    }
    Ok(map)
}

pub(crate) fn build_catalog(
    profile: &SlmpDeviceRangeProfile,
    registers: &BTreeMap<u16, u16>,
) -> Result<SlmpDeviceRangeCatalog, SlmpError> {
    let mut entries = Vec::with_capacity(64);
    for item in ORDERED_ITEMS {
        let row = row_for(item);
        let spec = profile
            .rules
            .get(item)
            .ok_or_else(|| SlmpError::new(format!("Missing range rule for item {item}.")))?;
        let raw_point_count = evaluate_point_count(spec, registers)?;
        let (point_count, family_note) =
            apply_profile_point_count_cap(profile.plc_profile, item, raw_point_count);
        let upper_bound = point_count_to_upper_bound(point_count);
        let supported = spec.kind != SlmpRangeValueKind::Unsupported;
        for (device, is_bit_device) in row.devices {
            let notation = resolve_notation(profile.plc_profile, device, row.notation);
            entries.push(SlmpDeviceRangeEntry {
                device: (*device).to_string(),
                category: row.category,
                is_bit_device: *is_bit_device,
                supported,
                lower_bound: 0,
                upper_bound,
                point_count,
                address_range: format_address_range(device, notation, upper_bound),
                notation,
                source: spec.source.to_string(),
                notes: merge_notes(spec.notes, family_note),
            });
        }
    }

    Ok(SlmpDeviceRangeCatalog {
        model: profile_label(profile.plc_profile).to_string(),
        model_code: 0,
        has_model_code: false,
        plc_profile: profile.plc_profile,
        entries,
    })
}

pub(crate) fn build_catalog_for_plc_profile(
    plc_profile: SlmpPlcProfile,
    registers: &BTreeMap<u16, u16>,
) -> Result<SlmpDeviceRangeCatalog, SlmpError> {
    let profile = resolve_profile_for_plc_profile(plc_profile);
    let mut catalog = build_catalog(&profile, registers)?;
    catalog.model = profile_label(plc_profile).to_string();
    catalog.plc_profile = plc_profile;
    Ok(catalog)
}

pub(crate) fn replace_fixed_point_count(
    mut catalog: SlmpDeviceRangeCatalog,
    device: &str,
    point_count: u32,
    source: &str,
    note: &str,
) -> SlmpDeviceRangeCatalog {
    let upper_bound = point_count_to_upper_bound(Some(point_count));
    for entry in &mut catalog.entries {
        if entry.device == device {
            entry.upper_bound = upper_bound;
            entry.point_count = Some(point_count);
            entry.address_range = format_address_range(&entry.device, entry.notation, upper_bound);
            entry.source = source.to_string();
            entry.notes = Some(match &entry.notes {
                Some(existing) if !existing.is_empty() => format!("{existing} {note}"),
                _ => note.to_string(),
            });
        }
    }
    catalog
}

pub(crate) fn profile_label(plc_profile: SlmpPlcProfile) -> &'static str {
    match plc_profile {
        SlmpPlcProfile::IqR => "IQ-R",
        SlmpPlcProfile::IqRRj71En71 => "iQ-R via RJ71EN71",
        SlmpPlcProfile::IqL => "iQ-L",
        SlmpPlcProfile::MxF => "MX-F",
        SlmpPlcProfile::MxR => "MX-R",
        SlmpPlcProfile::IqF => "IQ-F",
        SlmpPlcProfile::QCpu => "QCPU",
        SlmpPlcProfile::QCpuQj71E71100 => "QCPU via QJ71E71-100",
        SlmpPlcProfile::LCpu => "LCPU",
        SlmpPlcProfile::LCpuLj71E71100 => "LCPU via LJ71E71-100",
        SlmpPlcProfile::QnU => "QnU",
        SlmpPlcProfile::QnUQj71E71100 => "QnU via QJ71E71-100",
        SlmpPlcProfile::QnUDV => "QnUDV",
        SlmpPlcProfile::QnUDVQj71E71100 => "QnUDV via QJ71E71-100",
    }
}

fn evaluate_point_count(
    spec: &SlmpRangeValueSpec,
    registers: &BTreeMap<u16, u16>,
) -> Result<Option<u32>, SlmpError> {
    Ok(match spec.kind {
        SlmpRangeValueKind::Unsupported | SlmpRangeValueKind::Undefined => None,
        SlmpRangeValueKind::Fixed => Some(spec.fixed_value),
        SlmpRangeValueKind::WordRegister => Some(read_word(registers, spec.register)?),
        SlmpRangeValueKind::DWordRegister => Some(read_dword(registers, spec.register)?),
        SlmpRangeValueKind::WordRegisterClipped => {
            Some(read_word(registers, spec.register)?.min(spec.clip_value))
        }
        SlmpRangeValueKind::DWordRegisterClipped => {
            Some(read_dword(registers, spec.register)?.min(spec.clip_value))
        }
    })
}

fn apply_profile_point_count_cap(
    plc_profile: SlmpPlcProfile,
    item: &str,
    point_count: Option<u32>,
) -> (Option<u32>, Option<&'static str>) {
    let Some(value) = point_count else {
        return (None, None);
    };
    let Some(cap) = profile_point_count_cap(plc_profile, item) else {
        return (Some(value), None);
    };
    if value > cap {
        (
            Some(cap),
            Some("iQ-R SD point count is capped to the fixed family maximum."),
        )
    } else {
        (Some(value), None)
    }
}

fn profile_point_count_cap(plc_profile: SlmpPlcProfile, item: &str) -> Option<u32> {
    if !matches!(plc_profile, SlmpPlcProfile::IqR | SlmpPlcProfile::IqRRj71En71) {
        return None;
    }

    Some(match item {
        "X" | "Y" => 12_288,
        "M" | "B" | "SB" => 94_674_944,
        "F" | "V" | "L" => 32_768,
        "T" | "ST" | "C" => 5_259_712,
        "LT" | "LST" => 1_479_296,
        "LC" => 2_784_544,
        "D" | "W" | "SW" => 5_917_184,
        _ => return None,
    })
}

fn merge_notes(primary: Option<&'static str>, extra: Option<&'static str>) -> Option<String> {
    match (primary, extra) {
        (Some(left), Some(right)) if !left.is_empty() && !right.is_empty() => {
            Some(format!("{left} {right}"))
        }
        (Some(left), _) if !left.is_empty() => Some(left.to_string()),
        (_, Some(right)) if !right.is_empty() => Some(right.to_string()),
        _ => None,
    }
}

fn point_count_to_upper_bound(point_count: Option<u32>) -> Option<u32> {
    point_count.and_then(|value| value.checked_sub(1))
}

fn resolve_notation(
    plc_profile: SlmpPlcProfile,
    device: &str,
    default_notation: SlmpDeviceRangeNotation,
) -> SlmpDeviceRangeNotation {
    if plc_profile == SlmpPlcProfile::IqF && matches!(device, "X" | "Y") {
        return SlmpDeviceRangeNotation::Octal;
    }

    default_notation
}

fn format_address_range(
    device: &str,
    notation: SlmpDeviceRangeNotation,
    upper_bound: Option<u32>,
) -> Option<String> {
    let upper_bound = upper_bound?;
    Some(match notation {
        SlmpDeviceRangeNotation::Decimal => format!("{device}0-{device}{upper_bound}"),
        SlmpDeviceRangeNotation::Octal => {
            let upper_text = format!("{upper_bound:o}");
            let width = std::cmp::max(3, upper_text.len());
            format!(
                "{device}{start:0width$o}-{device}{end:0width$o}",
                start = 0u32,
                end = upper_bound,
                width = width
            )
        }
        SlmpDeviceRangeNotation::Hexadecimal => {
            let width = std::cmp::max(3, format!("{upper_bound:X}").len());
            format!(
                "{device}{start:0width$X}-{device}{end:0width$X}",
                start = 0u32,
                end = upper_bound,
                width = width
            )
        }
    })
}

fn read_word(registers: &BTreeMap<u16, u16>, register: u16) -> Result<u32, SlmpError> {
    registers
        .get(&register)
        .copied()
        .map(u32::from)
        .ok_or_else(|| SlmpError::new(format!("Device-range resolver is missing SD{register}.")))
}

fn read_dword(registers: &BTreeMap<u16, u16>, register: u16) -> Result<u32, SlmpError> {
    let low = registers
        .get(&register)
        .copied()
        .ok_or_else(|| SlmpError::new(format!("Device-range resolver is missing SD{register}.")))?;
    let high_register = register + 1;
    let high = registers.get(&high_register).copied().ok_or_else(|| {
        SlmpError::new(format!(
            "Device-range resolver is missing SD{high_register}."
        ))
    })?;
    Ok(u32::from(low) | (u32::from(high) << 16))
}

fn row_for(item: &str) -> SlmpDeviceRangeRow {
    match item {
        "X" => SlmpDeviceRangeRow {
            category: SlmpDeviceRangeCategory::Bit,
            devices: &[("X", true)],
            notation: SlmpDeviceRangeNotation::Hexadecimal,
        },
        "Y" => SlmpDeviceRangeRow {
            category: SlmpDeviceRangeCategory::Bit,
            devices: &[("Y", true)],
            notation: SlmpDeviceRangeNotation::Hexadecimal,
        },
        "M" => SlmpDeviceRangeRow {
            category: SlmpDeviceRangeCategory::Bit,
            devices: &[("M", true)],
            notation: SlmpDeviceRangeNotation::Decimal,
        },
        "B" => SlmpDeviceRangeRow {
            category: SlmpDeviceRangeCategory::Bit,
            devices: &[("B", true)],
            notation: SlmpDeviceRangeNotation::Hexadecimal,
        },
        "SB" => SlmpDeviceRangeRow {
            category: SlmpDeviceRangeCategory::Bit,
            devices: &[("SB", true)],
            notation: SlmpDeviceRangeNotation::Hexadecimal,
        },
        "F" => SlmpDeviceRangeRow {
            category: SlmpDeviceRangeCategory::Bit,
            devices: &[("F", true)],
            notation: SlmpDeviceRangeNotation::Decimal,
        },
        "V" => SlmpDeviceRangeRow {
            category: SlmpDeviceRangeCategory::Bit,
            devices: &[("V", true)],
            notation: SlmpDeviceRangeNotation::Decimal,
        },
        "L" => SlmpDeviceRangeRow {
            category: SlmpDeviceRangeCategory::Bit,
            devices: &[("L", true)],
            notation: SlmpDeviceRangeNotation::Decimal,
        },
        "S" => SlmpDeviceRangeRow {
            category: SlmpDeviceRangeCategory::Bit,
            devices: &[("S", true)],
            notation: SlmpDeviceRangeNotation::Decimal,
        },
        "D" => SlmpDeviceRangeRow {
            category: SlmpDeviceRangeCategory::Word,
            devices: &[("D", false)],
            notation: SlmpDeviceRangeNotation::Decimal,
        },
        "W" => SlmpDeviceRangeRow {
            category: SlmpDeviceRangeCategory::Word,
            devices: &[("W", false)],
            notation: SlmpDeviceRangeNotation::Hexadecimal,
        },
        "SW" => SlmpDeviceRangeRow {
            category: SlmpDeviceRangeCategory::Word,
            devices: &[("SW", false)],
            notation: SlmpDeviceRangeNotation::Hexadecimal,
        },
        "R" => SlmpDeviceRangeRow {
            category: SlmpDeviceRangeCategory::Word,
            devices: &[("R", false)],
            notation: SlmpDeviceRangeNotation::Decimal,
        },
        "T" => multi(
            SlmpDeviceRangeCategory::TimerCounter,
            SlmpDeviceRangeNotation::Decimal,
            T_DEVICES,
        ),
        "ST" => multi(
            SlmpDeviceRangeCategory::TimerCounter,
            SlmpDeviceRangeNotation::Decimal,
            ST_DEVICES,
        ),
        "C" => multi(
            SlmpDeviceRangeCategory::TimerCounter,
            SlmpDeviceRangeNotation::Decimal,
            C_DEVICES,
        ),
        "LT" => multi(
            SlmpDeviceRangeCategory::TimerCounter,
            SlmpDeviceRangeNotation::Decimal,
            LT_DEVICES,
        ),
        "LST" => multi(
            SlmpDeviceRangeCategory::TimerCounter,
            SlmpDeviceRangeNotation::Decimal,
            LST_DEVICES,
        ),
        "LC" => multi(
            SlmpDeviceRangeCategory::TimerCounter,
            SlmpDeviceRangeNotation::Decimal,
            LC_DEVICES,
        ),
        "Z" => SlmpDeviceRangeRow {
            category: SlmpDeviceRangeCategory::Index,
            devices: &[("Z", false)],
            notation: SlmpDeviceRangeNotation::Decimal,
        },
        "LZ" => SlmpDeviceRangeRow {
            category: SlmpDeviceRangeCategory::Index,
            devices: &[("LZ", false)],
            notation: SlmpDeviceRangeNotation::Decimal,
        },
        "ZR" => SlmpDeviceRangeRow {
            category: SlmpDeviceRangeCategory::FileRegister,
            devices: &[("ZR", false)],
            notation: SlmpDeviceRangeNotation::Decimal,
        },
        "RD" => SlmpDeviceRangeRow {
            category: SlmpDeviceRangeCategory::FileRegister,
            devices: &[("RD", false)],
            notation: SlmpDeviceRangeNotation::Decimal,
        },
        "SM" => SlmpDeviceRangeRow {
            category: SlmpDeviceRangeCategory::Bit,
            devices: &[("SM", true)],
            notation: SlmpDeviceRangeNotation::Decimal,
        },
        "SD" => SlmpDeviceRangeRow {
            category: SlmpDeviceRangeCategory::Word,
            devices: &[("SD", false)],
            notation: SlmpDeviceRangeNotation::Decimal,
        },
        _ => unreachable!("unsupported item {item}"),
    }
}

fn multi(
    category: SlmpDeviceRangeCategory,
    notation: SlmpDeviceRangeNotation,
    devices: &'static [(&'static str, bool)],
) -> SlmpDeviceRangeRow {
    SlmpDeviceRangeRow {
        category,
        devices,
        notation,
    }
}

fn create_profile(
    plc_profile: SlmpPlcProfile,
    register_start: u16,
    register_count: u16,
    rules: Vec<(&'static str, SlmpRangeValueSpec)>,
) -> SlmpDeviceRangeProfile {
    let mut map = BTreeMap::new();
    for (item, spec) in rules {
        map.insert(item, spec);
    }
    SlmpDeviceRangeProfile {
        plc_profile,
        register_start,
        register_count,
        rules: map,
    }
}

fn fixed(value: u32, source: &'static str) -> SlmpRangeValueSpec {
    SlmpRangeValueSpec {
        kind: SlmpRangeValueKind::Fixed,
        register: 0,
        fixed_value: value,
        clip_value: 0,
        source,
        notes: None,
    }
}

fn word_register(
    register: u16,
    source: &'static str,
    notes: Option<&'static str>,
) -> SlmpRangeValueSpec {
    SlmpRangeValueSpec {
        kind: SlmpRangeValueKind::WordRegister,
        register,
        fixed_value: 0,
        clip_value: 0,
        source,
        notes,
    }
}

fn dword_register(
    register: u16,
    source: &'static str,
    notes: Option<&'static str>,
) -> SlmpRangeValueSpec {
    SlmpRangeValueSpec {
        kind: SlmpRangeValueKind::DWordRegister,
        register,
        fixed_value: 0,
        clip_value: 0,
        source,
        notes,
    }
}

fn word_register_clipped(
    register: u16,
    clip_value: u32,
    source: &'static str,
    notes: Option<&'static str>,
) -> SlmpRangeValueSpec {
    SlmpRangeValueSpec {
        kind: SlmpRangeValueKind::WordRegisterClipped,
        register,
        fixed_value: 0,
        clip_value,
        source,
        notes,
    }
}

fn dword_register_clipped(
    register: u16,
    clip_value: u32,
    source: &'static str,
    notes: Option<&'static str>,
) -> SlmpRangeValueSpec {
    SlmpRangeValueSpec {
        kind: SlmpRangeValueKind::DWordRegisterClipped,
        register,
        fixed_value: 0,
        clip_value,
        source,
        notes,
    }
}

fn undefined(notes: &'static str) -> SlmpRangeValueSpec {
    SlmpRangeValueSpec {
        kind: SlmpRangeValueKind::Undefined,
        register: 0,
        fixed_value: 0,
        clip_value: 0,
        source: "No finite upper-bound register",
        notes: Some(notes),
    }
}

fn unsupported(notes: &'static str) -> SlmpRangeValueSpec {
    SlmpRangeValueSpec {
        kind: SlmpRangeValueKind::Unsupported,
        register: 0,
        fixed_value: 0,
        clip_value: 0,
        source: "Not supported",
        notes: Some(notes),
    }
}

fn create_iqr_profile() -> SlmpDeviceRangeProfile {
    create_profile(
        SlmpPlcProfile::IqR,
        260,
        50,
        vec![
            ("X", dword_register(260, "SD260-SD261 (32-bit)", None)),
            ("Y", dword_register(262, "SD262-SD263 (32-bit)", None)),
            ("M", dword_register(264, "SD264-SD265 (32-bit)", None)),
            ("B", dword_register(266, "SD266-SD267 (32-bit)", None)),
            ("SB", dword_register(268, "SD268-SD269 (32-bit)", None)),
            ("F", dword_register(270, "SD270-SD271 (32-bit)", None)),
            ("V", dword_register(272, "SD272-SD273 (32-bit)", None)),
            ("L", dword_register(274, "SD274-SD275 (32-bit)", None)),
            ("S", dword_register(276, "SD276-SD277 (32-bit)", None)),
            ("D", dword_register(280, "SD280-SD281 (32-bit)", None)),
            ("W", dword_register(282, "SD282-SD283 (32-bit)", None)),
            ("SW", dword_register(284, "SD284-SD285 (32-bit)", None)),
            (
                "R",
                dword_register_clipped(
                    306,
                    32768,
                    "SD306-SD307 (32-bit)",
                    Some("Upper bound is clipped to 32768."),
                ),
            ),
            ("T", dword_register(288, "SD288-SD289 (32-bit)", None)),
            ("ST", dword_register(290, "SD290-SD291 (32-bit)", None)),
            ("C", dword_register(292, "SD292-SD293 (32-bit)", None)),
            ("LT", dword_register(294, "SD294-SD295 (32-bit)", None)),
            ("LST", dword_register(296, "SD296-SD297 (32-bit)", None)),
            ("LC", dword_register(298, "SD298-SD299 (32-bit)", None)),
            ("Z", word_register(300, "SD300", None)),
            ("LZ", word_register(302, "SD302", None)),
            ("ZR", dword_register(306, "SD306-SD307 (32-bit)", None)),
            ("RD", dword_register(308, "SD308-SD309 (32-bit)", None)),
            ("SM", fixed(4096, "Fixed family limit")),
            ("SD", fixed(4096, "Fixed family limit")),
        ],
    )
}

fn create_iql_profile() -> SlmpDeviceRangeProfile {
    create_profile(
        SlmpPlcProfile::IqL,
        260,
        50,
        vec![
            ("X", dword_register(260, "SD260-SD261 (32-bit)", None)),
            ("Y", dword_register(262, "SD262-SD263 (32-bit)", None)),
            ("M", dword_register(264, "SD264-SD265 (32-bit)", None)),
            ("B", dword_register(266, "SD266-SD267 (32-bit)", None)),
            ("SB", dword_register(268, "SD268-SD269 (32-bit)", None)),
            ("F", dword_register(270, "SD270-SD271 (32-bit)", None)),
            ("V", dword_register(272, "SD272-SD273 (32-bit)", None)),
            ("L", dword_register(274, "SD274-SD275 (32-bit)", None)),
            ("S", dword_register(276, "SD276-SD277 (32-bit)", None)),
            ("D", dword_register(280, "SD280-SD281 (32-bit)", None)),
            ("W", dword_register(282, "SD282-SD283 (32-bit)", None)),
            ("SW", dword_register(284, "SD284-SD285 (32-bit)", None)),
            (
                "R",
                dword_register_clipped(
                    306,
                    32768,
                    "SD306-SD307 (32-bit)",
                    Some("Upper bound is clipped to 32768."),
                ),
            ),
            ("T", dword_register(288, "SD288-SD289 (32-bit)", None)),
            ("ST", dword_register(290, "SD290-SD291 (32-bit)", None)),
            ("C", dword_register(292, "SD292-SD293 (32-bit)", None)),
            ("LT", dword_register(294, "SD294-SD295 (32-bit)", None)),
            ("LST", dword_register(296, "SD296-SD297 (32-bit)", None)),
            ("LC", dword_register(298, "SD298-SD299 (32-bit)", None)),
            ("Z", word_register(300, "SD300", None)),
            ("LZ", word_register(302, "SD302", None)),
            ("ZR", dword_register(306, "SD306-SD307 (32-bit)", None)),
            ("RD", dword_register(308, "SD308-SD309 (32-bit)", None)),
            ("SM", fixed(4096, "Fixed family limit")),
            ("SD", fixed(4096, "Fixed family limit")),
        ],
    )
}

fn create_mxf_profile() -> SlmpDeviceRangeProfile {
    let mut profile = create_iqr_profile();
    profile.plc_profile = SlmpPlcProfile::MxF;
    profile
        .rules
        .insert("SM", fixed(10000, "Fixed family limit"));
    profile
        .rules
        .insert("SD", fixed(10000, "Fixed family limit"));
    profile
}

fn create_mxr_profile() -> SlmpDeviceRangeProfile {
    let mut profile = create_iqr_profile();
    profile.plc_profile = SlmpPlcProfile::MxR;
    profile
        .rules
        .insert("SM", fixed(4496, "Fixed family limit"));
    profile
        .rules
        .insert("SD", fixed(4496, "Fixed family limit"));
    profile
}

fn create_iqf_profile() -> SlmpDeviceRangeProfile {
    create_profile(
        SlmpPlcProfile::IqF,
        260,
        46,
        vec![
            (
                "X",
                dword_register(
                    260,
                    "SD260-SD261 (32-bit)",
                    Some("Manual addressing for iQ-F X devices is octal."),
                ),
            ),
            (
                "Y",
                dword_register(
                    262,
                    "SD262-SD263 (32-bit)",
                    Some("Manual addressing for iQ-F Y devices is octal."),
                ),
            ),
            ("M", dword_register(264, "SD264-SD265 (32-bit)", None)),
            ("B", dword_register(266, "SD266-SD267 (32-bit)", None)),
            ("SB", dword_register(268, "SD268-SD269 (32-bit)", None)),
            ("F", dword_register(270, "SD270-SD271 (32-bit)", None)),
            ("V", unsupported("Not supported on iQ-F.")),
            ("L", dword_register(274, "SD274-SD275 (32-bit)", None)),
            ("S", dword_register(276, "SD276-SD277 (32-bit)", None)),
            ("D", dword_register(280, "SD280-SD281 (32-bit)", None)),
            ("W", dword_register(282, "SD282-SD283 (32-bit)", None)),
            ("SW", dword_register(284, "SD284-SD285 (32-bit)", None)),
            ("R", dword_register(304, "SD304-SD305 (32-bit)", None)),
            ("T", dword_register(288, "SD288-SD289 (32-bit)", None)),
            ("ST", dword_register(290, "SD290-SD291 (32-bit)", None)),
            ("C", dword_register(292, "SD292-SD293 (32-bit)", None)),
            ("LT", unsupported("Not supported on iQ-F.")),
            ("LST", unsupported("Not supported on iQ-F.")),
            ("LC", dword_register(298, "SD298-SD299 (32-bit)", None)),
            ("Z", word_register(300, "SD300", None)),
            ("LZ", word_register(302, "SD302", None)),
            ("ZR", unsupported("Not supported on iQ-F.")),
            ("RD", unsupported("Not supported on iQ-F.")),
            ("SM", fixed(10000, "Fixed family limit")),
            ("SD", fixed(12000, "Fixed family limit")),
        ],
    )
}

fn create_qcpu_profile() -> SlmpDeviceRangeProfile {
    create_profile(
        SlmpPlcProfile::QCpu,
        290,
        15,
        vec![
            ("X", word_register(290, "SD290", None)),
            ("Y", word_register(291, "SD291", None)),
            (
                "M",
                word_register_clipped(
                    292,
                    32768,
                    "SD292",
                    Some("Upper bound is clipped to 32768."),
                ),
            ),
            (
                "B",
                word_register_clipped(
                    294,
                    32768,
                    "SD294",
                    Some("Upper bound is clipped to 32768."),
                ),
            ),
            ("SB", word_register(296, "SD296", None)),
            ("F", word_register(295, "SD295", None)),
            ("V", word_register(297, "SD297", None)),
            ("L", word_register(293, "SD293", None)),
            ("S", word_register(298, "SD298", None)),
            (
                "D",
                word_register_clipped(
                    302,
                    32768,
                    "SD302",
                    Some("Upper bound is clipped to 32768 and excludes extended area."),
                ),
            ),
            (
                "W",
                word_register_clipped(
                    303,
                    32768,
                    "SD303",
                    Some("Upper bound is clipped to 32768 and excludes extended area."),
                ),
            ),
            ("SW", word_register(304, "SD304", None)),
            ("R", fixed(32768, "Fixed family limit")),
            ("T", word_register(299, "SD299", None)),
            ("ST", word_register(300, "SD300", None)),
            ("C", word_register(301, "SD301", None)),
            ("LT", unsupported("Not supported on QCPU.")),
            ("LST", unsupported("Not supported on QCPU.")),
            ("LC", unsupported("Not supported on QCPU.")),
            ("Z", fixed(10, "Fixed family limit")),
            ("LZ", unsupported("Not supported on QCPU.")),
            (
                "ZR",
                undefined("No finite upper-bound register is defined for QCPU ZR."),
            ),
            ("RD", unsupported("Not supported on QCPU.")),
            ("SM", fixed(1024, "Fixed family limit")),
            ("SD", fixed(1024, "Fixed family limit")),
        ],
    )
}

fn create_lcpu_like_profile(
    plc_profile: SlmpPlcProfile,
    profile_name: &'static str,
) -> SlmpDeviceRangeProfile {
    let unsupported_note = match plc_profile {
        SlmpPlcProfile::LCpu => "Not supported on LCPU.",
        SlmpPlcProfile::QnU => "Not supported on QnU.",
        SlmpPlcProfile::QnUDV => "Not supported on QnUDV.",
        _ => profile_name,
    };

    create_profile(
        plc_profile,
        286,
        26,
        vec![
            ("X", word_register(290, "SD290", None)),
            ("Y", word_register(291, "SD291", None)),
            ("M", dword_register(286, "SD286-SD287 (32-bit)", None)),
            ("B", dword_register(288, "SD288-SD289 (32-bit)", None)),
            ("SB", word_register(296, "SD296", None)),
            ("F", word_register(295, "SD295", None)),
            ("V", word_register(297, "SD297", None)),
            ("L", word_register(293, "SD293", None)),
            ("S", word_register(298, "SD298", None)),
            ("D", dword_register(308, "SD308-SD309 (32-bit)", None)),
            ("W", dword_register(310, "SD310-SD311 (32-bit)", None)),
            ("SW", word_register(304, "SD304", None)),
            ("R", dword_register(306, "SD306-SD307 (32-bit)", None)),
            ("T", word_register(299, "SD299", None)),
            ("ST", word_register(300, "SD300", None)),
            ("C", word_register(301, "SD301", None)),
            ("LT", unsupported(unsupported_note)),
            ("LST", unsupported(unsupported_note)),
            ("LC", unsupported(unsupported_note)),
            ("Z", fixed(20, "Fixed family limit")),
            ("LZ", unsupported(unsupported_note)),
            ("ZR", dword_register(306, "SD306-SD307 (32-bit)", None)),
            ("RD", unsupported(unsupported_note)),
            ("SM", fixed(2048, "Fixed family limit")),
            ("SD", fixed(2048, "Fixed family limit")),
        ],
    )
}

fn create_lcpu_profile() -> SlmpDeviceRangeProfile {
    create_lcpu_like_profile(SlmpPlcProfile::LCpu, "LCPU")
}

fn create_qnu_profile() -> SlmpDeviceRangeProfile {
    create_lcpu_like_profile(SlmpPlcProfile::QnU, "QnU")
}

fn create_qnudv_profile() -> SlmpDeviceRangeProfile {
    create_lcpu_like_profile(SlmpPlcProfile::QnUDV, "QnUDV")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_snapshot(profile: &SlmpDeviceRangeProfile) -> BTreeMap<u16, u16> {
        let mut snapshot = BTreeMap::new();
        for offset in 0..profile.register_count {
            snapshot.insert(profile.register_start + offset, 0);
        }
        snapshot
    }

    fn entry<'a>(catalog: &'a SlmpDeviceRangeCatalog, device: &str) -> &'a SlmpDeviceRangeEntry {
        catalog
            .entries
            .iter()
            .find(|item| item.device == device)
            .unwrap()
    }

    fn insert_dword(snapshot: &mut BTreeMap<u16, u16>, register: u16, value: u32) {
        snapshot.insert(register, value as u16);
        snapshot.insert(register + 1, (value >> 16) as u16);
    }

    fn canonical_device_range_rules() -> serde_json::Value {
        serde_json::from_str(include_str!(
            "../tests/fixtures/slmp_device_range_rules.json"
        ))
        .unwrap()
    }

    fn canonical_rule_value(rule: &serde_json::Value) -> u32 {
        let kind = rule["kind"].as_str().unwrap();
        if kind.ends_with("clipped") {
            return rule["clip_value"].as_u64().unwrap() as u32 + 5;
        }
        123
    }

    fn canonical_register_snapshot(
        profile: &serde_json::Value,
        only_item: Option<&str>,
    ) -> BTreeMap<u16, u16> {
        let start = profile["register_start"].as_u64().unwrap() as u16;
        let count = profile["register_count"].as_u64().unwrap() as u16;
        let mut snapshot = BTreeMap::new();
        for offset in 0..count {
            snapshot.insert(start + offset, 0);
        }

        let rules = profile["rules"].as_object().unwrap();
        let selected_rules: Vec<&serde_json::Value> = match only_item {
            Some(item) => vec![rules.get(item).unwrap()],
            None => rules.values().collect(),
        };
        for rule in selected_rules {
            let Some(register) = rule.get("register").and_then(serde_json::Value::as_u64) else {
                continue;
            };
            let register = register as u16;
            let value = canonical_rule_value(rule);
            let kind = rule["kind"].as_str().unwrap();
            if kind.starts_with("dword-register") {
                insert_dword(&mut snapshot, register, value);
            } else if kind.starts_with("word-register") {
                snapshot.insert(register, value as u16);
            }
        }
        snapshot
    }

    fn canonical_expected_point_count(rule: &serde_json::Value) -> Option<u32> {
        let kind = rule["kind"].as_str().unwrap();
        match kind {
            "unsupported" | "undefined" => None,
            "fixed" => Some(rule["fixed_value"].as_u64().unwrap() as u32),
            _ if kind.ends_with("clipped") => {
                Some(canonical_rule_value(rule).min(rule["clip_value"].as_u64().unwrap() as u32))
            }
            _ => Some(canonical_rule_value(rule)),
        }
    }

    fn canonical_notation(value: &str) -> SlmpDeviceRangeNotation {
        match value {
            "base10" => SlmpDeviceRangeNotation::Decimal,
            "base8" => SlmpDeviceRangeNotation::Octal,
            "base16" => SlmpDeviceRangeNotation::Hexadecimal,
            other => panic!("unsupported notation {other}"),
        }
    }

    #[test]
    fn build_catalog_matches_canonical_device_range_rules_fixture() {
        let payload = canonical_device_range_rules();
        let rows = payload["rows"].as_object().unwrap();
        let profiles = payload["profiles"].as_object().unwrap();
        let notation_overrides = payload["notation_overrides"].as_object().unwrap();

        for (profile_name, profile_payload) in profiles {
            let plc_profile = SlmpPlcProfile::parse_label(profile_name).unwrap();

            for (item, rule) in profile_payload["rules"].as_object().unwrap() {
                let snapshot = canonical_register_snapshot(profile_payload, Some(item));
                let catalog = build_catalog_for_plc_profile(plc_profile, &snapshot).unwrap();
                let row = rows.get(item).unwrap();
                let expected_supported = rule["kind"].as_str().unwrap() != "unsupported";
                let expected_point_count = canonical_expected_point_count(rule);
                let expected_notation = notation_overrides
                    .get(profile_name)
                    .and_then(|profile| profile.get(item))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_else(|| row["notation"].as_str().unwrap());

                for device in row["devices"].as_array().unwrap() {
                    let device_name = device["device"].as_str().unwrap();
                    let entry = entry(&catalog, device_name);
                    assert_eq!(
                        entry.supported, expected_supported,
                        "{profile_name} {device_name}"
                    );
                    assert_eq!(
                        entry.point_count, expected_point_count,
                        "{profile_name} {device_name}"
                    );
                    assert_eq!(
                        entry.notation,
                        canonical_notation(expected_notation),
                        "{profile_name} {device_name}"
                    );
                }
            }
        }
    }

    #[test]
    fn build_catalog_qcpu_clips_and_leaves_conditional_bounds_open() {
        let plc_profile = SlmpPlcProfile::QCpu;
        let profile = resolve_profile_for_plc_profile(plc_profile);
        let mut snapshot = create_snapshot(&profile);
        snapshot.insert(290, 123);
        snapshot.insert(292, 50000);
        snapshot.insert(299, 90);
        snapshot.insert(302, 50000);
        snapshot.insert(303, 60000);

        let catalog = build_catalog_for_plc_profile(plc_profile, &snapshot).unwrap();

        assert_eq!(catalog.plc_profile, SlmpPlcProfile::QCpu);
        assert_eq!(entry(&catalog, "X").point_count, Some(123));
        assert_eq!(entry(&catalog, "X").upper_bound, Some(122));
        assert_eq!(
            entry(&catalog, "X").address_range.as_deref(),
            Some("X000-X07A")
        );
        assert_eq!(entry(&catalog, "M").point_count, Some(32768));
        assert_eq!(entry(&catalog, "M").upper_bound, Some(32767));
        assert_eq!(entry(&catalog, "D").point_count, Some(32768));
        assert_eq!(entry(&catalog, "D").upper_bound, Some(32767));
        assert_eq!(entry(&catalog, "TS").point_count, Some(90));
        assert_eq!(entry(&catalog, "TS").upper_bound, Some(89));
        assert_eq!(entry(&catalog, "TN").point_count, Some(90));
        assert_eq!(entry(&catalog, "TN").upper_bound, Some(89));
        assert!(entry(&catalog, "ZR").supported);
        assert_eq!(entry(&catalog, "ZR").point_count, None);
        assert_eq!(entry(&catalog, "ZR").upper_bound, None);
        assert_eq!(entry(&catalog, "ZR").address_range, None);
        assert_eq!(entry(&catalog, "Z").point_count, Some(10));
        assert_eq!(entry(&catalog, "Z").upper_bound, Some(9));
        assert_eq!(entry(&catalog, "Z").address_range.as_deref(), Some("Z0-Z9"));
    }

    #[test]
    fn unit_profile_uses_base_range_rules_but_keeps_selected_identity() {
        let plc_profile = SlmpPlcProfile::QCpuQj71E71100;
        let profile = resolve_profile_for_plc_profile(plc_profile);
        let mut snapshot = create_snapshot(&profile);
        snapshot.insert(290, 123);

        let catalog = build_catalog_for_plc_profile(plc_profile, &snapshot).unwrap();

        assert_eq!(catalog.model, "QCPU via QJ71E71-100");
        assert_eq!(catalog.plc_profile, SlmpPlcProfile::QCpuQj71E71100);
        assert_eq!(entry(&catalog, "X").point_count, Some(123));
        assert_eq!(
            entry(&catalog, "X").address_range.as_deref(),
            Some("X000-X07A")
        );
        assert_eq!(entry(&catalog, "Z").point_count, Some(10));
    }

    #[test]
    fn build_catalog_iqr_reads_dword_registers_and_caps_family_maximums() {
        let plc_profile = SlmpPlcProfile::IqR;
        let profile = resolve_profile_for_plc_profile(plc_profile);
        let mut snapshot = create_snapshot(&profile);
        insert_dword(&mut snapshot, 260, 12_289);
        insert_dword(&mut snapshot, 264, 94_674_945);
        insert_dword(&mut snapshot, 266, 94_674_945);
        insert_dword(&mut snapshot, 270, 32_769);
        insert_dword(&mut snapshot, 280, 5_917_185);
        insert_dword(&mut snapshot, 282, 5_917_185);
        insert_dword(&mut snapshot, 284, 5_917_185);
        insert_dword(&mut snapshot, 288, 5_259_713);
        insert_dword(&mut snapshot, 294, 1_479_297);
        insert_dword(&mut snapshot, 296, 1_479_297);
        insert_dword(&mut snapshot, 298, 2_784_545);
        insert_dword(&mut snapshot, 306, 0x0002_0001);

        let catalog = build_catalog_for_plc_profile(plc_profile, &snapshot).unwrap();

        assert_eq!(entry(&catalog, "X").point_count, Some(12_288));
        assert_eq!(entry(&catalog, "X").upper_bound, Some(12_287));
        assert_eq!(
            entry(&catalog, "X").address_range.as_deref(),
            Some("X0000-X2FFF")
        );
        assert_eq!(entry(&catalog, "M").point_count, Some(94_674_944));
        assert_eq!(entry(&catalog, "M").upper_bound, Some(94_674_943));
        assert_eq!(entry(&catalog, "B").point_count, Some(94_674_944));
        assert_eq!(
            entry(&catalog, "B").address_range.as_deref(),
            Some("B0000000-B5A49FFF")
        );
        assert_eq!(entry(&catalog, "F").point_count, Some(32_768));
        assert_eq!(entry(&catalog, "D").point_count, Some(5_917_184));
        assert_eq!(entry(&catalog, "W").point_count, Some(5_917_184));
        assert_eq!(
            entry(&catalog, "SW").address_range.as_deref(),
            Some("SW000000-SW5A49FF")
        );
        assert_eq!(entry(&catalog, "TN").point_count, Some(5_259_712));
        assert_eq!(entry(&catalog, "LTN").point_count, Some(1_479_296));
        assert_eq!(entry(&catalog, "LSTN").point_count, Some(1_479_296));
        assert_eq!(entry(&catalog, "LCN").point_count, Some(2_784_544));
        assert_eq!(entry(&catalog, "R").point_count, Some(32768));
        assert_eq!(entry(&catalog, "R").upper_bound, Some(32767));
        assert_eq!(
            entry(&catalog, "R").address_range.as_deref(),
            Some("R0-R32767")
        );
    }

    #[test]
    fn build_catalog_iqf_formats_x_and_y_in_octal() {
        let plc_profile = SlmpPlcProfile::IqF;
        let profile = resolve_profile_for_plc_profile(plc_profile);
        let mut snapshot = create_snapshot(&profile);
        snapshot.insert(260, 1024);
        snapshot.insert(261, 0);
        snapshot.insert(262, 1024);
        snapshot.insert(263, 0);

        let catalog = build_catalog_for_plc_profile(plc_profile, &snapshot).unwrap();

        assert_eq!(
            entry(&catalog, "X").notation,
            SlmpDeviceRangeNotation::Octal
        );
        assert_eq!(entry(&catalog, "X").point_count, Some(1024));
        assert_eq!(entry(&catalog, "X").upper_bound, Some(1023));
        assert_eq!(
            entry(&catalog, "X").address_range.as_deref(),
            Some("X0000-X1777")
        );

        assert_eq!(
            entry(&catalog, "Y").notation,
            SlmpDeviceRangeNotation::Octal
        );
        assert_eq!(entry(&catalog, "Y").point_count, Some(1024));
        assert_eq!(entry(&catalog, "Y").upper_bound, Some(1023));
        assert_eq!(
            entry(&catalog, "Y").address_range.as_deref(),
            Some("Y0000-Y1777")
        );
    }

    #[test]
    fn build_catalog_mx_profiles_keep_s_supported_from_sd276() {
        for plc_profile in [SlmpPlcProfile::MxF, SlmpPlcProfile::MxR] {
            let profile = resolve_profile_for_plc_profile(plc_profile);
            let mut snapshot = create_snapshot(&profile);
            insert_dword(&mut snapshot, 276, 123);

            let catalog = build_catalog_for_plc_profile(plc_profile, &snapshot).unwrap();

            assert!(entry(&catalog, "S").supported);
            assert_eq!(entry(&catalog, "S").source, "SD276-SD277 (32-bit)");
            assert_eq!(entry(&catalog, "S").point_count, Some(123));
            assert_eq!(
                entry(&catalog, "S").address_range.as_deref(),
                Some("S0-S122")
            );
        }
    }

    #[test]
    fn build_catalog_qnu_uses_sd300_for_st_family_and_fixed_z_limit() {
        let plc_profile = SlmpPlcProfile::QnU;
        let profile = resolve_profile_for_plc_profile(plc_profile);
        let mut snapshot = create_snapshot(&profile);
        snapshot.insert(300, 16);
        snapshot.insert(301, 1024);

        let catalog = build_catalog_for_plc_profile(plc_profile, &snapshot).unwrap();

        assert_eq!(catalog.plc_profile, SlmpPlcProfile::QnU);
        assert_eq!(entry(&catalog, "STS").point_count, Some(16));
        assert_eq!(entry(&catalog, "STS").upper_bound, Some(15));
        assert_eq!(
            entry(&catalog, "STS").address_range.as_deref(),
            Some("STS0-STS15")
        );
        assert_eq!(entry(&catalog, "STC").point_count, Some(16));
        assert_eq!(entry(&catalog, "STC").upper_bound, Some(15));
        assert_eq!(
            entry(&catalog, "STC").address_range.as_deref(),
            Some("STC0-STC15")
        );
        assert_eq!(entry(&catalog, "STN").point_count, Some(16));
        assert_eq!(entry(&catalog, "STN").upper_bound, Some(15));
        assert_eq!(
            entry(&catalog, "STN").address_range.as_deref(),
            Some("STN0-STN15")
        );
        assert_eq!(entry(&catalog, "CS").point_count, Some(1024));
        assert_eq!(entry(&catalog, "CS").upper_bound, Some(1023));
        assert_eq!(
            entry(&catalog, "CS").address_range.as_deref(),
            Some("CS0-CS1023")
        );
        assert_eq!(entry(&catalog, "Z").point_count, Some(20));
        assert_eq!(entry(&catalog, "Z").upper_bound, Some(19));
        assert_eq!(
            entry(&catalog, "Z").address_range.as_deref(),
            Some("Z0-Z19")
        );
    }
}
