"""SQLAlchemy models for the track_sessions feature."""

from __future__ import annotations

import uuid
from typing import TYPE_CHECKING, Self

from racing_coach_core.schemas.telemetry import SessionFrame
from sqlalchemy import Index, Integer, String
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from racing_coach_server.database.base import Base
from racing_coach_server.database.mixins import TimestampMixin

if TYPE_CHECKING:
    from racing_coach_server.telemetry.models import Telemetry
    from racing_coach_server.track_sessions.laps.models import Lap


class TrackSession(TimestampMixin, Base):
    """Model representing a track session."""

    __tablename__ = "track_session"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True)
    track_id: Mapped[int] = mapped_column(Integer, nullable=False)
    track_name: Mapped[str] = mapped_column(String(255), nullable=False)
    track_config_name: Mapped[str | None] = mapped_column(String(255), nullable=True)
    track_type: Mapped[str] = mapped_column(String(50), nullable=False)
    car_id: Mapped[int] = mapped_column(Integer, nullable=False)
    car_name: Mapped[str] = mapped_column(String(255), nullable=False)
    car_class_id: Mapped[int] = mapped_column(Integer, nullable=False)
    series_id: Mapped[int] = mapped_column(Integer, nullable=False)
    session_type: Mapped[str] = mapped_column(String(255), nullable=False)

    # Relationships
    laps: Mapped[list[Lap]] = relationship(
        "Lap", back_populates="track_session", cascade="all, delete-orphan", init=False
    )
    telemetry_frames: Mapped[list[Telemetry]] = relationship(
        "Telemetry",
        back_populates="track_session",
        cascade="all, delete-orphan",
        init=False,
    )

    # Indexes
    __table_args__ = (
        Index("idx_session_track_id", "track_id"),
        Index("idx_session_car_id", "car_id"),
        Index("idx_session_track_id_car_id", "track_id", "car_id"),
    )

    @classmethod
    def from_session_frame(cls, session_frame: SessionFrame) -> Self:
        return cls(
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

    def to_session_frame(self) -> SessionFrame:
        """Convert TrackSession to SessionFrame."""
        return SessionFrame(
            timestamp=self.created_at,
            session_id=self.id,
            track_id=self.track_id,
            track_name=self.track_name,
            track_config_name=self.track_config_name,
            track_type=self.track_type,
            car_id=self.car_id,
            car_name=self.car_name,
            car_class_id=self.car_class_id,
            series_id=self.series_id,
            session_type=self.session_type,
        )
