//! Data models for the Racing Coach API.
//!
//! These structs mirror the Python Pydantic models from racing-coach-core
//! and are used for serializing requests and deserializing responses.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Tire Data Types
// ============================================================================

/// Temperature data for a single tire (left, middle, right across the tread).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TireSideData {
    pub left: f64,
    pub middle: f64,
    pub right: f64,
}

/// Tire temperatures for all four tires.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TireTemps {
    #[serde(rename = "LF")]
    pub lf: TireSideData,
    #[serde(rename = "RF")]
    pub rf: TireSideData,
    #[serde(rename = "LR")]
    pub lr: TireSideData,
    #[serde(rename = "RR")]
    pub rr: TireSideData,
}

/// Tire wear for all four tires.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TireWear {
    #[serde(rename = "LF")]
    pub lf: TireSideData,
    #[serde(rename = "RF")]
    pub rf: TireSideData,
    #[serde(rename = "LR")]
    pub lr: TireSideData,
    #[serde(rename = "RR")]
    pub rr: TireSideData,
}

/// Brake line pressure per wheel.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BrakeLinePressure {
    #[serde(rename = "LF")]
    pub lf: f64,
    #[serde(rename = "RF")]
    pub rf: f64,
    #[serde(rename = "LR")]
    pub lr: f64,
    #[serde(rename = "RR")]
    pub rr: f64,
}

// ============================================================================
// Telemetry Types
// ============================================================================

/// A single frame of driving telemetry data.
///
/// Mirrors Python `TelemetryFrame` from racing-coach-core.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TelemetryFrame {
    // Time
    pub timestamp: DateTime<Utc>,
    pub session_time: f64,

    // Lap Information
    pub lap_number: i32,
    pub lap_distance_pct: f64,
    pub lap_distance: f64,
    pub current_lap_time: f64,
    pub last_lap_time: f64,
    pub best_lap_time: f64,

    // Vehicle State
    pub speed: f64,
    pub rpm: f64,
    pub gear: i32,

    // Driver Inputs
    pub throttle: f64,
    pub brake: f64,
    pub clutch: f64,
    pub steering_angle: f64,

    // Vehicle Dynamics
    pub lateral_acceleration: f64,
    pub longitudinal_acceleration: f64,
    pub vertical_acceleration: f64,
    pub yaw_rate: f64,
    pub roll_rate: f64,
    pub pitch_rate: f64,

    // Vehicle Velocity
    pub velocity_x: f64,
    pub velocity_y: f64,
    pub velocity_z: f64,

    // Vehicle Orientation
    pub yaw: f64,
    pub pitch: f64,
    pub roll: f64,

    // GPS Position
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,

    // Tire Data
    pub tire_temps: TireTemps,
    pub tire_wear: TireWear,
    pub brake_line_pressure: BrakeLinePressure,

    // Track Conditions
    pub track_temp: f64,
    pub track_wetness: i32,
    pub air_temp: f64,

    // Session State
    pub session_flags: i32,
    pub track_surface: i32,
    pub on_pit_road: bool,
}

/// Lap telemetry containing all frames for a lap.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LapTelemetry {
    pub frames: Vec<TelemetryFrame>,
    pub lap_time: Option<f64>,
}

/// Session metadata frame.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionFrame {
    pub timestamp: DateTime<Utc>,
    pub session_id: Uuid,

    // Track
    pub track_id: i32,
    pub track_name: String,
    pub track_config_name: Option<String>,
    #[serde(default = "default_track_type")]
    pub track_type: String,

    // Car
    pub car_id: i32,
    pub car_name: String,
    pub car_class_id: i32,

    // Series
    pub series_id: i32,

    // Session
    pub session_type: String,
}

fn default_track_type() -> String {
    "road course".to_string()
}

// ============================================================================
// Metrics Types
// ============================================================================

/// Comprehensive braking metrics for a braking zone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrakingMetrics {
    // Location and timing
    pub braking_point_distance: f64,
    pub braking_point_speed: f64,
    pub end_distance: f64,

    // Performance metrics
    pub max_brake_pressure: f64,
    pub braking_duration: f64,
    pub minimum_speed: f64,

    // Advanced metrics
    pub initial_deceleration: f64,
    pub average_deceleration: f64,
    pub braking_efficiency: f64,

    // Trail braking
    pub has_trail_braking: bool,
    pub trail_brake_distance: f64,
    pub trail_brake_percentage: f64,
}

