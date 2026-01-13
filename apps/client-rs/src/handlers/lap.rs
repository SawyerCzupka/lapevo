use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::api::{
    LapTelemetry, ServerAPIClient, SessionFrame, TelemetryFrame as ApiTelemetryFrame,
};
use crate::events::{LapCompletePayload, RacingEvent, RacingEventKind};
use eventbus::{EventHandler, HandlerContext};

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
}

impl LapHandler {
    pub fn new(client: Arc<ServerAPIClient>, session: Arc<SessionFrame>) -> Self {
        Self {
            client,
            session,
            state: Mutex::new(LapHandlerState {
                current_lap: -1,
                frame_count: 0,
                valid: true,
                frame_buffer: Vec::with_capacity(4000), // ~60 fps * ~60 seconds
                last_lap_time: None,
            }),
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

        // Convert frame to API format and buffer it
        let api_frame: ApiTelemetryFrame = frame.as_ref().into();
        state.frame_buffer.push(api_frame);

        // Track validity (on track surface = 3)
        if state.valid && frame.track_surface != 3 {
            state.valid = false;
        }

        // Track lap time for completed laps
        if frame.last_lap_time > 0.0 {
            state.last_lap_time = Some(frame.last_lap_time as f64);
        }

        // Detect lap change
        if frame.lap_number != state.current_lap && state.current_lap >= 0 {
            let lap_number = state.current_lap;
            let frame_count = state.frame_count;
            let valid = state.valid;
            let lap_time = state.last_lap_time;

            info!(
                "Lap {} complete after {} frames. Valid: {}, Time: {:?}",
                lap_number, frame_count, valid, lap_time
            );

            // Take ownership of buffered frames
            let frames = std::mem::replace(
                &mut state.frame_buffer,
                Vec::with_capacity(4000),
            );
            let frames = Arc::new(frames);

            // Generate lap ID
            let lap_id = Uuid::new_v4();

            // Upload lap telemetry to server
            let lap_telemetry = LapTelemetry {
                frames: frames.as_ref().clone(),
                lap_time,
            };

            match self
                .client
                .upload_lap(&lap_telemetry, &self.session, Some(lap_id))
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

            // Publish LapComplete event for downstream handlers
            ctx.publish(RacingEvent::LapComplete(LapCompletePayload {
                lap_number,
                lap_time_ms: lap_time.map(|t| (t * 1000.0) as u64),
                lap_time,
                frame_count,
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
        debug!("Lap {} frame {}", state.current_lap, state.frame_count);
    }
}
