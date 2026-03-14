use lapevo_sdk::BrakingMetrics;

mod coaching_feedback_tests {
    use super::*;

    fn make_metrics(braking_point_distance: f64, minimum_speed: f64) -> BrakingMetrics {
        BrakingMetrics {
            braking_point_distance,
            braking_point_speed: 70.0,
            end_distance: braking_point_distance + 80.0,
            max_brake_pressure: 0.95,
            braking_duration: 1.5,
            minimum_speed,
            initial_deceleration: -15.0,
            average_deceleration: -12.0,
            braking_efficiency: 12.6,
            has_trail_braking: false,
            trail_brake_distance: 0.0,
            trail_brake_percentage: 0.0,
        }
    }

    #[test]
    fn reference_lap_stores_zones() {
        use lapevo_client::coaching::feedback::ReferenceLap;

        let mut reference = ReferenceLap::new();
        let zone = make_metrics(500.0, 30.0);
        reference.add_zone(zone.clone(), 0.35);

        assert_eq!(reference.zones().len(), 1);
        assert_eq!(reference.zones()[0].metrics.braking_point_distance, 500.0);
    }

    #[test]
    fn match_zone_finds_closest_within_threshold() {
        use lapevo_client::coaching::feedback::ReferenceLap;

        let mut reference = ReferenceLap::new();
        reference.add_zone(make_metrics(200.0, 40.0), 0.15);
        reference.add_zone(make_metrics(500.0, 30.0), 0.35);
        reference.add_zone(make_metrics(900.0, 25.0), 0.65);

        // Should match the zone at 500m
        let matched = reference.match_zone(510.0);
        assert!(matched.is_some());
        assert_eq!(
            matched.unwrap().metrics.braking_point_distance,
            500.0
        );

        // Too far from any zone
        let matched = reference.match_zone(700.0);
        assert!(matched.is_none());
    }

    #[test]
    fn match_zone_returns_none_when_empty() {
        use lapevo_client::coaching::feedback::ReferenceLap;

        let reference = ReferenceLap::new();
        assert!(reference.match_zone(500.0).is_none());
    }

    #[test]
    fn compare_braking_detects_braked_too_early() {
        use lapevo_client::coaching::feedback::{compare_braking, BrakingPointDelta};

        let reference = make_metrics(500.0, 30.0);
        let current = make_metrics(480.0, 30.0); // braked 20m earlier

        let result = compare_braking(&reference, &current);
        assert!(result.is_some());
        let deviation = result.unwrap();
        assert!(matches!(
            deviation.braking_point,
            Some(BrakingPointDelta::TooEarly(_))
        ));
    }

    #[test]
    fn compare_braking_detects_braked_too_late() {
        use lapevo_client::coaching::feedback::{compare_braking, BrakingPointDelta};

        let reference = make_metrics(500.0, 30.0);
        let current = make_metrics(520.0, 30.0); // braked 20m later

        let result = compare_braking(&reference, &current);
        assert!(result.is_some());
        let deviation = result.unwrap();
        assert!(matches!(
            deviation.braking_point,
            Some(BrakingPointDelta::TooLate(_))
        ));
    }

    #[test]
    fn compare_braking_detects_too_much_speed_at_apex() {
        use lapevo_client::coaching::feedback::compare_braking;

        let reference = make_metrics(500.0, 30.0);
        let current = make_metrics(500.0, 38.0); // 8 m/s faster at apex

        let result = compare_braking(&reference, &current);
        assert!(result.is_some());
        let deviation = result.unwrap();
        assert!(deviation.min_speed_delta.is_some());
    }

    #[test]
    fn compare_braking_returns_none_when_within_tolerance() {
        use lapevo_client::coaching::feedback::compare_braking;

        let reference = make_metrics(500.0, 30.0);
        let current = make_metrics(503.0, 31.0); // tiny differences

        let result = compare_braking(&reference, &current);
        assert!(result.is_none());
    }

    #[test]
    fn generate_feedback_produces_immediate_and_reminder_text() {
        use lapevo_client::coaching::feedback::{
            compare_braking, generate_feedback, generate_reminder,
        };

        let reference = make_metrics(500.0, 30.0);
        let current = make_metrics(480.0, 30.0);

        let deviation = compare_braking(&reference, &current).unwrap();
        let feedback = generate_feedback(&deviation);
        let reminder = generate_reminder(&deviation);

        assert!(!feedback.is_empty());
        assert!(!reminder.is_empty());
        // Feedback should mention braking earlier
        assert!(feedback.to_lowercase().contains("early") || feedback.to_lowercase().contains("soon"));
    }
}
