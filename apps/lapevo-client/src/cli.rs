use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "lapevo", about = "Lapevo telemetry client")]
pub struct Cli {
    /// Server URL for API calls
    #[arg(long, default_value = "http://localhost:8000", global = true)]
    pub server_url: String,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Replay telemetry from an IBT file
    Replay {
        /// Path to IBT file
        #[arg(short, long)]
        file: PathBuf,

        /// Playback speed multiplier
        #[arg(short, long, default_value_t = 5.0)]
        speed: f64,

        /// Run without interactive UI (process file once and exit)
        #[arg(long)]
        daemon: bool,
    },

    /// Connect to live iRacing session (Windows only, future)
    Live {
        /// Polling interval in milliseconds
        #[arg(long, default_value_t = 16)]
        poll_interval_ms: u64,
    },

    /// Connect to remote telemetry stream (future)
    Network {
        /// Remote server address
        #[arg(short, long)]
        address: String,

        /// Remote server port
        #[arg(short, long, default_value_t = 9000)]
        port: u16,
    },
}
