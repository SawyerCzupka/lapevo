//! Racing Coach API client module.
//!
//! This module provides an HTTP client for communicating with the Racing Coach server.
//!
//! # Example
//!
//! ```ignore
//! use lapevo_sdk::{ServerAPIClient, LapTelemetry, SessionFrame};
//!
//! async fn upload_data(lap: &LapTelemetry, session: &SessionFrame) -> Result<(), Box<dyn std::error::Error>> {
//!     let client = ServerAPIClient::new("http://localhost:8000")?;
//!
//!     // Upload lap telemetry
//!     let response = client.upload_lap(lap, session, None).await?;
//!     println!("Uploaded lap: {}", response.lap_id);
//!     Ok(())
//! }
//! ```

pub mod auth;
mod client;
mod error;
mod models;

pub use auth::{has_credentials, StoredCredentials};
pub use client::{AuthResult, ServerAPIClient};
pub use error::{ApiError, ApiResult};
pub use models::{
    // Telemetry types
    BrakeLinePressure,
    // Metrics types
    BrakingMetrics,
    CornerMetrics,
    LapMetrics,
    LapTelemetry,
    // Request types
    LapUploadRequest,
    // Response types
    LapUploadResponse,
    MetricsUploadRequest,
    MetricsUploadResponse,
    SessionFrame,
    TelemetryFrame,
    TireSideData,
    TireTemps,
    TireWear,
    // Track boundary types
    TrackBoundaryListResponse,
    TrackBoundaryResponse,
    TrackBoundarySummary,
    // Corner segment types
    CornerSegmentListResponse,
    CornerSegmentResponse,
};
