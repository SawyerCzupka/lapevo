//! Lateral position calculation from track boundaries.
//!
//! This module calculates a car's lateral position relative to track boundaries
//! using GPS coordinate projection.

use crate::algs::error::{AlgsError, AlgsResult};
use lapevo_sdk::{TelemetryFrame, TrackBoundaryResponse};

/// Calculate lateral position for a single point on the track.
///
/// Returns a value where:
/// - `-1.0` = left boundary
/// -  `0.0` = track center
/// - `+1.0` = right boundary
///
/// Values can exceed `[-1, 1]` if the car is outside the track boundaries.
///
/// # Arguments
///
/// * `boundary` - Track boundary data with GPS coordinates
/// * `lap_distance_pct` - Normalized lap distance (0.0 to 1.0)
/// * `latitude` - Car's GPS latitude
/// * `longitude` - Car's GPS longitude
pub fn get_lateral_position(
    boundary: &TrackBoundaryResponse,
    lap_distance_pct: f64,
    latitude: f64,
    longitude: f64,
) -> f64 {
    let grid_size = boundary.grid_size as usize;
    if grid_size == 0 {
        return 0.0;
    }

    // Normalize lap distance to [0, 1)
    let normalized_pct = lap_distance_pct.rem_euclid(1.0);

    // Calculate fractional grid index
    let idx_float = normalized_pct * (grid_size as f64);
    let idx_low = (idx_float.floor() as usize).min(grid_size - 1);
    let idx_high = (idx_low + 1) % grid_size;
    let t = idx_float.fract();

    // Interpolate boundary positions at this lap distance
    let (left_lat, left_lon) = interpolate_boundary(
        boundary.left_latitude[idx_low],
        boundary.left_longitude[idx_low],
        boundary.left_latitude[idx_high],
        boundary.left_longitude[idx_high],
        t,
    );

    let (right_lat, right_lon) = interpolate_boundary(
        boundary.right_latitude[idx_low],
        boundary.right_longitude[idx_low],
        boundary.right_latitude[idx_high],
        boundary.right_longitude[idx_high],
        t,
    );

    // Project car position onto track width vector
    project_onto_track(left_lat, left_lon, right_lat, right_lon, latitude, longitude)
}

/// Compute lateral positions for a slice of telemetry frames.
///
/// Returns a `Vec<f64>` with the lateral position for each frame.
///
/// # Arguments
///
/// * `boundary` - Track boundary data with GPS coordinates
/// * `frames` - Slice of telemetry frames
///
/// # Errors
///
/// Returns `InvalidBoundary` if the boundary data is empty or malformed.
pub fn compute_lateral_positions(
    boundary: &TrackBoundaryResponse,
    frames: &[TelemetryFrame],
) -> AlgsResult<Vec<f64>> {
    if boundary.grid_size == 0 || boundary.grid_distance_pct.is_empty() {
        return Err(AlgsError::InvalidBoundary {
            reason: "boundary has no grid data".to_string(),
        });
    }

    if boundary.left_latitude.len() != boundary.grid_size as usize
        || boundary.right_latitude.len() != boundary.grid_size as usize
    {
        return Err(AlgsError::InvalidBoundary {
            reason: "boundary coordinate arrays do not match grid_size".to_string(),
        });
    }

    let positions: Vec<f64> = frames
        .iter()
        .map(|frame| {
            get_lateral_position(boundary, frame.lap_distance_pct, frame.latitude, frame.longitude)
        })
        .collect();

    Ok(positions)
}

/// Linear interpolation between two boundary points.
#[inline]
fn interpolate_boundary(
    lat1: f64,
    lon1: f64,
    lat2: f64,
    lon2: f64,
    t: f64,
) -> (f64, f64) {
    let lat = lat1 + t * (lat2 - lat1);
    let lon = lon1 + t * (lon2 - lon1);
    (lat, lon)
}

/// Project car position onto the track width vector.
///
/// Returns lateral position where -1 = left edge, 0 = center, +1 = right edge.
#[inline]
fn project_onto_track(
    left_lat: f64,
    left_lon: f64,
    right_lat: f64,
    right_lon: f64,
    car_lat: f64,
    car_lon: f64,
) -> f64 {
    // Track width vector: left -> right
    let track_lat = right_lat - left_lat;
    let track_lon = right_lon - left_lon;

    // Car vector: left boundary -> car position
    let car_delta_lat = car_lat - left_lat;
    let car_delta_lon = car_lon - left_lon;

    // Project car position onto track width vector
    let track_len_sq = track_lat * track_lat + track_lon * track_lon;
    if track_len_sq < 1e-12 {
        // Track width is essentially zero at this point
        return 0.0;
    }

    let dot = car_delta_lat * track_lat + car_delta_lon * track_lon;
    let projection = dot / track_len_sq;

    // Convert projection (0 = left, 1 = right) to lateral position (-1 = left, +1 = right)
    2.0 * projection - 1.0
}
