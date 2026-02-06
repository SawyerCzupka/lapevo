use std::sync::Arc;

use lapevo_eventbus::{EventBus, HandlerRegistry};
use lapevo_sdk::ServerAPIClient;
use lapevo_telemetry::{SourceStatus, TelemetryError, TelemetrySource, TelemetryStream};
use thiserror::Error;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use crate::events::RacingEvent;
use crate::handlers::{LapHandler, LogHandler, MetricsHandler};
use crate::pos_service::{PositionService, PositionState};
use crate::telem::extract_session_frame;

/// Errors that can occur during session execution.
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Event bus error: {0}")]
    EventBus(String),

    #[error("Source error: {0}")]
    Source(#[from] TelemetryError),
}

/// Runs a session with ANY telemetry source.
///
/// Accepts a boxed TelemetryStream, sets up the event bus and handlers,
/// then processes frames until the stream ends. Returns Ok(()) when
/// the stream ends normally.
pub async fn run_session(
    client: &Arc<ServerAPIClient>,
    mut stream: Box<dyn TelemetryStream>,
    token: CancellationToken,
) -> Result<(), SessionError> {
    // Extract session info
    let session_info = stream.session();
    let session = Arc::new(extract_session_frame(session_info));

    info!(
        "Session: {} - {} ({})",
        session.track_name, session.car_name, session.session_type
    );

    let bus = EventBus::new(10000);

    let (mut pos_service, tx) = PositionService::new();

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

    // Run telemetry collection (publisher) using the stream
    let cancel = token.clone();
    let publisher = tokio::spawn(async move {
        let mut published_count: u64 = 0;

        loop {
            tokio::select! {
                frame = stream.next_frame() => {
                    let Some(frame) = frame else { break };

                    // Update position service
                    if let Err(error) = tx.send(PositionState {
                        lap_dist_pct: frame.lap_distance_pct,
                        lap_number: frame.lap_number,
                    }) {
                        error!("Failed to send position state: {error}");
                        break;
                    }

                    // Publish telemetry frame to event bus
                    if let Err(error) = bus.publish(RacingEvent::TelemetryFrameCollected(Arc::new(frame))) {
                        error!("Failed to publish telemetry frame: {error}");
                    }
                    published_count += 1;
                }
                _ = cancel.cancelled() => {
                    info!("[Telemetry Publisher] Cancelled");
                    break;
                }
            }
        }

        info!(
            "[Telemetry Publisher] Finished - total frames published: {}",
            published_count
        );
    });

    // Await publisher completion (stream EOF or cancellation)
    let _ = publisher.await;

    // Sleep to allow handlers to finish working
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    info!("Shutting down session...");

    // Signal shutdown
    registry.shutdown();

    // Wait for handlers to finish
    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}

/// Runs the client loop driven by a `TelemetrySource`.
///
/// Outer loop: `wait_for_ready` → inner loop: `wait_for_session` → `run_session`.
/// After the inner loop exits, checks `source.status()`:
/// - `Unavailable` → exit (single-use source like IBT replay, or sim exited)
/// - Otherwise → loop back to `wait_for_ready` (future live source reconnect)
pub async fn run_client_loop(
    client: &Arc<ServerAPIClient>,
    mut source: Box<dyn TelemetrySource>,
    token: CancellationToken,
) -> Result<(), SessionError> {
    loop {
        info!("{}", source.status_message());

        tokio::select! {
            result = source.wait_for_ready() => { result?; }
            _ = token.cancelled() => {
                info!("Shutdown requested, exiting.");
                return Ok(());
            }
        }

        info!("Source ready");

        loop {
            let session = tokio::select! {
                result = source.wait_for_session() => { result? }
                _ = token.cancelled() => {
                    info!("Shutdown requested, exiting.");
                    return Ok(());
                }
            };

            let Some(session) = session else { break };

            info!(
                "Session started: {} - {} ({})",
                session.info.track_name, session.info.car_name, session.info.session_type
            );

            if let Err(e) = run_session(client, session.stream, token.clone()).await {
                error!("Session error: {e}");
            }

            if token.is_cancelled() {
                info!("Shutdown requested, exiting.");
                return Ok(());
            }

            info!("Session ended, waiting for next session...");
        }

        if source.status() == SourceStatus::Unavailable {
            info!("Source exhausted, exiting.");
            return Ok(());
        }

        info!("Source disconnected, returning to standby...");
    }
}
