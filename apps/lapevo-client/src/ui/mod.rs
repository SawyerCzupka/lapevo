use std::sync::Arc;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::terminal;
use lapevo_iracing::IbtReplaySource;
use lapevo_sdk::ServerAPIClient;
use lapevo_telemetry::{TelemetryError, TelemetrySource};
use thiserror::Error;
use tokio_util::sync::CancellationToken;

use crate::session::{run_session, SessionError};
use crate::source::ReplayConfig;

/// Errors that can occur in interactive mode.
#[derive(Debug, Error)]
pub enum InteractiveError {
    #[error("Telemetry error: {0}")]
    Telemetry(#[from] TelemetryError),

    #[error("Session error: {0}")]
    Session(#[from] SessionError),

    #[error("Terminal error: {0}")]
    Terminal(#[from] std::io::Error),
}

/// Replay-specific interactive mode.
///
/// Loop: wait for 's' to start session, 'q' to quit.
pub async fn run_interactive_replay(
    client: Arc<ServerAPIClient>,
    config: ReplayConfig,
) -> Result<(), InteractiveError> {
    // Enable raw mode for key input
    terminal::enable_raw_mode()?;

    let result = interactive_loop(&client, &config).await;

    // Always disable raw mode before returning
    terminal::disable_raw_mode()?;

    result
}

async fn interactive_loop(
    client: &Arc<ServerAPIClient>,
    config: &ReplayConfig,
) -> Result<(), InteractiveError> {
    loop {
        println!("\r\n[IDLE] Press 's' to start a session, 'q' to quit.\r");

        // Block on keypress (in a blocking thread to not stall tokio)
        let key = tokio::task::spawn_blocking(|| {
            loop {
                if let Ok(Event::Key(KeyEvent {
                    code,
                    kind: KeyEventKind::Press,
                    ..
                })) = event::read()
                {
                    return code;
                }
            }
        })
        .await
        .expect("Key reader task panicked");

        match key {
            KeyCode::Char('s') => {
                terminal::disable_raw_mode()?;
                println!("\n[STARTING SESSION]");

                let mut source =
                    IbtReplaySource::new(config.file_path.clone(), config.speed)?;
                if let Some(session) = source.wait_for_session().await? {
                    run_session(client, session.stream, CancellationToken::new(), None).await?;
                }

                println!("\n[SESSION COMPLETE]");
                terminal::enable_raw_mode()?;
            }
            KeyCode::Char('q') => {
                println!("\r\n[EXITING]\r");
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
