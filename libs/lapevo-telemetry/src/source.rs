use async_trait::async_trait;

use crate::{SessionInfo, TelemetryError, TelemetryStream};

/// Connection/lifecycle status for a telemetry source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceStatus {
    /// Source not available (sim not running, file not found, etc.)
    Unavailable,
    /// Source available but not actively driving (in menus, garage, etc.)
    Idle,
    /// Actively driving - frames are being streamed
    Active,
}

/// An active driving session with its metadata and frame stream.
pub struct ActiveSession {
    /// Metadata about the session (track, car, etc.)
    pub info: SessionInfo,
    /// The frame stream for this session
    pub stream: Box<dyn TelemetryStream>,
}

/// A telemetry source that manages connection lifecycle and produces streams.
///
/// This trait handles the "outer loop" - waiting for the sim, detecting
/// when driving starts, and producing TelemetryStream instances for each session.
///
/// Implementations define their own waiting strategies appropriate to their
/// source type (file, shared memory, network, etc.)
#[async_trait]
pub trait TelemetrySource: Send {
    /// Wait until the source becomes available.
    ///
    /// Blocks (with efficient async sleep) until the source is ready to
    /// potentially produce sessions.
    async fn wait_for_ready(&mut self) -> Result<(), TelemetryError>;

    /// Wait until an active driving session begins.
    ///
    /// Returns `Some(session)` when driving begins, or `None` if the source
    /// becomes unavailable while waiting (e.g., sim exited, replay exhausted).
    async fn wait_for_session(&mut self) -> Result<Option<ActiveSession>, TelemetryError>;

    /// Check current status without blocking.
    fn status(&self) -> SourceStatus;

    /// Human-readable description of current state.
    fn status_message(&self) -> String;
}
