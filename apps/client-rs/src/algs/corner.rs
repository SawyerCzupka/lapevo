//! Corner detection and metrics extraction.
//!
//! Provides two corner detection modes:
//! - Auto-detection based on steering angle thresholds
//! - Segment-based using predefined corner boundaries

use crate::algs::error::{AlgsError, AlgsResult};
use crate::algs::lateral::compute_lateral_positions;
use crate::algs::types::{CornerSegment, ExtractConfig};
use crate::api::{CornerMetrics, TelemetryFrame, TrackBoundaryResponse};

/// Extract corners using steering threshold detection (auto-detect mode).
///
/// Corners are detected when steering angle exceeds the threshold.
/// - Turn-in: steering exceeds threshold
/// - Exit: steering drops below threshold
/// - Apex: maximum |lateral_acceleration| within the corner
///
/// # Arguments
///
/// * `frames` - Slice of telemetry frames ordered by time
/// * `config` - Extraction configuration with thresholds
///
/// # Errors
///
/// Returns `EmptySequence` if frames is empty.
pub fn extract_corners_auto(
    frames: &[TelemetryFrame],
    config: &ExtractConfig,
) -> AlgsResult<Vec<CornerMetrics>> {
    if frames.is_empty() {
        return Err(AlgsError::EmptySequence);
    }

    let mut corners = Vec::new();
    let mut in_corner = false;
    let mut turn_in_idx: usize = 0;

    for (idx, frame) in frames.iter().enumerate() {
        let is_turning = frame.steering_angle.abs() > config.steering_threshold;

        if !in_corner && is_turning {
            // Corner entry detected
            in_corner = true;
            turn_in_idx = idx;
        } else if in_corner && !is_turning {
            // Corner exit detected
            in_corner = false;
            let exit_idx = idx;

            if let Some(metrics) =
                compute_corner_metrics(frames, turn_in_idx, exit_idx, None, config)
            {
                corners.push(metrics);
            }
        }
    }

    // Handle corner that extends to end of sequence
    if in_corner {
        let exit_idx = frames.len() - 1;
        if let Some(metrics) =
            compute_corner_metrics(frames, turn_in_idx, exit_idx, None, config)
        {
            corners.push(metrics);
        }
    }

    Ok(corners)
}

