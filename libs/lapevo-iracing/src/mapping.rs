use crate::error::Result;
use crate::raw_frame::RawFrame;
use crate::vars::VariableSchema;
use lapevo_telemetry::TelemetryFrame;

/// Maps a raw iRacing frame to the common TelemetryFrame.
///
/// Variable names follow iRacing's naming convention:
/// - SessionTime, Lap, LapDistPct, LapDist, LapCurrentLapTime, LapLastLapTime, LapBestLapTime
/// - Speed, RPM, Gear
/// - Throttle, Brake, Clutch, SteeringWheelAngle
/// - LatAccel, LongAccel, VertAccel, YawRate, RollRate, PitchRate
/// - Lat, Lon, Alt
/// - PlayerTrackSurface (3 = on track)
pub fn map_to_telemetry_frame(frame: &RawFrame, schema: &VariableSchema) -> Result<TelemetryFrame> {
    Ok(TelemetryFrame {
        session_time: frame.get_f64(schema, "SessionTime").unwrap_or(0.0),

        lap_number: frame.get_i32(schema, "Lap").unwrap_or(0),
        lap_distance_pct: frame.get_f32(schema, "LapDistPct").unwrap_or(0.0),
        lap_distance: frame.get_f32(schema, "LapDist").unwrap_or(0.0),
        current_lap_time: frame.get_f32(schema, "LapCurrentLapTime").unwrap_or(0.0),
        last_lap_time: frame.get_f32(schema, "LapLastLapTime").unwrap_or(0.0),
        best_lap_time: frame.get_f32(schema, "LapBestLapTime").unwrap_or(0.0),

        speed: frame.get_f32(schema, "Speed").unwrap_or(0.0),
        rpm: frame.get_f32(schema, "RPM").unwrap_or(0.0),
        gear: frame.get_i32(schema, "Gear").unwrap_or(0),

        throttle: frame.get_f32(schema, "Throttle").unwrap_or(0.0),
        brake: frame.get_f32(schema, "Brake").unwrap_or(0.0),
        clutch: frame.get_f32(schema, "Clutch").unwrap_or(0.0),
        steering_angle: frame.get_f32(schema, "SteeringWheelAngle").unwrap_or(0.0),

        lat_accel: frame.get_f32(schema, "LatAccel").unwrap_or(0.0),
        lon_accel: frame.get_f32(schema, "LongAccel").unwrap_or(0.0),
        vert_accel: frame.get_f32(schema, "VertAccel").unwrap_or(0.0),
        yaw_rate: frame.get_f32(schema, "YawRate").unwrap_or(0.0),
        roll_rate: frame.get_f32(schema, "RollRate").unwrap_or(0.0),
        pitch_rate: frame.get_f32(schema, "PitchRate").unwrap_or(0.0),

        latitude: frame.get_f64(schema, "Lat").unwrap_or(0.0),
        longitude: frame.get_f64(schema, "Lon").unwrap_or(0.0),
        altitude: frame.get_f32(schema, "Alt").unwrap_or(0.0),

        on_track: frame.get_i32(schema, "PlayerTrackSurface").map(|v| v == 3).unwrap_or(false),
    })
}
