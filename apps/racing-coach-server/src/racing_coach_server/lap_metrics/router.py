"""API router for lap_metrics endpoints."""

import logging
from uuid import UUID

from fastapi import APIRouter, HTTPException
from racing_coach_core.algs.events import BrakingMetrics, CornerMetrics

from racing_coach_server.database.engine import transactional_session
from racing_coach_server.dependencies import AsyncSessionDep, LapMetricsServiceDep
from racing_coach_server.lap_metrics.schemas import (
    LapMetricsResponse,
    LapMetricsUploadRequest,
    LapMetricsUploadResponse,
)
from racing_coach_server.track_sessions.laps.exceptions import LapNotFoundError

logger = logging.getLogger(__name__)

router = APIRouter()


@router.post(
    "/lap",
    response_model=LapMetricsUploadResponse,
    tags=["lap_metrics"],
    operation_id="uploadLapMetrics",
)
async def upload_lap_metrics(
    request: LapMetricsUploadRequest,
    lap_metrics_service: LapMetricsServiceDep,
    db: AsyncSessionDep,
) -> LapMetricsUploadResponse:
    """
    Upload metrics for a lap.

    Accepts extracted lap metrics and stores them in the database.
    If metrics already exist for the lap, they are replaced (upsert pattern).
    """
    try:
        lap_id = UUID(request.lap_id)
    except ValueError as e:
        raise HTTPException(status_code=400, detail="Invalid lap_id format") from e

    try:
        async with transactional_session(db):
            db_metrics = await lap_metrics_service.add_or_update_lap_metrics(
                lap_metrics=request.lap_metrics,
                lap_id=lap_id,
            )

            logger.info(f"Successfully uploaded metrics for lap {lap_id}")

            return LapMetricsUploadResponse(
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
    tags=["lap_metrics"],
    operation_id="getLapMetrics",
)
async def get_lap_metrics(
    lap_id: str,
    lap_metrics_service: LapMetricsServiceDep,
) -> LapMetricsResponse:
    """
    Get metrics for a specific lap.

    Returns all metrics including braking zones and corner analysis.
    """
    try:
        uuid_lap_id = UUID(lap_id)
    except ValueError as e:
        raise HTTPException(status_code=400, detail="Invalid lap_id format") from e

    db_metrics = await lap_metrics_service.get_lap_metrics(uuid_lap_id)

    if not db_metrics:
        raise HTTPException(status_code=404, detail=f"Metrics not found for lap {lap_id}")

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
