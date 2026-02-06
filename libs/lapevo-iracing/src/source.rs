use std::path::PathBuf;

use async_trait::async_trait;
use lapevo_telemetry::{
    ActiveSession, PlaybackControls, SourceStatus, TelemetryError, TelemetrySource,
    TelemetryStream,
};

use crate::playback::IbtPlayback;

/// IBT file replay source implementing `TelemetrySource`.
///
/// Single-use source: returns one session from the IBT file, then signals completion.
pub struct IbtReplaySource {
    path: PathBuf,
    speed: f64,
    consumed: bool,
    last_controls: Option<PlaybackControls>,
}

impl IbtReplaySource {
    /// Create a new IBT replay source, validating the file exists.
    pub fn new(path: PathBuf, speed: f64) -> Result<Self, TelemetryError> {
        if !path.exists() {
            return Err(TelemetryError::FileNotFound(path));
        }
        Ok(Self {
            path,
            speed,
            consumed: false,
            last_controls: None,
        })
    }

    /// Returns playback controls from the last opened session, if any.
    pub fn controls(&self) -> Option<&PlaybackControls> {
        self.last_controls.as_ref()
    }
}

#[async_trait]
impl TelemetrySource for IbtReplaySource {
    async fn wait_for_ready(&mut self) -> Result<(), TelemetryError> {
        // File was validated at construction - always ready
        Ok(())
    }

    async fn wait_for_session(&mut self) -> Result<Option<ActiveSession>, TelemetryError> {
        if self.consumed {
            return Ok(None);
        }
        self.consumed = true;

        let (stream, controls) = IbtPlayback::open(&self.path, self.speed)?;
        let info = stream.session().clone();
        self.last_controls = Some(controls);

        Ok(Some(ActiveSession {
            info,
            stream: Box::new(stream),
        }))
    }

    fn status(&self) -> SourceStatus {
        if self.consumed {
            SourceStatus::Unavailable
        } else {
            SourceStatus::Idle
        }
    }

    fn status_message(&self) -> String {
        if self.consumed {
            "Replay complete".into()
        } else {
            "IBT replay ready".into()
        }
    }
}
