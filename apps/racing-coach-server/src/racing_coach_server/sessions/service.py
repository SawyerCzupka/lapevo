"""Service for session and lap management."""

import logging
from typing import Any
from uuid import UUID

from racing_coach_core.schemas.telemetry import SessionFrame
from sqlalchemy import desc, select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from racing_coach_server.telemetry.models import Lap, LapMetricsDB, TrackSession

logger = logging.getLogger(__name__)


class SessionService:
    """Service for track session and lap operations."""

    def __init__(self, db: AsyncSession) -> None:
        self.db = db

    # === Session Operations ===

    async def add_or_get_session(self, session_frame: SessionFrame) -> TrackSession:
        """
        Idempotent session creation - returns existing session if found,
        creates new one otherwise.

        Args:
            session_frame: The session information from the client

        Returns:
            TrackSession: The existing or newly created session
        """
        stmt = select(TrackSession).where(TrackSession.id == session_frame.session_id)
        result = await self.db.execute(stmt)
        existing_session = result.scalar_one_or_none()

        if existing_session:
            logger.debug(f"Found existing session with ID {session_frame.session_id}")
            return existing_session

        new_session = TrackSession(
            id=session_frame.session_id,
            track_id=session_frame.track_id,
            track_name=session_frame.track_name,
            track_config_name=session_frame.track_config_name,
            track_type=session_frame.track_type,
            car_id=session_frame.car_id,
            car_name=session_frame.car_name,
            car_class_id=session_frame.car_class_id,
            series_id=session_frame.series_id,
            session_type=session_frame.session_type,
        )
        self.db.add(new_session)
        await self.db.flush()
        logger.info(f"Created new session with ID {new_session.id}")
        return new_session

    async def get_latest_session(self) -> TrackSession | None:
        """
        Get the most recent track session.

        Returns:
            TrackSession | None: The latest session or None if no sessions exist
        """
        stmt = select(TrackSession).order_by(desc(TrackSession.created_at)).limit(1)
        result = await self.db.execute(stmt)
        session = result.scalar_one_or_none()

        if session:
            logger.debug(f"Found latest session with ID {session.id}")
        else:
            logger.debug("No sessions found in database")

        return session

    async def get_all_sessions(self) -> list[TrackSession]:
        """
        Get all track sessions ordered by created_at descending.

        Returns:
            list[TrackSession]: All sessions with their lap counts
        """
        stmt = (
            select(TrackSession)
            .options(selectinload(TrackSession.laps))
            .order_by(desc(TrackSession.created_at))
        )
        result = await self.db.execute(stmt)
        sessions = result.scalars().all()

        logger.debug(f"Found {len(sessions)} sessions")
        return list(sessions)

    async def get_session_by_id(self, session_id: UUID) -> TrackSession | None:
        """
        Get a specific session by ID with its laps.

        Args:
            session_id: The ID of the session

        Returns:
            TrackSession | None: The session or None if not found
        """
        stmt = (
            select(TrackSession)
            .where(TrackSession.id == session_id)
            .options(selectinload(TrackSession.laps))
        )
        result = await self.db.execute(stmt)
        session = result.scalar_one_or_none()

        if session:
            logger.debug(f"Found session with ID {session_id}")
        else:
            logger.debug(f"No session found with ID {session_id}")

        return session

    # === Lap Operations ===

    async def get_laps_for_session(self, session_id: UUID) -> list[Lap]:
        """
        Get all laps for a session ordered by lap number.

        Args:
            session_id: The ID of the session

        Returns:
            list[Lap]: The laps for the session
        """
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
        """
        Get a specific lap by ID with its metrics.

        Args:
            lap_id: The ID of the lap

        Returns:
            Lap | None: The lap or None if not found
        """
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

        Returns:
            Lap: The created lap record
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
