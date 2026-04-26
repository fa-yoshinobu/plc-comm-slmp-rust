use std::fmt;
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
pub enum SlmpPlcFamily {
    IqF,
    IqR,
    IqL,
    MxF,
    MxR,
    QCpu,
    LCpu,
    QnU,
    QnUDV,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlmpPlcFamilyDefaults {
    pub frame_type: SlmpFrameType,
    pub compatibility_mode: SlmpCompatibilityMode,
}

impl SlmpPlcFamily {
    pub fn canonical_name(self) -> &'static str {
        match self {
            Self::IqF => "iq-f",
            Self::IqR => "iq-r",
            Self::IqL => "iq-l",
            Self::MxF => "mx-f",
            Self::MxR => "mx-r",
            Self::QCpu => "qcpu",
            Self::LCpu => "lcpu",
            Self::QnU => "qnu",
            Self::QnUDV => "qnudv",
        }
    }

    pub fn parse_label(value: &str) -> Option<Self> {
        match value
            .trim()
            .to_ascii_lowercase()
            .replace(['-', '_'], "")
            .as_str()
        {
            "iqf" => Some(Self::IqF),
            "iqr" => Some(Self::IqR),
            "iql" => Some(Self::IqL),
            "mxf" => Some(Self::MxF),
            "mxr" => Some(Self::MxR),
            "qcpu" => Some(Self::QCpu),
            "lcpu" => Some(Self::LCpu),
            "qnu" => Some(Self::QnU),
            "qnudv" => Some(Self::QnUDV),
            _ => None,
        }
    }

    pub fn defaults(self) -> SlmpPlcFamilyDefaults {
        match self {
            Self::IqF => SlmpPlcFamilyDefaults {
                frame_type: SlmpFrameType::Frame3E,
                compatibility_mode: SlmpCompatibilityMode::Legacy,
            },
            Self::IqR | Self::IqL | Self::MxF | Self::MxR => SlmpPlcFamilyDefaults {
                frame_type: SlmpFrameType::Frame4E,
                compatibility_mode: SlmpCompatibilityMode::Iqr,
            },
            Self::QCpu | Self::LCpu | Self::QnU | Self::QnUDV => SlmpPlcFamilyDefaults {
                frame_type: SlmpFrameType::Frame3E,
                compatibility_mode: SlmpCompatibilityMode::Legacy,
            },
        }
    }

    pub fn address_family(self) -> Self {
        match self {
            Self::IqL => Self::IqR,
            other => other,
        }
    }

    pub fn uses_iqf_xy_octal(self) -> bool {
        matches!(self.address_family(), Self::IqF)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SlmpTraceDirection {
    Send,
    Receive,
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
            module_io: 0x03FF,
            multidrop: 0x00,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SlmpDeviceAddress {
    pub code: SlmpDeviceCode,
    pub number: u32,
}

impl SlmpDeviceAddress {
    pub const fn new(code: SlmpDeviceCode, number: u32) -> Self {
        Self { code, number }
    }
}

impl fmt::Display for SlmpDeviceAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.code, self.number)
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
pub struct SlmpBlockWriteOptions {
    pub split_mixed_blocks: bool,
    pub retry_mixed_on_error: bool,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SlmpExtensionSpec {
    pub extension_specification: u16,
    pub extension_specification_modification: u8,
    pub device_modification_index: u8,
    pub device_modification_flags: u8,
    pub direct_memory_specification: u8,
}

impl Default for SlmpExtensionSpec {
    fn default() -> Self {
        Self {
            extension_specification: 0x0000,
            extension_specification_modification: 0x00,
            device_modification_index: 0x00,
            device_modification_flags: 0x00,
            direct_memory_specification: 0x00,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SlmpQualifiedDeviceAddress {
    pub device: SlmpDeviceAddress,
    pub extension_specification: Option<u16>,
    pub direct_memory_specification: Option<u8>,
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
    pub plc_family: SlmpPlcFamily,
    pub frame_type: SlmpFrameType,
    pub compatibility_mode: SlmpCompatibilityMode,
    pub target: SlmpTargetAddress,
    pub transport_mode: SlmpTransportMode,
    pub monitoring_timer: u16,
}

impl SlmpConnectionOptions {
    pub fn new(host: impl Into<String>, family: SlmpPlcFamily) -> Self {
        let defaults = family.defaults();
        Self {
            host: host.into(),
            port: 1025,
            timeout: Duration::from_secs(3),
            plc_family: family,
            frame_type: defaults.frame_type,
            compatibility_mode: defaults.compatibility_mode,
            target: SlmpTargetAddress::default(),
            transport_mode: SlmpTransportMode::Tcp,
            monitoring_timer: 0x0010,
        }
    }

    pub fn set_plc_family(&mut self, family: SlmpPlcFamily) {
        let defaults = family.defaults();
        self.plc_family = family;
        self.frame_type = defaults.frame_type;
        self.compatibility_mode = defaults.compatibility_mode;
    }
}