/// Extract corners from predefined segments.
///
/// Uses corner segment definitions to identify corner boundaries.
/// Lateral positions are computed internally when a track boundary is provided.
///
/// # Arguments
///
/// * `frames` - Slice of telemetry frames ordered by time
/// * `segments` - Predefined corner segment definitions
/// * `track_length` - Track length in meters
/// * `boundary` - Optional track boundary for lateral position computation
/// * `config` - Extraction configuration with thresholds
///
/// # Errors
///
/// Returns `EmptySequence` if frames is empty, or `EmptyCornerSegment` if a
/// segment contains no frames.
pub fn extract_corners_from_segments(
    frames: &[TelemetryFrame],
    segments: &[CornerSegment],
    track_length: f64,
    boundary: Option<&TrackBoundaryResponse>,
    config: &ExtractConfig,
) -> AlgsResult<Vec<CornerMetrics>> {
    if frames.is_empty() {
        return Err(AlgsError::EmptySequence);
    }

    // Compute lateral positions if boundary is available
    let lateral_positions = boundary
        .map(|b| compute_lateral_positions(b, frames))
        .transpose()?;

    let mut corners = Vec::new();

    for segment in segments {
        // Find frames within this segment's distance range
        let segment_indices: Vec<usize> = frames
            .iter()
            .enumerate()
            .filter_map(|(idx, frame)| {
                if is_in_segment(frame.lap_distance, segment, track_length) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();

        if segment_indices.is_empty() {
            return Err(AlgsError::EmptyCornerSegment {
                corner_number: segment.corner_number,
                start: segment.start_distance,
                end: segment.end_distance,
            });
        }

        let turn_in_idx = *segment_indices.first().unwrap();
        let exit_idx = *segment_indices.last().unwrap();

        if let Some(metrics) = compute_corner_metrics_with_lateral(
            frames,
            turn_in_idx,
            exit_idx,
            lateral_positions.as_deref(),
            &segment_indices,
            config,
        ) {
            corners.push(metrics);
        }
    }

    Ok(corners)
}

/// Check if a distance falls within a corner segment.
fn is_in_segment(lap_distance: f64, segment: &CornerSegment, _track_length: f64) -> bool {
    let start = segment.start_distance;
    let end = segment.end_distance;

    if start <= end {
        // Normal segment (doesn't wrap around start/finish)
        lap_distance >= start && lap_distance <= end
    } else {
        // Segment wraps around start/finish line
        lap_distance >= start || lap_distance <= end
    }
}

/// Compute corner metrics for a detected corner (auto-detect mode).
fn compute_corner_metrics(
    frames: &[TelemetryFrame],
    turn_in_idx: usize,
    exit_idx: usize,
    _lateral_positions: Option<&[f64]>,
    config: &ExtractConfig,
) -> Option<CornerMetrics> {
    if turn_in_idx >= exit_idx || exit_idx >= frames.len() {
        return None;
    }

    let turn_in = &frames[turn_in_idx];
    let exit = &frames[exit_idx];

    // Find apex using max lateral acceleration
    let apex_idx = find_apex_by_lateral_g(frames, turn_in_idx, exit_idx);
    let apex = &frames[apex_idx];

    // Find throttle application point (first throttle > threshold after apex)
    let throttle_idx = find_throttle_application(frames, apex_idx, exit_idx, config);
    let throttle_frame = &frames[throttle_idx];

    // Compute frame range statistics
    let stats = compute_frame_range_stats(frames, turn_in_idx, exit_idx);

    // Compute time in corner
    let time_in_corner = exit.session_time - turn_in.session_time;

    // Compute corner distance (handle wrap-around)
    let corner_distance = if exit.lap_distance >= turn_in.lap_distance {
        exit.lap_distance - turn_in.lap_distance
    } else {
        // Wrapped around start/finish
        exit.lap_distance + (turn_in.lap_distance - turn_in.lap_distance.floor())
    };

    Some(CornerMetrics {
        turn_in_distance: turn_in.lap_distance,
        apex_distance: apex.lap_distance,
        exit_distance: exit.lap_distance,
        throttle_application_distance: throttle_frame.lap_distance,
        turn_in_speed: turn_in.speed,
        apex_speed: apex.speed,
        exit_speed: exit.speed,
        throttle_application_speed: throttle_frame.speed,
        max_lateral_g: stats.max_lateral_g,
        time_in_corner,
        corner_distance,
        max_steering_angle: stats.max_steering_angle,
        speed_loss: turn_in.speed - apex.speed,
        speed_gain: exit.speed - apex.speed,
    })
}

/// Compute corner metrics with lateral position data (segment-based mode).
fn compute_corner_metrics_with_lateral(
    frames: &[TelemetryFrame],
    turn_in_idx: usize,
    exit_idx: usize,
    lateral_positions: Option<&[f64]>,
    segment_indices: &[usize],
    config: &ExtractConfig,
) -> Option<CornerMetrics> {
    if turn_in_idx >= exit_idx || exit_idx >= frames.len() {
        return None;
    }

    let turn_in = &frames[turn_in_idx];
    let exit = &frames[exit_idx];

    // Find apex using lateral position if available, otherwise fall back to lateral G
    let apex_idx = if let Some(positions) = lateral_positions {
        find_apex_by_lateral_position(frames, segment_indices, positions)
    } else {
        find_apex_by_lateral_g(frames, turn_in_idx, exit_idx)
    };
    let apex = &frames[apex_idx];

    // Find throttle application point
    let throttle_idx = find_throttle_application(frames, apex_idx, exit_idx, config);
    let throttle_frame = &frames[throttle_idx];

    // Compute frame range statistics
    let stats = compute_frame_range_stats(frames, turn_in_idx, exit_idx);

    let time_in_corner = exit.session_time - turn_in.session_time;

    let corner_distance = if exit.lap_distance >= turn_in.lap_distance {
        exit.lap_distance - turn_in.lap_distance
    } else {
        exit.lap_distance + (turn_in.lap_distance - turn_in.lap_distance.floor())
    };

    Some(CornerMetrics {
        turn_in_distance: turn_in.lap_distance,
        apex_distance: apex.lap_distance,
        exit_distance: exit.lap_distance,
        throttle_application_distance: throttle_frame.lap_distance,
        turn_in_speed: turn_in.speed,
        apex_speed: apex.speed,
        exit_speed: exit.speed,
        throttle_application_speed: throttle_frame.speed,
        max_lateral_g: stats.max_lateral_g,
        time_in_corner,
        corner_distance,
        max_steering_angle: stats.max_steering_angle,
        speed_loss: turn_in.speed - apex.speed,
        speed_gain: exit.speed - apex.speed,
    })
}

/// Find apex index using maximum lateral acceleration.
fn find_apex_by_lateral_g(frames: &[TelemetryFrame], start_idx: usize, end_idx: usize) -> usize {
    frames[start_idx..=end_idx]
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| {
            a.lateral_acceleration
                .abs()
                .partial_cmp(&b.lateral_acceleration.abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(idx, _)| start_idx + idx)
        .unwrap_or(start_idx)
}

/// Find apex index using lateral position.
///
/// For a left corner (negative steering), apex is at minimum lateral position (closest to left).
/// For a right corner (positive steering), apex is at maximum lateral position (closest to right).
fn find_apex_by_lateral_position(
    frames: &[TelemetryFrame],
    segment_indices: &[usize],
    lateral_positions: &[f64],
) -> usize {
    if segment_indices.is_empty() {
        return 0;
    }

    // Determine corner direction from average steering
    let avg_steering: f64 = segment_indices
        .iter()
        .map(|&idx| frames[idx].steering_angle)
        .sum::<f64>()
        / segment_indices.len() as f64;

    let is_left_corner = avg_steering < 0.0;

    if is_left_corner {
        // Left corner: apex = minimum lateral position (closest to left edge)
        segment_indices
            .iter()
            .copied()
            .min_by(|&a, &b| {
                lateral_positions[a]
                    .partial_cmp(&lateral_positions[b])
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(segment_indices[0])
    } else {
        // Right corner: apex = maximum lateral position (closest to right edge)
        segment_indices
            .iter()
            .copied()
            .max_by(|&a, &b| {
                lateral_positions[a]
                    .partial_cmp(&lateral_positions[b])
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(segment_indices[0])
    }
}

/// Find the first frame with throttle above threshold after the apex.
fn find_throttle_application(
    frames: &[TelemetryFrame],
    apex_idx: usize,
    exit_idx: usize,
    config: &ExtractConfig,
) -> usize {
    for idx in apex_idx..=exit_idx {
        if frames[idx].throttle > config.throttle_threshold {
            return idx;
        }
    }
    // No throttle application found, return exit
    exit_idx
}

/// Statistics computed over a frame range.
struct FrameRangeStats {
    max_lateral_g: f64,
    max_steering_angle: f64,
}

/// Compute statistics over a range of frames.
fn compute_frame_range_stats(
    frames: &[TelemetryFrame],
    start_idx: usize,
    end_idx: usize,
) -> FrameRangeStats {
    let mut max_lateral_g = 0.0f64;
    let mut max_steering_angle = 0.0f64;

    for frame in &frames[start_idx..=end_idx] {
        max_lateral_g = max_lateral_g.max(frame.lateral_acceleration.abs());
        max_steering_angle = max_steering_angle.max(frame.steering_angle.abs());
    }

    FrameRangeStats {
        max_lateral_g,
        max_steering_angle,
    }
}
