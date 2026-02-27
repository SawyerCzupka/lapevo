"""SQLAlchemy models for the lap_metrics feature."""

from __future__ import annotations

import uuid
from typing import TYPE_CHECKING

from sqlalchemy import Boolean, Float, ForeignKey, Index, Integer, UniqueConstraint
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from racing_coach_server.database.base import Base
from racing_coach_server.database.mixins import TimestampMixin

if TYPE_CHECKING:
    from racing_coach_server.track_sessions.laps.models import Lap


class LapMetricsDB(TimestampMixin, Base):
    """Model representing aggregate metrics for a lap."""

    __tablename__ = "lap_metrics"

    # Non-default fields first
    lap_id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), ForeignKey("lap.id", ondelete="CASCADE"), nullable=False
    )

    lap_time: Mapped[float | None] = mapped_column(Float, nullable=True)
    total_corners: Mapped[int] = mapped_column(Integer, nullable=False)
    total_braking_zones: Mapped[int] = mapped_column(Integer, nullable=False)
    average_corner_speed: Mapped[float] = mapped_column(Float, nullable=False)
    max_speed: Mapped[float] = mapped_column(Float, nullable=False)
    min_speed: Mapped[float] = mapped_column(Float, nullable=False)

    # Fields with defaults come after
    id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), primary_key=True, default_factory=uuid.uuid4, init=False
    )

    # Relationships
    lap: Mapped[Lap] = relationship("Lap", back_populates="metrics", init=False)
    braking_zones: Mapped[list[BrakingMetricsDB]] = relationship(
        "BrakingMetricsDB",
        back_populates="lap_metrics",
        cascade="all, delete-orphan",
        init=False,
        order_by="BrakingMetricsDB.zone_number",
    )
    corners: Mapped[list[CornerMetricsDB]] = relationship(
        "CornerMetricsDB",
        back_populates="lap_metrics",
        cascade="all, delete-orphan",
        init=False,
        order_by="CornerMetricsDB.corner_number",
    )

    # Constraints and indexes
    __table_args__ = (
        UniqueConstraint("lap_id", name="uq_lap_metrics_lap_id"),
        Index("idx_lap_metrics_lap_id", "lap_id"),
    )


class BrakingMetricsDB(Base):
    """Model representing metrics for a braking zone."""

    __tablename__ = "braking_metrics"

    # Non-default fields first
    lap_metrics_id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True),
        ForeignKey("lap_metrics.id", ondelete="CASCADE"),
        nullable=False,
    )
    zone_number: Mapped[int] = mapped_column(Integer, nullable=False)

    # Location and timing
    braking_point_distance: Mapped[float] = mapped_column(Float, nullable=False)
    braking_point_speed: Mapped[float] = mapped_column(Float, nullable=False)
    end_distance: Mapped[float] = mapped_column(Float, nullable=False)

    # Performance metrics
    max_brake_pressure: Mapped[float] = mapped_column(Float, nullable=False)
    braking_duration: Mapped[float] = mapped_column(Float, nullable=False)
    minimum_speed: Mapped[float] = mapped_column(Float, nullable=False)

    # Advanced metrics
    initial_deceleration: Mapped[float] = mapped_column(Float, nullable=False)
    average_deceleration: Mapped[float] = mapped_column(Float, nullable=False)
    braking_efficiency: Mapped[float] = mapped_column(Float, nullable=False)

    # Trail braking
    has_trail_braking: Mapped[bool] = mapped_column(Boolean, nullable=False)
    trail_brake_distance: Mapped[float] = mapped_column(Float, nullable=False)
    trail_brake_percentage: Mapped[float] = mapped_column(Float, nullable=False)

    # Fields with defaults come after
    id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), primary_key=True, default_factory=uuid.uuid4, init=False
    )

    # Relationship
    lap_metrics: Mapped[LapMetricsDB] = relationship(
        "LapMetricsDB", back_populates="braking_zones", init=False
    )

    # Indexes
    __table_args__ = (
        Index("idx_braking_metrics_lap_metrics_id", "lap_metrics_id"),
        Index("idx_braking_metrics_zone_number", "lap_metrics_id", "zone_number"),
    )


class CornerMetricsDB(Base):
    """Model representing metrics for a corner."""

    __tablename__ = "corner_metrics"

    # Non-default fields first
    lap_metrics_id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True),
        ForeignKey("lap_metrics.id", ondelete="CASCADE"),
        nullable=False,
    )
    corner_number: Mapped[int] = mapped_column(Integer, nullable=False)

    # Key corner points (distances)
    turn_in_distance: Mapped[float] = mapped_column(Float, nullable=False)
    apex_distance: Mapped[float] = mapped_column(Float, nullable=False)
    exit_distance: Mapped[float] = mapped_column(Float, nullable=False)
    throttle_application_distance: Mapped[float] = mapped_column(Float, nullable=False)

    # Speeds at key points
    turn_in_speed: Mapped[float] = mapped_column(Float, nullable=False)
    apex_speed: Mapped[float] = mapped_column(Float, nullable=False)
    exit_speed: Mapped[float] = mapped_column(Float, nullable=False)
    throttle_application_speed: Mapped[float] = mapped_column(Float, nullable=False)

    # Performance metrics
    max_lateral_g: Mapped[float] = mapped_column(Float, nullable=False)
    time_in_corner: Mapped[float] = mapped_column(Float, nullable=False)
    corner_distance: Mapped[float] = mapped_column(Float, nullable=False)

    # Steering metrics
    max_steering_angle: Mapped[float] = mapped_column(Float, nullable=False)

    # Speed delta
    speed_loss: Mapped[float] = mapped_column(Float, nullable=False)
    speed_gain: Mapped[float] = mapped_column(Float, nullable=False)

    # Fields with defaults come after
    id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), primary_key=True, default_factory=uuid.uuid4, init=False
    )

    # Relationship
    lap_metrics: Mapped[LapMetricsDB] = relationship(
        "LapMetricsDB", back_populates="corners", init=False
    )

    # Indexes
    __table_args__ = (
        Index("idx_corner_metrics_lap_metrics_id", "lap_metrics_id"),
        Index("idx_corner_metrics_corner_number", "lap_metrics_id", "corner_number"),
    )
