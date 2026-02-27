"""Schemas for the lap_comparison API responses."""

from pydantic import BaseModel


class BrakingZoneComparison(BaseModel):
    """Comparison data for a braking zone between two laps."""

    # Zone identification
    zone_index: int  # Index in the baseline lap's braking zones
    matched_zone_index: int | None  # Index in comparison lap (None if no match found)

    # Distance matching info
    baseline_distance: float  # Braking point distance in baseline lap
    comparison_distance: float | None  # Braking point distance in comparison lap (None if no match)
    distance_delta: (
        float | None
    )  # Difference in braking point (negative = earlier, positive = later)

    # Performance deltas (positive = better in comparison lap, negative = worse)
    braking_point_speed_delta: float | None  # Speed difference at braking point
    max_brake_pressure_delta: float | None  # Brake pressure difference
    braking_duration_delta: float | None  # Duration difference (negative = shorter)
    minimum_speed_delta: float | None  # Minimum speed difference
    braking_efficiency_delta: float | None  # Efficiency difference
    trail_braking_comparison: str | None  # "both", "baseline_only", "comparison_only", "neither"


class CornerComparison(BaseModel):
    """Comparison data for a corner between two laps."""

    # Corner identification
    corner_index: int  # Index in the baseline lap's corners
    matched_corner_index: int | None  # Index in comparison lap (None if no match found)

    # Distance matching info
    baseline_apex_distance: float  # Apex distance in baseline lap
    comparison_apex_distance: float | None  # Apex distance in comparison lap (None if no match)
    distance_delta: float | None  # Difference in apex distance

    # Speed deltas (positive = faster in comparison lap)
    turn_in_speed_delta: float | None
    apex_speed_delta: float | None
    exit_speed_delta: float | None

    # Performance deltas
    max_lateral_g_delta: float | None  # Lateral G difference (positive = more G)
    time_in_corner_delta: float | None  # Time difference (negative = faster)
    corner_distance_delta: float | None  # Distance difference


class LapComparisonSummary(BaseModel):
    """Summary statistics for lap comparison."""

    # Lap info
    baseline_lap_id: str
    comparison_lap_id: str
    baseline_lap_time: float | None
    comparison_lap_time: float | None
    lap_time_delta: float | None  # Negative = comparison lap faster

    # Overall metrics deltas
    max_speed_delta: float | None
    min_speed_delta: float | None
    average_corner_speed_delta: float | None

    # Zone/corner counts
    total_braking_zones_baseline: int
    total_braking_zones_comparison: int
    total_corners_baseline: int
    total_corners_comparison: int
    matched_braking_zones: int  # Number of zones matched between laps
    matched_corners: int  # Number of corners matched between laps


class LapComparisonResponse(BaseModel):
    """Complete lap comparison response."""

    summary: LapComparisonSummary
    braking_zone_comparisons: list[BrakingZoneComparison]
    corner_comparisons: list[CornerComparison]
