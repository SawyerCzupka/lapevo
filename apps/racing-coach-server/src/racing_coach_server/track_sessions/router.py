"""FastAPI route handlers for the track_sessions feature."""

import logging
from uuid import UUID

from fastapi import APIRouter, HTTPException, status
from racing_coach_core.schemas.telemetry import SessionFrame

from racing_coach_server.database.engine import transactional_session
from racing_coach_server.dependencies import AsyncSessionDep, TrackSessionServiceDep
from racing_coach_server.track_sessions.schemas import (
    LapSummary,
    SessionDetailResponse,
    SessionListResponse,
    SessionSummary,
)

logger = logging.getLogger(__name__)

router = APIRouter()


@router.get(
    "",
    response_model=SessionListResponse,
    operation_id="listSessions",
    tags=["track_sessions"],
)
async def list_sessions(
    session_service: TrackSessionServiceDep,
) -> SessionListResponse:
    """List all sessions, ordered by creation date (most recent first)."""
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
    "/latest",
    response_model=SessionFrame,
    operation_id="getLatestSession",
    tags=["track_sessions"],
)
async def get_latest_session(
    session_service: TrackSessionServiceDep,
) -> SessionFrame:
    """Retrieve the most recent track session."""
    try:
        latest_session = await session_service.get_latest_session()

        if not latest_session:
            raise HTTPException(status_code=404, detail="No sessions found.")

        return latest_session.to_session_frame()

    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error retrieving latest session: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Server error: {str(e)}") from e


@router.get(
    "/{session_id}",
    response_model=SessionDetailResponse,
    operation_id="getSessionDetail",
    tags=["track_sessions"],
)
async def get_session(
    session_id: UUID,
    session_service: TrackSessionServiceDep,
) -> SessionDetailResponse:
    """Get session details including all laps."""
    try:
        session = await session_service.get_session_by_id(session_id)

        if not session:
            raise HTTPException(status_code=404, detail=f"Session {session_id} not found")

        laps = session.laps

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
    tags=["track_sessions"],
)
async def delete_session(
    session_id: UUID,
    db: AsyncSessionDep,
    session_service: TrackSessionServiceDep,
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
