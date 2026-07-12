use std::fmt;
use std::sync::OnceLock;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SlmpTransportMode {
    Tcp,
    Udp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SlmpFrameType {
    Frame3E,
    Frame4E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SlmpCompatibilityMode {
    Legacy,
    Iqr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SlmpRemoteMode {
    Normal,
    Force,
}

impl SlmpRemoteMode {
    pub const fn wire_value(self) -> u16 {
        match self {
            Self::Normal => 0x0001,
            Self::Force => 0x0003,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SlmpRemoteClearMode {
    NoClear,
    ClearExceptLatch,
    ClearAll,
}

impl SlmpRemoteClearMode {
    pub const fn wire_value(self) -> u16 {
        match self {
            Self::NoClear => 0,
            Self::ClearExceptLatch => 1,
            Self::ClearAll => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SlmpPlcProfile {
    IqF,
    IqR,
    IqRRj71En71,
    IqL,
    MxF,
    MxR,
    QCpu,
    QCpuQj71E71100,
    LCpu,
    LCpuLj71E71100,
    QnU,
    QnUQj71E71100,
    QnUDV,
    QnUDVQj71E71100,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlmpPlcProfileDefaults {
    pub frame_type: SlmpFrameType,
    pub compatibility_mode: SlmpCompatibilityMode,
}

/// Canonical metadata used to select and describe one PLC profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlmpPlcProfileDescriptor {
    pub canonical_name: &'static str,
    pub display_name: &'static str,
    pub connectable: bool,
    pub base_profile: Option<&'static str>,
}

impl SlmpPlcProfile {
    pub const ALL: [Self; 14] = [
        Self::IqF,
        Self::IqR,
        Self::IqRRj71En71,
        Self::IqL,
        Self::MxF,
        Self::MxR,
        Self::QCpu,
        Self::QCpuQj71E71100,
        Self::LCpu,
        Self::LCpuLj71E71100,
        Self::QnU,
        Self::QnUQj71E71100,
        Self::QnUDV,
        Self::QnUDVQj71E71100,
    ];

    /// Return the profiles that can be used to open a connection.
    ///
    /// The abstract `melsec:qcpu` base profile is intentionally excluded;
    /// callers must choose its concrete module route instead.
    pub fn available_connection_profiles() -> &'static [Self] {
        &[
            Self::IqF,
            Self::IqR,
            Self::IqRRj71En71,
            Self::IqL,
            Self::MxF,
            Self::MxR,
            Self::QCpuQj71E71100,
            Self::LCpu,
            Self::LCpuLj71E71100,
            Self::QnU,
            Self::QnUQj71E71100,
            Self::QnUDV,
            Self::QnUDVQj71E71100,
        ]
    }

    pub fn canonical_name(self) -> &'static str {
        match self {
            Self::IqF => "melsec:iq-f",
            Self::IqR => "melsec:iq-r",
            Self::IqRRj71En71 => "melsec:iq-r:rj71en71",
            Self::IqL => "melsec:iq-l",
            Self::MxF => "melsec:mx-f",
            Self::MxR => "melsec:mx-r",
            Self::QCpu => "melsec:qcpu",
            Self::QCpuQj71E71100 => "melsec:qcpu:qj71e71-100",
            Self::LCpu => "melsec:lcpu",
            Self::LCpuLj71E71100 => "melsec:lcpu:lj71e71-100",
            Self::QnU => "melsec:qnu",
            Self::QnUQj71E71100 => "melsec:qnu:qj71e71-100",
            Self::QnUDV => "melsec:qnudv",
            Self::QnUDVQj71E71100 => "melsec:qnudv:qj71e71-100",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::IqF => "MELSEC iQ-F (built-in)",
            Self::IqR => "MELSEC iQ-R (built-in)",
            Self::IqRRj71En71 => "MELSEC iQ-R (RJ71EN71)",
            Self::IqL => "MELSEC iQ-L (built-in)",
            Self::MxF => "MELSEC MX-F (built-in)",
            Self::MxR => "MELSEC MX-R (built-in)",
            Self::QCpu => "MELSEC-Q (base profile)",
            Self::QCpuQj71E71100 => "MELSEC-Q (QJ71E71-100)",
            Self::LCpu => "MELSEC-L (built-in)",
            Self::LCpuLj71E71100 => "MELSEC-L (LJ71E71-100)",
            Self::QnU => "MELSEC QnU (built-in)",
            Self::QnUQj71E71100 => "MELSEC QnU (QJ71E71-100)",
            Self::QnUDV => "MELSEC QnUDV (built-in)",
            Self::QnUDVQj71E71100 => "MELSEC QnUDV (QJ71E71-100)",
        }
    }

    pub fn base_profile(self) -> Option<&'static str> {
        match self {
            Self::IqRRj71En71 => Some("melsec:iq-r"),
            Self::MxF | Self::MxR => Some("melsec:iq-r"),
            Self::QCpu => Some("melsec:qnu"),
            Self::QCpuQj71E71100 => Some("melsec:qcpu"),
            Self::LCpuLj71E71100 => Some("melsec:lcpu"),
            Self::QnUQj71E71100 => Some("melsec:qnu"),
            Self::QnUDVQj71E71100 => Some("melsec:qnudv"),
            _ => None,
        }
    }

    pub fn parse_label(value: &str) -> Option<Self> {
        let normalized = value.trim();
        match normalized {
            "melsec:iq-f" => Some(Self::IqF),
            "melsec:iq-r" => Some(Self::IqR),
            "melsec:iq-r:rj71en71" => Some(Self::IqRRj71En71),
            "melsec:iq-l" => Some(Self::IqL),
            "melsec:mx-f" => Some(Self::MxF),
            "melsec:mx-r" => Some(Self::MxR),
            "melsec:qcpu" => Some(Self::QCpu),
            "melsec:qcpu:qj71e71-100" => Some(Self::QCpuQj71E71100),
            "melsec:lcpu" => Some(Self::LCpu),
            "melsec:lcpu:lj71e71-100" => Some(Self::LCpuLj71E71100),
            "melsec:qnu" => Some(Self::QnU),
            "melsec:qnu:qj71e71-100" => Some(Self::QnUQj71E71100),
            "melsec:qnudv" => Some(Self::QnUDV),
            "melsec:qnudv:qj71e71-100" => Some(Self::QnUDVQj71E71100),
            _ => None,
        }
    }

    pub fn defaults(self) -> SlmpPlcProfileDefaults {
        match self {
            Self::IqF => SlmpPlcProfileDefaults {
                frame_type: SlmpFrameType::Frame3E,
                compatibility_mode: SlmpCompatibilityMode::Legacy,
            },
            Self::IqR | Self::IqRRj71En71 | Self::IqL | Self::MxF | Self::MxR => {
                SlmpPlcProfileDefaults {
                    frame_type: SlmpFrameType::Frame4E,
                    compatibility_mode: SlmpCompatibilityMode::Iqr,
                }
            }
            Self::QCpu | Self::LCpu | Self::QnU | Self::QnUDV => SlmpPlcProfileDefaults {
                frame_type: SlmpFrameType::Frame3E,
                compatibility_mode: SlmpCompatibilityMode::Legacy,
            },
            Self::QCpuQj71E71100
            | Self::LCpuLj71E71100
            | Self::QnUQj71E71100
            | Self::QnUDVQj71E71100 => SlmpPlcProfileDefaults {
                frame_type: SlmpFrameType::Frame4E,
                compatibility_mode: SlmpCompatibilityMode::Legacy,
            },
        }
    }

    pub fn address_profile(self) -> Self {
        match self {
            Self::IqRRj71En71 => Self::IqR,
            Self::QCpuQj71E71100 => Self::QCpu,
            Self::LCpuLj71E71100 => Self::LCpu,
            Self::QnUQj71E71100 => Self::QnU,
            Self::QnUDVQj71E71100 => Self::QnUDV,
            _ => self,
        }
    }

    pub fn range_profile(self) -> Self {
        self
    }

    pub fn is_base_profile(self) -> bool {
        matches!(self, Self::QCpu)
    }

    pub fn validate_connection_selectable(self) -> Result<(), crate::error::SlmpError> {
        if self.is_base_profile() {
            return Err(crate::error::SlmpError::new(
                "melsec:qcpu is a base profile; use melsec:qcpu:qj71e71-100.",
            ));
        }
        Ok(())
    }

    pub fn uses_iqf_xy_octal(self) -> bool {
        matches!(self.address_profile(), Self::IqF)
    }

    pub fn uses_iqr_protocol(self) -> bool {
        matches!(
            self.defaults().compatibility_mode,
            SlmpCompatibilityMode::Iqr
        )
    }
}

/// Return all canonical profiles with display, connection, and base-profile metadata.
///
/// The abstract `melsec:qcpu` entry is included with `connectable` set to
/// `false` so selectors can explain why it cannot be opened directly.
pub fn plc_profile_descriptors() -> &'static [SlmpPlcProfileDescriptor] {
    static PROFILE_DESCRIPTORS: OnceLock<Vec<SlmpPlcProfileDescriptor>> = OnceLock::new();

    PROFILE_DESCRIPTORS
        .get_or_init(|| {
            SlmpPlcProfile::ALL
                .iter()
                .map(|profile| SlmpPlcProfileDescriptor {
                    canonical_name: profile.canonical_name(),
                    display_name: profile.display_name(),
                    connectable: !profile.is_base_profile(),
                    base_profile: profile.base_profile(),
                })
                .collect()
        })
        .as_slice()
}

#[cfg(test)]
mod plc_profile_descriptor_tests {
    use super::*;
    use serde_json::Value;
    use std::collections::BTreeSet;

    #[test]
    fn profile_descriptors_match_canonical_profile_metadata() {
        let fixture = include_str!("../tests/fixtures/slmp_ethernet_profiles.json");
        let expected: Value = serde_json::from_str(fixture).unwrap();
        let expected_profiles = expected["profiles"].as_object().unwrap();
        let descriptors = plc_profile_descriptors();
        let expected_names: BTreeSet<_> = expected_profiles.keys().map(String::as_str).collect();
        let actual_names: BTreeSet<_> = descriptors
            .iter()
            .map(|descriptor| descriptor.canonical_name)
            .collect();

        assert_eq!(actual_names, expected_names);

        for descriptor in descriptors {
            let profile = &expected_profiles[descriptor.canonical_name];
            assert_eq!(
                descriptor.display_name,
                profile["display_name"].as_str().unwrap()
            );
            assert_eq!(
                descriptor.connectable,
                profile["role"].as_str() != Some("base")
            );
            assert_eq!(descriptor.base_profile, profile["base_profile"].as_str());
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SlmpTrafficStats {
    pub request_count: u64,
    pub tx_bytes: u64,
    pub rx_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum SlmpCommand {
    DeviceRead = 0x0401,
    DeviceWrite = 0x1401,
    DeviceReadRandom = 0x0403,
    DeviceWriteRandom = 0x1402,
    DeviceReadBlock = 0x0406,
    DeviceWriteBlock = 0x1406,
    MonitorRegister = 0x0801,
    Monitor = 0x0802,
    ReadTypeName = 0x0101,
    LabelArrayRead = 0x041A,
    LabelArrayWrite = 0x141A,
    LabelReadRandom = 0x041C,
    LabelWriteRandom = 0x141B,
    MemoryRead = 0x0613,
    MemoryWrite = 0x1613,
    ExtendUnitRead = 0x0601,
    ExtendUnitWrite = 0x1601,
    RemoteRun = 0x1001,
    RemoteStop = 0x1002,
    RemotePause = 0x1003,
    RemoteLatchClear = 0x1005,
    RemoteReset = 0x1006,
    RemotePasswordUnlock = 0x1630,
    RemotePasswordLock = 0x1631,
    SelfTest = 0x0619,
    ClearError = 0x1617,
}

impl SlmpCommand {
    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[repr(u16)]
pub enum SlmpDeviceCode {
    SM = 0x0091,
    SD = 0x00A9,
    X = 0x009C,
    Y = 0x009D,
    M = 0x0090,
    L = 0x0092,
    F = 0x0093,
    V = 0x0094,
    B = 0x00A0,
    S = 0x0098,
    D = 0x00A8,
    W = 0x00B4,
    TS = 0x00C1,
    TC = 0x00C0,
    TN = 0x00C2,
    LTS = 0x0051,
    LTC = 0x0050,
    LTN = 0x0052,
    STS = 0x00C7,
    STC = 0x00C6,
    STN = 0x00C8,
    LSTS = 0x0059,
    LSTC = 0x0058,
    LSTN = 0x005A,
    LCC = 0x0054,
    LCS = 0x0055,
    LCN = 0x0056,
    CS = 0x00C4,
    CC = 0x00C3,
    CN = 0x00C5,
    SB = 0x00A1,
    SW = 0x00B5,
    DX = 0x00A2,
    DY = 0x00A3,
    Z = 0x00CC,
    LZ = 0x0062,
    R = 0x00AF,
    ZR = 0x00B0,
    RD = 0x002C,
    G = 0x00AB,
    HG = 0x002E,
}

impl SlmpDeviceCode {
    pub fn as_u16(self) -> u16 {
        self as u16
    }

    pub fn as_u8(self) -> u8 {
        (self.as_u16() & 0x00FF) as u8
    }

    pub fn prefix(self) -> &'static str {
        match self {
            Self::SM => "SM",
            Self::SD => "SD",
            Self::X => "X",
            Self::Y => "Y",
            Self::M => "M",
            Self::L => "L",
            Self::F => "F",
            Self::V => "V",
            Self::B => "B",
            Self::S => "S",
            Self::D => "D",
            Self::W => "W",
            Self::TS => "TS",
            Self::TC => "TC",
            Self::TN => "TN",
            Self::LTS => "LTS",
            Self::LTC => "LTC",
            Self::LTN => "LTN",
            Self::STS => "STS",
            Self::STC => "STC",
            Self::STN => "STN",
            Self::LSTS => "LSTS",
            Self::LSTC => "LSTC",
            Self::LSTN => "LSTN",
            Self::LCC => "LCC",
            Self::LCS => "LCS",
            Self::LCN => "LCN",
            Self::CS => "CS",
            Self::CC => "CC",
            Self::CN => "CN",
            Self::SB => "SB",
            Self::SW => "SW",
            Self::DX => "DX",
            Self::DY => "DY",
            Self::Z => "Z",
            Self::LZ => "LZ",
            Self::R => "R",
            Self::ZR => "ZR",
            Self::RD => "RD",
            Self::G => "G",
            Self::HG => "HG",
        }
    }

    pub fn is_hex_addressed(self) -> bool {
        matches!(
            self,
            Self::X | Self::Y | Self::B | Self::W | Self::SB | Self::SW | Self::DX | Self::DY
        )
    }

    pub fn is_bit_device(self) -> bool {
        matches!(
            self,
            Self::SM
                | Self::X
                | Self::Y
                | Self::M
                | Self::L
                | Self::F
                | Self::V
                | Self::B
                | Self::S
                | Self::TS
                | Self::TC
                | Self::LTS
                | Self::LTC
                | Self::STS
                | Self::STC
                | Self::LSTS
                | Self::LSTC
                | Self::CS
                | Self::CC
                | Self::LCS
                | Self::LCC
                | Self::SB
                | Self::DX
                | Self::DY
        )
    }

    pub fn is_word_device(self) -> bool {
        matches!(
            self,
            Self::SD
                | Self::D
                | Self::W
                | Self::TN
                | Self::LTN
                | Self::STN
                | Self::LSTN
                | Self::CN
                | Self::LCN
                | Self::SW
                | Self::Z
                | Self::LZ
                | Self::R
                | Self::ZR
                | Self::RD
                | Self::G
                | Self::HG
        )
    }

    pub fn is_word_batchable(self) -> bool {
        matches!(
            self,
            Self::SD
                | Self::D
                | Self::W
                | Self::TN
                | Self::LTN
                | Self::STN
                | Self::LSTN
                | Self::CN
                | Self::LCN
                | Self::SW
                | Self::Z
                | Self::LZ
                | Self::R
                | Self::ZR
                | Self::RD
        )
    }

    pub fn parse_prefix(prefix: &str) -> Option<Self> {
        match prefix {
            "LSTS" => Some(Self::LSTS),
            "LSTC" => Some(Self::LSTC),
            "LSTN" => Some(Self::LSTN),
            "LTS" => Some(Self::LTS),
            "LTC" => Some(Self::LTC),
            "LTN" => Some(Self::LTN),
            "STS" => Some(Self::STS),
            "STC" => Some(Self::STC),
            "STN" => Some(Self::STN),
            "SM" => Some(Self::SM),
            "SD" => Some(Self::SD),
            "TS" => Some(Self::TS),
            "TC" => Some(Self::TC),
            "TN" => Some(Self::TN),
            "CS" => Some(Self::CS),
            "CC" => Some(Self::CC),
            "CN" => Some(Self::CN),
            "SB" => Some(Self::SB),
            "SW" => Some(Self::SW),
            "DX" => Some(Self::DX),
            "DY" => Some(Self::DY),
            "LCS" => Some(Self::LCS),
            "LCC" => Some(Self::LCC),
            "LCN" => Some(Self::LCN),
            "LZ" => Some(Self::LZ),
            "ZR" => Some(Self::ZR),
            "RD" => Some(Self::RD),
            "HG" => Some(Self::HG),
            "X" => Some(Self::X),
            "Y" => Some(Self::Y),
            "M" => Some(Self::M),
            "L" => Some(Self::L),
            "F" => Some(Self::F),
            "V" => Some(Self::V),
            "B" => Some(Self::B),
            "S" => Some(Self::S),
            "D" => Some(Self::D),
            "W" => Some(Self::W),
            "Z" => Some(Self::Z),
            "R" => Some(Self::R),
            "G" => Some(Self::G),
            _ => None,
        }
    }
}

impl fmt::Display for SlmpDeviceCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.prefix())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlmpModuleIo;

impl SlmpModuleIo {
    /// SLMP request destination module I/O numbers from specification SH080956.
    pub const CONTROL_SYSTEM_CPU: u16 = 0x03D0;
    pub const STANDBY_SYSTEM_CPU: u16 = 0x03D1;
    pub const SYSTEM_A_CPU: u16 = 0x03D2;
    pub const SYSTEM_B_CPU: u16 = 0x03D3;
    pub const MULTIPLE_CPU_1: u16 = 0x03E0;
    pub const MULTIPLE_CPU_2: u16 = 0x03E1;
    pub const MULTIPLE_CPU_3: u16 = 0x03E2;
    pub const MULTIPLE_CPU_4: u16 = 0x03E3;
    pub const REMOTE_HEAD_1: u16 = Self::MULTIPLE_CPU_1;
    pub const REMOTE_HEAD_2: u16 = Self::MULTIPLE_CPU_2;
    pub const CONTROL_SYSTEM_REMOTE_HEAD: u16 = Self::CONTROL_SYSTEM_CPU;
    pub const STANDBY_SYSTEM_REMOTE_HEAD: u16 = Self::STANDBY_SYSTEM_CPU;
    pub const OWN_STATION: u16 = 0x03FF;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SlmpTargetAddress {
    pub network: u8,
    pub station: u8,
    pub module_io: u16,
    pub multidrop: u8,
}

impl Default for SlmpTargetAddress {
    fn default() -> Self {
        Self {
            network: 0x00,
            station: 0xFF,
            module_io: SlmpModuleIo::OWN_STATION,
            multidrop: 0x00,
        }
    }
}

/// Immutable, profile-bound semantic device address.
///
/// The code, wire number, and PLC profile can be read through accessors but
/// cannot be changed after construction.
///
/// ```compile_fail
/// use plc_comm_slmp::{SlmpDeviceAddress, SlmpDeviceCode, SlmpPlcProfile};
/// let mut address = SlmpDeviceAddress::new(SlmpDeviceCode::X, 8, SlmpPlcProfile::IqF);
/// address.number = 16;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SlmpDeviceAddress {
    code: SlmpDeviceCode,
    number: u32,
    plc_profile: SlmpPlcProfile,
}

impl SlmpDeviceAddress {
    pub const fn new(code: SlmpDeviceCode, number: u32, plc_profile: SlmpPlcProfile) -> Self {
        Self {
            code,
            number,
            plc_profile,
        }
    }

    pub const fn code(self) -> SlmpDeviceCode {
        self.code
    }

    pub const fn number(self) -> u32 {
        self.number
    }

    pub const fn plc_profile(self) -> SlmpPlcProfile {
        self.plc_profile
    }
}

impl fmt::Display for SlmpDeviceAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let number = if matches!(self.code, SlmpDeviceCode::X | SlmpDeviceCode::Y)
            && self.plc_profile.uses_iqf_xy_octal()
        {
            format!("{:o}", self.number).to_ascii_uppercase()
        } else if self.code.is_hex_addressed() {
            format!("{:X}", self.number)
        } else {
            self.number.to_string()
        };
        write!(f, "{}{number}", self.code)
    }
}

/// Profile-independent wire address for maintainer probes and frame tests.
///
/// Normal client operations intentionally accept only [`SlmpDeviceAddress`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RawSlmpDeviceAddress {
    pub code: SlmpDeviceCode,
    pub number: u32,
}

impl RawSlmpDeviceAddress {
    pub const fn new(code: SlmpDeviceCode, number: u32) -> Self {
        Self { code, number }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SlmpTypeNameInfo {
    pub model: String,
    pub model_code: u16,
    pub has_model_code: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SlmpCpuOperationStatus {
    Unknown,
    Run,
    Stop,
    Pause,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SlmpCpuOperationState {
    pub status: SlmpCpuOperationStatus,
    pub raw_status_word: u16,
    pub raw_code: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SlmpBlockRead {
    pub device: SlmpDeviceAddress,
    pub points: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SlmpBlockWrite {
    pub device: SlmpDeviceAddress,
    pub values: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct SlmpRandomReadResult {
    pub word_values: Vec<u16>,
    pub dword_values: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct SlmpBlockReadResult {
    pub word_values: Vec<u16>,
    pub bit_values: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SlmpLabelArrayReadPoint {
    pub label: String,
    pub unit_specification: u8,
    pub array_data_length: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SlmpLabelArrayWritePoint {
    pub label: String,
    pub unit_specification: u8,
    pub array_data_length: u16,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SlmpLabelRandomWritePoint {
    pub label: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SlmpLabelArrayReadResult {
    pub data_type_id: u8,
    pub unit_specification: u8,
    pub array_data_length: u16,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SlmpLabelRandomReadResult {
    pub data_type_id: u8,
    pub spare: u8,
    pub read_data_length: u16,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SlmpLongTimerResult {
    pub index: u32,
    pub device: String,
    pub current_value: u32,
    pub contact: bool,
    pub coil: bool,
    pub status_word: u16,
    pub raw_words: Vec<u16>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SlmpExtensionSpec {
    pub extension_specification: u16,
    pub extension_specification_modification: u8,
    pub device_modification_index: u8,
    pub device_modification_flags: u8,
    pub direct_memory_specification: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SlmpDeviceModification {
    IndexZ(u8),
    IndexLz(u8),
    Indirect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SlmpQualifiedDeviceAddress {
    pub device: SlmpDeviceAddress,
    pub extension_specification: Option<u16>,
    pub direct_memory_specification: Option<u8>,
    pub modification: Option<SlmpDeviceModification>,
}

impl SlmpQualifiedDeviceAddress {
    pub const fn with_modification(mut self, modification: SlmpDeviceModification) -> Self {
        self.modification = Some(modification);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SlmpNamedTarget {
    pub name: String,
    pub target: SlmpTargetAddress,
}

#[derive(Debug, Clone)]
pub struct SlmpConnectionOptions {
    pub host: String,
    pub port: u16,
    pub timeout: Duration,
    pub tcp_keepalive: Option<Duration>,
    pub(crate) plc_profile: SlmpPlcProfile,
    pub(crate) frame_type: SlmpFrameType,
    pub(crate) compatibility_mode: SlmpCompatibilityMode,
    pub target: SlmpTargetAddress,
    pub transport_mode: SlmpTransportMode,
    pub monitoring_timer: u16,
    pub(crate) strict_profile: bool,
}

impl SlmpConnectionOptions {
    pub fn new(
        host: impl Into<String>,
        port: u16,
        transport_mode: SlmpTransportMode,
        target: SlmpTargetAddress,
        plc_profile: SlmpPlcProfile,
    ) -> Result<Self, crate::error::SlmpError> {
        plc_profile.validate_connection_selectable()?;
        if port == 0 {
            return Err(crate::error::SlmpError::new(
                "port is required and must be in range 1..=65535",
            ));
        }
        let defaults = plc_profile.defaults();
        Ok(Self {
            host: host.into(),
            port,
            timeout: Duration::from_secs(3),
            tcp_keepalive: Some(Duration::from_secs(30)),
            plc_profile,
            frame_type: defaults.frame_type,
            compatibility_mode: defaults.compatibility_mode,
            target,
            transport_mode,
            monitoring_timer: 0x0010,
            strict_profile: true,
        })
    }

    pub fn plc_profile(&self) -> SlmpPlcProfile {
        self.plc_profile
    }

    pub fn frame_type(&self) -> SlmpFrameType {
        self.frame_type
    }

    pub fn compatibility_mode(&self) -> SlmpCompatibilityMode {
        self.compatibility_mode
    }

    pub fn set_plc_profile(
        &mut self,
        plc_profile: SlmpPlcProfile,
    ) -> Result<(), crate::error::SlmpError> {
        plc_profile.validate_connection_selectable()?;
        let defaults = plc_profile.defaults();
        self.plc_profile = plc_profile;
        self.frame_type = defaults.frame_type;
        self.compatibility_mode = defaults.compatibility_mode;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{SlmpPlcProfile, SlmpTargetAddress, SlmpTransportMode};

    #[test]
    fn plc_profile_parse_label_accepts_only_canonical_profile_text() {
        assert_eq!(
            SlmpPlcProfile::parse_label("melsec:iq-r"),
            Some(SlmpPlcProfile::IqR)
        );
        assert_eq!(
            SlmpPlcProfile::parse_label("melsec:iq-r:rj71en71"),
            Some(SlmpPlcProfile::IqRRj71En71)
        );
        assert_eq!(
            SlmpPlcProfile::parse_label("melsec:qcpu:qj71e71-100"),
            Some(SlmpPlcProfile::QCpuQj71E71100)
        );
        assert_eq!(SlmpPlcProfile::parse_label("MELSEC:IQ-F"), None);
        assert_eq!(SlmpPlcProfile::parse_label("iq-r"), None);
        assert_eq!(SlmpPlcProfile::parse_label("iqr"), None);
        assert_eq!(SlmpPlcProfile::parse_label("q"), None);
        assert_eq!(SlmpPlcProfile::parse_label("qnudvcpu"), None);
    }

    #[test]
    fn iq_l_uses_its_own_profile_for_address_rules() {
        assert!(!SlmpPlcProfile::IqL.uses_iqf_xy_octal());
    }

    #[test]
    fn unit_profile_keeps_frame_and_compatibility_axes_independent() {
        let profile = SlmpPlcProfile::QCpuQj71E71100;
        let defaults = profile.defaults();

        assert_eq!(defaults.frame_type, super::SlmpFrameType::Frame4E);
        assert_eq!(
            defaults.compatibility_mode,
            super::SlmpCompatibilityMode::Legacy
        );
        assert_eq!(profile.address_profile(), SlmpPlcProfile::QCpu);
        assert_eq!(profile.range_profile(), SlmpPlcProfile::QCpuQj71E71100);
    }

    #[test]
    fn iqr_unit_profile_uses_iqr_address_rules() {
        let profile = SlmpPlcProfile::IqRRj71En71;
        let defaults = profile.defaults();

        assert_eq!(defaults.frame_type, super::SlmpFrameType::Frame4E);
        assert_eq!(
            defaults.compatibility_mode,
            super::SlmpCompatibilityMode::Iqr
        );
        assert_eq!(profile.address_profile(), SlmpPlcProfile::IqR);
        assert_eq!(profile.range_profile(), SlmpPlcProfile::IqRRj71En71);
    }

    #[test]
    fn connection_options_reject_base_qcpu_profile() {
        assert!(
            super::SlmpConnectionOptions::new(
                "127.0.0.1",
                1025,
                SlmpTransportMode::Tcp,
                SlmpTargetAddress::default(),
                SlmpPlcProfile::QCpu
            )
            .is_err()
        );
    }

    #[test]
    fn connection_options_use_approved_time_defaults_and_reject_zero_port() {
        let target = SlmpTargetAddress {
            network: 1,
            station: 2,
            module_io: 0x03FF,
            multidrop: 0,
        };
        let options = super::SlmpConnectionOptions::new(
            "127.0.0.1",
            1025,
            SlmpTransportMode::Tcp,
            target,
            SlmpPlcProfile::IqR,
        )
        .unwrap();

        assert_eq!(options.timeout, std::time::Duration::from_secs(3));
        assert_eq!(options.monitoring_timer, 0x0010);
        assert_eq!(
            options.tcp_keepalive,
            Some(std::time::Duration::from_secs(30))
        );
        assert_eq!(options.target, target);

        assert!(
            super::SlmpConnectionOptions::new(
                "127.0.0.1",
                0,
                SlmpTransportMode::Tcp,
                target,
                SlmpPlcProfile::IqR,
            )
            .is_err()
        );
    }
}
