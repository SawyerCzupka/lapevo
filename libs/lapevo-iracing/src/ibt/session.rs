use crate::error::{IracingError, Result};
use lapevo_telemetry::SessionInfo;
use serde::Deserialize;

/// Minimal deserialization target for iRacing session YAML.
/// We only extract what we need for SessionInfo.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct IracingSessionYaml {
    #[serde(default)]
    weekend_info: Option<WeekendInfoYaml>,
    #[serde(default)]
    session_info: Option<SessionInfoYaml>,
    #[serde(default)]
    driver_info: Option<DriverInfoYaml>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct WeekendInfoYaml {
    #[serde(default)]
    track_name: String,
    #[serde(rename = "TrackID", default)]
    track_id: Option<i32>,
    #[serde(default)]
    track_length: Option<String>,
    #[serde(default)]
    track_display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SessionInfoYaml {
    #[serde(default)]
    sessions: Option<Vec<SessionYaml>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SessionYaml {
    #[serde(default)]
    session_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DriverInfoYaml {
    #[serde(default)]
    drivers: Option<Vec<DriverYaml>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DriverYaml {
    #[serde(rename = "CarID", default)]
    car_id: Option<i32>,
    #[serde(default)]
    car_screen_name: Option<String>,
}

/// Parse iRacing session YAML into our common SessionInfo.
pub fn parse_session_info(yaml: &str, tick_rate: f64) -> Result<SessionInfo> {
    let parsed: IracingSessionYaml = serde_yaml::from_str(yaml).map_err(|e| {
        IracingError::Yaml(format!("failed to parse session YAML: {}", e))
    })?;

    let weekend = parsed.weekend_info.unwrap_or(WeekendInfoYaml {
        track_name: String::new(),
        track_id: None,
        track_length: None,
        track_display_name: None,
    });

    let track_length = weekend
        .track_length
        .as_deref()
        .and_then(parse_track_length)
        .unwrap_or(0.0);

    let track_name = weekend
        .track_display_name
        .unwrap_or(weekend.track_name);

    let session_type = parsed
        .session_info
        .and_then(|si| si.sessions)
        .and_then(|sessions| sessions.into_iter().next())
        .and_then(|s| s.session_type)
        .unwrap_or_default();

    let (car_id, car_name) = parsed
        .driver_info
        .and_then(|di| di.drivers)
        .and_then(|drivers| drivers.into_iter().next())
        .map(|d| (d.car_id.unwrap_or(0), d.car_screen_name.unwrap_or_default()))
        .unwrap_or((0, String::new()));

    Ok(SessionInfo {
        track_name,
        track_id: weekend.track_id.unwrap_or(0),
        track_length,
        car_name,
        car_id,
        session_type,
        tick_rate,
    })
}

/// Parse track length string like "6.1441 km" into meters.
pub(crate) fn parse_track_length(s: &str) -> Option<f32> {
    let s = s.trim();
    if let Some(km_str) = s.strip_suffix("km").or_else(|| s.strip_suffix("Km")) {
        km_str.trim().parse::<f32>().ok().map(|km| km * 1000.0)
    } else if let Some(mi_str) = s.strip_suffix("mi") {
        mi_str.trim().parse::<f32>().ok().map(|mi| mi * 1609.34)
    } else {
        s.parse::<f32>().ok()
    }
}
