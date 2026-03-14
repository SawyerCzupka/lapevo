use lapevo_sdk::BrakingMetrics;

/// Distance threshold (meters) for matching braking zones across laps.
const ZONE_MATCH_THRESHOLD_M: f64 = 30.0;

/// Minimum braking point difference (meters) to consider significant.
const BRAKING_POINT_THRESHOLD_M: f64 = 8.0;
/// Minimum speed difference (m/s) at apex to consider significant.
const MIN_SPEED_THRESHOLD_MS: f64 = 3.0;

/// A reference braking zone with its track position.
#[derive(Debug, Clone)]
pub struct ReferenceZone {
    pub metrics: BrakingMetrics,
    /// Lap distance percentage (0.0–1.0) at the braking point.
    pub braking_point_pct: f32,
}

/// Stores braking zones from a reference lap for comparison.
#[derive(Debug, Clone, Default)]
pub struct ReferenceLap {
    zones: Vec<ReferenceZone>,
}

impl ReferenceLap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_zone(&mut self, metrics: BrakingMetrics, braking_point_pct: f32) {
        self.zones.push(ReferenceZone {
            metrics,
            braking_point_pct,
        });
    }

    pub fn zones(&self) -> &[ReferenceZone] {
        &self.zones
    }

    /// Find the reference zone closest to the given braking point distance.
    /// Returns `None` if no zone is within `ZONE_MATCH_THRESHOLD_M`.
    pub fn match_zone(&self, braking_point_distance: f64) -> Option<&ReferenceZone> {
        self.zones
            .iter()
            .filter(|z| {
                (z.metrics.braking_point_distance - braking_point_distance).abs()
                    < ZONE_MATCH_THRESHOLD_M
            })
            .min_by(|a, b| {
                let da = (a.metrics.braking_point_distance - braking_point_distance).abs();
                let db = (b.metrics.braking_point_distance - braking_point_distance).abs();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Returns true if the reference lap has any zones stored.
    pub fn is_populated(&self) -> bool {
        !self.zones.is_empty()
    }
}

// --- Comparison types ---

#[derive(Debug, Clone)]
pub enum BrakingPointDelta {
    /// Braked N meters before the reference point.
    TooEarly(f64),
    /// Braked N meters after the reference point.
    TooLate(f64),
}

/// A measurable deviation from the reference braking zone.
#[derive(Debug, Clone)]
pub struct BrakingDeviation {
    /// Braking point delta, if significant.
    pub braking_point: Option<BrakingPointDelta>,
    /// Minimum speed delta (positive = faster than reference), if significant.
    pub min_speed_delta: Option<f64>,
}

// --- Comparison logic ---

/// Compare a current braking zone against a reference.
/// Returns `None` if all metrics are within tolerance.
pub fn compare_braking(
    reference: &BrakingMetrics,
    current: &BrakingMetrics,
) -> Option<BrakingDeviation> {
    let distance_delta = current.braking_point_distance - reference.braking_point_distance;

    let braking_point = if distance_delta < -BRAKING_POINT_THRESHOLD_M {
        Some(BrakingPointDelta::TooEarly(distance_delta.abs()))
    } else if distance_delta > BRAKING_POINT_THRESHOLD_M {
        Some(BrakingPointDelta::TooLate(distance_delta))
    } else {
        None
    };

    let speed_delta = current.minimum_speed - reference.minimum_speed;
    let min_speed_delta = if speed_delta.abs() > MIN_SPEED_THRESHOLD_MS {
        Some(speed_delta)
    } else {
        None
    };

    if braking_point.is_none() && min_speed_delta.is_none() {
        return None;
    }

    Some(BrakingDeviation {
        braking_point,
        min_speed_delta,
    })
}

// --- Feedback text generation ---

/// Generate immediate feedback text (spoken right after the braking zone).
pub fn generate_feedback(deviation: &BrakingDeviation) -> String {
    let mut parts = Vec::new();

    match &deviation.braking_point {
        Some(BrakingPointDelta::TooEarly(meters)) => {
            parts.push(format!(
                "You braked about {:.0} meters too early.",
                meters
            ));
        }
        Some(BrakingPointDelta::TooLate(meters)) => {
            parts.push(format!(
                "You braked about {:.0} meters too late.",
                meters
            ));
        }
        None => {}
    }

    if let Some(delta) = deviation.min_speed_delta {
        if delta > 0.0 {
            parts.push("You carried too much speed through the corner.".to_string());
        } else {
            parts.push("You over-slowed into the corner.".to_string());
        }
    }

    parts.join(" ")
}

/// Generate a reminder text (spoken before the same zone on the next lap).
pub fn generate_reminder(deviation: &BrakingDeviation) -> String {
    let mut parts = Vec::new();

    match &deviation.braking_point {
        Some(BrakingPointDelta::TooEarly(_)) => {
            parts.push("Brake a little later this time.".to_string());
        }
        Some(BrakingPointDelta::TooLate(_)) => {
            parts.push("Brake a little sooner this time.".to_string());
        }
        None => {}
    }

    if let Some(delta) = deviation.min_speed_delta {
        if delta > 0.0 {
            parts.push("Use more brakes and slow down more.".to_string());
        } else {
            parts.push("Carry more speed into the corner.".to_string());
        }
    }

    parts.join(" ")
}
