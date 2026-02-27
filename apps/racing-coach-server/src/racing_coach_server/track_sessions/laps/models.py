"""SQLAlchemy model for the laps sub-feature."""

from __future__ import annotations

import uuid
from typing import TYPE_CHECKING

from sqlalchemy import Boolean, Float, ForeignKey, Index, Integer, UniqueConstraint
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from racing_coach_server.database.base import Base
from racing_coach_server.database.mixins import TimestampMixin

if TYPE_CHECKING:
    from racing_coach_server.lap_metrics.models import LapMetricsDB
    from racing_coach_server.telemetry.models import Telemetry
    from racing_coach_server.track_sessions.models import TrackSession


class Lap(TimestampMixin, Base):
    """Model representing a lap in a session."""

    __tablename__ = "lap"

    track_session_id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True),
        ForeignKey("track_session.id", ondelete="CASCADE"),
        nullable=False,
    )
    lap_number: Mapped[int] = mapped_column(Integer, nullable=False)
    lap_time: Mapped[float | None] = mapped_column(Float, nullable=True)
    is_valid: Mapped[bool] = mapped_column(Boolean, default=True)

    # Relationships
    track_session: Mapped[TrackSession] = relationship(
        "TrackSession", back_populates="laps", init=False
    )
    telemetry_frames: Mapped[list[Telemetry]] = relationship(
        "Telemetry", back_populates="lap", cascade="all, delete-orphan", init=False
    )
    metrics: Mapped[LapMetricsDB | None] = relationship(
        "LapMetricsDB",
        back_populates="lap",
        cascade="all, delete-orphan",
        init=False,
        uselist=False,
    )
    id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), primary_key=True, default_factory=uuid.uuid4
    )

    # Indexes and constraints
    __table_args__ = (
        UniqueConstraint("track_session_id", "lap_number", name="uq_track_session_id_lap_number"),
        Index("idx_track_session_id_lap_number", "track_session_id", "lap_number"),
    )