/// Comprehensive corner metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CornerMetrics {
    // Key corner points (distances)
    pub turn_in_distance: f64,
    pub apex_distance: f64,
    pub exit_distance: f64,
    pub throttle_application_distance: f64,

    // Speeds at key points
    pub turn_in_speed: f64,
    pub apex_speed: f64,
    pub exit_speed: f64,
    pub throttle_application_speed: f64,

    // Performance metrics
    pub max_lateral_g: f64,
    pub time_in_corner: f64,
    pub corner_distance: f64,

    // Steering metrics
    pub max_steering_angle: f64,

    // Speed delta
    pub speed_loss: f64,
    pub speed_gain: f64,
}

/// Combined metrics for an entire lap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LapMetrics {
    pub lap_number: i32,
    pub lap_time: Option<f64>,

    // Collections of all events in the lap
    pub braking_zones: Vec<BrakingMetrics>,
    pub corners: Vec<CornerMetrics>,

    // Lap-wide statistics
    pub total_corners: i32,
    pub total_braking_zones: i32,
    pub average_corner_speed: f64,
    pub max_speed: f64,
    pub min_speed: f64,
}

// ============================================================================
// Track Boundary Types
// ============================================================================

/// Summary of a track boundary for listing purposes.
#[derive(Debug, Clone, Deserialize)]
pub struct TrackBoundarySummary {
    pub id: Uuid,
    pub track_id: i32,
    pub track_name: String,
    pub track_config_name: Option<String>,
    pub grid_size: i32,
    pub track_length: Option<f64>,
    pub created_at: DateTime<Utc>,
}

/// Response from track boundary list endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct TrackBoundaryListResponse {
    pub boundaries: Vec<TrackBoundarySummary>,
    pub total: i32,
}

/// Full track boundary data with coordinate arrays.
#[derive(Debug, Clone, Deserialize)]
pub struct TrackBoundaryResponse {
    pub id: Uuid,
    pub track_id: i32,
    pub track_name: String,
    pub track_config_name: Option<String>,

    // Boundary coordinate arrays
    pub grid_distance_pct: Vec<f64>,
    pub left_latitude: Vec<f64>,
    pub left_longitude: Vec<f64>,
    pub right_latitude: Vec<f64>,
    pub right_longitude: Vec<f64>,

    // Metadata
    pub grid_size: i32,
    pub source_left_frames: i32,
    pub source_right_frames: i32,
    pub track_length: Option<f64>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Corner Segment Types
// ============================================================================

/// A corner segment defining a section of track.
#[derive(Debug, Clone, Deserialize)]
pub struct CornerSegmentResponse {
    pub id: Uuid,
    pub corner_number: i32,
    pub start_distance: f64,
    pub end_distance: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response from corner segments list endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct CornerSegmentListResponse {
    pub corners: Vec<CornerSegmentResponse>,
    pub total: i32,
}

// ============================================================================
// Auth Types
// ============================================================================

/// Response from the /auth/me endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct UserResponse {
    pub user_id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub email_verified: bool,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Request Types
// ============================================================================

/// Request body for lap telemetry upload.
#[derive(Debug, Clone, Serialize)]
pub struct LapUploadRequest {
    pub lap: LapTelemetry,
    pub session: SessionFrame,
}

/// Request body for lap metrics upload.
#[derive(Debug, Clone, Serialize)]
pub struct MetricsUploadRequest {
    pub lap_metrics: LapMetrics,
    pub lap_id: String,
}

// ============================================================================
// Response Types
// ============================================================================

/// Response from lap telemetry upload.
#[derive(Debug, Clone, Deserialize)]
pub struct LapUploadResponse {
    pub status: String,
    pub message: String,
    pub lap_id: String,
}

/// Response from lap metrics upload.
#[derive(Debug, Clone, Deserialize)]
pub struct MetricsUploadResponse {
    pub status: String,
    pub message: String,
    pub lap_metrics_id: String,
}
