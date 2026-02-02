//! Accelerated replay connection for IBT files
//!
//! Provides the same API as `ReplayConnection` but with configurable playback speed.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use futures::{Stream, StreamExt};
use pitwall::driver::{Driver, SyncState};
use pitwall::provider::Provider;
use pitwall::stream::ThrottleExt;
use pitwall::{FrameAdapter, FramePacket, Result, SessionInfo, UpdateRate, VariableSchema};
use tokio::sync::broadcast;
use tokio_stream::wrappers::{BroadcastStream, errors::BroadcastStreamRecvError};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

use super::accelerated_provider::AcceleratedReplayProvider;

/// Accelerated replay connection from IBT file
///
/// Provides the same API as `ReplayConnection` but with configurable playback speed.
/// This allows replaying telemetry data faster (e.g., 10x) or slower (e.g., 0.5x)
/// than real-time, which is useful for development and testing.
pub struct AcceleratedReplayConnection {
    /// Frame broadcast sender (for creating subscribers)
    frame_tx: Option<broadcast::Sender<Arc<FramePacket>>>,

    /// Session broadcast sender (for creating subscribers)
    session_tx: broadcast::Sender<Arc<SessionInfo>>,

    /// Shared sync state for current_frame()/current_session() access
    sync_state: Arc<SyncState>,

    /// Variable schema
    schema: Arc<VariableSchema>,

    /// Source frequency
    source_hz: f64,

    /// Configured playback speed
    speed: f64,

    /// Cancellation token for stopping tasks
    cancel: CancellationToken,
}

impl AcceleratedReplayConnection {
    /// Open an IBT file for accelerated replay.
    ///
    /// # Arguments
    /// * `path` - Path to the IBT file
    /// * `speed` - Playback speed multiplier (e.g., 10.0 for 10x speed, 0.5 for half speed)
    ///
    /// Speed is clamped to the range [0.1, 100.0]
    ///
    /// Waits for the first frame to be available before returning to ensure
    /// the connection is fully initialized and ready for subscriptions.
    pub async fn open<P: AsRef<Path>>(path: P, speed: f64) -> Result<Self> {
        let path = path.as_ref();
        info!(
            "Opening IBT file for accelerated replay: {}",
            path.display()
        );

        // Create provider and extract metadata
        let provider = AcceleratedReplayProvider::new(path, speed)?;
        let schema = provider.schema();
        let source_hz = provider.tick_rate();
        let actual_speed = provider.speed();

        // Spawn driver tasks
        let channels = Driver::spawn(provider);

        // Wait for first frame to be available via broadcast
        let mut frame_rx = channels.frame_tx.subscribe();
        let timeout = Duration::from_secs(5);
        let frame_result = tokio::time::timeout(timeout, async {
            // Wait for first frame via broadcast receiver
            frame_rx.recv().await.ok()
        })
        .await;

        if frame_result.is_err() || frame_result.as_ref().ok().and_then(|o| o.as_ref()).is_none() {
            warn!("Timeout waiting for first frame from replay file");
        }

        // Wait for session info to be available (parsed from YAML in background task)
        let mut session_rx = channels.session_tx.subscribe();
        let session_result = tokio::time::timeout(timeout, async {
            session_rx.recv().await.ok()
        })
        .await;

        if session_result.is_err()
            || session_result.as_ref().ok().and_then(|o| o.as_ref()).is_none()
        {
            warn!("Timeout waiting for session info from replay file");
        }

        info!(
            "Accelerated replay connection opened ({}Hz, {}x speed)",
            source_hz, actual_speed
        );

        Ok(Self {
            frame_tx: Some(channels.frame_tx),
            session_tx: channels.session_tx,
            sync_state: channels.sync_state,
            schema,
            source_hz,
            speed: actual_speed,
            cancel: channels.cancel,
        })
    }

    /// Subscribe to telemetry frames
    ///
    /// Returns an async stream of typed telemetry frames. The type parameter `T`
    /// must implement `FrameAdapter`, which is typically done via `#[derive(PitwallFrame)]`.
    ///
    /// # Arguments
    /// * `rate` - The desired update rate for the stream
    pub fn subscribe<T>(&self, rate: UpdateRate) -> impl Stream<Item = T> + 'static
    where
        T: FrameAdapter + Send + 'static,
    {
        // Validate schema once at subscription time
        let validation = T::validate_schema(&self.schema).expect("Schema validation failed");

        // Create base frame stream from broadcast channel
        let frames = BroadcastStream::new(
            self.frame_tx.as_ref().expect("frame sender already released").subscribe(),
        )
        .filter_map(|result| async move {
                match result {
                    Ok(frame) => Some(frame),
                    Err(BroadcastStreamRecvError::Lagged(n)) => {
                        warn!("Frame subscriber lagged by {} frames", n);
                        None
                    }
                }
            });

        // Apply rate control and adaptation
        let effective_rate = rate.normalize(self.source_hz);

        match effective_rate {
            UpdateRate::Native => {
                // Direct adaptation, no throttling
                frames
                    .map(move |packet| T::adapt(&packet, &validation))
                    .boxed()
            }
            UpdateRate::Max(hz) => {
                // Throttle then adapt
                let interval = Duration::from_secs_f64(1.0 / hz as f64);
                frames
                    .throttle(interval)
                    .map(move |packet| T::adapt(&packet, &validation))
                    .boxed()
            }
        }
    }

    /// Get session updates as a stream
    pub fn session_updates(&self) -> impl Stream<Item = Arc<SessionInfo>> + 'static {
        BroadcastStream::new(self.session_tx.subscribe()).filter_map(|result| async move {
            match result {
                Ok(session) => Some(session),
                Err(BroadcastStreamRecvError::Lagged(n)) => {
                    warn!("Session subscriber lagged by {} updates", n);
                    None
                }
            }
        })
    }

    /// Get current session info (if available) - synchronous access
    pub fn current_session(&self) -> Option<Arc<SessionInfo>> {
        self.sync_state.current_session.read().unwrap().clone()
    }

    /// Get current frame (if available) - synchronous access
    pub fn current_frame(&self) -> Option<Arc<FramePacket>> {
        self.sync_state.current_frame.read().unwrap().clone()
    }

    /// Get the source telemetry frequency
    pub fn source_hz(&self) -> f64 {
        self.source_hz
    }

    /// Get the configured playback speed
    pub fn speed(&self) -> f64 {
        self.speed
    }

    /// Release the internal frame sender so the broadcast channel will close
    /// when the Driver finishes producing frames. Call after all `subscribe()` calls.
    pub fn release_frame_sender(&mut self) {
        self.frame_tx.take();
    }

    /// Get the variable schema
    pub fn schema(&self) -> &VariableSchema {
        &self.schema
    }
}

impl Drop for AcceleratedReplayConnection {
    fn drop(&mut self) {
        debug!("Dropping accelerated replay connection");
        // Cancel tasks on drop for clean shutdown
        self.cancel.cancel();
    }
}
