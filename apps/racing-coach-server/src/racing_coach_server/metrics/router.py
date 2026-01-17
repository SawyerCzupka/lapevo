"""API router for metrics endpoints."""

import logging
from uuid import UUID

from fastapi import APIRouter, HTTPException, Query
from racing_coach_core.algs.events import BrakingMetrics, CornerMetrics

from racing_coach_server.database.engine import transactional_session
from racing_coach_server.dependencies import AsyncSessionDep, MetricsServiceDep
from racing_coach_server.metrics.comparison_schemas import LapComparisonResponse
from racing_coach_server.metrics.comparison_service import LapComparisonService
from racing_coach_server.metrics.schemas import (
    LapMetricsResponse,
    MetricsUploadRequest,
    MetricsUploadResponse,
)
from racing_coach_server.sessions.exceptions import LapNotFoundError

logger = logging.getLogger(__name__)

router = APIRouter()


@router.post(
    "/lap",
    response_model=MetricsUploadResponse,
    tags=["metrics"],
    operation_id="uploadLapMetrics",
)
async def upload_lap_metrics(
    request: MetricsUploadRequest,
    metrics_service: MetricsServiceDep,
    db: AsyncSessionDep,
) -> MetricsUploadResponse:
    """
    Upload metrics for a lap.

    This endpoint accepts extracted lap metrics and stores them in the database.
    If metrics already exist for the lap, they are replaced (upsert pattern).
    """
    try:
        lap_id = UUID(request.lap_id)
    except ValueError as e:
        raise HTTPException(status_code=400, detail="Invalid lap_id format") from e

    try:
        async with transactional_session(db):
            db_metrics = await metrics_service.add_or_update_lap_metrics(
                lap_metrics=request.lap_metrics,
                lap_id=lap_id,
            )

            logger.info(f"Successfully uploaded metrics for lap {lap_id}")

            return MetricsUploadResponse(
                status="success",
                message=f"Metrics uploaded for lap {lap_id}",
                lap_metrics_id=str(db_metrics.id),
            )

    except LapNotFoundError as e:
        logger.warning(f"Lap not found when uploading metrics: {e}")
        raise HTTPException(status_code=404, detail=str(e)) from e

    except Exception as e:
        logger.error(f"Error uploading lap metrics: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Failed to upload metrics: {str(e)}") from e


@router.get(
    "/lap/{lap_id}",
    response_model=LapMetricsResponse,
    tags=["metrics"],
    operation_id="getLapMetrics",
)
async def get_lap_metrics(
    lap_id: str,
    metrics_service: MetricsServiceDep,
) -> LapMetricsResponse:
    """
    Get metrics for a specific lap.

    Returns all metrics including braking zones and corner analysis.
    """
    try:
        uuid_lap_id = UUID(lap_id)
    except ValueError as e:
        raise HTTPException(status_code=400, detail="Invalid lap_id format") from e

    db_metrics = await metrics_service.get_lap_metrics(uuid_lap_id)

    if not db_metrics:
        raise HTTPException(status_code=404, detail=f"Metrics not found for lap {lap_id}")

    # Convert database models to response schema
    return LapMetricsResponse(
        lap_id=str(db_metrics.lap_id),
        lap_time=db_metrics.lap_time,
        total_corners=db_metrics.total_corners,
        total_braking_zones=db_metrics.total_braking_zones,
        average_corner_speed=db_metrics.average_corner_speed,
        max_speed=db_metrics.max_speed,
        min_speed=db_metrics.min_speed,
        braking_zones=[
            BrakingMetrics(
                braking_point_distance=b.braking_point_distance,
                braking_point_speed=b.braking_point_speed,
                end_distance=b.end_distance,
                max_brake_pressure=b.max_brake_pressure,
                braking_duration=b.braking_duration,
                minimum_speed=b.minimum_speed,
                initial_deceleration=b.initial_deceleration,
                average_deceleration=b.average_deceleration,
                braking_efficiency=b.braking_efficiency,
                has_trail_braking=b.has_trail_braking,
                trail_brake_distance=b.trail_brake_distance,
                trail_brake_percentage=b.trail_brake_percentage,
            )
            for b in db_metrics.braking_zones
        ],
        corners=[
            CornerMetrics(
                turn_in_distance=c.turn_in_distance,
                apex_distance=c.apex_distance,
                exit_distance=c.exit_distance,
                throttle_application_distance=c.throttle_application_distance,
                turn_in_speed=c.turn_in_speed,
                apex_speed=c.apex_speed,
                exit_speed=c.exit_speed,
                throttle_application_speed=c.throttle_application_speed,
                max_lateral_g=c.max_lateral_g,
                time_in_corner=c.time_in_corner,
                corner_distance=c.corner_distance,
                max_steering_angle=c.max_steering_angle,
                speed_loss=c.speed_loss,
                speed_gain=c.speed_gain,
            )
            for c in db_metrics.corners
        ],
    )


@router.get(
    "/compare",
    response_model=LapComparisonResponse,
    tags=["metrics"],
    operation_id="compareLaps",
)
async def compare_laps(
    metrics_service: MetricsServiceDep,
    lap_id_1: str = Query(..., description="UUID of the baseline lap"),
    lap_id_2: str = Query(..., description="UUID of the lap to compare against baseline"),
) -> LapComparisonResponse:
    """
    Compare two laps and return detailed performance deltas.

    This endpoint compares metrics from two laps and returns:
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

    # Get metrics for both laps
    baseline_metrics = await metrics_service.get_lap_metrics(uuid_lap_id_1)
    if not baseline_metrics:
        raise HTTPException(
            status_code=404, detail=f"Metrics not found for baseline lap {lap_id_1}"
        )

    comparison_metrics = await metrics_service.get_lap_metrics(uuid_lap_id_2)
    if not comparison_metrics:
        raise HTTPException(
            status_code=404, detail=f"Metrics not found for comparison lap {lap_id_2}"
        )

    # Compare laps
    comparison = LapComparisonService.compare_laps(baseline_metrics, comparison_metrics)

    logger.info(
        f"Compared laps {lap_id_1} vs {lap_id_2}: "
        f"time delta = {comparison.summary.lap_time_delta}s, "
        f"matched {comparison.summary.matched_corners}/{comparison.summary.total_corners_baseline} corners"  # noqa: E501
    )

    return comparison
