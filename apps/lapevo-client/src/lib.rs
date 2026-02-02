pub mod algs;
pub mod events;
pub mod handlers;
pub mod pitwall_ext;
pub mod telem;

mod config;
mod pos_service;

use std::sync::Arc;

use crate::pos_service::PositionState;
pub use config::Config;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::terminal;
use lapevo_eventbus::{EventBus, HandlerRegistry};
use futures::StreamExt;
use handlers::{LapHandler, LogHandler, MetricsHandler};
use pitwall::UpdateRate;
pub use pitwall_ext::AcceleratedReplayConnection;
use pos_service::PositionService;
use telem::{TelemetryFrame, extract_session_frame};
use tokio::sync::watch;
use tracing::{error, info};

use lapevo_sdk::ServerAPIClient;
use crate::events::RacingEvent;

/// Interactive replay mode: idle loop waiting for keypresses.
/// `s` starts a session, `q` quits.
pub async fn run_replay_mode(ibt_path: &str, speed: f64) {
    // Create and authenticate API client once (reused across sessions)
    let client = Arc::new(
        ServerAPIClient::new("http://localhost:8000").expect("Failed to create API client"),
    );

    match client.load_stored_token().await {
        Ok(true) => {
            info!("Loaded stored authentication token — validating...");
            if let Err(e) = client.validate_credentials("racing-client").await {
                error!("Credential validation failed: {}", e);
            }
        }
        Ok(false) => info!("No stored credentials found - uploads will require authentication"),
        Err(e) => error!("Failed to load stored credentials: {}", e),
    }

    // Enable raw mode for key input
    terminal::enable_raw_mode().expect("Failed to enable raw mode");

    loop {
        println!("\r\n[IDLE] Press 's' to start a session, 'q' to quit.\r");

        // Block on keypress (in a blocking thread to not stall tokio)
        let key = tokio::task::spawn_blocking(|| loop {
            if let Ok(Event::Key(KeyEvent {
                code,
                kind: KeyEventKind::Press,
                ..
            })) = event::read()
            {
                return code;
            }
        })
        .await
        .expect("Key reader task panicked");

        match key {
            KeyCode::Char('s') => {
                terminal::disable_raw_mode().expect("Failed to disable raw mode");
                println!("\n[STARTING SESSION]");
                run_single_session(&client, ibt_path, speed).await;
                println!("\n[SESSION COMPLETE]");
                terminal::enable_raw_mode().expect("Failed to enable raw mode");
            }
            KeyCode::Char('q') => {
                println!("\r\n[EXITING]\r");
                break;
            }
            _ => {}
        }
    }

    terminal::disable_raw_mode().expect("Failed to disable raw mode");
}

/// Runs a single replay session end-to-end: opens connection, registers handlers,
/// plays back telemetry, then shuts down cleanly.
pub async fn run_single_session(client: &Arc<ServerAPIClient>, ibt_path: &str, speed: f64) {
    // Open IBT replay connection
    let mut connection = AcceleratedReplayConnection::open(ibt_path, speed)
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
    let stream = connection.subscribe::<TelemetryFrame>(UpdateRate::Max(60));
    connection.release_frame_sender();
    let publisher = tokio::spawn(async move {
        let _connection = connection; // Keep alive to prevent Drop from cancelling Driver
        let mut stream = stream;
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

    // Await publisher completion (replay EOF) instead of sleeping
    let _ = publisher.await;

    info!("Shutting down session...");

    // Signal shutdown
    registry.shutdown();

    // Wait for handlers to finish
    for handle in handles {
        let _ = handle.await;
    }
}
