use thiserror::Error;

#[derive(Debug, Error)]
pub enum IracingError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid IBT file: {0}")]
    InvalidIbt(String),

    #[error("parse error in {context}: {details}")]
    Parse { context: String, details: String },

    #[error("missing variable: {0}")]
    MissingVariable(String),

    #[error("YAML error: {0}")]
    Yaml(String),

    #[error("unsupported version: expected {expected}, got {found}")]
    Version { expected: i32, found: i32 },

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, IracingError>;

impl From<IracingError> for lapevo_telemetry::TelemetryError {
    fn from(e: IracingError) -> Self {
        match e {
            IracingError::Io(e) => lapevo_telemetry::TelemetryError::Io(e),
            IracingError::MissingVariable(v) => lapevo_telemetry::TelemetryError::MissingVariable(v),
            other => lapevo_telemetry::TelemetryError::Other(other.to_string()),
        }
    }
}
