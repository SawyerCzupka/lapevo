use std::sync::Arc;

use chrono::Utc;
use eventbus::EventBus;
use futures::StreamExt;
use pitwall::{PitwallFrame, SessionInfo, UpdateRate};
use tokio::sync::watch;
use uuid::Uuid;

use crate::api::{self, SessionFrame};
use crate::events::RacingEvent;
use crate::pitwall_ext::AcceleratedReplayConnection;
use crate::pos_service::PositionState;

#[derive(Debug, PitwallFrame)]
pub struct TelemetryFrame {
    // Time
    #[field_name = "SessionTime"]
    pub session_time: f64,

    // Lap Information
    #[field_name = "Lap"]
    pub lap_number: i32,
    #[field_name = "LapDistPct"]
    pub lap_distance_pct: f32,
    #[field_name = "LapDist"]
    pub lap_distance: f32,
    #[field_name = "LapCurrentLapTime"]
    pub current_lap_time: f32,
    #[field_name = "LapLastLapTime"]
    pub last_lap_time: f32,
    #[field_name = "LapBestLapTime"]
    pub best_lap_time: f32,

    // Vehicle State
    #[field_name = "Speed"]
    pub speed: f32,
    #[field_name = "RPM"]
    pub rpm: f32,
    #[field_name = "Gear"]
    pub gear: i32,

    // Driver Inputs
    #[field_name = "Throttle"]
    pub throttle: f32,
    #[field_name = "Brake"]
    pub brake: f32,
    #[field_name = "Clutch"]
    pub clutch: f32,
    #[field_name = "SteeringWheelAngle"]
    pub steering_angle: f32,

    // Vehicle Dynamics
    #[field_name = "LatAccel"]
    pub lateral_acceleration: f32,
    #[field_name = "LongAccel"]
    pub longitudinal_acceleration: f32,
    #[field_name = "VertAccel"]
    pub vertical_acceleration: f32,
    #[field_name = "YawRate"]
    pub yaw_rate: f32,
    #[field_name = "RollRate"]
    pub roll_rate: f32,
    #[field_name = "PitchRate"]
    pub pitch_rate: f32,

    // GPS Position
    #[field_name = "Lat"]
    pub latitude: f64,
    #[field_name = "Lon"]
    pub longitude: f64,
    #[field_name = "Alt"]
    pub altitude: f32,

    // Session State
    #[field_name = "PlayerTrackSurface"]
    pub track_surface: i32,
}

impl From<&TelemetryFrame> for api::TelemetryFrame {
    fn from(frame: &TelemetryFrame) -> Self {
        Self {
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
            lateral_acceleration: frame.lateral_acceleration as f64,
            longitudinal_acceleration: frame.longitudinal_acceleration as f64,
            vertical_acceleration: frame.vertical_acceleration as f64,
            yaw_rate: frame.yaw_rate as f64,
            roll_rate: frame.roll_rate as f64,
            pitch_rate: frame.pitch_rate as f64,
            velocity_x: 0.0, // Not collected in pitwall frame
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
            track_surface: frame.track_surface,
            on_pit_road: false,
        }
    }
}

impl From<TelemetryFrame> for api::TelemetryFrame {
    fn from(frame: TelemetryFrame) -> Self {
        Self::from(&frame)
    }
}

pub async fn read_telemetry_print() {
    let connection = AcceleratedReplayConnection::open(
        "../../sample_data/ligierjsp320_bathurst 2025-11-17 18-15-16.ibt",
        5.0,
    )
    .await
    .unwrap();

    let mut stream = connection.subscribe::<TelemetryFrame>(UpdateRate::Max(60));

    while let Some(frame) = stream.next().await {
        println!(
            "Speed: {:.1} mph, RPM: {}, Gear: {}",
            frame.speed, frame.rpm, frame.gear
        )
    }
}

pub async fn read_telemetry_eventbus(
    bus: EventBus<RacingEvent>,
    speed: f64,
    pos_tx: watch::Sender<PositionState>,
) {
    let connection = AcceleratedReplayConnection::open(
        "../../../sample_data/ligierjsp320_bathurst 2025-11-17 18-15-16.ibt",
        speed,
    )
    .await
    .unwrap();

    let mut stream = connection.subscribe::<TelemetryFrame>(UpdateRate::Max(60));
    let mut published_count: u64 = 0;

    while let Some(frame) = stream.next().await {
        match pos_tx.send(PositionState {
            lap_dist_pct: frame.lap_distance_pct,
            lap_number: frame.lap_number,
        }) {
            Ok(_) => {}
            Err(error) => {
                println!("Error Msg: '{error}'");
                panic!();
            }
        }

        match bus.publish(RacingEvent::TelemetryFrameCollected(Arc::new(frame))) {
            Ok(_) => {}
            Err(error) => {
                println!("Error Msg: {error}")
            }
        }
        published_count += 1;
    }

    println!(
        "[Telemetry Publisher] Finished - total frames published: {}",
        published_count
    );
}

/// Extract a SessionFrame from pitwall's SessionInfo.
///
/// This function extracts the relevant session metadata from an iRacing IBT file's
/// session info, converting it to the format expected by the server API.
///
/// Returns `None` if required fields are missing (driver info, drivers list).
pub fn extract_session_frame(session: &SessionInfo) -> Option<SessionFrame> {
    let weekend = &session.weekend_info;
    let driver_info = session.driver_info.as_ref()?;
    let drivers = driver_info.drivers.as_ref()?;
    let driver = drivers.first()?;

    // Get the current session's type from the session list
    let current_session_num = session.session_info.current_session_num as usize;
    let session_type = session
        .session_info
        .sessions
        .get(current_session_num)
        .map(|s| s.session_type.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    Some(SessionFrame {
        timestamp: Utc::now(),
        session_id: Uuid::new_v4(),
        track_id: weekend.track_id.unwrap_or(0),
        track_name: weekend.track_display_name.clone(),
        track_config_name: weekend.track_config_name.clone(),
        track_type: weekend
            .track_type
            .clone()
            .unwrap_or_else(|| "road course".to_string()),
        car_id: driver.car_id.unwrap_or(0),
        car_name: driver.car_screen_name.clone().unwrap_or_default(),
        car_class_id: driver.car_class_id.unwrap_or(0),
        series_id: 0, // Not directly available in DriverInfo
        session_type,
    })
}
