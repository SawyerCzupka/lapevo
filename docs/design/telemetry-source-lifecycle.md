# Telemetry Source Lifecycle Architecture

## Problem Statement

The current `lapevo-client` is designed for single-session, run-to-completion operation. Users want to:

1. Start the client once (e.g., at computer startup)
2. Have it run in the background with minimal resource usage when idle
3. Automatically detect when iRacing starts and begin collecting telemetry
4. Seamlessly handle multiple sessions over hours/days without manual intervention
5. Return to idle state when sessions end, ready for the next one

The current `TelemetryStream` trait only handles frame iteration and has no concept of:
- Waiting for a sim to become available
- Detecting when real driving starts vs. being in menus/garage
- Session boundaries and transitions

## Proposed Solution

Introduce a **two-tier trait hierarchy** that separates concerns:

1. **`TelemetrySource`** - Manages connection lifecycle and session detection
2. **`TelemetryStream`** - Handles frame-by-frame telemetry (unchanged)

This allows each source type (IBT replay, live iRacing, network) to implement its own waiting and detection strategy while keeping the frame streaming logic uniform.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     TelemetrySource                             │
│  Responsibilities:                                              │
│  • Wait for sim/source availability                             │
│  • Detect session start (driving begins)                        │
│  • Detect session end (driving stops, sim exits)                │
│  • Produce TelemetryStream instances for each session           │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ produces
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     TelemetryStream                             │
│  Responsibilities:                                              │
│  • Iterate frames for a single driving session                  │
│  • Provide session metadata                                     │
│  • Signal session end via None return                           │
└─────────────────────────────────────────────────────────────────┘
```

## State Machine

The `TelemetrySource` trait encapsulates this state machine:

```
┌─────────────────────────────────────────────────────────────────┐
│                      UNAVAILABLE                                │
│  • Source not ready (sim not running, no connection)            │
│  • wait_for_ready() blocks here with efficient async sleep      │
│  • Polling interval: 2-3 seconds                                │
│  • CPU usage: ~0% (thread parking)                              │
└─────────────────────┬───────────────────────────────────────────┘
                      │ Source becomes available
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                         IDLE                                    │
│  • Source available but not actively driving                    │
│  • wait_for_session() blocks here                               │
│  • Polling interval: 100ms (watching for track entry)           │
│  • CPU usage: <1%                                               │
└─────────────────────┬───────────────────────────────────────────┘
                      │ Driving session detected
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                        ACTIVE                                   │
│  • TelemetryStream returned, frames flowing                     │
│  • 60Hz frame collection                                        │
│  • CPU usage: 2-5%                                              │
└─────────────────────┬───────────────────────────────────────────┘
                      │ Session ends
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                    SESSION_ENDED                                │
│  • Transient state after stream returns None                    │
│  • Loops back to IDLE (if sim still running)                    │
│  • Loops back to UNAVAILABLE (if sim exited)                    │
└─────────────────────────────────────────────────────────────────┘
```

## Trait Definitions

### TelemetrySource Trait

```rust
// libs/lapevo-telemetry/src/source.rs

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
    /// Session ended, may transition back to Idle or Unavailable
    SessionEnded,
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
    /// This method blocks (with efficient async sleep) until the source
    /// is ready to potentially produce sessions. It does NOT wait for
    /// driving to start - that's `wait_for_session`'s job.
    ///
    /// # Returns
    /// - `Ok(())` when the source is ready
    /// - `Err(_)` if an unrecoverable error occurs
    ///
    /// # Implementation Notes
    /// - IBT Replay: Returns immediately (file validated at construction)
    /// - Live iRacing: Polls for shared memory existence every 2-3 seconds
    /// - Network: Attempts connection with exponential backoff
    async fn wait_for_ready(&mut self) -> Result<(), TelemetryError>;

    /// Wait until an active driving session begins.
    ///
    /// This method blocks until the user is actively driving on track,
    /// then returns an ActiveSession containing the stream.
    ///
    /// # Returns
    /// - `Ok(Some(session))` when driving begins
    /// - `Ok(None)` if the source becomes unavailable while waiting
    /// - `Err(_)` if an unrecoverable error occurs
    ///
    /// # Implementation Notes
    /// - IBT Replay: Returns immediately on first call, None on subsequent calls
    /// - Live iRacing: Polls IsOnTrack at ~10Hz until true
    /// - Network: Waits for session-start message from server
    async fn wait_for_session(&mut self) -> Result<Option<ActiveSession>, TelemetryError>;

    /// Check current status without blocking.
    ///
    /// Useful for UI display, logging, and health checks.
    fn status(&self) -> SourceStatus;

    /// Human-readable description of current state.
    ///
    /// Examples:
    /// - "Waiting for iRacing to start..."
    /// - "iRacing running - waiting for track session..."
    /// - "Collecting telemetry"
    fn status_message(&self) -> &str;
}
```

### TelemetryStream Trait (Unchanged)

```rust
// libs/lapevo-telemetry/src/stream.rs

