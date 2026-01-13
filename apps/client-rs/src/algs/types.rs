//! Types and constants for telemetry analysis algorithms.

use crate::api::CornerSegmentResponse;

/// Threshold constants for event detection.
pub mod thresholds {
    /// Minimum brake application to consider as braking (5%).
    pub const BRAKE: f64 = 0.05;
    /// Minimum steering angle to consider as turning (15%).
    pub const STEERING: f64 = 0.15;
    /// Minimum throttle application to consider as acceleration (5%).
    pub const THROTTLE: f64 = 0.05;
}

/// Configuration for metrics extraction.
#[derive(Debug, Clone)]
pub struct ExtractConfig {
    pub brake_threshold: f64,
    pub steering_threshold: f64,
    pub throttle_threshold: f64,
}

impl Default for ExtractConfig {
    fn default() -> Self {
        Self {
            brake_threshold: thresholds::BRAKE,
            steering_threshold: thresholds::STEERING,
            throttle_threshold: thresholds::THROTTLE,
        }
    }
}

/// Corner detection mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CornerDetectionMode {
    /// Auto-detect corners from steering threshold crossings.
    Auto,
    /// Use provided segments only, return empty if none.
    Segments,
    /// Use segments if available, otherwise fall back to auto-detect.
    #[default]
    SegmentsWithFallback,
}

/// Corner segment input for segment-based extraction.
///
/// Simplified version of `CornerSegmentResponse` for algorithm use.
#[derive(Debug, Clone)]
pub struct CornerSegment {
    pub corner_number: i32,
    /// Start distance in meters from start/finish line.
    pub start_distance: f64,
    /// End distance in meters from start/finish line.
    pub end_distance: f64,
}

impl From<&CornerSegmentResponse> for CornerSegment {
    fn from(response: &CornerSegmentResponse) -> Self {
        Self {
            corner_number: response.corner_number,
            start_distance: response.start_distance,
            end_distance: response.end_distance,
        }
    }
}

impl From<CornerSegmentResponse> for CornerSegment {
    fn from(response: CornerSegmentResponse) -> Self {
        Self::from(&response)
    }
}

/// Trail braking detection results.
#[derive(Debug, Clone, Default)]
pub struct TrailBrakingInfo {
    /// Whether trail braking was detected.
    pub has_trail_braking: bool,
    /// Track distance covered while trail braking (meters).
    pub distance: f64,
    /// Average brake pressure during trail braking phase (0.0-1.0).
    pub percentage: f64,
}
