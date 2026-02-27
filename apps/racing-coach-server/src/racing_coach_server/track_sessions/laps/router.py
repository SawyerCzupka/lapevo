"""FastAPI route handlers for the laps sub-feature."""

import logging
from uuid import UUID

from fastapi import APIRouter, HTTPException

from racing_coach_server.dependencies import LapServiceDep, TelemetryServiceDep
from racing_coach_server.track_sessions.laps.schemas import (
    LapDetailResponse,
    LapTelemetryResponse,
    TelemetryFrameResponse,
)

logger = logging.getLogger(__name__)

router = APIRouter()


@router.get(
    "/{lap_id}",
    response_model=LapDetailResponse,
    operation_id="getSessionLapDetail",
    tags=["track_sessions"],
)
async def get_lap(
    session_id: UUID,
    lap_id: UUID,
    lap_service: LapServiceDep,
) -> LapDetailResponse:
    """Get detailed information about a specific lap."""
    try:
        lap = await lap_service.get_lap_by_id(lap_id)

        if not lap:
            raise HTTPException(status_code=404, detail=f"Lap {lap_id} not found")

        if lap.track_session_id != session_id:
            raise HTTPException(
                status_code=404,
                detail=f"Lap {lap_id} does not belong to session {session_id}",
            )

        return LapDetailResponse(
            lap_id=str(lap.id),
            session_id=str(lap.track_session_id),
            lap_number=lap.lap_number,
            lap_time=lap.lap_time,
            is_valid=lap.is_valid,
            track_name=lap.track_session.track_name,
            track_config_name=lap.track_session.track_config_name,
            car_name=lap.track_session.car_name,
            has_metrics=lap.metrics is not None,
            created_at=lap.created_at,
        )

    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error getting lap {lap_id}: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Server error: {str(e)}") from e


@router.get(
    "/{lap_id}/telemetry",
    response_model=LapTelemetryResponse,
    operation_id="getLapTelemetry",
    tags=["track_sessions"],
)
async def get_lap_telemetry(
    session_id: UUID,
    lap_id: UUID,
    lap_service: LapServiceDep,
    telemetry_service: TelemetryServiceDep,
) -> LapTelemetryResponse:
    """
    Get all telemetry frames for a specific lap.

    Returns telemetry data including position, speed, inputs, and dynamics
    for visualization and analysis.
    """
    try:
        lap = await lap_service.get_lap_by_id(lap_id)

        if not lap:
            raise HTTPException(status_code=404, detail=f"Lap {lap_id} not found")

        if lap.track_session_id != session_id:
            raise HTTPException(
                status_code=404,
                detail=f"Lap {lap_id} does not belong to session {session_id}",
            )

        telemetry_frames = await telemetry_service.get_telemetry_for_lap(lap_id)

        if not telemetry_frames:
            raise HTTPException(
                status_code=404,
                detail=f"No telemetry data found for lap {lap_id}",
            )

        frames = [
            TelemetryFrameResponse(
                timestamp=frame.timestamp,
                session_time=frame.session_time,
                lap_number=frame.lap_number,
                lap_distance_pct=frame.lap_distance_pct,
                lap_distance=frame.lap_distance,
                current_lap_time=frame.current_lap_time,
                speed=frame.speed,
                rpm=frame.rpm,
                gear=frame.gear,
                throttle=frame.throttle,
                brake=frame.brake,
                clutch=frame.clutch,
                steering_angle=frame.steering_angle,
                lateral_acceleration=frame.lateral_acceleration,
                longitudinal_acceleration=frame.longitudinal_acceleration,
                vertical_acceleration=frame.vertical_acceleration,
                yaw_rate=frame.yaw_rate,
                roll_rate=frame.roll_rate,
                pitch_rate=frame.pitch_rate,
                velocity_x=frame.velocity_x,
                velocity_y=frame.velocity_y,
                velocity_z=frame.velocity_z,
                yaw=frame.yaw,
                pitch=frame.pitch,
                roll=frame.roll,
                latitude=frame.latitude,
                longitude=frame.longitude,
                altitude=frame.altitude,
                track_temp=frame.track_temp,
                air_temp=frame.air_temp,
            )
            for frame in telemetry_frames
        ]

        return LapTelemetryResponse(
            lap_id=str(lap_id),
            session_id=str(session_id),
            lap_number=lap.lap_number,
            frame_count=len(frames),
            frames=frames,
        )

    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error getting telemetry for lap {lap_id}: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Server error: {str(e)}") from e