use async_trait::async_trait;
use crate::{SessionInfo, TelemetryFrame};

/// Async stream of telemetry frames for a single driving session.
///
/// This trait is focused purely on frame iteration. Lifecycle management
/// (waiting for sim, detecting session boundaries) is handled by TelemetrySource.
#[async_trait]
pub trait TelemetryStream: Send {
    /// Receive next frame.
    ///
    /// # Returns
    /// - `Some(frame)` for each telemetry frame
    /// - `None` when the session ends (triggers transition back to Source)
    async fn next_frame(&mut self) -> Option<TelemetryFrame>;

    /// Session metadata.
    fn session(&self) -> &SessionInfo;
}
```

## Implementation Examples

### IBT Replay Source

The simplest implementation - file-based, single session, no waiting logic needed.

```rust
pub struct IbtReplaySource {
    path: PathBuf,
    speed: f64,
    consumed: bool,
}

impl IbtReplaySource {
    pub fn new(path: PathBuf, speed: f64) -> Result<Self, TelemetryError> {
        // Validate file exists at construction time
        if !path.exists() {
            return Err(TelemetryError::FileNotFound(path));
        }
        Ok(Self { path, speed, consumed: false })
    }
}

#[async_trait]
impl TelemetrySource for IbtReplaySource {
    async fn wait_for_ready(&mut self) -> Result<(), TelemetryError> {
        // File was validated at construction - always ready
        Ok(())
    }

    async fn wait_for_session(&mut self) -> Result<Option<ActiveSession>, TelemetryError> {
        // Single-use source - returns None after first session
        if self.consumed {
            return Ok(None);
        }
        self.consumed = true;

        let (stream, _controls) = IbtPlayback::open(&self.path, self.speed)?;
        let info = stream.session().clone();

        Ok(Some(ActiveSession {
            info,
            stream: Box::new(stream),
        }))
    }

    fn status(&self) -> SourceStatus {
        if self.consumed {
            SourceStatus::SessionEnded
        } else {
            SourceStatus::Active
        }
    }

    fn status_message(&self) -> &str {
        if self.consumed {
            "Replay complete"
        } else {
            "IBT replay ready"
        }
    }
}
```

### Live iRacing Source

Full lifecycle management with efficient polling.

```rust
pub struct LiveIRacingSource {
    config: LiveConfig,
    connection: Option<IRacingConnection>,
    current_session_num: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct LiveConfig {
    /// How often to check if iRacing is running (when not connected)
    pub ready_poll_interval: Duration,
    /// How often to check for track entry (when connected but not driving)
    pub session_poll_interval: Duration,
    /// Grace period before declaring session ended when IsOnTrack goes false
    pub off_track_grace_period: Duration,
}

impl Default for LiveConfig {
    fn default() -> Self {
        Self {
            ready_poll_interval: Duration::from_secs(2),
            session_poll_interval: Duration::from_millis(100),
            off_track_grace_period: Duration::from_secs(5),
        }
    }
}

#[async_trait]
impl TelemetrySource for LiveIRacingSource {
    async fn wait_for_ready(&mut self) -> Result<(), TelemetryError> {
        loop {
            // Check if shared memory exists
            if self.try_connect()? {
                return Ok(());
            }

            // Efficient async sleep - yields to executor, no CPU burn
            tokio::time::sleep(self.config.ready_poll_interval).await;
        }
    }

    async fn wait_for_session(&mut self) -> Result<Option<ActiveSession>, TelemetryError> {
        let conn = self.connection.as_mut()
            .ok_or(TelemetryError::NotConnected)?;

        loop {
            match conn.poll_state()? {
                SimState::Disconnected => {
                    // Sim exited - caller should loop back to wait_for_ready
                    self.connection = None;
                    return Ok(None);
                }
                SimState::InMenus | SimState::InGarage => {
                    // Sim running but not driving yet
                    tokio::time::sleep(self.config.session_poll_interval).await;
                }
                SimState::OnTrack { session_info, session_num } => {
                    // Check if this is a new session (different SessionNum)
                    let is_new_session = self.current_session_num != Some(session_num);
                    self.current_session_num = Some(session_num);

                    if is_new_session {
                        return Ok(Some(ActiveSession {
                            info: session_info,
                            stream: Box::new(conn.create_frame_stream(
                                self.config.off_track_grace_period
                            )),
                        }));
                    }

                    // Same session, keep waiting
                    tokio::time::sleep(self.config.session_poll_interval).await;
                }
            }
        }
    }

