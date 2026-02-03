/// Metadata about a telemetry session.
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub track_name: String,
    pub track_id: i32,
    pub track_length: f32,
    pub car_name: String,
    pub car_id: i32,
    pub session_type: String,
    pub tick_rate: f64,
}
