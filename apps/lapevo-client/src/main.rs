use std::sync::Arc;

use clap::Parser;
use lapevo_client::cli::{Cli, Command};
use lapevo_client::session::run_session;
use lapevo_client::source::{
    LiveConfig, NetworkConfig, ReplayConfig, create_live_source, create_network_source,
    create_replay_source,
};
use lapevo_client::ui::run_interactive_replay;
use lapevo_sdk::{AuthResult, ServerAPIClient};
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

    match cli.command {
        Command::Replay {
            file,
            speed,
            daemon,
        } => {
            let config = ReplayConfig {
                file_path: file,
                speed,
            };

            if daemon {
                // Daemon mode: run once and exit
                let handle = create_replay_source(&config)?;
                run_session(&client, handle.stream).await?;
            } else {
                // Interactive mode: loop on keypresses
                run_interactive_replay(client, config).await?;
            }
        }

        Command::Live { poll_interval_ms } => {
            let config = LiveConfig { poll_interval_ms };
            let handle = create_live_source(&config)?;
            run_session(&client, handle.stream).await?;
        }

        Command::Network { address, port } => {
            let config = NetworkConfig { address, port };
            let handle = create_network_source(&config)?;
            run_session(&client, handle.stream).await?;
        }
    }

    Ok(())
}
