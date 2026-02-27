"""Unit tests for LapComparisonService."""

import pytest
from racing_coach_server.lap_comparison.service import LapComparisonService

from tests.polyfactories import BrakingMetricsDBFactory, CornerMetricsDBFactory, LapMetricsDBFactory


@pytest.mark.unit
class TestLapComparisonService:
    """Unit tests for lap comparison logic."""

    def test_compare_identical_laps(
        self,
        lap_metrics_db_factory: LapMetricsDBFactory,
        braking_metrics_db_factory: BrakingMetricsDBFactory,
        corner_metrics_db_factory: CornerMetricsDBFactory,
    ) -> None:
        """Test comparing two identical laps produces zero deltas."""
        baseline = lap_metrics_db_factory.build(
            lap_time=90.0,
            max_speed=100.0,
            min_speed=40.0,
            average_corner_speed=50.0,
            total_corners=2,
            total_braking_zones=2,
        )

        baseline.braking_zones = [
            braking_metrics_db_factory.build(
                lap_metrics_id=baseline.id,
                zone_number=1,
                braking_point_distance=0.25,
                braking_point_speed=75.0,
            ),
            braking_metrics_db_factory.build(
                lap_metrics_id=baseline.id,
                zone_number=2,
                braking_point_distance=0.75,
                braking_point_speed=80.0,
            ),
        ]

        baseline.corners = [
            corner_metrics_db_factory.build(
                lap_metrics_id=baseline.id,
                corner_number=1,
                apex_distance=0.30,
                apex_speed=45.0,
            ),
            corner_metrics_db_factory.build(
                lap_metrics_id=baseline.id,
                corner_number=2,
                apex_distance=0.80,
                apex_speed=48.0,
            ),
        ]

        comparison = lap_metrics_db_factory.build(
            lap_time=90.0,
            max_speed=100.0,
            min_speed=40.0,
            average_corner_speed=50.0,
            total_corners=2,
            total_braking_zones=2,
        )

        comparison.braking_zones = [
            braking_metrics_db_factory.build(
                lap_metrics_id=comparison.id,
                zone_number=1,
                braking_point_distance=0.25,
                braking_point_speed=75.0,
            ),
            braking_metrics_db_factory.build(
                lap_metrics_id=comparison.id,
                zone_number=2,
                braking_point_distance=0.75,
                braking_point_speed=80.0,
            ),
        ]

        comparison.corners = [
            corner_metrics_db_factory.build(
                lap_metrics_id=comparison.id,
                corner_number=1,
                apex_distance=0.30,
                apex_speed=45.0,
            ),
            corner_metrics_db_factory.build(
                lap_metrics_id=comparison.id,
                corner_number=2,
                apex_distance=0.80,
                apex_speed=48.0,
            ),
        ]

        result = LapComparisonService.compare_laps(baseline, comparison)

        assert result.summary.lap_time_delta == 0.0
        assert result.summary.max_speed_delta == 0.0
        assert result.summary.min_speed_delta == 0.0
        assert result.summary.matched_braking_zones == 2
        assert result.summary.matched_corners == 2

        assert len(result.braking_zone_comparisons) == 2
        for bc in result.braking_zone_comparisons:
            assert bc.matched_zone_index is not None
            assert bc.distance_delta == pytest.approx(0.0, abs=0.001)
            assert bc.braking_point_speed_delta == pytest.approx(0.0, abs=0.001)

        assert len(result.corner_comparisons) == 2
        for cc in result.corner_comparisons:
            assert cc.matched_corner_index is not None
            assert cc.distance_delta == pytest.approx(0.0, abs=0.001)
            assert cc.apex_speed_delta == pytest.approx(0.0, abs=0.001)

    def test_compare_laps_with_improvements(
        self,
        lap_metrics_db_factory: LapMetricsDBFactory,
        braking_metrics_db_factory: BrakingMetricsDBFactory,
        corner_metrics_db_factory: CornerMetricsDBFactory,
    ) -> None:
        """Test comparison shows positive deltas when comparison lap is faster."""
        baseline = lap_metrics_db_factory.build(
            lap_time=92.0,
            max_speed=95.0,
            average_corner_speed=45.0,
        )
        baseline.braking_zones = [
            braking_metrics_db_factory.build(
                lap_metrics_id=baseline.id,
                braking_point_distance=0.25,
                braking_point_speed=70.0,
            )
        ]
        baseline.corners = [
            corner_metrics_db_factory.build(
                lap_metrics_id=baseline.id,
                apex_distance=0.30,
                apex_speed=45.0,
            )
        ]

        comparison = lap_metrics_db_factory.build(
            lap_time=90.0,
            max_speed=98.0,
            average_corner_speed=48.0,
        )
        comparison.braking_zones = [
            braking_metrics_db_factory.build(
                lap_metrics_id=comparison.id,
                braking_point_distance=0.25,
                braking_point_speed=75.0,
            )
        ]
        comparison.corners = [
            corner_metrics_db_factory.build(
                lap_metrics_id=comparison.id,
                apex_distance=0.30,
                apex_speed=48.0,
            )
        ]

        result = LapComparisonService.compare_laps(baseline, comparison)

        assert result.summary.lap_time_delta == -2.0
        assert result.summary.max_speed_delta == 3.0
        assert result.summary.average_corner_speed_delta == 3.0
        assert result.braking_zone_comparisons[0].braking_point_speed_delta == 5.0
        assert result.corner_comparisons[0].apex_speed_delta == 3.0

    def test_distance_based_matching(
        self,
        lap_metrics_db_factory: LapMetricsDBFactory,
        corner_metrics_db_factory: CornerMetricsDBFactory,
    ) -> None:
        """Test that corners are matched by closest distance."""
        baseline = lap_metrics_db_factory.build()
        baseline.corners = [
            corner_metrics_db_factory.build(apex_distance=0.20),
            corner_metrics_db_factory.build(apex_distance=0.50),
            corner_metrics_db_factory.build(apex_distance=0.80),
        ]

        comparison = lap_metrics_db_factory.build()
        comparison.corners = [
            corner_metrics_db_factory.build(apex_distance=0.22),
            corner_metrics_db_factory.build(apex_distance=0.78),
            corner_metrics_db_factory.build(apex_distance=0.52),
        ]

        result = LapComparisonService.compare_laps(baseline, comparison)

        assert len(result.corner_comparisons) == 3
        assert all(cc.matched_corner_index is not None for cc in result.corner_comparisons)

        assert result.corner_comparisons[0].baseline_apex_distance == pytest.approx(0.20)
        assert result.corner_comparisons[0].comparison_apex_distance == pytest.approx(0.22)
        assert result.corner_comparisons[1].baseline_apex_distance == pytest.approx(0.50)
        assert result.corner_comparisons[1].comparison_apex_distance == pytest.approx(0.52)
        assert result.corner_comparisons[2].baseline_apex_distance == pytest.approx(0.80)
        assert result.corner_comparisons[2].comparison_apex_distance == pytest.approx(0.78)

    def test_unmatched_zones(
        self,
        lap_metrics_db_factory: LapMetricsDBFactory,
        braking_metrics_db_factory: BrakingMetricsDBFactory,
    ) -> None:
        """Test that zones that don't match show as unmatched."""
        baseline = lap_metrics_db_factory.build()
        baseline.braking_zones = [
            braking_metrics_db_factory.build(braking_point_distance=0.25),
            braking_metrics_db_factory.build(braking_point_distance=0.75),
        ]

        comparison = lap_metrics_db_factory.build()
        comparison.braking_zones = [
            braking_metrics_db_factory.build(braking_point_distance=0.50),
        ]

        result = LapComparisonService.compare_laps(baseline, comparison)

        assert len(result.braking_zone_comparisons) == 2
        unmatched_zones = [
            bc for bc in result.braking_zone_comparisons if bc.matched_zone_index is None
        ]
        assert len(unmatched_zones) >= 1

    def test_trail_braking_comparison(
        self,
        lap_metrics_db_factory: LapMetricsDBFactory,
        braking_metrics_db_factory: BrakingMetricsDBFactory,
    ) -> None:
        """Test trail braking comparison between laps."""
        baseline = lap_metrics_db_factory.build()
        baseline.braking_zones = [
            braking_metrics_db_factory.build(
                braking_point_distance=0.25,
                has_trail_braking=True,
            )
        ]

        comparison = lap_metrics_db_factory.build()
        comparison.braking_zones = [
            braking_metrics_db_factory.build(
                braking_point_distance=0.25,
                has_trail_braking=False,
            )
        ]

        result = LapComparisonService.compare_laps(baseline, comparison)

        assert result.braking_zone_comparisons[0].trail_braking_comparison == "baseline_only"

    def test_empty_laps_comparison(
        self,
        lap_metrics_db_factory: LapMetricsDBFactory,
    ) -> None:
        """Test comparison with no braking zones or corners."""
        baseline = lap_metrics_db_factory.build()
        baseline.braking_zones = []
        baseline.corners = []

        comparison = lap_metrics_db_factory.build()
        comparison.braking_zones = []
        comparison.corners = []

        result = LapComparisonService.compare_laps(baseline, comparison)

        assert len(result.braking_zone_comparisons) == 0
        assert len(result.corner_comparisons) == 0
        assert result.summary.matched_braking_zones == 0
        assert result.summary.matched_corners == 0
