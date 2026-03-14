//! Integration test: publish events to a bus and verify coaching service behavior.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use lapevo_eventbus::EventBus;
use lapevo_sdk::BrakingMetrics;
use lapevo_tts::{AudioClip, Result as TtsResult, TtsBackend};
use tokio_util::sync::CancellationToken;

use lapevo_client::coaching::CoachingService;
use lapevo_client::events::{BrakingZonePayload, RacingEvent};
use lapevo_client::pos_service::PositionService;

/// Mock TTS backend that records all spoken text.
struct MockTtsBackend {
    spoken: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl TtsBackend for MockTtsBackend {
    async fn synth(&self, text: &str) -> TtsResult<AudioClip> {
        self.spoken.lock().unwrap().push(text.to_string());
        Ok(AudioClip {
            samples: vec![],
            sample_rate: 24000,
            channels: 1,
        })
    }
}

fn make_metrics(braking_point_distance: f64, minimum_speed: f64) -> BrakingMetrics {
    BrakingMetrics {
        braking_point_distance,
        braking_point_speed: 70.0,
        end_distance: braking_point_distance + 80.0,
        max_brake_pressure: 0.95,
        braking_duration: 1.5,
        minimum_speed,
        initial_deceleration: -15.0,
        average_deceleration: -12.0,
        braking_efficiency: 12.6,
        has_trail_braking: false,
        trail_brake_distance: 0.0,
        trail_brake_percentage: 0.0,
    }
}

#[tokio::test]
#[ignore = "requires audio device (rodio AudioPlayer); run manually on a system with audio"]
async fn coaching_service_speaks_feedback_on_deviation() {
    let spoken = Arc::new(Mutex::new(Vec::new()));
    let backend = MockTtsBackend {
        spoken: spoken.clone(),
    };
    let tts = Arc::new(
        lapevo_tts::Tts::with_backend(backend).expect("mock TTS init — requires audio device"),
    );

    let bus = EventBus::<RacingEvent>::new(100);
    let (pos_service, _pos_tx) = PositionService::new();
    let cancel = CancellationToken::new();

    let service = CoachingService::new(bus.clone(), tts, pos_service, cancel.clone());
    let handle = service.spawn();

    // Give the service time to subscribe
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Publish reference lap braking zone (lap 1)
    bus.publish(RacingEvent::BrakingZoneDetected(BrakingZonePayload {
        metrics: make_metrics(500.0, 30.0),
        lap_number: 1,
        braking_point_pct: 0.35,
    }))
    .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Publish a deviated braking zone on lap 2 (braked 25m too early)
    bus.publish(RacingEvent::BrakingZoneDetected(BrakingZonePayload {
        metrics: make_metrics(475.0, 30.0),
        lap_number: 2,
        braking_point_pct: 0.33,
    }))
    .unwrap();

    // Wait for TTS to be called
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    cancel.cancel();
    let _ = handle.await;

    let spoken = spoken.lock().unwrap();
    assert!(
        !spoken.is_empty(),
        "Expected TTS feedback but nothing was spoken"
    );
    assert!(
        spoken[0].to_lowercase().contains("early"),
        "Expected feedback about braking early, got: {}",
        spoken[0]
    );
}
