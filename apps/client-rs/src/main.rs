use client_rs::run_events;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

use client_rs::api;

#[tokio::main]
async fn main() {
    // Set log level by RUST_LOG if set or default to `info`
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(false)
        .with_line_number(true)
        .init();

    info!("Racing Coach Client v{}", env!("CARGO_PKG_VERSION"));

    // let config = Config::new("http://localhost:8000");
    // client_rs::run(&config);

    println!("Hello World!");

    let client = api::ServerAPIClient::new(String::from("http://localhost:8000")).unwrap();

    if client.load_stored_token().await.unwrap() {
        client
            .validate_credentials("sawyer_laptop")
            .await
            .unwrap();
    } else {
        client.authenticate("sawyer_laptop").await.unwrap();
    }

    // let boundaries = client.fetch_track_boundaries().await.unwrap();

    // let my_boundary = client
    //     .fetch_track_boundary(boundaries.boundaries[0].id)
    //     .await
    //     .unwrap();

    // let debug_str = format!(
    //     "[Boundary] Track: {}, # points: {}",
    //     my_boundary.track_name, my_boundary.source_left_frames
    // );
    // println!("{debug_str}");

    run_events().await;

    // read_telemetry().await;

    // tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    println!("Main Done.");
}
