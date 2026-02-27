"""Service for track session management."""

import logging
from uuid import UUID

from racing_coach_core.schemas.telemetry import SessionFrame
from sqlalchemy import desc, select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from racing_coach_server.track_sessions.models import TrackSession

logger = logging.getLogger(__name__)


class TrackSessionService:
    """Service for track session operations."""

    def __init__(self, db: AsyncSession) -> None:
        self.db = db

    async def add_or_get_session(self, session_frame: SessionFrame) -> TrackSession:
        """
        Idempotent session creation - returns existing session if found,
        creates new one otherwise.
        """
        stmt = select(TrackSession).where(TrackSession.id == session_frame.session_id)
        result = await self.db.execute(stmt)
        existing_session = result.scalar_one_or_none()

        if existing_session:
            logger.debug(f"Found existing session with ID {session_frame.session_id}")
            return existing_session

        new_session = TrackSession.from_session_frame(session_frame)
        self.db.add(new_session)
        await self.db.flush()
        logger.info(f"Created new session with ID {new_session.id}")
        return new_session

    async def get_latest_session(self) -> TrackSession | None:
        """Get the most recent track session."""
        stmt = select(TrackSession).order_by(desc(TrackSession.created_at)).limit(1)
        result = await self.db.execute(stmt)
        session = result.scalar_one_or_none()

        if session:
            logger.debug(f"Found latest session with ID {session.id}")
        else:
            logger.debug("No sessions found in database")

        return session

    async def get_all_sessions(self) -> list[TrackSession]:
        """Get all track sessions ordered by created_at descending."""
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
        """Get a specific session by ID with its laps."""
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

    async def delete_session(self, session_id: UUID) -> bool:
        """
        Delete a track session and all associated data.

        Cascade deletes are configured in the model, so this will also delete:
        - All Lap records
        - All Telemetry frames
        - All LapMetricsDB records and their children
        """
        session = await self.get_session_by_id(session_id)
        if session is None:
            logger.debug(f"Cannot delete session {session_id}: not found")
            return False

        await self.db.delete(session)
        await self.db.flush()
        logger.info(f"Deleted session {session_id} and all associated data")
        return True
