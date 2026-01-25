"""FastAPI route handlers for sessions feature."""

import logging
from uuid import UUID

from fastapi import APIRouter, HTTPException, status

from racing_coach_server.database.engine import transactional_session
from racing_coach_server.dependencies import AsyncSessionDep, SessionServiceDep, TelemetryServiceDep
from racing_coach_server.sessions.schemas import (
    LapDetailResponse,
    LapSummary,
    LapTelemetryResponse,
    SessionDetailResponse,
    SessionListResponse,
    SessionSummary,
    TelemetryFrameResponse,
)

logger = logging.getLogger(__name__)

router = APIRouter()


@router.get(
    "",
    response_model=SessionListResponse,
    operation_id="getSessionsList",
    tags=["sessions"],
)
async def list_sessions(
    session_service: SessionServiceDep,
) -> SessionListResponse:
    """
    List all sessions.

    Returns sessions ordered by creation date (most recent first).
    """
    try:
        sessions = await session_service.get_all_sessions()

        session_summaries = [
            SessionSummary(
                session_id=str(session.id),
                track_id=session.track_id,
                track_name=session.track_name,
                track_config_name=session.track_config_name,
                track_type=session.track_type,
                car_id=session.car_id,
                car_name=session.car_name,
                car_class_id=session.car_class_id,
                series_id=session.series_id,
                lap_count=len(session.laps),
                created_at=session.created_at,
            )
            for session in sessions
        ]

        return SessionListResponse(
            sessions=session_summaries,
            total=len(session_summaries),
        )

    except Exception as e:
        logger.error(f"Error listing sessions: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Server error: {str(e)}") from e


@router.get(
    "/{session_id}",
    response_model=SessionDetailResponse,
    operation_id="getSessionDetail",
    tags=["sessions"],
)
async def get_session(
    session_id: UUID,
    session_service: SessionServiceDep,
) -> SessionDetailResponse:
    """Get session details including all laps."""
    try:
        session = await session_service.get_session_by_id(session_id)

        if not session:
            raise HTTPException(status_code=404, detail=f"Session {session_id} not found")

        laps = await session_service.get_laps_for_session(session_id)

        lap_summaries = [
            LapSummary(
                lap_id=str(lap.id),
                lap_number=lap.lap_number,
                lap_time=lap.lap_time,
                is_valid=lap.is_valid,
                has_metrics=lap.metrics is not None,
                created_at=lap.created_at,
            )
            for lap in laps
        ]

        return SessionDetailResponse(
            session_id=str(session.id),
            track_id=session.track_id,
            track_name=session.track_name,
            track_config_name=session.track_config_name,
            track_type=session.track_type,
            car_id=session.car_id,
            car_name=session.car_name,
            car_class_id=session.car_class_id,
            series_id=session.series_id,
            laps=lap_summaries,
            created_at=session.created_at,
        )

    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error getting session {session_id}: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Server error: {str(e)}") from e


@router.delete(
    "/{session_id}",
    status_code=status.HTTP_204_NO_CONTENT,
    operation_id="deleteSession",
    tags=["sessions"],
)
async def delete_session(
    session_id: UUID,
    db: AsyncSessionDep,
    session_service: SessionServiceDep,
) -> None:
    """
    Delete a track session and all associated data.

    This permanently deletes the session along with all its laps,
    telemetry frames, and metrics. This action cannot be undone.
    """
    try:
        async with transactional_session(db):
            deleted = await session_service.delete_session(session_id)

            if not deleted:
                raise HTTPException(
                    status_code=status.HTTP_404_NOT_FOUND,
                    detail=f"Session {session_id} not found",
                )
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error deleting session {session_id}: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Server error: {str(e)}") from e


@router.get(
    "/{session_id}/laps/{lap_id}",
    response_model=LapDetailResponse,
    operation_id="getSessionLapDetail",
    tags=["sessions"],
)
async def get_lap(
    session_id: UUID,
    lap_id: UUID,
    session_service: SessionServiceDep,
) -> LapDetailResponse:
    """Get detailed information about a specific lap."""
    try:
        lap = await session_service.get_lap_by_id(lap_id)

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
    "/{session_id}/laps/{lap_id}/telemetry",
    response_model=LapTelemetryResponse,
    operation_id="getLapTelemetry",
    tags=["sessions"],
)
async def get_lap_telemetry(
    session_id: UUID,
    lap_id: UUID,
    session_service: SessionServiceDep,
    telemetry_service: TelemetryServiceDep,
) -> LapTelemetryResponse:
    """
    Get all telemetry frames for a specific lap.

    Returns telemetry data including position, speed, inputs, and dynamics
    for visualization and analysis.
    """
    try:
        # Verify lap exists and belongs to session
        lap = await session_service.get_lap_by_id(lap_id)

        if not lap:
            raise HTTPException(status_code=404, detail=f"Lap {lap_id} not found")

        if lap.track_session_id != session_id:
            raise HTTPException(
                status_code=404,
                detail=f"Lap {lap_id} does not belong to session {session_id}",
            )

        # Get telemetry frames
        telemetry_frames = await telemetry_service.get_telemetry_for_lap(lap_id)

        if not telemetry_frames:
            raise HTTPException(
                status_code=404,
                detail=f"No telemetry data found for lap {lap_id}",
            )

        # Convert to response format
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
