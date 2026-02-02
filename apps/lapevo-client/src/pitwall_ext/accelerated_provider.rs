//! Accelerated replay provider for IBT files
//!
//! Implements the `Provider` trait with configurable playback speed.

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use pitwall::provider::Provider;
use pitwall::{FramePacket, IbtReader, Result, VariableSchema};
use tokio::time::{Duration, Interval, interval};
use tracing::{debug, info, trace};

/// Replay provider that reads from IBT files at configurable speed
pub struct AcceleratedReplayProvider {
    /// IBT file reader
    reader: IbtReader,

    /// Frame pacing interval (adjusted for speed)
    interval: Interval,

    /// Cached schema
    schema: Arc<VariableSchema>,

    /// Native tick rate from IBT
    tick_rate: f64,

    /// Configured playback speed
    speed: f64,
}

impl AcceleratedReplayProvider {
    /// Create a new accelerated replay provider from an IBT file
    ///
    /// # Arguments
    /// * `path` - Path to the IBT file
    /// * `speed` - Playback speed multiplier (e.g., 10.0 for 10x speed, 0.5 for half speed)
    ///
    /// Speed is clamped to the range [0.1, 100.0]
    pub fn new<P: AsRef<Path>>(path: P, speed: f64) -> Result<Self> {
        let reader = IbtReader::open(&path)?;

        // Get metadata
        let total_frames = reader.total_frames();
        let tick_rate = reader.tick_rate();

        // Get the variable schema from the reader
        let schema = Arc::new(reader.variables().clone());

        // Clamp speed to reasonable range
        let speed = speed.clamp(0.1, 100.0);

        info!(
            "Opened IBT file: {} frames at {}Hz, playback speed {}x",
            total_frames, tick_rate, speed
        );

        // Calculate frame interval for pacing (adjusted for speed)
        let frame_interval = Duration::from_secs_f64(1.0 / (tick_rate * speed));
        let interval = interval(frame_interval);

        Ok(Self {
            reader,
            interval,
            schema,
            tick_rate,
            speed,
        })
    }

    /// Get the variable schema
    pub fn schema(&self) -> Arc<VariableSchema> {
        Arc::clone(&self.schema)
    }

    /// Get the configured playback speed
    pub fn speed(&self) -> f64 {
        self.speed
    }
}

#[async_trait]
impl Provider for AcceleratedReplayProvider {
    async fn next_frame(&mut self) -> Result<Option<FramePacket>> {
        // Check if we've reached the end
        let total_frames = self.reader.total_frames();
        if self.reader.current_frame() >= total_frames {
            debug!("Reached end of replay");
            return Ok(None);
        }

        // Wait for next frame timing (pacing)
        self.interval.tick().await;

        // Read next frame data directly from IBT reader
        let (frame_data, tick, session_version) = match self.reader.read_next_frame()? {
            Some(data) => data,
            None => {
                debug!("No more frames from reader");
                return Ok(None);
            }
        };

        trace!(
            "Frame {}/{}: tick={}, session_version={}",
            self.reader.current_frame(),
            total_frames,
            tick,
            session_version
        );

        let packet = FramePacket::new(frame_data, tick, session_version, Arc::clone(&self.schema));

        Ok(Some(packet))
    }

    async fn session_yaml(&mut self, _version: u32) -> Result<Option<String>> {
        // Get cleaned YAML from IBT file
        // IBT files have static session info, version parameter is ignored
        self.reader.session_yaml()
    }

    fn tick_rate(&self) -> f64 {
        self.tick_rate
    }
}
