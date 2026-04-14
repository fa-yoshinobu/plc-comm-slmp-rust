use crate::client::SlmpClient;
use crate::error::SlmpError;
use crate::model::{SlmpDeviceAddress, SlmpDeviceCode, SlmpTypeNameInfo};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SlmpDeviceRangeFamily {
    IqR,
    MxF,
    MxR,
    IqF,
    QCpu,
    LCpu,
    QnU,
    QnUDV,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SlmpDeviceRangeCategory {
    Bit,
    Word,
    TimerCounter,
    Index,
    FileRefresh,
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
    /// PLC model text when known, or a user-selected family label.
    pub model: String,
    /// Model code when known. Zero when the caller selected the family explicitly.
    pub model_code: u16,
    pub has_model_code: bool,
    pub family: SlmpDeviceRangeFamily,
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
    pub(crate) family: SlmpDeviceRangeFamily,
    pub(crate) register_start: u16,
    pub(crate) register_count: u16,
    rules: BTreeMap<&'static str, SlmpRangeValueSpec>,
}

const ORDERED_ITEMS: &[&str] = &[
    "X", "Y", "M", "B", "SB", "F", "V", "L", "S", "D", "W", "SW", "R", "T", "ST", "C", "LT",
    "LST", "LC", "Z", "LZ", "ZR", "RD", "SM", "SD",
];

const T_DEVICES: &[(&str, bool)] = &[("TS", true), ("TC", true), ("TN", false)];
const ST_DEVICES: &[(&str, bool)] = &[("STS", true), ("STC", true), ("STN", false)];
const C_DEVICES: &[(&str, bool)] = &[("CS", true), ("CC", true), ("CN", false)];
const LT_DEVICES: &[(&str, bool)] = &[("LTS", true), ("LTC", true), ("LTN", false)];
const LST_DEVICES: &[(&str, bool)] = &[("LSTS", true), ("LSTC", true), ("LSTN", false)];
const LC_DEVICES: &[(&str, bool)] = &[("LCS", true), ("LCC", true), ("LCN", false)];

pub(crate) fn normalize_model(model: &str) -> String {
    model.trim().trim_end_matches('\0').trim().to_ascii_uppercase()
}

pub(crate) fn resolve_family(type_info: &SlmpTypeNameInfo) -> Result<SlmpDeviceRangeFamily, SlmpError> {
    if type_info.has_model_code {
        if let Some(family) = family_from_model_code(type_info.model_code) {
            return Ok(family);
        }
    }

    let normalized = normalize_model(&type_info.model);
    if let Some(family) = family_from_model_name(&normalized) {
        return Ok(family);
    }

    let code_text = if type_info.has_model_code {
        format!("0x{:04X}", type_info.model_code)
    } else {
        "none".to_string()
    };
    Err(SlmpError::new(format!(
        "Unsupported PLC model for device-range rules: model='{normalized}', model_code={code_text}."
    )))
}

pub(crate) fn resolve_profile(type_info: &SlmpTypeNameInfo) -> Result<SlmpDeviceRangeProfile, SlmpError> {
    Ok(match resolve_family(type_info)? {
        SlmpDeviceRangeFamily::IqR => create_iqr_profile(),
        SlmpDeviceRangeFamily::MxF => create_mxf_profile(),
        SlmpDeviceRangeFamily::MxR => create_mxr_profile(),
        SlmpDeviceRangeFamily::IqF => create_iqf_profile(),
        SlmpDeviceRangeFamily::QCpu => create_qcpu_profile(),
        SlmpDeviceRangeFamily::LCpu => create_lcpu_profile(),
        SlmpDeviceRangeFamily::QnU => create_qnu_profile(),
        SlmpDeviceRangeFamily::QnUDV => create_qnudv_profile(),
    })
}

pub(crate) fn resolve_profile_for_family(family: SlmpDeviceRangeFamily) -> SlmpDeviceRangeProfile {
    match family {
        SlmpDeviceRangeFamily::IqR => create_iqr_profile(),
        SlmpDeviceRangeFamily::MxF => create_mxf_profile(),
        SlmpDeviceRangeFamily::MxR => create_mxr_profile(),
        SlmpDeviceRangeFamily::IqF => create_iqf_profile(),
        SlmpDeviceRangeFamily::QCpu => create_qcpu_profile(),
        SlmpDeviceRangeFamily::LCpu => create_lcpu_profile(),
        SlmpDeviceRangeFamily::QnU => create_qnu_profile(),
        SlmpDeviceRangeFamily::QnUDV => create_qnudv_profile(),
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
    type_info: &SlmpTypeNameInfo,
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
        let point_count = evaluate_point_count(spec, registers)?;
        let upper_bound = point_count_to_upper_bound(point_count);
        let supported = spec.kind != SlmpRangeValueKind::Unsupported;
        for (device, is_bit_device) in row.devices {
            let notation = resolve_notation(profile.family, device, row.notation);
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
                notes: spec.notes.map(str::to_string),
            });
        }
    }

    Ok(SlmpDeviceRangeCatalog {
        model: normalize_model(&type_info.model),
        model_code: type_info.model_code,
        has_model_code: type_info.has_model_code,
        family: profile.family,
        entries,
    })
}

