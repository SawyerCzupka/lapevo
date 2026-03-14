pub mod feedback;

use std::sync::Arc;

use lapevo_eventbus::EventBus;
use lapevo_tts::Tts;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::events::{RacingEvent, RacingEventKind};
use crate::pos_service::PositionService;
use feedback::{compare_braking, generate_feedback, generate_reminder, ReferenceLap};

/// Lead distance (as fraction of lap) to trigger reminders before the braking zone.
/// e.g., 0.02 = remind ~2% of lap distance before the braking point.
const REMINDER_LEAD_PCT: f32 = 0.02;

pub struct CoachingService {
    bus: EventBus<RacingEvent>,
    tts: Arc<Tts>,
    position: PositionService,
    cancel: CancellationToken,
}

impl CoachingService {
    pub fn new(
        bus: EventBus<RacingEvent>,
        tts: Arc<Tts>,
        position: PositionService,
        cancel: CancellationToken,
    ) -> Self {
        Self {
            bus,
            tts,
            position,
            cancel,
        }
    }

    /// Spawn the coaching service as a tokio task. Returns the join handle.
    pub fn spawn(self) -> JoinHandle<()> {
        tokio::spawn(self.run())
    }

    async fn run(self) {
        let mut braking_rx = self.bus.subscribe(RacingEventKind::BrakingZoneDetected);
        let mut lap_rx = self.bus.subscribe(RacingEventKind::LapComplete);

        let mut reference = ReferenceLap::new();
        let mut reference_lap_number: Option<i32> = None;

        info!("CoachingService started");

        loop {
            tokio::select! {
                _ = self.cancel.cancelled() => {
                    info!("CoachingService shutting down");
                    break;
                }

                result = braking_rx.recv() => {
                    let Ok(RacingEvent::BrakingZoneDetected(payload)) = result else {
                        if matches!(result, Err(tokio::sync::broadcast::error::RecvError::Closed)) {
                            break;
                        }
                        continue;
                    };

                    if !reference.is_populated()
                        || reference_lap_number == Some(payload.lap_number)
                    {
                        // Still on the reference lap — accumulate zones.
                        reference.add_zone(
                            payload.metrics.clone(),
                            payload.braking_point_pct,
                        );
                        reference_lap_number = Some(payload.lap_number);
                        info!(
                            "Reference zone recorded at {:.0}m (lap {})",
                            payload.metrics.braking_point_distance, payload.lap_number
                        );
                        continue;
                    }

                    // Compare against reference.
                    let Some(ref_zone) =
                        reference.match_zone(payload.metrics.braking_point_distance)
                    else {
                        info!(
                            "No reference zone near {:.0}m — skipping",
                            payload.metrics.braking_point_distance
                        );
                        continue;
                    };

                    let Some(deviation) =
                        compare_braking(&ref_zone.metrics, &payload.metrics)
                    else {
                        continue; // within tolerance
                    };

                    // Immediate feedback
                    let feedback_text = generate_feedback(&deviation);
                    info!("Coaching feedback: {feedback_text}");
                    let tts = self.tts.clone();
                    let feedback_text_clone = feedback_text.clone();
                    tokio::spawn(async move {
                        if let Err(e) = tts.speak(&feedback_text_clone).await {
                            warn!("TTS error (feedback): {e}");
                        }
                    });

                    // Schedule reminder for next lap
                    let reminder_text = generate_reminder(&deviation);
                    let trigger_pct =
                        (ref_zone.braking_point_pct - REMINDER_LEAD_PCT).max(0.0);
                    self.schedule_reminder(trigger_pct, reminder_text);
                }

                result = lap_rx.recv() => {
                    let Ok(RacingEvent::LapComplete(payload)) = result else {
                        if matches!(result, Err(tokio::sync::broadcast::error::RecvError::Closed)) {
                            break;
                        }
                        continue;
                    };

                    info!(
                        "CoachingService: lap {} complete (ref lap: {:?})",
                        payload.lap_number, reference_lap_number
                    );
                }
            }
        }
    }

    fn schedule_reminder(&self, trigger_pct: f32, message: String) {
        let mut pos = self.position.clone();
        let tts = self.tts.clone();
        let cancel = self.cancel.clone();

        tokio::spawn(async move {
            tokio::select! {
                _ = cancel.cancelled() => {}
                _ = async {
                    pos.wait_for_next_lap().await;
                    pos.wait_until_position(trigger_pct).await;
                    info!("Coaching reminder: {message}");
                    if let Err(e) = tts.speak(&message).await {
                        warn!("TTS error (reminder): {e}");
                    }
                } => {}
            }
        });
    }
}
