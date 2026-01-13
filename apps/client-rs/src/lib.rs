pub mod algs;
pub mod api;
pub mod events;
pub mod handlers;
pub mod pitwall_ext;
pub mod telem;

mod config;
mod pos_service;

use std::sync::Arc;

use crate::pos_service::PositionState;
pub use config::Config;
use eventbus::{EventBus, HandlerRegistry};
use futures::StreamExt;
use handlers::{LapHandler, LogHandler, MetricsHandler};
use pitwall::UpdateRate;
pub use pitwall_ext::AcceleratedReplayConnection;
use pos_service::PositionService;
use telem::{TelemetryFrame, extract_session_frame};
use tokio::sync::watch;
use tokio::time::sleep;
use tracing::{error, info};

use crate::api::ServerAPIClient;
use crate::events::RacingEvent;

pub async fn run_events() {
    // Open IBT replay connection
    let connection = AcceleratedReplayConnection::open(
        "../../sample_data/ligierjsp320_bathurst 2025-11-17 18-15-16.ibt",
        10f64,
    )
    .await
    .expect("Failed to open replay file");

    // Extract session info from replay file
    let session_info = connection
        .current_session()
        .expect("No session info in replay");
    let session =
        Arc::new(extract_session_frame(&session_info).expect("Failed to extract session frame"));

    info!(
        "Session: {} - {} ({:?})",
        session.track_name, session.car_name, session.session_type
    );

    // Create API client
    let client = Arc::new(
        ServerAPIClient::new("http://localhost:8000").expect("Failed to create API client"),
    );

    // Load stored auth token (continue even if not authenticated)
    match client.load_stored_token().await {
        Ok(true) => info!("Loaded stored authentication token"),
        Ok(false) => info!("No stored credentials found - uploads will require authentication"),
        Err(e) => error!("Failed to load stored credentials: {}", e),
    }

    let bus = EventBus::new(10000);

    let (tx, rx) = watch::channel(PositionState::default());
    let mut pos_service = PositionService::new(rx.clone());

    // Set up handler registry
    let mut registry = HandlerRegistry::new();
    registry.register(LapHandler::new(client.clone(), session.clone()));
    registry.register(LogHandler::new(500));
    registry.register(MetricsHandler::new(client.clone()));

    // Start all handlers
    let handles = registry.run(bus.clone());

    tokio::spawn(async move {
        let state = pos_service.wait_until_position(0.8).await;

        println!("[POS_SVC_USER] At 80% Lap Percentage!");
        println!("[POS_SVC_USER] State: {state}");
    });

    // Run telemetry collection (publisher) using the existing connection
    let bus_clone = bus.clone();
    tokio::spawn(async move {
        let mut stream = connection.subscribe::<TelemetryFrame>(UpdateRate::Max(60));
        let mut published_count: u64 = 0;

        while let Some(frame) = stream.next().await {
            // Update position service
            if let Err(error) = tx.send(PositionState {
                lap_dist_pct: frame.lap_distance_pct,
                lap_number: frame.lap_number,
            }) {
                error!("Failed to send position state: {error}");
                break;
            }

            // Publish telemetry frame to event bus
            if let Err(error) =
                bus_clone.publish(RacingEvent::TelemetryFrameCollected(Arc::new(frame)))
            {
                error!("Failed to publish telemetry frame: {error}");
            }
            published_count += 1;
        }

        info!(
            "[Telemetry Publisher] Finished - total frames published: {}",
            published_count
        );
    });

    sleep(std::time::Duration::from_secs(60)).await;

    println!("Shutting down...");

    // Signal shutdown
    registry.shutdown();

    // Wait for handlers to finish
    for handle in handles {
        let _ = handle.await;
    }
}

/// Main entry point for the library logic.
pub fn run(config: &Config) {
    println!("Server: {}", config.server_url);
}
