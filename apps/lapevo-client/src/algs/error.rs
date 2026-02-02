//! Error types for the algs module.

use thiserror::Error;

/// Result type alias for algorithm operations.
pub type AlgsResult<T> = Result<T, AlgsError>;

/// Errors that can occur during telemetry analysis.
#[derive(Debug, Error)]
pub enum AlgsError {
    /// Empty telemetry sequence provided.
    #[error("cannot extract metrics from empty telemetry sequence")]
    EmptySequence,

    /// Insufficient frames for analysis.
    #[error("insufficient frames for analysis: need at least {required}, got {actual}")]
    InsufficientFrames { required: usize, actual: usize },

    /// Track boundary data is invalid or insufficient.
    #[error("invalid track boundary: {reason}")]
    InvalidBoundary { reason: String },

    /// Corner segment references invalid frame range.
    #[error("corner segment {corner_number} has no frames in range [{start:.3}, {end:.3}]")]
    EmptyCornerSegment {
        corner_number: i32,
        start: f64,
        end: f64,
    },

    /// Track length required but not provided.
    #[error("track length required for segment-based corner extraction")]
    MissingTrackLength,
}
