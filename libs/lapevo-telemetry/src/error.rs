use thiserror::Error;

#[derive(Debug, Error)]
pub enum TelemetryError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("missing variable: {0}")]
    MissingVariable(String),

    #[error("session not available")]
    NoSession,

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, TelemetryError>;
