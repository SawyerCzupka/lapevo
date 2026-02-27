"""API router for the lap_comparison endpoint."""

import logging
from uuid import UUID

from fastapi import APIRouter, HTTPException, Query

from racing_coach_server.dependencies import LapMetricsServiceDep
from racing_coach_server.lap_comparison.schemas import LapComparisonResponse
from racing_coach_server.lap_comparison.service import LapComparisonService

logger = logging.getLogger(__name__)

router = APIRouter()


@router.get(
    "",
    response_model=LapComparisonResponse,
    tags=["lap_comparison"],
    operation_id="compareLaps",
)
async def compare_laps(
    lap_metrics_service: LapMetricsServiceDep,
    lap_id_1: str = Query(..., description="UUID of the baseline lap"),
    lap_id_2: str = Query(..., description="UUID of the lap to compare against baseline"),
) -> LapComparisonResponse:
    """
    Compare two laps and return detailed performance deltas.

    Returns:
    - Summary statistics (lap time delta, speed deltas, etc.)
    - Per-braking-zone comparisons with matched zones and deltas
    - Per-corner comparisons with matched corners and deltas

    Zones and corners are matched based on distance (closest match within threshold).
    """
    try:
        uuid_lap_id_1 = UUID(lap_id_1)
        uuid_lap_id_2 = UUID(lap_id_2)
    except ValueError as e:
        raise HTTPException(status_code=400, detail="Invalid lap_id format") from e

    baseline_metrics = await lap_metrics_service.get_lap_metrics(uuid_lap_id_1)
    if not baseline_metrics:
        raise HTTPException(
            status_code=404, detail=f"Metrics not found for baseline lap {lap_id_1}"
        )

    comparison_metrics = await lap_metrics_service.get_lap_metrics(uuid_lap_id_2)
    if not comparison_metrics:
        raise HTTPException(
            status_code=404, detail=f"Metrics not found for comparison lap {lap_id_2}"
        )

    comparison = LapComparisonService.compare_laps(baseline_metrics, comparison_metrics)

    logger.info(
        f"Compared laps {lap_id_1} vs {lap_id_2}: "
        f"time delta = {comparison.summary.lap_time_delta}s, "
        f"matched {comparison.summary.matched_corners}/{comparison.summary.total_corners_baseline} corners"  # noqa: E501
    )

    return comparison
