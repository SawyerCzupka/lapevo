//! Racing telemetry analysis algorithms.
//!
//! This module provides pure functions for extracting performance metrics
//! from racing telemetry data. All functions accept `&[TelemetryFrame]` slices
//! and return `Result<T, AlgsError>`.
//!
//! # Example
//!
//! ```ignore
//! use lapevo_client::algs::{extract_lap_metrics, ExtractConfig, CornerDetectionMode};
//! use lapevo_sdk::TelemetryFrame;
//!
//! let config = ExtractConfig::default();
//! let metrics = extract_lap_metrics(
//!     &frames,
//!     Some(1),
//!     Some(92.456),
//!     &config,
//!     None,
//!     None,
//!     CornerDetectionMode::Auto,
//! )?;
//! ```

mod braking;
mod corner;
mod error;
mod lateral;
mod metrics;
mod types;

// Re-export public API
pub use error::{AlgsError, AlgsResult};
pub use types::{thresholds, CornerDetectionMode, CornerSegment, ExtractConfig, TrailBrakingInfo};

pub use braking::extract_braking_zones;
pub use corner::{extract_corners_auto, extract_corners_from_segments};
pub use lateral::{compute_lateral_positions, get_lateral_position};
pub use metrics::extract_lap_metrics;
