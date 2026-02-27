"""Service for reference lap queries."""

import logging

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from racing_coach_server.lap_metrics.models import LapMetricsDB
from racing_coach_server.track_sessions.laps.models import Lap
from racing_coach_server.track_sessions.models import TrackSession

logger = logging.getLogger(__name__)


class LapReferenceService:
    """Service for querying reference (best) laps for a track+car combination."""

    def __init__(self, db: AsyncSession) -> None:
        self.db = db

    async def get_best_lap(self, track_id: int, car_id: int) -> Lap | None:
        """
        Get the fastest valid lap for a track+car combination.

        Returns the lap with the lowest lap_time that is valid and has a non-null lap_time,
        with metrics eagerly loaded.

        Args:
            track_id: The iRacing track ID
            car_id: The iRacing car ID

        Returns:
            Lap | None: The best lap with metrics loaded, or None if no qualifying lap exists
        """
        stmt = (
            select(Lap)
            .join(TrackSession)
            .where(
                TrackSession.track_id == track_id,
                TrackSession.car_id == car_id,
                Lap.is_valid.is_(True),
                Lap.lap_time.isnot(None),
            )
            .options(
                selectinload(Lap.metrics).selectinload(LapMetricsDB.braking_zones),
                selectinload(Lap.metrics).selectinload(LapMetricsDB.corners),
            )
            .order_by(Lap.lap_time.asc())
            .limit(1)
        )
        result = await self.db.execute(stmt)
        lap = result.scalar_one_or_none()

        if lap:
            logger.debug(
                f"Found best lap for track={track_id} car={car_id}: {lap.id}, Time: {lap.lap_time}"
            )
        else:
            logger.debug(f"No valid lap found for track={track_id} car={car_id}")

        return lap