pub(crate) fn build_catalog_for_family(
    family: SlmpDeviceRangeFamily,
    registers: &BTreeMap<u16, u16>,
) -> Result<SlmpDeviceRangeCatalog, SlmpError> {
    let profile = resolve_profile_for_family(family);
    build_catalog(
        &SlmpTypeNameInfo {
            model: family_label(family).to_string(),
            model_code: 0,
            has_model_code: false,
        },
        &profile,
        registers,
    )
}

pub(crate) fn family_label(family: SlmpDeviceRangeFamily) -> &'static str {
    match family {
        SlmpDeviceRangeFamily::IqR => "IQ-R",
        SlmpDeviceRangeFamily::MxF => "MX-F",
        SlmpDeviceRangeFamily::MxR => "MX-R",
        SlmpDeviceRangeFamily::IqF => "IQ-F",
        SlmpDeviceRangeFamily::QCpu => "QCPU",
        SlmpDeviceRangeFamily::LCpu => "LCPU",
        SlmpDeviceRangeFamily::QnU => "QnU",
        SlmpDeviceRangeFamily::QnUDV => "QnUDV",
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

fn point_count_to_upper_bound(point_count: Option<u32>) -> Option<u32> {
    point_count.and_then(|value| value.checked_sub(1))
}

fn resolve_notation(
    family: SlmpDeviceRangeFamily,
    device: &str,
    default_notation: SlmpDeviceRangeNotation,
) -> SlmpDeviceRangeNotation {
    if family == SlmpDeviceRangeFamily::IqF && matches!(device, "X" | "Y") {
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
    let high = registers
        .get(&high_register)
        .copied()
        .ok_or_else(|| SlmpError::new(format!("Device-range resolver is missing SD{high_register}.")))?;
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
            category: SlmpDeviceRangeCategory::FileRefresh,
            devices: &[("ZR", false)],
            notation: SlmpDeviceRangeNotation::Decimal,
        },
        "RD" => SlmpDeviceRangeRow {
            category: SlmpDeviceRangeCategory::FileRefresh,
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

fn family_from_model_code(model_code: u16) -> Option<SlmpDeviceRangeFamily> {
    Some(match model_code {
        0x0250 | 0x0251 | 0x0041 | 0x0042 | 0x0043 | 0x0044 | 0x004B | 0x004C | 0x0230 => {
            SlmpDeviceRangeFamily::QCpu
        }
        0x0260 | 0x0261 | 0x0262 | 0x0263 | 0x0268 | 0x0269 | 0x026A | 0x0266 | 0x026B
        | 0x0267 | 0x026C | 0x026D | 0x026E => SlmpDeviceRangeFamily::QnU,
        0x0366 | 0x0367 | 0x0368 | 0x036A | 0x036C => SlmpDeviceRangeFamily::QnUDV,
        0x0543 | 0x0541 | 0x0544 | 0x0545 | 0x0542 | 0x48C0 | 0x48C1 | 0x48C2 | 0x48C3
        | 0x0641 => SlmpDeviceRangeFamily::LCpu,
        0x48A0 | 0x48A1 | 0x48A2 | 0x4800 | 0x4801 | 0x4802 | 0x4803 | 0x4804 | 0x4805
        | 0x4806 | 0x4807 | 0x4808 | 0x4809 | 0x4841 | 0x4842 | 0x4843 | 0x4844 | 0x4851
        | 0x4852 | 0x4853 | 0x4854 | 0x4891 | 0x4892 | 0x4893 | 0x4894 | 0x4820 | 0x4E01
        | 0x4860 | 0x4861 | 0x4862 | 0x0642 => SlmpDeviceRangeFamily::IqR,
        0x48E9 | 0x48EA | 0x48EB | 0x48EE | 0x48EF => SlmpDeviceRangeFamily::MxR,
        0x4A21 | 0x4A23 | 0x4A24 | 0x4A29 | 0x4A2B | 0x4A2C | 0x4A31 | 0x4A33 | 0x4A34
        | 0x4A41 | 0x4A43 | 0x4A44 | 0x4A49 | 0x4A4B | 0x4A4C | 0x4A51 | 0x4A53 | 0x4A54
        | 0x4A91 | 0x4A92 | 0x4A93 | 0x4A99 | 0x4A9A | 0x4A9B | 0x4AA9 | 0x4AB1 | 0x4AB9
        | 0x4B0D | 0x4B0E | 0x4B0F | 0x4B14 | 0x4B15 | 0x4B16 | 0x4B1B | 0x4B1C | 0x4B1D
        | 0x4B4E | 0x4B4F | 0x4B50 | 0x4B51 | 0x4B55 | 0x4B56 | 0x4B57 | 0x4B58 | 0x4B5C
        | 0x4B5D | 0x4B5E | 0x4B5F => SlmpDeviceRangeFamily::IqF,
        _ => return None,
    })
}

fn family_from_model_name(model: &str) -> Option<SlmpDeviceRangeFamily> {
    const PREFIXES: &[(&str, SlmpDeviceRangeFamily)] = &[
        ("Q04UDPV", SlmpDeviceRangeFamily::QnUDV),
        ("Q06UDPV", SlmpDeviceRangeFamily::QnUDV),
        ("Q13UDPV", SlmpDeviceRangeFamily::QnUDV),
        ("Q26UDPV", SlmpDeviceRangeFamily::QnUDV),
        ("Q03UDV", SlmpDeviceRangeFamily::QnUDV),
        ("Q04UDV", SlmpDeviceRangeFamily::QnUDV),
        ("Q06UDV", SlmpDeviceRangeFamily::QnUDV),
        ("Q13UDV", SlmpDeviceRangeFamily::QnUDV),
        ("Q26UDV", SlmpDeviceRangeFamily::QnUDV),
        ("Q00UJ", SlmpDeviceRangeFamily::QnU),
        ("Q00U", SlmpDeviceRangeFamily::QnU),
        ("Q01U", SlmpDeviceRangeFamily::QnU),
        ("Q02U", SlmpDeviceRangeFamily::QnU),
        ("Q03UD", SlmpDeviceRangeFamily::QnU),
        ("Q04UD", SlmpDeviceRangeFamily::QnU),
        ("Q06UD", SlmpDeviceRangeFamily::QnU),
        ("Q10UD", SlmpDeviceRangeFamily::QnU),
        ("Q13UD", SlmpDeviceRangeFamily::QnU),
        ("Q20UD", SlmpDeviceRangeFamily::QnU),
        ("Q26UD", SlmpDeviceRangeFamily::QnU),
        ("Q50UDEH", SlmpDeviceRangeFamily::QnU),
        ("Q100UDEH", SlmpDeviceRangeFamily::QnU),
        ("FX5UC", SlmpDeviceRangeFamily::IqF),
        ("FX5UJ", SlmpDeviceRangeFamily::IqF),
        ("FX5U", SlmpDeviceRangeFamily::IqF),
        ("FX5S", SlmpDeviceRangeFamily::IqF),
        ("MXF100-", SlmpDeviceRangeFamily::MxF),
        ("MXF", SlmpDeviceRangeFamily::MxF),
        ("MXR", SlmpDeviceRangeFamily::MxR),
        ("LJ72GF15-T2", SlmpDeviceRangeFamily::LCpu),
        ("L02SCPU", SlmpDeviceRangeFamily::LCpu),
        ("L02CPU", SlmpDeviceRangeFamily::LCpu),
        ("L06CPU", SlmpDeviceRangeFamily::LCpu),
        ("L26CPU", SlmpDeviceRangeFamily::LCpu),
        ("L04HCPU", SlmpDeviceRangeFamily::LCpu),
        ("L08HCPU", SlmpDeviceRangeFamily::LCpu),
        ("L16HCPU", SlmpDeviceRangeFamily::LCpu),
        ("L32HCPU", SlmpDeviceRangeFamily::LCpu),
        ("RJ72GF15-T2", SlmpDeviceRangeFamily::IqR),
        ("NZ2GF-ETB", SlmpDeviceRangeFamily::IqR),
        ("MI5122-VW", SlmpDeviceRangeFamily::IqR),
        ("QS001CPU", SlmpDeviceRangeFamily::QCpu),
        ("Q00JCPU", SlmpDeviceRangeFamily::QCpu),
        ("Q00CPU", SlmpDeviceRangeFamily::QCpu),
        ("Q01CPU", SlmpDeviceRangeFamily::QCpu),
        ("Q02", SlmpDeviceRangeFamily::QCpu),
        ("Q06", SlmpDeviceRangeFamily::QCpu),
        ("Q12", SlmpDeviceRangeFamily::QCpu),
        ("Q25", SlmpDeviceRangeFamily::QCpu),
        ("R", SlmpDeviceRangeFamily::IqR),
    ];

    PREFIXES
        .iter()
        .find(|(prefix, _)| model.starts_with(prefix))
        .map(|(_, family)| *family)
}

fn create_profile(
    family: SlmpDeviceRangeFamily,
    register_start: u16,
    register_count: u16,
    rules: Vec<(&'static str, SlmpRangeValueSpec)>,
) -> SlmpDeviceRangeProfile {
    let mut map = BTreeMap::new();
    for (item, spec) in rules {
        map.insert(item, spec);
    }
    SlmpDeviceRangeProfile {
        family,
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

fn word_register(register: u16, source: &'static str, notes: Option<&'static str>) -> SlmpRangeValueSpec {
    SlmpRangeValueSpec {
        kind: SlmpRangeValueKind::WordRegister,
        register,
        fixed_value: 0,
        clip_value: 0,
        source,
        notes,
    }
}

fn dword_register(register: u16, source: &'static str, notes: Option<&'static str>) -> SlmpRangeValueSpec {
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
        SlmpDeviceRangeFamily::IqR,
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
    profile.family = SlmpDeviceRangeFamily::MxF;
    profile.rules.insert("S", unsupported("Not supported on MX-F."));
    profile.rules.insert("SM", fixed(10000, "Fixed family limit"));
    profile.rules.insert("SD", fixed(10000, "Fixed family limit"));
    profile
}

fn create_mxr_profile() -> SlmpDeviceRangeProfile {
    let mut profile = create_iqr_profile();
    profile.family = SlmpDeviceRangeFamily::MxR;
    profile.rules.insert("S", unsupported("Not supported on MX-R."));
    profile.rules.insert("SM", fixed(4496, "Fixed family limit"));
    profile.rules.insert("SD", fixed(4496, "Fixed family limit"));
    profile
}

fn create_iqf_profile() -> SlmpDeviceRangeProfile {
    create_profile(
        SlmpDeviceRangeFamily::IqF,
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
            ("S", unsupported("Not supported on iQ-F.")),
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
        SlmpDeviceRangeFamily::QCpu,
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

fn create_lcpu_like_profile(family: SlmpDeviceRangeFamily, family_name: &'static str) -> SlmpDeviceRangeProfile {
    let unsupported_note = match family {
        SlmpDeviceRangeFamily::LCpu => "Not supported on LCPU.",
        SlmpDeviceRangeFamily::QnU => "Not supported on QnU.",
        SlmpDeviceRangeFamily::QnUDV => "Not supported on QnUDV.",
        _ => family_name,
    };

    create_profile(
        family,
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
            (
                "R",
                dword_register_clipped(
                    306,
                    32768,
                    "SD306-SD307 (32-bit)",
                    Some("Upper bound is clipped to 32768."),
                ),
            ),
            ("T", word_register(299, "SD299", None)),
            ("ST", word_register(300, "SD300", None)),
            ("C", word_register(301, "SD301", None)),
            ("LT", unsupported(unsupported_note)),
            ("LST", unsupported(unsupported_note)),
            ("LC", unsupported(unsupported_note)),
            (
                "Z",
                word_register(
                    305,
                    "SD305",
                    Some("Requires ZZ = FFFFh for the reported upper bound."),
                ),
            ),
            ("LZ", unsupported(unsupported_note)),
            ("ZR", dword_register(306, "SD306-SD307 (32-bit)", None)),
            ("RD", unsupported(unsupported_note)),
            ("SM", fixed(2048, "Fixed family limit")),
            ("SD", fixed(2048, "Fixed family limit")),
        ],
    )
}

fn create_lcpu_profile() -> SlmpDeviceRangeProfile {
    create_lcpu_like_profile(SlmpDeviceRangeFamily::LCpu, "LCPU")
}

fn create_qnu_profile() -> SlmpDeviceRangeProfile {
    create_lcpu_like_profile(SlmpDeviceRangeFamily::QnU, "QnU")
}

fn create_qnudv_profile() -> SlmpDeviceRangeProfile {
    create_lcpu_like_profile(SlmpDeviceRangeFamily::QnUDV, "QnUDV")
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
        catalog.entries.iter().find(|item| item.device == device).unwrap()
    }

    #[test]
    fn normalize_model_trims_and_upcases() {
        assert_eq!(normalize_model(" R120PCPU\0 "), "R120PCPU");
        assert_eq!(normalize_model("fx5u-32mr/ds"), "FX5U-32MR/DS");
    }

    #[test]
    fn resolve_family_uses_model_code_and_name_rules() {
        let qnudv = resolve_family(&SlmpTypeNameInfo {
            model: "Q03UDVCPU".to_string(),
            model_code: 0x0366,
            has_model_code: true,
        })
        .unwrap();
        let mxf = resolve_family(&SlmpTypeNameInfo {
            model: "MXF100-8-N32".to_string(),
            model_code: 0,
            has_model_code: false,
        })
        .unwrap();

        assert_eq!(qnudv, SlmpDeviceRangeFamily::QnUDV);
        assert_eq!(mxf, SlmpDeviceRangeFamily::MxF);
    }

    #[test]
    fn build_catalog_qcpu_clips_and_leaves_conditional_bounds_open() {
        let type_info = SlmpTypeNameInfo {
            model: "Q00CPU".to_string(),
            model_code: 0x0251,
            has_model_code: true,
        };
        let profile = resolve_profile(&type_info).unwrap();
        let mut snapshot = create_snapshot(&profile);
        snapshot.insert(290, 123);
        snapshot.insert(292, 50000);
        snapshot.insert(299, 90);
        snapshot.insert(302, 50000);
        snapshot.insert(303, 60000);

        let catalog = build_catalog(&type_info, &profile, &snapshot).unwrap();

        assert_eq!(catalog.family, SlmpDeviceRangeFamily::QCpu);
        assert_eq!(entry(&catalog, "X").point_count, Some(123));
        assert_eq!(entry(&catalog, "X").upper_bound, Some(122));
        assert_eq!(entry(&catalog, "X").address_range.as_deref(), Some("X000-X07A"));
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
    fn build_catalog_iqr_reads_dword_registers_and_expands_long_families() {
        let type_info = SlmpTypeNameInfo {
            model: "R120PCPU".to_string(),
            model_code: 0x4844,
            has_model_code: true,
        };
        let profile = resolve_profile(&type_info).unwrap();
        let mut snapshot = create_snapshot(&profile);
        snapshot.insert(260, 0x5678);
        snapshot.insert(261, 0x1234);
        snapshot.insert(294, 0x4321);
        snapshot.insert(295, 0x0001);
        snapshot.insert(306, 0x0001);
        snapshot.insert(307, 0x0002);

        let catalog = build_catalog(&type_info, &profile, &snapshot).unwrap();

        assert_eq!(entry(&catalog, "X").point_count, Some(0x1234_5678));
        assert_eq!(entry(&catalog, "X").upper_bound, Some(0x1234_5677));
        assert_eq!(
            entry(&catalog, "X").address_range.as_deref(),
            Some("X00000000-X12345677")
        );
        assert_eq!(entry(&catalog, "LTN").point_count, Some(0x0001_4321));
        assert_eq!(entry(&catalog, "LTN").upper_bound, Some(0x0001_4320));
        assert_eq!(entry(&catalog, "LTS").point_count, Some(0x0001_4321));
        assert_eq!(entry(&catalog, "LTS").upper_bound, Some(0x0001_4320));
        assert_eq!(entry(&catalog, "R").point_count, Some(32768));
        assert_eq!(entry(&catalog, "R").upper_bound, Some(32767));
        assert_eq!(entry(&catalog, "R").address_range.as_deref(), Some("R0-R32767"));
    }

    #[test]
    fn build_catalog_iqf_formats_x_and_y_in_octal() {
        let type_info = SlmpTypeNameInfo {
            model: "FX5UC-32MT/D".to_string(),
            model_code: 0x4A91,
            has_model_code: true,
        };
        let profile = resolve_profile(&type_info).unwrap();
        let mut snapshot = create_snapshot(&profile);
        snapshot.insert(260, 1024);
        snapshot.insert(261, 0);
        snapshot.insert(262, 1024);
        snapshot.insert(263, 0);

        let catalog = build_catalog(&type_info, &profile, &snapshot).unwrap();

        assert_eq!(entry(&catalog, "X").notation, SlmpDeviceRangeNotation::Octal);
        assert_eq!(entry(&catalog, "X").point_count, Some(1024));
        assert_eq!(entry(&catalog, "X").upper_bound, Some(1023));
        assert_eq!(entry(&catalog, "X").address_range.as_deref(), Some("X0000-X1777"));

        assert_eq!(entry(&catalog, "Y").notation, SlmpDeviceRangeNotation::Octal);
        assert_eq!(entry(&catalog, "Y").point_count, Some(1024));
        assert_eq!(entry(&catalog, "Y").upper_bound, Some(1023));
        assert_eq!(entry(&catalog, "Y").address_range.as_deref(), Some("Y0000-Y1777"));
    }

    #[test]
    fn build_catalog_qnu_uses_sd300_for_st_family_and_sd305_for_z() {
        let type_info = SlmpTypeNameInfo {
            model: "Q03UDECPU".to_string(),
            model_code: 0x0268,
            has_model_code: true,
        };
        let profile = resolve_profile(&type_info).unwrap();
        let mut snapshot = create_snapshot(&profile);
        snapshot.insert(300, 16);
        snapshot.insert(301, 1024);
        snapshot.insert(305, 20);

        let catalog = build_catalog(&type_info, &profile, &snapshot).unwrap();

        assert_eq!(catalog.family, SlmpDeviceRangeFamily::QnU);
        assert_eq!(entry(&catalog, "STS").point_count, Some(16));
        assert_eq!(entry(&catalog, "STS").upper_bound, Some(15));
        assert_eq!(entry(&catalog, "STS").address_range.as_deref(), Some("STS0-STS15"));
        assert_eq!(entry(&catalog, "STC").point_count, Some(16));
        assert_eq!(entry(&catalog, "STC").upper_bound, Some(15));
        assert_eq!(entry(&catalog, "STC").address_range.as_deref(), Some("STC0-STC15"));
        assert_eq!(entry(&catalog, "STN").point_count, Some(16));
        assert_eq!(entry(&catalog, "STN").upper_bound, Some(15));
        assert_eq!(entry(&catalog, "STN").address_range.as_deref(), Some("STN0-STN15"));
        assert_eq!(entry(&catalog, "CS").point_count, Some(1024));
        assert_eq!(entry(&catalog, "CS").upper_bound, Some(1023));
        assert_eq!(entry(&catalog, "CS").address_range.as_deref(), Some("CS0-CS1023"));
        assert_eq!(entry(&catalog, "Z").point_count, Some(20));
        assert_eq!(entry(&catalog, "Z").upper_bound, Some(19));
        assert_eq!(entry(&catalog, "Z").address_range.as_deref(), Some("Z0-Z19"));
    }
}
