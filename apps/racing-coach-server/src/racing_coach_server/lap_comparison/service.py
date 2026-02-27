"""Service for comparing laps and computing performance deltas."""

from racing_coach_server.lap_comparison.schemas import (
    BrakingZoneComparison,
    CornerComparison,
    LapComparisonResponse,
    LapComparisonSummary,
)
from racing_coach_server.lap_metrics.models import BrakingMetricsDB, CornerMetricsDB, LapMetricsDB

# Distance threshold for matching zones/corners (10% of lap distance)
DISTANCE_MATCH_THRESHOLD = 0.10


class LapComparisonService:
    """Service for comparing two laps and computing performance deltas."""

    @staticmethod
    def compare_laps(
        baseline_metrics: LapMetricsDB,
        comparison_metrics: LapMetricsDB,
    ) -> LapComparisonResponse:
        """
        Compare two laps and return detailed performance deltas.

        Args:
            baseline_metrics: Metrics for the baseline lap
            comparison_metrics: Metrics for the lap to compare against baseline
        """
        braking_comparisons = LapComparisonService._compare_braking_zones(
            baseline_metrics.braking_zones,
            comparison_metrics.braking_zones,
        )

        corner_comparisons = LapComparisonService._compare_corners(
            baseline_metrics.corners,
            comparison_metrics.corners,
        )

        summary = LapComparisonService._compute_summary(
            baseline_metrics,
            comparison_metrics,
            braking_comparisons,
            corner_comparisons,
        )

        return LapComparisonResponse(
            summary=summary,
            braking_zone_comparisons=braking_comparisons,
            corner_comparisons=corner_comparisons,
        )

    @staticmethod
    def _compare_braking_zones(
        baseline_zones: list[BrakingMetricsDB],
        comparison_zones: list[BrakingMetricsDB],
    ) -> list[BrakingZoneComparison]:
        """Compare braking zones using distance-based matching."""
        comparisons: list[BrakingZoneComparison] = []
        matched_comparison_indices: set[int] = set()

        for baseline_idx, baseline_zone in enumerate(baseline_zones):
            best_match_idx: int | None = None
            best_match_distance_diff: float = float("inf")

            for comp_idx, comp_zone in enumerate(comparison_zones):
                if comp_idx in matched_comparison_indices:
                    continue

                distance_diff = abs(
                    baseline_zone.braking_point_distance - comp_zone.braking_point_distance
                )

                if (
                    distance_diff < best_match_distance_diff
                    and distance_diff < DISTANCE_MATCH_THRESHOLD
                ):
                    best_match_idx = comp_idx
                    best_match_distance_diff = distance_diff

            if best_match_idx is not None:
                matched_comparison_indices.add(best_match_idx)
                comp_zone = comparison_zones[best_match_idx]

                comparison = BrakingZoneComparison(
                    zone_index=baseline_idx,
                    matched_zone_index=best_match_idx,
                    baseline_distance=baseline_zone.braking_point_distance,
                    comparison_distance=comp_zone.braking_point_distance,
                    distance_delta=comp_zone.braking_point_distance
                    - baseline_zone.braking_point_distance,
                    braking_point_speed_delta=comp_zone.braking_point_speed
                    - baseline_zone.braking_point_speed,
                    max_brake_pressure_delta=comp_zone.max_brake_pressure
                    - baseline_zone.max_brake_pressure,
                    braking_duration_delta=comp_zone.braking_duration
                    - baseline_zone.braking_duration,
                    minimum_speed_delta=comp_zone.minimum_speed - baseline_zone.minimum_speed,
                    braking_efficiency_delta=comp_zone.braking_efficiency
                    - baseline_zone.braking_efficiency,
                    trail_braking_comparison=LapComparisonService._compare_trail_braking(
                        baseline_zone.has_trail_braking,
                        comp_zone.has_trail_braking,
                    ),
                )
            else:
                comparison = BrakingZoneComparison(
                    zone_index=baseline_idx,
                    matched_zone_index=None,
                    baseline_distance=baseline_zone.braking_point_distance,
                    comparison_distance=None,
                    distance_delta=None,
                    braking_point_speed_delta=None,
                    max_brake_pressure_delta=None,
                    braking_duration_delta=None,
                    minimum_speed_delta=None,
                    braking_efficiency_delta=None,
                    trail_braking_comparison=None,
                )

            comparisons.append(comparison)

        return comparisons

    @staticmethod
    def _compare_corners(
        baseline_corners: list[CornerMetricsDB],
        comparison_corners: list[CornerMetricsDB],
    ) -> list[CornerComparison]:
        """Compare corners using distance-based matching."""
        comparisons: list[CornerComparison] = []
        matched_comparison_indices: set[int] = set()

        for baseline_idx, baseline_corner in enumerate(baseline_corners):
            best_match_idx: int | None = None
            best_match_distance_diff: float = float("inf")

            for comp_idx, comp_corner in enumerate(comparison_corners):
                if comp_idx in matched_comparison_indices:
                    continue

                distance_diff = abs(baseline_corner.apex_distance - comp_corner.apex_distance)

                if (
                    distance_diff < best_match_distance_diff
                    and distance_diff < DISTANCE_MATCH_THRESHOLD
                ):
                    best_match_idx = comp_idx
                    best_match_distance_diff = distance_diff

            if best_match_idx is not None:
                matched_comparison_indices.add(best_match_idx)
                comp_corner = comparison_corners[best_match_idx]

                comparison = CornerComparison(
                    corner_index=baseline_idx,
                    matched_corner_index=best_match_idx,
                    baseline_apex_distance=baseline_corner.apex_distance,
                    comparison_apex_distance=comp_corner.apex_distance,
                    distance_delta=comp_corner.apex_distance - baseline_corner.apex_distance,
                    turn_in_speed_delta=comp_corner.turn_in_speed - baseline_corner.turn_in_speed,
                    apex_speed_delta=comp_corner.apex_speed - baseline_corner.apex_speed,
                    exit_speed_delta=comp_corner.exit_speed - baseline_corner.exit_speed,
                    max_lateral_g_delta=comp_corner.max_lateral_g - baseline_corner.max_lateral_g,
                    time_in_corner_delta=comp_corner.time_in_corner
                    - baseline_corner.time_in_corner,
                    corner_distance_delta=comp_corner.corner_distance
                    - baseline_corner.corner_distance,
                )
            else:
                comparison = CornerComparison(
                    corner_index=baseline_idx,
                    matched_corner_index=None,
                    baseline_apex_distance=baseline_corner.apex_distance,
                    comparison_apex_distance=None,
                    distance_delta=None,
                    turn_in_speed_delta=None,
                    apex_speed_delta=None,
                    exit_speed_delta=None,
                    max_lateral_g_delta=None,
                    time_in_corner_delta=None,
                    corner_distance_delta=None,
                )

            comparisons.append(comparison)

        return comparisons

    @staticmethod
    def _compare_trail_braking(baseline_has: bool, comparison_has: bool) -> str:
        """Compare trail braking usage between two laps."""
        if baseline_has and comparison_has:
            return "both"
        elif baseline_has and not comparison_has:
            return "baseline_only"
        elif not baseline_has and comparison_has:
            return "comparison_only"
        else:
            return "neither"

    @staticmethod
    def _compute_summary(
        baseline_metrics: LapMetricsDB,
        comparison_metrics: LapMetricsDB,
        braking_comparisons: list[BrakingZoneComparison],
        corner_comparisons: list[CornerComparison],
    ) -> LapComparisonSummary:
        """Compute summary statistics for the lap comparison."""
        lap_time_delta = None
        if baseline_metrics.lap_time is not None and comparison_metrics.lap_time is not None:
            lap_time_delta = comparison_metrics.lap_time - baseline_metrics.lap_time

        matched_braking_zones = sum(
            1 for bc in braking_comparisons if bc.matched_zone_index is not None
        )
        matched_corners = sum(1 for cc in corner_comparisons if cc.matched_corner_index is not None)

        return LapComparisonSummary(
            baseline_lap_id=str(baseline_metrics.lap_id),
            comparison_lap_id=str(comparison_metrics.lap_id),
            baseline_lap_time=baseline_metrics.lap_time,
            comparison_lap_time=comparison_metrics.lap_time,
            lap_time_delta=lap_time_delta,
            max_speed_delta=comparison_metrics.max_speed - baseline_metrics.max_speed,
            min_speed_delta=comparison_metrics.min_speed - baseline_metrics.min_speed,
            average_corner_speed_delta=comparison_metrics.average_corner_speed
            - baseline_metrics.average_corner_speed,
            total_braking_zones_baseline=baseline_metrics.total_braking_zones,
            total_braking_zones_comparison=comparison_metrics.total_braking_zones,
            total_corners_baseline=baseline_metrics.total_corners,
            total_corners_comparison=comparison_metrics.total_corners,
            matched_braking_zones=matched_braking_zones,
            matched_corners=matched_corners,
        )
