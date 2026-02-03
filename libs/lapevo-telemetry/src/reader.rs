use crate::error::Result;
use crate::frame::TelemetryFrame;
use crate::session::SessionInfo;

/// Batch/direct access to all telemetry data (e.g. reading a file).
pub trait TelemetryReader: Send {
    /// Read all frames into memory.
    fn read_all(&self) -> Result<Vec<TelemetryFrame>>;

    /// Iterate frames lazily.
    fn frames(&self) -> Box<dyn Iterator<Item = Result<TelemetryFrame>> + '_>;

    /// Total frame count.
    fn frame_count(&self) -> usize;

    /// Session metadata.
    fn session(&self) -> &SessionInfo;
}
