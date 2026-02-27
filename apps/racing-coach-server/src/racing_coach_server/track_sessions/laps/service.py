"""Service for lap management within track sessions."""

import logging
from typing import Any
from uuid import UUID

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from racing_coach_server.lap_metrics.models import LapMetricsDB
from racing_coach_server.track_sessions.laps.models import Lap

logger = logging.getLogger(__name__)


class LapService:
    """Service for lap operations."""

    def __init__(self, db: AsyncSession) -> None:
        self.db = db

    async def get_laps_for_session(self, session_id: UUID) -> list[Lap]:
        """Get all laps for a session ordered by lap number."""
        stmt = (
            select(Lap)
            .where(Lap.track_session_id == session_id)
            .options(selectinload(Lap.metrics))
            .order_by(Lap.lap_number)
        )
        result = await self.db.execute(stmt)
        laps = result.scalars().all()

        logger.debug(f"Found {len(laps)} laps for session {session_id}")
        return list(laps)

    async def get_lap_by_id(self, lap_id: UUID) -> Lap | None:
        """Get a specific lap by ID with its metrics and session eagerly loaded."""
        stmt = (
            select(Lap)
            .where(Lap.id == lap_id)
            .options(
                selectinload(Lap.metrics).selectinload(LapMetricsDB.braking_zones),
                selectinload(Lap.metrics).selectinload(LapMetricsDB.corners),
                selectinload(Lap.track_session),
            )
        )
        result = await self.db.execute(stmt)
        lap = result.scalar_one_or_none()

        if lap:
            logger.debug(f"Found lap with ID {lap_id}")
        else:
            logger.debug(f"No lap found with ID {lap_id}")

        return lap

    async def add_lap(
        self,
        track_session_id: UUID,
        lap_number: int,
        lap_time: float | None = None,
        is_valid: bool = False,
        lap_id: UUID | None = None,
    ) -> Lap:
        """
        Create a lap record for a session.

        Args:
            track_session_id: The ID of the track session
            lap_number: The lap number
            lap_time: Optional lap time (nullable)
            is_valid: Whether the lap is valid
            lap_id: Optional client-provided UUID. If not provided, server generates one.
        """
        lap_kwargs: dict[str, Any] = {
            "track_session_id": track_session_id,
            "lap_number": lap_number,
            "lap_time": lap_time,
            "is_valid": is_valid,
        }
        if lap_id is not None:
            lap_kwargs["id"] = lap_id

        lap = Lap(**lap_kwargs)

        logger.info(f"Lap Id: {lap.id} | Lap_id: {lap_id}")
        self.db.add(lap)
        await self.db.flush()
        logger.info(f"Created lap {lap_number} for session {track_session_id} with ID {lap.id}")

        if lap.id != lap_id:
            logger.warning("Actual lap ID does not match provided lap id.")
        return lap
