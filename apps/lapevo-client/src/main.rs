use std::sync::Arc;

use clap::Parser;
use lapevo_client::cli::{Cli, Command};
use lapevo_client::session::run_client_loop;
use lapevo_client::source::ReplayConfig;
use lapevo_client::ui::run_interactive_replay;
use lapevo_iracing::IbtReplaySource;
use lapevo_sdk::{AuthResult, ServerAPIClient};
use tokio_util::sync::CancellationToken;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(false)
        .with_line_number(true)
        .init();

    info!("Lapevo Client v{}", env!("CARGO_PKG_VERSION"));

    let cli = Cli::parse();

    // Authenticate once at startup
    let (client, auth_result) =
        ServerAPIClient::new_with_auth(&cli.server_url, "lapevo-client", true, true).await?;
    let client = Arc::new(client);

    if auth_result == AuthResult::Unauthenticated {
        info!("Client is not authenticated - uploads may require authentication");
        // could also crash the program here if it MUST have auth.
    }

    // Set up graceful shutdown
    let token = CancellationToken::new();
    let shutdown_token = token.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
        info!("Ctrl+C received, shutting down...");
        shutdown_token.cancel();
    });

    match cli.command {
        Command::Replay {
            file,
            speed,
            daemon,
        } => {
            if daemon {
                let source = IbtReplaySource::new(file, speed)?;
                run_client_loop(&client, Box::new(source), token).await?;
            } else {
                let config = ReplayConfig {
                    file_path: file,
                    speed,
                };
                run_interactive_replay(client, config).await?;
            }
        }

        Command::Live { .. } => {
            eprintln!("Live mode is not yet implemented (requires Windows shared memory).");
            std::process::exit(1);
        }

        Command::Network { .. } => {
            eprintln!("Network mode is not yet implemented.");
            std::process::exit(1);
        }
    }

    Ok(())
}
