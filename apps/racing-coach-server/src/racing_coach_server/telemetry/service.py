"""Service for raw telemetry frame data management."""

import logging
from uuid import UUID

from racing_coach_core.schemas.telemetry import TelemetrySequence
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from racing_coach_server.telemetry.models import Telemetry

logger = logging.getLogger(__name__)


class TelemetryService:
    """Service for telemetry frame data operations."""

    def __init__(self, db: AsyncSession) -> None:
        self.db = db

    async def add_telemetry_sequence(
        self,
        telemetry_sequence: TelemetrySequence,
        lap_id: UUID,
        session_id: UUID,
    ) -> None:
        """
        Batch insert telemetry frames for a lap.

        Args:
            telemetry_sequence: The sequence of telemetry frames to add
            lap_id: The ID of the lap
            session_id: The ID of the session
        """
        frames: list[Telemetry] = []
        for frame in telemetry_sequence.frames:
            telemetry = Telemetry.from_telemetry_frame(
                frame, track_session_id=session_id, lap_id=lap_id
            )
            frames.append(telemetry)

        self.db.add_all(frames)
        logger.info(f"Added {len(frames)} telemetry frames for lap {lap_id}")

    async def get_telemetry_for_lap(self, lap_id: UUID) -> list[Telemetry]:
        """
        Get all telemetry frames for a specific lap, ordered by session time.

        Args:
            lap_id: The ID of the lap

        Returns:
            list[Telemetry]: The telemetry frames for the lap
        """
        stmt = select(Telemetry).where(Telemetry.lap_id == lap_id).order_by(Telemetry.session_time)
        result = await self.db.execute(stmt)
        frames = result.scalars().all()

        logger.debug(f"Found {len(frames)} telemetry frames for lap {lap_id}")
        return list(frames)
