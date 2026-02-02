use clap::Parser;
use lapevo_client::run_replay_mode;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

#[derive(Parser)]
#[command(name = "racing-coach-client", about = "Racing Coach iRacing client")]
struct Cli {
    /// IBT file path for replay mode
    #[arg(
        long,
        default_value = "../../sample_data/ligierjsp320_bathurst 2025-11-17 18-15-16.ibt"
    )]
    ibt_path: String,

    /// Playback speed multiplier
    #[arg(long, default_value_t = 5.0)]
    speed: f64,
}

#[tokio::main]
async fn main() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(false)
        .with_line_number(true)
        .init();

    info!("Racing Coach Client v{}", env!("CARGO_PKG_VERSION"));

    let cli = Cli::parse();

    run_replay_mode(&cli.ibt_path, cli.speed).await;
}
