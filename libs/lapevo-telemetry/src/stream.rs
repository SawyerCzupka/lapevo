use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use async_trait::async_trait;

use crate::frame::TelemetryFrame;
use crate::session::SessionInfo;

/// Async stream of telemetry frames (live or replay playback).
#[async_trait]
pub trait TelemetryStream: Send {
    /// Receive next frame. Returns None when source ends.
    async fn next_frame(&mut self) -> Option<TelemetryFrame>;

    /// Session metadata.
    fn session(&self) -> &SessionInfo;
}

/// Controls for replay-style streams (speed, pause).
#[derive(Debug, Clone)]
pub struct PlaybackControls {
    speed: Arc<AtomicU64>,
    paused: Arc<AtomicBool>,
}

impl PlaybackControls {
    pub fn new(initial_speed: f64) -> Self {
        Self {
            speed: Arc::new(AtomicU64::new(initial_speed.to_bits())),
            paused: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set_speed(&self, multiplier: f64) {
        self.speed.store(multiplier.to_bits(), Ordering::Relaxed);
    }

    pub fn pause(&self) {
        self.paused.store(true, Ordering::Relaxed);
    }

    pub fn resume(&self) {
        self.paused.store(false, Ordering::Relaxed);
    }

    pub fn speed(&self) -> f64 {
        f64::from_bits(self.speed.load(Ordering::Relaxed))
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }
}
