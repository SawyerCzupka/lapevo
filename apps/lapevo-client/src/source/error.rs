use thiserror::Error;

/// Errors that can occur when creating or using a telemetry source.
#[derive(Error, Debug)]
pub enum SourceError {
    #[error("Failed to open IBT file: {0}")]
    IbtOpen(#[from] lapevo_iracing::IracingError),

    #[error("Live mode not available on this platform")]
    LiveNotAvailable,

    #[error("Network connection failed: {0}")]
    NetworkConnection(String),

    #[error("Source not implemented: {0}")]
    NotImplemented(String),
}
