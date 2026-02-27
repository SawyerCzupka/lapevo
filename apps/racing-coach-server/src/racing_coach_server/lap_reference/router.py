"""API router for the lap_reference endpoint."""

import logging

from fastapi import APIRouter, HTTPException, Query
from racing_coach_core.algs.events import BrakingMetrics, CornerMetrics

from racing_coach_server.dependencies import LapReferenceServiceDep
from racing_coach_server.lap_reference.schemas import ReferenceLapResponse

logger = logging.getLogger(__name__)

router = APIRouter()


@router.get(
    "",
    response_model=ReferenceLapResponse,
    tags=["lap_reference"],
    operation_id="getReferenceLap",
)
async def get_reference_lap(
    lap_reference_service: LapReferenceServiceDep,
    track_id: int = Query(..., description="iRacing track ID"),
    car_id: int = Query(..., description="iRacing car ID"),
) -> ReferenceLapResponse:
    """
    Get the best reference lap for a track+car combination.

    Returns the fastest valid lap with all metrics (braking zones, corners, lap-level stats).
    Returns 404 if no qualifying lap exists or if the lap has no computed metrics.
    """
    lap = await lap_reference_service.get_best_lap(track_id, car_id)

    if lap is None or lap.metrics is None:
        raise HTTPException(
            status_code=404,
            detail=f"No reference lap found for track_id={track_id}, car_id={car_id}",
        )

    metrics = lap.metrics

    return ReferenceLapResponse(
        lap_id=str(lap.id),
        lap_time=metrics.lap_time,
        total_corners=metrics.total_corners,
        total_braking_zones=metrics.total_braking_zones,
        average_corner_speed=metrics.average_corner_speed,
        max_speed=metrics.max_speed,
        min_speed=metrics.min_speed,
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
            for b in metrics.braking_zones
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
            for c in metrics.corners
        ],
    )
