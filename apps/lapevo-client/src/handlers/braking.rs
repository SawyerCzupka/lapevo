use std::sync::{Arc, Mutex};

use crate::algs::thresholds;
use crate::events::{BrakingZonePayload, RacingEvent, RacingEventKind};
use async_trait::async_trait;
use lapevo_eventbus::{EventHandler, HandlerContext};
use lapevo_sdk::BrakingMetrics;
use lapevo_telemetry::TelemetryFrame;
use tracing::info;

/// Accumulated data for an in-progress braking zone.
struct ActiveZoneData {
    frames: Vec<Arc<TelemetryFrame>>,
    max_pressure: f32,
    min_speed: f32,
    start_lap: i32,
}

/// Mutable state for the braking handler.
#[derive(Default)]
struct BrakingHandlerState {
    /// `None` = idle (watching for braking), `Some` = currently in a zone.
    active_zone: Option<ActiveZoneData>,
}

impl BrakingHandlerState {
    /// Process a single telemetry frame, returning a payload if a zone just completed.
    fn process_frame(
        &mut self,
        frame: &Arc<TelemetryFrame>,
        brake_threshold: f32,
        steering_threshold: f32,
    ) -> Option<BrakingZonePayload> {
        // Lap rollover: discard any in-progress zone.
        if let Some(zone) = &self.active_zone
            && zone.start_lap != frame.lap_number
        {
            self.active_zone = None;
        }

        if frame.brake > brake_threshold {
            // Braking: start or continue accumulating.
            match &mut self.active_zone {
                Some(zone) => {
                    zone.max_pressure = zone.max_pressure.max(frame.brake);
                    zone.min_speed = zone.min_speed.min(frame.speed);
                    zone.frames.push(Arc::clone(frame));
                }
                None => {
                    self.active_zone = Some(ActiveZoneData {
                        frames: vec![Arc::clone(frame)],
                        max_pressure: frame.brake,
                        min_speed: frame.speed,
                        start_lap: frame.lap_number,
                    });
                }
            }
            None
        } else {
            // Not braking: finalize if a zone was active.
            self.active_zone.take().map(|zone| BrakingZonePayload {
                metrics: finalize_braking_zone(
                    &zone.frames,
                    zone.max_pressure,
                    zone.min_speed,
                    steering_threshold,
                ),
                lap_number: zone.start_lap,
            })
        }
    }
}

/// Detects braking zones in real-time and emits `BrakingZoneDetected` events.
pub struct BrakingHandler {
    state: Mutex<BrakingHandlerState>,
    brake_threshold: f32,
    steering_threshold: f32,
}

impl BrakingHandler {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(BrakingHandlerState::default()),
            brake_threshold: thresholds::BRAKE as f32,
            steering_threshold: thresholds::STEERING as f32,
        }
    }
}

impl Default for BrakingHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventHandler<RacingEvent> for BrakingHandler {
    fn handles(&self) -> RacingEventKind {
        RacingEventKind::TelemetryFrameCollected
    }

    fn name(&self) -> &'static str {
        "BrakingHandler"
    }

    async fn handle(&self, event: RacingEvent, ctx: &HandlerContext<RacingEvent>) {
        let RacingEvent::TelemetryFrameCollected(frame) = event else {
            return;
        };

        let maybe_payload = {
            let mut state = self.state.lock().unwrap();
            state.process_frame(&frame, self.brake_threshold, self.steering_threshold)
        }; // lock dropped before publish

        if let Some(payload) = maybe_payload {
            info!(
                "BrakingZone: \nStart Dist: {},\nEntry Speed: {}, \nMin Speed: {}",
                payload.metrics.braking_point_distance,
                payload.metrics.braking_point_speed,
                payload.metrics.minimum_speed
            );
            ctx.publish(RacingEvent::BrakingZoneDetected(payload));
        }
    }
}

// ---------------------------------------------------------------------------
// Metrics finalization helpers
// ---------------------------------------------------------------------------

/// Calculate deceleration (m/s²) between two frames.
fn calculate_deceleration(start: &TelemetryFrame, end: &TelemetryFrame) -> f64 {
    let dt = end.session_time - start.session_time;
    if dt.abs() > 1e-6 {
        (end.speed as f64 - start.speed as f64) / dt
    } else {
        0.0
    }
}

/// Detect trail braking (simultaneous braking and steering) within a frame slice.
fn detect_trail_braking(
    frames: &[Arc<TelemetryFrame>],
    steering_threshold: f32,
) -> (bool, f64, f64) {
    let mut count = 0usize;
    let mut total_pressure = 0.0f64;
    let mut first: Option<&TelemetryFrame> = None;
    let mut last: Option<&TelemetryFrame> = None;

    for f in frames {
        if f.brake > 0.0 && f.steering_angle.abs() > steering_threshold {
            count += 1;
            total_pressure += f.brake as f64;
            first = first.or(Some(f));
            last = Some(f);
        }
    }

    if count == 0 {
        return (false, 0.0, 0.0);
    }

    let distance = match (first, last) {
        (Some(a), Some(b)) => (b.lap_distance - a.lap_distance).abs() as f64,
        _ => 0.0,
    };

    (true, distance, total_pressure / count as f64)
}

/// Build `BrakingMetrics` from an accumulated frame buffer.
fn finalize_braking_zone(
    frames: &[Arc<TelemetryFrame>],
    max_pressure: f32,
    min_speed: f32,
    steering_threshold: f32,
) -> BrakingMetrics {
    let (Some(start), Some(end)) = (frames.first(), frames.last()) else {
        return BrakingMetrics {
            braking_point_distance: 0.0,
            braking_point_speed: 0.0,
            end_distance: 0.0,
            max_brake_pressure: 0.0,
            braking_duration: 0.0,
            minimum_speed: 0.0,
            initial_deceleration: 0.0,
            average_deceleration: 0.0,
            braking_efficiency: 0.0,
            has_trail_braking: false,
            trail_brake_distance: 0.0,
            trail_brake_percentage: 0.0,
        };
    };

    let braking_duration = end.session_time - start.session_time;
    let average_deceleration = calculate_deceleration(start, end);

    // Initial deceleration over first 5 frames.
    let initial_end = frames.len().min(5).saturating_sub(1);
    let initial_deceleration = if initial_end > 0 {
        calculate_deceleration(start, &frames[initial_end])
    } else {
        0.0
    };

    let max_pressure_f64 = max_pressure as f64;
    let braking_efficiency = if max_pressure_f64 > 0.0 {
        average_deceleration.abs() / max_pressure_f64
    } else {
        0.0
    };

    let (has_trail_braking, trail_brake_distance, trail_brake_percentage) =
        detect_trail_braking(frames, steering_threshold);

    BrakingMetrics {
        braking_point_distance: start.lap_distance as f64,
        braking_point_speed: start.speed as f64,
        end_distance: end.lap_distance as f64,
        max_brake_pressure: max_pressure_f64,
        braking_duration,
        minimum_speed: min_speed as f64,
        initial_deceleration,
        average_deceleration,
        braking_efficiency,
        has_trail_braking,
        trail_brake_distance,
        trail_brake_percentage,
    }
}
