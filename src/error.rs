use crate::error_codes::{end_code_name, is_remote_password_end_code};
use crate::model::SlmpCommand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SlmpErrorKind {
    General,
    Timeout,
    Cancelled,
    Closed,
    NotConnected,
    Transport,
    MalformedResponse,
    PlcEndCode,
    ProfileFeature,
    OutcomeUnknown,
}

/// Machine-readable reason why a state-changing command has an unknown outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SlmpOutcomeUnknownReason {
    Timeout,
    Cancelled,
    Closed,
    Transport,
    MalformedResponse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlmpErrorInfo {
    pub network: u8,
    pub station: u8,
    pub module_io: u16,
    pub multidrop: u8,
    pub command: u16,
    pub subcommand: u16,
    pub raw: Vec<u8>,
}

impl SlmpErrorInfo {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 9 {
            return None;
        }
        let raw = data[..9].to_vec();
        Some(Self {
            network: raw[0],
            station: raw[1],
            module_io: u16::from_le_bytes([raw[2], raw[3]]),
            multidrop: raw[4],
            command: u16::from_le_bytes([raw[5], raw[6]]),
            subcommand: u16::from_le_bytes([raw[7], raw[8]]),
            raw,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlmpProfileFeatureErrorInfo {
    pub profile_id: String,
    pub feature_key: String,
    pub state: String,
    pub evidence: Option<String>,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("{message}")]
pub struct SlmpError {
    pub kind: SlmpErrorKind,
    pub message: String,
    pub end_code: Option<u16>,
    pub command: Option<SlmpCommand>,
    pub subcommand: Option<u16>,
    pub error_info: Option<SlmpErrorInfo>,
    pub profile_feature: Option<SlmpProfileFeatureErrorInfo>,
    pub outcome_unknown_reason: Option<SlmpOutcomeUnknownReason>,
}

impl SlmpError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            kind: SlmpErrorKind::General,
            message: message.into(),
            end_code: None,
            command: None,
            subcommand: None,
            error_info: None,
            profile_feature: None,
            outcome_unknown_reason: None,
        }
    }

    pub(crate) fn timeout(message: impl Into<String>) -> Self {
        Self {
            kind: SlmpErrorKind::Timeout,
            message: message.into(),
            end_code: None,
            command: None,
            subcommand: None,
            error_info: None,
            profile_feature: None,
            outcome_unknown_reason: None,
        }
    }

    pub(crate) fn closed(message: impl Into<String>) -> Self {
        Self::of_kind(SlmpErrorKind::Closed, message)
    }

    pub(crate) fn transport(message: impl Into<String>) -> Self {
        Self::of_kind(SlmpErrorKind::Transport, message)
    }

    pub(crate) fn malformed_response(message: impl Into<String>) -> Self {
        Self::of_kind(SlmpErrorKind::MalformedResponse, message)
    }

    fn of_kind(kind: SlmpErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            end_code: None,
            command: None,
            subcommand: None,
            error_info: None,
            profile_feature: None,
            outcome_unknown_reason: None,
        }
    }

    pub(crate) fn malformed_with_context(
        message: impl Into<String>,
        command: SlmpCommand,
        subcommand: u16,
    ) -> Self {
        let mut error = Self::malformed_response(message);
        error.command = Some(command);
        error.subcommand = Some(subcommand);
        error
    }

    pub(crate) fn outcome_unknown(
        reason: SlmpOutcomeUnknownReason,
        source: SlmpError,
        command: SlmpCommand,
        subcommand: u16,
    ) -> Self {
        Self {
            kind: SlmpErrorKind::OutcomeUnknown,
            message: format!(
                "state-changing SLMP command outcome is unknown ({reason:?}): {}",
                source.message
            ),
            end_code: source.end_code,
            command: Some(command),
            subcommand: Some(subcommand),
            error_info: source.error_info,
            profile_feature: source.profile_feature,
            outcome_unknown_reason: Some(reason),
        }
    }

    pub fn with_context(
        message: impl Into<String>,
        end_code: Option<u16>,
        command: Option<SlmpCommand>,
        subcommand: Option<u16>,
    ) -> Self {
        Self {
            kind: if end_code.is_some() {
                SlmpErrorKind::PlcEndCode
            } else {
                SlmpErrorKind::General
            },
            message: message.into(),
            end_code,
            command,
            subcommand,
            error_info: None,
            profile_feature: None,
            outcome_unknown_reason: None,
        }
    }

    pub fn with_error_info(
        message: impl Into<String>,
        end_code: Option<u16>,
        command: Option<SlmpCommand>,
        subcommand: Option<u16>,
        error_info: Option<SlmpErrorInfo>,
    ) -> Self {
        Self {
            kind: if end_code.is_some() {
                SlmpErrorKind::PlcEndCode
            } else {
                SlmpErrorKind::General
            },
            message: message.into(),
            end_code,
            command,
            subcommand,
            error_info,
            profile_feature: None,
            outcome_unknown_reason: None,
        }
    }

    pub fn profile_feature(
        profile_id: impl Into<String>,
        feature_key: impl Into<String>,
        state: impl Into<String>,
        evidence: Option<String>,
    ) -> Self {
        let profile_id = profile_id.into();
        let feature_key = feature_key.into();
        let state = state.into();
        let evidence_text = evidence
            .as_ref()
            .map(|value| format!(" Evidence: {value}."))
            .unwrap_or_default();
        let message = format!(
            "Feature '{feature_key}' is {state} for plc_profile '{profile_id}'.{evidence_text}"
        );
        Self {
            kind: SlmpErrorKind::ProfileFeature,
            message,
            end_code: None,
            command: None,
            subcommand: None,
            error_info: None,
            profile_feature: Some(SlmpProfileFeatureErrorInfo {
                profile_id,
                feature_key,
                state,
                evidence,
            }),
            outcome_unknown_reason: None,
        }
    }

    pub fn is_profile_feature_error(&self) -> bool {
        matches!(self.kind, SlmpErrorKind::ProfileFeature)
    }

    pub fn is_timeout(&self) -> bool {
        matches!(self.kind, SlmpErrorKind::Timeout)
    }

    pub fn is_outcome_unknown(&self) -> bool {
        matches!(self.kind, SlmpErrorKind::OutcomeUnknown)
    }

    pub fn end_code_name(&self) -> Option<&'static str> {
        self.end_code.map(end_code_name)
    }

    pub fn is_remote_password_error(&self) -> bool {
        self.end_code.is_some_and(is_remote_password_end_code)
    }
}

impl From<std::io::Error> for SlmpError {
    fn from(value: std::io::Error) -> Self {
        if value.kind() == std::io::ErrorKind::TimedOut {
            Self::timeout(value.to_string())
        } else {
            Self::transport(value.to_string())
        }
    }
}
