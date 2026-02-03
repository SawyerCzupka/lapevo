use chrono::Utc;

use lapevo_sdk::TelemetryFrame as ApiTelemetryFrame;
use lapevo_sdk::SessionFrame;
use lapevo_telemetry::TelemetryFrame;
use lapevo_telemetry::SessionInfo;

/// Convert a common TelemetryFrame to the API format.
pub fn to_api_frame(frame: &TelemetryFrame) -> ApiTelemetryFrame {
    ApiTelemetryFrame {
        timestamp: Utc::now(),
        session_time: frame.session_time,
        lap_number: frame.lap_number,
        lap_distance_pct: frame.lap_distance_pct as f64,
        lap_distance: frame.lap_distance as f64,
        current_lap_time: frame.current_lap_time as f64,
        last_lap_time: frame.last_lap_time as f64,
        best_lap_time: frame.best_lap_time as f64,
        speed: frame.speed as f64,
        rpm: frame.rpm as f64,
        gear: frame.gear,
        throttle: frame.throttle as f64,
        brake: frame.brake as f64,
        clutch: frame.clutch as f64,
        steering_angle: frame.steering_angle as f64,
        lateral_acceleration: frame.lat_accel as f64,
        longitudinal_acceleration: frame.lon_accel as f64,
        vertical_acceleration: frame.vert_accel as f64,
        yaw_rate: frame.yaw_rate as f64,
        roll_rate: frame.roll_rate as f64,
        pitch_rate: frame.pitch_rate as f64,
        velocity_x: 0.0,
        velocity_y: 0.0,
        velocity_z: 0.0,
        yaw: 0.0,
        pitch: 0.0,
        roll: 0.0,
        latitude: frame.latitude,
        longitude: frame.longitude,
        altitude: frame.altitude as f64,
        tire_temps: Default::default(),
        tire_wear: Default::default(),
        brake_line_pressure: Default::default(),
        track_temp: 0.0,
        track_wetness: 0,
        air_temp: 0.0,
        session_flags: 0,
        track_surface: if frame.on_track { 3 } else { 0 },
        on_pit_road: false,
    }
}

/// Extract a SessionFrame from lapevo-telemetry's SessionInfo.
pub fn extract_session_frame(session: &SessionInfo) -> SessionFrame {
    SessionFrame {
        timestamp: Utc::now(),
        session_id: uuid::Uuid::new_v4(),
        track_id: session.track_id,
        track_name: session.track_name.clone(),
        track_config_name: None,
        track_type: "road course".to_string(),
        car_id: session.car_id,
        car_name: session.car_name.clone(),
        car_class_id: 0,
        series_id: 0,
        session_type: session.session_type.clone(),
    }
}
