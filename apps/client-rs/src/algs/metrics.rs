//! Lap-level metrics aggregation.
//!
//! Provides the main entry point for extracting comprehensive metrics from
//! a lap's telemetry data.

use crate::algs::braking::extract_braking_zones;
use crate::algs::corner::{extract_corners_auto, extract_corners_from_segments};
use crate::algs::error::{AlgsError, AlgsResult};
use crate::algs::types::{CornerDetectionMode, CornerSegment, ExtractConfig};
use crate::api::{LapMetrics, TelemetryFrame, TrackBoundaryResponse};

/// Extract comprehensive metrics from a lap's telemetry.
///
/// This is the main entry point for metrics extraction. It orchestrates
/// braking zone detection, corner detection, and lap-wide statistics.
///
/// # Arguments
///
/// * `frames` - Telemetry frames for the lap, ordered by time
/// * `lap_number` - Optional lap number (defaults to first frame's lap_number)
/// * `lap_time` - Optional lap time in seconds
/// * `config` - Extraction configuration with thresholds
/// * `corner_segments` - Optional predefined corner segments
/// * `boundary` - Optional track boundary for lateral position computation
/// * `corner_mode` - How to detect corners
///
/// # Example
///
/// ```ignore
/// use client_rs::algs::{extract_lap_metrics, ExtractConfig, CornerDetectionMode};
///
/// let config = ExtractConfig::default();
/// let metrics = extract_lap_metrics(
///     &frames,
///     Some(1),
///     Some(92.456),
///     &config,
///     None,
///     None,
///     CornerDetectionMode::Auto,
/// )?;
///
/// println!("Found {} braking zones and {} corners",
///     metrics.total_braking_zones, metrics.total_corners);
/// ```
///
/// # Errors
///
/// Returns `EmptySequence` if frames is empty, or propagates errors from
/// braking/corner extraction.
pub fn extract_lap_metrics(
    frames: &[TelemetryFrame],
    lap_number: Option<i32>,
    lap_time: Option<f64>,
    config: &ExtractConfig,
    corner_segments: Option<&[CornerSegment]>,
    boundary: Option<&TrackBoundaryResponse>,
    corner_mode: CornerDetectionMode,
) -> AlgsResult<LapMetrics> {
    if frames.is_empty() {
        return Err(AlgsError::EmptySequence);
    }

    let lap_number = lap_number.unwrap_or(frames[0].lap_number);

    // Extract braking zones
    let braking_zones = extract_braking_zones(frames, config)?;

    // Determine corner extraction method based on mode
    let corners = match corner_mode {
        CornerDetectionMode::Auto => extract_corners_auto(frames, config)?,

        CornerDetectionMode::Segments => {
            match (corner_segments, get_track_length(boundary)) {
                (Some(segs), Some(len)) if !segs.is_empty() => {
                    extract_corners_from_segments(frames, segs, len, boundary, config)?
                }
                _ => Vec::new(), // Segments mode with no segments returns empty
            }
        }

        CornerDetectionMode::SegmentsWithFallback => {
            match (corner_segments, get_track_length(boundary)) {
                (Some(segs), Some(len)) if !segs.is_empty() => {
                    extract_corners_from_segments(frames, segs, len, boundary, config)?
                }
                _ => extract_corners_auto(frames, config)?,
            }
        }
    };

    // Compute lap-wide statistics
    let (max_speed, min_speed) = frames
        .iter()
        .fold((f64::MIN, f64::MAX), |(max, min), f| {
            (max.max(f.speed), min.min(f.speed))
        });

    let average_corner_speed = if corners.is_empty() {
        0.0
    } else {
        corners.iter().map(|c| c.apex_speed).sum::<f64>() / corners.len() as f64
    };

    let total_corners = corners.len() as i32;
    let total_braking_zones = braking_zones.len() as i32;

    Ok(LapMetrics {
        lap_number,
        lap_time,
        braking_zones,
        corners,
        total_corners,
        total_braking_zones,
        average_corner_speed,
        max_speed,
        min_speed,
    })
}

/// Get track length from boundary if available.
fn get_track_length(boundary: Option<&TrackBoundaryResponse>) -> Option<f64> {
    boundary.and_then(|b| b.track_length)
}
