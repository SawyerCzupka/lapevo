//! Braking zone detection and metrics extraction.
//!
//! Uses a state machine approach to detect braking zones and extract
//! comprehensive metrics including trail braking detection.

use crate::algs::error::{AlgsError, AlgsResult};
use crate::algs::types::{ExtractConfig, TrailBrakingInfo};
use crate::api::{BrakingMetrics, TelemetryFrame};

/// State machine states for braking zone detection.
#[derive(Debug, Clone)]
enum BrakingState {
    /// Waiting for brake application above threshold.
    Idle,
    /// Currently in a braking zone.
    Braking {
        start_idx: usize,
        max_pressure: f64,
        min_speed: f64,
    },
}

/// Extract all braking zones from a telemetry sequence.
///
/// # Arguments
///
/// * `frames` - Slice of telemetry frames ordered by time
/// * `config` - Extraction configuration with thresholds
///
/// # Returns
///
/// A vector of `BrakingMetrics` for each detected braking zone.
///
/// # Errors
///
/// Returns `EmptySequence` if frames is empty.
pub fn extract_braking_zones(
    frames: &[TelemetryFrame],
    config: &ExtractConfig,
) -> AlgsResult<Vec<BrakingMetrics>> {
    if frames.is_empty() {
        return Err(AlgsError::EmptySequence);
    }

    let mut zones = Vec::new();
    let mut state = BrakingState::Idle;

    for (idx, frame) in frames.iter().enumerate() {
        let is_braking = frame.brake > config.brake_threshold;

        match &mut state {
            BrakingState::Idle => {
                if is_braking {
                    state = BrakingState::Braking {
                        start_idx: idx,
                        max_pressure: frame.brake,
                        min_speed: frame.speed,
                    };
                }
            }
            BrakingState::Braking {
                start_idx,
                max_pressure,
                min_speed,
            } => {
                if is_braking {
                    // Update tracking values
                    *max_pressure = max_pressure.max(frame.brake);
                    *min_speed = min_speed.min(frame.speed);
                } else {
                    // Braking zone ended - finalize metrics
                    let metrics = finalize_braking_zone(
                        frames,
                        *start_idx,
                        idx.saturating_sub(1),
                        *max_pressure,
                        *min_speed,
                        config,
                    );
                    zones.push(metrics);
                    state = BrakingState::Idle;
                }
            }
        }
    }

    // Handle case where braking continues to end of sequence
    if let BrakingState::Braking {
        start_idx,
        max_pressure,
        min_speed,
    } = state
    {
        let metrics = finalize_braking_zone(
            frames,
            start_idx,
            frames.len() - 1,
            max_pressure,
            min_speed,
            config,
        );
        zones.push(metrics);
    }

    Ok(zones)
}

/// Finalize braking zone metrics calculation.
fn finalize_braking_zone(
    frames: &[TelemetryFrame],
    start_idx: usize,
    end_idx: usize,
    max_pressure: f64,
    min_speed: f64,
    config: &ExtractConfig,
) -> BrakingMetrics {
    let start_frame = &frames[start_idx];
    let end_frame = &frames[end_idx];

    // Calculate duration
    let braking_duration = end_frame.session_time - start_frame.session_time;

    // Calculate initial deceleration (first 5 frames or available)
    let initial_decel_end = (start_idx + 5).min(end_idx);
    let initial_deceleration = calculate_deceleration(frames, start_idx, initial_decel_end);

    // Calculate average deceleration
    let average_deceleration = calculate_deceleration(frames, start_idx, end_idx);

    // Calculate braking efficiency (deceleration per unit brake pressure)
    let braking_efficiency = if max_pressure > 0.0 {
        average_deceleration.abs() / max_pressure
    } else {
        0.0
    };

    // Detect trail braking
    let trail_braking = detect_trail_braking(frames, start_idx, end_idx, config);

    BrakingMetrics {
        braking_point_distance: start_frame.lap_distance,
        braking_point_speed: start_frame.speed,
        end_distance: end_frame.lap_distance,
        max_brake_pressure: max_pressure,
        braking_duration,
        minimum_speed: min_speed,
        initial_deceleration,
        average_deceleration,
        braking_efficiency,
        has_trail_braking: trail_braking.has_trail_braking,
        trail_brake_distance: trail_braking.distance,
        trail_brake_percentage: trail_braking.percentage,
    }
}

/// Calculate deceleration between two frame indices.
///
/// Returns deceleration in m/s^2 (negative value indicates slowing down).
fn calculate_deceleration(frames: &[TelemetryFrame], start_idx: usize, end_idx: usize) -> f64 {
    if start_idx >= end_idx || start_idx >= frames.len() || end_idx >= frames.len() {
        return 0.0;
    }

    let start = &frames[start_idx];
    let end = &frames[end_idx];

    let speed_delta = end.speed - start.speed;
    let time_delta = end.session_time - start.session_time;

    if time_delta.abs() < 1e-6 {
        0.0
    } else {
        speed_delta / time_delta
    }
}

/// Detect trail braking within a braking zone.
///
/// Trail braking occurs when the driver is both braking and steering
/// (turning into the corner while still on the brakes).
fn detect_trail_braking(
    frames: &[TelemetryFrame],
    start_idx: usize,
    end_idx: usize,
    config: &ExtractConfig,
) -> TrailBrakingInfo {
    let mut trail_brake_frames = 0usize;
    let mut total_brake_pressure = 0.0;
    let mut first_trail_idx: Option<usize> = None;
    let mut last_trail_idx: Option<usize> = None;

    for idx in start_idx..=end_idx {
        let frame = &frames[idx];
        let is_braking = frame.brake > config.brake_threshold;
        let is_steering = frame.steering_angle.abs() > config.steering_threshold;

        if is_braking && is_steering {
            trail_brake_frames += 1;
            total_brake_pressure += frame.brake;
            if first_trail_idx.is_none() {
                first_trail_idx = Some(idx);
            }
            last_trail_idx = Some(idx);
        }
    }

    if trail_brake_frames == 0 {
        return TrailBrakingInfo::default();
    }

    // Calculate trail brake distance
    let distance = match (first_trail_idx, last_trail_idx) {
        (Some(first), Some(last)) => {
            frames[last].lap_distance - frames[first].lap_distance
        }
        _ => 0.0,
    };

    // Handle wrap-around (trail braking across start/finish line)
    let distance = if distance < 0.0 {
        // This shouldn't normally happen within a single braking zone,
        // but handle it gracefully
        distance.abs()
    } else {
        distance
    };

    TrailBrakingInfo {
        has_trail_braking: true,
        distance,
        percentage: total_brake_pressure / trail_brake_frames as f64,
    }
}
