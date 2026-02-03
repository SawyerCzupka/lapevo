/// Common telemetry frame shared across all sim racing game integrations.
#[derive(Debug, Clone)]
pub struct TelemetryFrame {
    // Time
    pub session_time: f64,

    // Lap info
    pub lap_number: i32,
    pub lap_distance_pct: f32,
    pub lap_distance: f32,
    pub current_lap_time: f32,
    pub last_lap_time: f32,
    pub best_lap_time: f32,

    // Vehicle state
    pub speed: f32,
    pub rpm: f32,
    pub gear: i32,

    // Driver inputs
    pub throttle: f32,
    pub brake: f32,
    pub clutch: f32,
    pub steering_angle: f32,

    // Vehicle dynamics
    pub lat_accel: f32,
    pub lon_accel: f32,
    pub vert_accel: f32,
    pub yaw_rate: f32,
    pub roll_rate: f32,
    pub pitch_rate: f32,

    // Position
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f32,

    // Surface
    pub on_track: bool,
}
