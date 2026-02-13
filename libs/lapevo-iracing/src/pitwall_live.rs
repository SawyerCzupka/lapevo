//! Live telemetry via pitwall wrapper.
//!
//! Implements `TelemetrySource` and `TelemetryStream` by wrapping pitwall's
//! `LiveConnection`. This is a temporary bridge until native shared memory
//! support is implemented in lapevo-iracing.

use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use futures::stream::StreamExt;
use futures::Stream;
use pitwall::connection::live::LiveConnection;
use pitwall::{Pitwall, PitwallFrame, SessionInfo as PitwallSessionInfo, UpdateRate};
use tokio::time::{Duration, sleep};
use tracing::{info, warn};

use lapevo_telemetry::{
    ActiveSession, SessionInfo, SourceStatus, TelemetryError, TelemetryFrame, TelemetrySource,
    TelemetryStream,
};

use crate::ibt::parse_track_length;

// ---------------------------------------------------------------------------
// PitwallFrame-derived struct for zero-copy telemetry extraction
// ---------------------------------------------------------------------------

#[derive(PitwallFrame)]
struct IracingFrame {
    #[field_name = "SessionTime"]
    session_time: f64,

    #[field_name = "Lap"]
    lap: i32,
    #[field_name = "LapDistPct"]
    lap_dist_pct: f32,
    #[field_name = "LapDist"]
    lap_dist: f32,
    #[field_name = "LapCurrentLapTime"]
    lap_current_lap_time: f32,
    #[field_name = "LapLastLapTime"]
    lap_last_lap_time: f32,
    #[field_name = "LapBestLapTime"]
    lap_best_lap_time: f32,

    #[field_name = "Speed"]
    speed: f32,
    #[field_name = "RPM"]
    rpm: f32,
    #[field_name = "Gear"]
    gear: i32,

    #[field_name = "Throttle"]
    throttle: f32,
    #[field_name = "Brake"]
    brake: f32,
    #[field_name = "Clutch"]
    clutch: f32,
    #[field_name = "SteeringWheelAngle"]
    steering_wheel_angle: f32,

    #[field_name = "LatAccel"]
    lat_accel: f32,
    #[field_name = "LongAccel"]
    long_accel: f32,
    #[field_name = "VertAccel"]
    vert_accel: f32,
    #[field_name = "YawRate"]
    yaw_rate: f32,
    #[field_name = "RollRate"]
    roll_rate: f32,
    #[field_name = "PitchRate"]
    pitch_rate: f32,

    #[field_name = "Lat"]
    lat: f64,
    #[field_name = "Lon"]
    lon: f64,
    #[field_name = "Alt"]
    alt: f32,

    #[field_name = "PlayerTrackSurface"]
    player_track_surface: i32,
}

// ---------------------------------------------------------------------------
// Conversion helpers
// ---------------------------------------------------------------------------

fn convert_frame(f: IracingFrame) -> TelemetryFrame {
    TelemetryFrame {
        session_time: f.session_time,
        lap_number: f.lap,
        lap_distance_pct: f.lap_dist_pct,
        lap_distance: f.lap_dist,
        current_lap_time: f.lap_current_lap_time,
        last_lap_time: f.lap_last_lap_time,
        best_lap_time: f.lap_best_lap_time,
        speed: f.speed,
        rpm: f.rpm,
        gear: f.gear,
        throttle: f.throttle,
        brake: f.brake,
        clutch: f.clutch,
        steering_angle: f.steering_wheel_angle,
        lat_accel: f.lat_accel,
        lon_accel: f.long_accel,
        vert_accel: f.vert_accel,
        yaw_rate: f.yaw_rate,
        roll_rate: f.roll_rate,
        pitch_rate: f.pitch_rate,
        latitude: f.lat,
        longitude: f.lon,
        altitude: f.alt,
        on_track: f.player_track_surface == 3,
    }
}

fn convert_session_info(pw: &PitwallSessionInfo, source_hz: f64) -> SessionInfo {
    let track_name = if !pw.weekend_info.track_display_name.is_empty() {
        pw.weekend_info.track_display_name.clone()
    } else {
        pw.weekend_info.track_name.clone()
    };

    let track_length = parse_track_length(&pw.weekend_info.track_length).unwrap_or(0.0);

    // Find the current driver's car via driver_car_idx
    let driver_car_idx = pw
        .driver_info
        .as_ref()
        .and_then(|di| di.driver_car_idx)
        .unwrap_or(0);

    let (car_id, car_name) = pw
        .driver_info
        .as_ref()
        .and_then(|di| di.drivers.as_ref())
        .and_then(|drivers| drivers.iter().find(|d| d.car_idx == driver_car_idx))
        .map(|d| {
            (
                d.car_id.unwrap_or(0),
                d.car_screen_name.clone().unwrap_or_default(),
            )
        })
        .unwrap_or((0, String::new()));

    let current_num = pw.session_info.current_session_num;
    let session_type = pw
        .session_info
        .sessions
        .get(current_num as usize)
        .map(|s| s.session_type.clone())
        .unwrap_or_default();

    SessionInfo {
        track_name,
        track_id: pw.weekend_info.track_id.unwrap_or(0),
        track_length,
        car_name,
        car_id,
        session_type,
        tick_rate: source_hz,
    }
}

