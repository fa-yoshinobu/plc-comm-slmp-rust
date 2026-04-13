use crate::model::SlmpCommand;

#[derive(Debug, Clone, thiserror::Error)]
#[error("{message}")]
pub struct SlmpError {
    pub message: String,
    pub end_code: Option<u16>,
    pub command: Option<SlmpCommand>,
    pub subcommand: Option<u16>,
}

impl SlmpError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            end_code: None,
            command: None,
            subcommand: None,
        }
    }

    pub fn with_context(
        message: impl Into<String>,
        end_code: Option<u16>,
        command: Option<SlmpCommand>,
        subcommand: Option<u16>,
    ) -> Self {
        Self {
            message: message.into(),
            end_code,
            command,
            subcommand,
        }
    }
}

impl From<std::io::Error> for SlmpError {
    fn from(value: std::io::Error) -> Self {
        Self::new(value.to_string())
    }
}
