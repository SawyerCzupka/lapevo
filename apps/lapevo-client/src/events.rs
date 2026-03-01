use std::sync::Arc;

use uuid::Uuid;

use lapevo_sdk::{BrakingMetrics, SessionFrame, TelemetryFrame as ApiTelemetryFrame};
use lapevo_telemetry::TelemetryFrame;
use lapevo_eventbus::EventLike;

/// Discriminant enum for channel routing (no payload, just identifies event kind).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RacingEventKind {
    TelemetryFrameCollected,
    LapComplete,
    BrakingZoneDetected,
}

/// Main event enum for racing telemetry events.
///
/// Large, frequent payloads (like TelemetryFrame) use Arc for zero-copy broadcast.
/// Small, infrequent payloads (like LapCompletePayload) are cloned directly.
#[derive(Clone, Debug)]
pub enum RacingEvent {
    TelemetryFrameCollected(Arc<TelemetryFrame>),
    LapComplete(LapCompletePayload),
    BrakingZoneDetected(BrakingZonePayload),
}

impl EventLike for RacingEvent {
    type Kind = RacingEventKind;

    fn kind(&self) -> Self::Kind {
        match self {
            RacingEvent::TelemetryFrameCollected(_) => RacingEventKind::TelemetryFrameCollected,
            RacingEvent::LapComplete(_) => RacingEventKind::LapComplete,
            RacingEvent::BrakingZoneDetected(_) => RacingEventKind::BrakingZoneDetected,
        }
    }

    fn all_kinds() -> impl Iterator<Item = Self::Kind> {
        [
            RacingEventKind::TelemetryFrameCollected,
            RacingEventKind::LapComplete,
            RacingEventKind::BrakingZoneDetected,
        ]
        .into_iter()
    }
}

/// Completed braking zone data emitted in real-time as driver releases the brake.
#[derive(Clone, Debug)]
pub struct BrakingZonePayload {
    pub metrics: BrakingMetrics,
    pub lap_number: i32,
}

/// Completed lap data.
#[derive(Clone, Debug)]
pub struct LapCompletePayload {
    pub lap_number: i32,
    // pub lap_time_ms: Option<u64>,
    /// Lap time in seconds (from last_lap_time telemetry field).
    pub lap_time: Option<f64>,
    pub frame_count: usize,
    /// Telemetry frames collected during this lap (converted to API format).
    pub frames: Arc<Vec<ApiTelemetryFrame>>,
    /// UUID for this lap (used for server upload).
    pub lap_id: Uuid,
    /// Session metadata from the replay/live session.
    pub session: Arc<SessionFrame>,
}