// ---------------------------------------------------------------------------
// PitwallLiveSource
// ---------------------------------------------------------------------------

/// Live telemetry source backed by pitwall's `LiveConnection`.
///
/// Designed for always-on daemon operation: never reports `Unavailable`,
/// automatically reconnects when iRacing restarts.
pub struct PitwallLiveSource {
    connection: Option<Arc<LiveConnection>>,
    current_session_num: Option<i32>,
}

impl PitwallLiveSource {
    pub fn new() -> Self {
        Self {
            connection: None,
            current_session_num: None,
        }
    }
}

#[async_trait]
impl TelemetrySource for PitwallLiveSource {
    async fn wait_for_ready(&mut self) -> Result<(), TelemetryError> {
        // Drop stale connection so pitwall resources are freed
        self.connection = None;
        self.current_session_num = None;

        info!("Waiting for iRacing...");

        loop {
            match Pitwall::connect().await {
                Ok(conn) => {
                    info!(
                        "Connected to iRacing live telemetry ({:.0}Hz)",
                        conn.source_hz()
                    );
                    self.connection = Some(Arc::new(conn));
                    return Ok(());
                }
                Err(e) => {
                    warn!("iRacing not available ({e}), retrying in 5s...");
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn wait_for_session(&mut self) -> Result<Option<ActiveSession>, TelemetryError> {
        let conn = match &self.connection {
            Some(c) => c.clone(),
            None => return Ok(None),
        };

        // Resolve the pitwall SessionInfo for a *new* session_num.
        let pw_session = if let Some(existing) = conn.current_session() {
            let num = existing.session_info.current_session_num;
            if Some(num) != self.current_session_num {
                // Fresh session already available
                existing
            } else {
                // Wait for session_num to change (or stream to end)
                match self.wait_for_new_session(&conn, num).await {
                    Some(s) => s,
                    None => {
                        info!("Connection lost while waiting for session transition");
                        return Ok(None);
                    }
                }
            }
        } else {
            // No session yet - wait for the first one
            let mut updates = conn.session_updates();
            match updates.next().await {
                Some(s) => s,
                None => {
                    info!("Connection lost while waiting for first session");
                    return Ok(None);
                }
            }
        };

        let session_num = pw_session.session_info.current_session_num;
        self.current_session_num = Some(session_num);

        let session_info = convert_session_info(&pw_session, conn.source_hz());

        info!(
            "Session started: {} - {} ({}) [session_num={}]",
            session_info.track_name, session_info.car_name, session_info.session_type, session_num
        );

        let frame_stream = conn.subscribe::<IracingFrame>(UpdateRate::Native);

        let stream = PitwallLiveStream {
            frame_stream: Box::pin(frame_stream),
            connection: conn,
            session_info,
            session_num,
        };

        Ok(Some(ActiveSession {
            info: stream.session_info.clone(),
            stream: Box::new(stream),
        }))
    }

    fn status(&self) -> SourceStatus {
        // Never return Unavailable - the daemon should always loop back
        SourceStatus::Idle
    }

    fn status_message(&self) -> String {
        if self.connection.is_some() {
            "Connected to iRacing".into()
        } else {
            "Waiting for iRacing".into()
        }
    }
}

impl PitwallLiveSource {
    /// Block until `current_session_num` differs from `prev_num`, or the
    /// session stream ends (connection lost).
    async fn wait_for_new_session(
        &self,
        conn: &Arc<LiveConnection>,
        prev_num: i32,
    ) -> Option<Arc<PitwallSessionInfo>> {
        info!("Waiting for next session (current session_num={prev_num})...");
        let mut updates = conn.session_updates();
        loop {
            match updates.next().await {
                Some(s) if s.session_info.current_session_num != prev_num => return Some(s),
                Some(_) => continue,
                None => return None, // connection dropped
            }
        }
    }
}

// ---------------------------------------------------------------------------
// PitwallLiveStream
// ---------------------------------------------------------------------------

/// Frame stream backed by a pitwall subscription.
///
/// Returns `None` from `next_frame()` when:
/// - The pitwall frame stream ends (iRacing exited)
/// - `current_session_num` changes (session transition)
struct PitwallLiveStream {
    frame_stream: Pin<Box<dyn Stream<Item = IracingFrame> + Send>>,
    connection: Arc<LiveConnection>,
    session_info: SessionInfo,
    session_num: i32,
}

#[async_trait]
impl TelemetryStream for PitwallLiveStream {
    async fn next_frame(&mut self) -> Option<TelemetryFrame> {
        let iracing_frame = self.frame_stream.next().await?;

        // Detect session transition
        if let Some(session) = self.connection.current_session() {
            if session.session_info.current_session_num != self.session_num {
                info!(
                    "Session transition detected ({}->{}), ending stream",
                    self.session_num, session.session_info.current_session_num
                );
                return None;
            }
        }

        Some(convert_frame(iracing_frame))
    }

    fn session(&self) -> &SessionInfo {
        &self.session_info
    }
}
