use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};
use uuid::Uuid;

use lapevo_sdk::{
    LapTelemetry, ServerAPIClient, SessionFrame, TelemetryFrame as ApiTelemetryFrame,
};
use crate::events::{LapCompletePayload, RacingEvent, RacingEventKind};
use crate::telem::to_api_frame;
use lapevo_eventbus::{EventHandler, HandlerContext};

const FRAME_BUFFER_CAPACITY: usize = 4000;
const LAP_COMPLETION_THRESHOLD: f32 = 0.95;

/// Detects lap completion by monitoring lap_number changes.
///
/// On lap completion, this handler:
/// 1. Uploads the lap telemetry to the server
/// 2. Publishes a `LapComplete` event with the frames for downstream handlers
pub struct LapHandler {
    client: Arc<ServerAPIClient>,
    session: Arc<SessionFrame>,
    state: Mutex<LapHandlerState>,
}

struct LapHandlerState {
    current_lap: i32,
    frame_count: usize,
    valid: bool,
    /// Buffer of frames for the current lap (converted to API format).
    frame_buffer: Vec<ApiTelemetryFrame>,
    /// Lap time from the last frame's last_lap_time field.
    last_lap_time: Option<f64>,
    current_lap_time_cache: Option<f64>,
    last_distance_pct: f32,
}

impl Default for LapHandlerState {
    fn default() -> Self {
        LapHandlerState {
            current_lap: -1,
            frame_count: 0,
            valid: true,
            frame_buffer: Vec::with_capacity(FRAME_BUFFER_CAPACITY),
            last_lap_time: None,
            current_lap_time_cache: None,
            last_distance_pct: 0.0,
        }
    }
}

impl LapHandler {
    pub fn new(client: Arc<ServerAPIClient>, session: Arc<SessionFrame>) -> Self {
        Self {
            client,
            session,
            state: Mutex::new(LapHandlerState::default()),
        }
    }

    /// Uploads lap telemetry to the server.
    async fn upload_lap(
        &self,
        lap_number: i32,
        lap_id: Uuid,
        is_valid: bool,
        frames: &[ApiTelemetryFrame],
        lap_time: Option<f64>,
    ) {
        let lap_telemetry = LapTelemetry {
            frames: frames.to_vec(),
            lap_time,
        };

        match self
            .client
            .upload_lap(&lap_telemetry, &self.session, Some(lap_id), is_valid)
            .await
        {
            Ok(response) => {
                info!(
                    "Uploaded lap {} (id: {}, server lap_id: {})",
                    lap_number, lap_id, response.lap_id
                );
            }
            Err(e) => {
                warn!("Failed to upload lap {}: {}", lap_number, e);
                // Continue anyway - downstream handlers can still process locally
            }
        }
    }
}

#[async_trait]
impl EventHandler<RacingEvent> for LapHandler {
    fn handles(&self) -> RacingEventKind {
        RacingEventKind::TelemetryFrameCollected
    }

    fn name(&self) -> &'static str {
        "LapHandler"
    }

    async fn handle(&self, event: RacingEvent, ctx: &HandlerContext<RacingEvent>) {
        let RacingEvent::TelemetryFrameCollected(frame) = event else {
            return;
        };

        let mut state = self.state.lock().await;
        state.frame_count += 1;

        let api_frame: ApiTelemetryFrame = to_api_frame(frame.as_ref());
        state.frame_buffer.push(api_frame);

        // Track validity (on track surface = 3)
        // More performant to add this check for each new frame to avoid scanning the entire vector later
        if state.valid && !frame.on_track {
            state.valid = false;
        }

        // // Track lap time for completed laps
        // if frame.last_lap_time > 0.0 {
        //     state.last_lap_time = Some(frame.last_lap_time as f64);
        // }

        // Detect lap change
        if frame.lap_number != state.current_lap && state.current_lap >= 0 {
            // Ignore incomplete laps
            if state.last_distance_pct < LAP_COMPLETION_THRESHOLD {
                state.current_lap = frame.lap_number;
                state.frame_buffer.clear();
                info!(
                    "Ignoring lap change to {} due to low lap distance percentage '{}'.",
                    frame.lap_number, frame.lap_distance_pct
                );
                return;
            }

            info!(
                "Lap {} complete after {} frames. Valid: {}, Time: {:?}, CurTime: {:?}",
                state.current_lap,
                state.frame_count,
                state.valid,
                state.last_lap_time,
                state.current_lap_time_cache
            );

            // TODO: Ensure this is necessary, because this seems weird.
            // Take ownership of buffered frames
            let frames = std::mem::replace(
                &mut state.frame_buffer,
                Vec::with_capacity(FRAME_BUFFER_CAPACITY),
            );
            let frames = Arc::new(frames);

            // Generate lap ID and upload to server
            let lap_id = Uuid::new_v4();
            self.upload_lap(
                state.current_lap,
                lap_id,
                state.valid,
                &frames,
                state.current_lap_time_cache,
            )
            .await;

            // Publish LapComplete event for downstream handlers
            ctx.publish(RacingEvent::LapComplete(LapCompletePayload {
                lap_number: state.current_lap,
                lap_time: state.current_lap_time_cache,
                frame_count: state.frame_count,
                frames,
                lap_id,
                session: self.session.clone(),
            }));

            // Reset for new lap
            state.frame_count = 0;
            state.valid = true;
            state.last_lap_time = None;
        }

        state.current_lap = frame.lap_number;
        state.last_distance_pct = frame.lap_distance_pct;
        if frame.current_lap_time > 0.0 {
            state.current_lap_time_cache = Some(frame.current_lap_time as f64);
        }
        debug!("Lap {} frame {}", state.current_lap, state.frame_count);
    }
}
