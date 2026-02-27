"""FastAPI route handlers for the telemetry feature."""

import logging
from uuid import UUID

from fastapi import APIRouter, HTTPException
from racing_coach_core.schemas.responses import LapUploadResponse
from racing_coach_core.schemas.telemetry import LapTelemetry, SessionFrame

from racing_coach_server.database.engine import transactional_session
from racing_coach_server.dependencies import (
    AsyncSessionDep,
    LapServiceDep,
    TelemetryServiceDep,
    TrackSessionServiceDep,
)

logger = logging.getLogger(__name__)

router = APIRouter()


@router.post(
    "/lap",
    response_model=LapUploadResponse,
    tags=["telemetry"],
    operation_id="uploadLap",
)
async def upload_lap(
    lap: LapTelemetry,
    session: SessionFrame,
    track_session_service: TrackSessionServiceDep,
    lap_service: LapServiceDep,
    telemetry_service: TelemetryServiceDep,
    db: AsyncSessionDep,
    lap_id: UUID | None = None,
    is_valid: bool = False,
) -> LapUploadResponse:
    """
    Upload a lap with telemetry data.

    Args:
        lap: The lap telemetry data
        session: The session frame with track/car info
        lap_id: Optional client-provided UUID for the lap. If not provided, server generates one.
    """
    logger.info(f"Router lap_id: {lap_id}")

    try:
        async with transactional_session(db):
            lap_number = lap.frames[0].lap_number

            db_track_session = await track_session_service.add_or_get_session(session)

            db_lap = await lap_service.add_lap(
                track_session_id=db_track_session.id,
                lap_number=lap_number,
                lap_id=lap_id,
                is_valid=is_valid,
                lap_time=lap.lap_time,
            )

            await telemetry_service.add_telemetry_sequence(
                telemetry_sequence=lap, lap_id=db_lap.id, session_id=db_track_session.id
            )

            logger.info(f"Successfully uploaded lap {lap_number} with {len(lap.frames)} frames")

            return LapUploadResponse(
                status="success",
                message=f"Received lap {lap_number} with {len(lap.frames)} frames",
                lap_id=str(db_lap.id),
            )

    except Exception as e:
        logger.error(f"Error uploading lap: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Server error: {str(e)}") from e
