mod error;

pub use error::SourceError;

use std::path::PathBuf;

use lapevo_iracing::IbtPlayback;
use lapevo_telemetry::{PlaybackControls, TelemetryStream};

/// Configuration for IBT file replay.
#[derive(Debug, Clone)]
pub struct ReplayConfig {
    pub file_path: PathBuf,
    pub speed: f64,
}

/// Configuration for live iRacing connection.
#[derive(Debug, Clone)]
pub struct LiveConfig {
    pub poll_interval_ms: u64,
}

/// Configuration for network telemetry stream.
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub address: String,
    pub port: u16,
}

/// Handle to a created telemetry source.
pub struct SourceHandle {
    pub stream: Box<dyn TelemetryStream>,
    /// Playback controls (only Some for replay sources).
    pub controls: Option<PlaybackControls>,
}

/// Create a telemetry source from replay configuration.
pub fn create_replay_source(config: &ReplayConfig) -> Result<SourceHandle, SourceError> {
    let (stream, controls) = IbtPlayback::open(&config.file_path, config.speed)?;
    Ok(SourceHandle {
        stream: Box::new(stream),
        controls: Some(controls),
    })
}

/// Create a live iRacing telemetry source.
pub fn create_live_source(_config: &LiveConfig) -> Result<SourceHandle, SourceError> {
    Err(SourceError::NotImplemented("Live mode".to_string()))
}

/// Create a network telemetry source.
pub fn create_network_source(_config: &NetworkConfig) -> Result<SourceHandle, SourceError> {
    Err(SourceError::NotImplemented("Network mode".to_string()))
}
