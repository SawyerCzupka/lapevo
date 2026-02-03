use std::path::Path;

use async_trait::async_trait;
use tokio::time::{Duration, sleep};

use crate::error::Result;
use crate::ibt::IbtFile;
use crate::mapping::map_to_telemetry_frame;
use lapevo_telemetry::{PlaybackControls, SessionInfo, TelemetryFrame, TelemetryStream};

/// IBT playback stream implementing `TelemetryStream` with speed control.
pub struct IbtPlayback {
    ibt: IbtFile,
    controls: PlaybackControls,
    current_frame: usize,
    frame_interval_secs: f64,
}

impl IbtPlayback {
    /// Open an IBT file for playback at the given speed multiplier.
    /// Returns the stream and a cloned PlaybackControls handle.
    pub fn open<P: AsRef<Path>>(path: P, speed: f64) -> Result<(Self, PlaybackControls)> {
        let ibt = IbtFile::open(path)?;
        let tick_rate = ibt.tick_rate();
        let frame_interval_secs = 1.0 / tick_rate;

        let controls = PlaybackControls::new(speed.clamp(0.1, 100.0));
        let controls_clone = controls.clone();

        Ok((
            Self {
                ibt,
                controls,
                current_frame: 0,
                frame_interval_secs,
            },
            controls_clone,
        ))
    }
}

#[async_trait]
impl TelemetryStream for IbtPlayback {
    async fn next_frame(&mut self) -> Option<TelemetryFrame> {
        // Wait while paused
        while self.controls.is_paused() {
            sleep(Duration::from_millis(50)).await;
        }

        if self.current_frame >= self.ibt.total_frames() {
            return None;
        }

        let raw = self.ibt.raw_frame(self.current_frame).ok()?;
        let frame = map_to_telemetry_frame(&raw, self.ibt.schema()).ok()?;
        self.current_frame += 1;

        // Delay based on speed
        let speed = self.controls.speed().max(0.1);
        let delay = Duration::from_secs_f64(self.frame_interval_secs / speed);
        sleep(delay).await;

        Some(frame)
    }

    fn session(&self) -> &SessionInfo {
        use lapevo_telemetry::TelemetryReader;
        self.ibt.session()
    }
}