    fn status(&self) -> SourceStatus {
        match &self.connection {
            None => SourceStatus::Unavailable,
            Some(conn) => match conn.current_state() {
                SimState::Disconnected => SourceStatus::Unavailable,
                SimState::InMenus | SimState::InGarage => SourceStatus::Idle,
                SimState::OnTrack { .. } => SourceStatus::Active,
            }
        }
    }

    fn status_message(&self) -> &str {
        match self.status() {
            SourceStatus::Unavailable => "Waiting for iRacing to start...",
            SourceStatus::Idle => "iRacing running - waiting for track session...",
            SourceStatus::Active => "Collecting telemetry",
            SourceStatus::SessionEnded => "Session ended",
        }
    }
}
```

### Live iRacing Frame Stream

The stream handles the off-track grace period to avoid premature session termination.

```rust
pub struct LiveFrameStream {
    connection: Arc<IRacingConnection>,
    session_info: SessionInfo,
    off_track_grace: Duration,
    off_track_since: Option<Instant>,
}

#[async_trait]
impl TelemetryStream for LiveFrameStream {
    async fn next_frame(&mut self) -> Option<TelemetryFrame> {
        loop {
            // Wait for data-valid event (efficient, event-driven)
            self.connection.wait_for_data().await;

            let frame = self.connection.read_frame()?;

            if frame.on_track {
                // Reset off-track timer
                self.off_track_since = None;
                return Some(frame);
            } else {
                // Track off-track duration
                let off_track_start = *self.off_track_since
                    .get_or_insert_with(Instant::now);

                if off_track_start.elapsed() > self.off_track_grace {
                    // Grace period exceeded - end session
                    return None;
                }

                // Still in grace period - continue streaming
                // (captures pit stops, brief off-tracks, etc.)
                return Some(frame);
            }
        }
    }

    fn session(&self) -> &SessionInfo {
        &self.session_info
    }
}
```

## Client Main Loop

With this architecture, the client main loop becomes elegant and declarative:

```rust
// apps/lapevo-client/src/main.rs

async fn run_client_loop(
    mut source: Box<dyn TelemetrySource>,
    client: Arc<ServerAPIClient>,
) -> Result<()> {
    loop {
        // Phase 1: Wait for source to be ready
        info!("{}", source.status_message());
        source.wait_for_ready().await?;
        info!("Source ready");

        // Phase 2: Process sessions until source becomes unavailable
        while let Some(session) = source.wait_for_session().await? {
            info!(
                "Session started: {} - {} ({})",
                session.info.track_name,
                session.info.car_name,
                session.info.session_type
            );

            // Run existing session processing logic
            if let Err(e) = run_session(&client, session.stream).await {
                error!("Session error: {}", e);
            }

            info!("Session ended, waiting for next session...");
        }

        // Source became unavailable - loop back to wait_for_ready
        info!("Source disconnected, returning to standby...");
    }
}
```

## Resource Usage Summary

| State | CPU | Memory | Network |
|-------|-----|--------|---------|
| Unavailable (waiting for sim) | ~0% | ~5-10 MB | None |
| Idle (sim running, not driving) | <1% | +session cache | Minimal |
| Active (collecting) | 2-5% | +frame buffers (~50-100 MB) | Active uploads |

## Migration Path

### Phase 1: Add New Traits (Non-Breaking)

1. Add `TelemetrySource` trait and `SourceStatus` enum to `lapevo-telemetry`
2. Add `ActiveSession` struct
3. Keep existing `TelemetryStream` trait unchanged

### Phase 2: Implement Sources

1. Create `IbtReplaySource` implementing `TelemetrySource`
2. Create `LiveIRacingSource` implementing `TelemetrySource`
3. Update `create_replay_source()` to return `Box<dyn TelemetrySource>`

### Phase 3: Update Client

1. Replace single-session `run_session()` call with `run_client_loop()`
2. Update CLI to create appropriate source type
3. Add status display for background operation

### Phase 4: Cleanup

1. Remove `SourceHandle` struct (replaced by `TelemetrySource` trait)
2. Update tests to use new architecture

## Open Questions

1. **Graceful shutdown**: How should Ctrl+C be handled in the background loop?
   - Suggestion: Use `tokio::select!` with a shutdown signal channel

2. **Configuration**: Should polling intervals be CLI args or config file?
   - Suggestion: Config file with CLI overrides

3. **Logging/Observability**: How to surface status to user in daemon mode?
   - Suggestion: Optional status file or simple HTTP health endpoint

4. **Session deduplication**: What if user rapidly enters/exits track?
   - Suggestion: Minimum session duration threshold (e.g., 30 seconds)
