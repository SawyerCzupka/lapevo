"""SQLAlchemy model for raw telemetry frames."""

from __future__ import annotations

import uuid
from datetime import datetime
from typing import TYPE_CHECKING, Self

from racing_coach_core import TelemetryFrame
from sqlalchemy import Boolean, DateTime, Float, ForeignKey, Index, Integer, func
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from racing_coach_server.database.base import Base

if TYPE_CHECKING:
    from racing_coach_server.track_sessions.laps.models import Lap
    from racing_coach_server.track_sessions.models import TrackSession


class Telemetry(Base):
    """Model representing a single telemetry frame."""

    __tablename__ = "telemetry"

    track_session_id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True),
        ForeignKey("track_session.id", ondelete="CASCADE"),
        nullable=False,
    )
    lap_id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), ForeignKey("lap.id", ondelete="CASCADE"), nullable=False
    )

    # Time fields
    timestamp: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        primary_key=True,
        server_default=func.now(),
        nullable=False,
    )
    session_time: Mapped[float] = mapped_column(Float, nullable=False)

    # Lap information
    lap_number: Mapped[int] = mapped_column(Integer, nullable=False)
    lap_distance_pct: Mapped[float] = mapped_column(Float, nullable=False)
    lap_distance: Mapped[float] = mapped_column(Float, nullable=False)
    current_lap_time: Mapped[float] = mapped_column(Float, nullable=False)
    last_lap_time: Mapped[float | None] = mapped_column(Float, nullable=True)
    best_lap_time: Mapped[float | None] = mapped_column(Float, nullable=True)

    # Vehicle state
    speed: Mapped[float] = mapped_column(Float, nullable=False)
    rpm: Mapped[float] = mapped_column(Float, nullable=False)
    gear: Mapped[int] = mapped_column(Integer, nullable=False)

    # Driver inputs
    throttle: Mapped[float] = mapped_column(Float, nullable=False)
    brake: Mapped[float] = mapped_column(Float, nullable=False)
    clutch: Mapped[float] = mapped_column(Float, nullable=False)
    steering_angle: Mapped[float] = mapped_column(Float, nullable=False)

    # Vehicle dynamics
    lateral_acceleration: Mapped[float] = mapped_column(Float, nullable=False)
    longitudinal_acceleration: Mapped[float] = mapped_column(Float, nullable=False)
    vertical_acceleration: Mapped[float] = mapped_column(Float, nullable=False)
    yaw_rate: Mapped[float] = mapped_column(Float, nullable=False)
    roll_rate: Mapped[float] = mapped_column(Float, nullable=False)
    pitch_rate: Mapped[float] = mapped_column(Float, nullable=False)

    # Vehicle velocity
    velocity_x: Mapped[float] = mapped_column(Float, nullable=False)
    velocity_y: Mapped[float] = mapped_column(Float, nullable=False)
    velocity_z: Mapped[float] = mapped_column(Float, nullable=False)

    # Vehicle orientation
    yaw: Mapped[float] = mapped_column(Float, nullable=False)
    pitch: Mapped[float] = mapped_column(Float, nullable=False)
    roll: Mapped[float] = mapped_column(Float, nullable=False)

    # GPS position
    latitude: Mapped[float] = mapped_column(Float, nullable=False)
    longitude: Mapped[float] = mapped_column(Float, nullable=False)
    altitude: Mapped[float] = mapped_column(Float, nullable=False)

    # Tire data - flattened for better query performance
    # Left Front
    lf_tire_temp_left: Mapped[float | None] = mapped_column(Float, nullable=True)
    lf_tire_temp_middle: Mapped[float | None] = mapped_column(Float, nullable=True)
    lf_tire_temp_right: Mapped[float | None] = mapped_column(Float, nullable=True)
    lf_tire_wear_left: Mapped[float | None] = mapped_column(Float, nullable=True)
    lf_tire_wear_middle: Mapped[float | None] = mapped_column(Float, nullable=True)
    lf_tire_wear_right: Mapped[float | None] = mapped_column(Float, nullable=True)
    lf_brake_pressure: Mapped[float | None] = mapped_column(Float, nullable=True)

    # Right Front
    rf_tire_temp_left: Mapped[float | None] = mapped_column(Float, nullable=True)
    rf_tire_temp_middle: Mapped[float | None] = mapped_column(Float, nullable=True)
    rf_tire_temp_right: Mapped[float | None] = mapped_column(Float, nullable=True)
    rf_tire_wear_left: Mapped[float | None] = mapped_column(Float, nullable=True)
    rf_tire_wear_middle: Mapped[float | None] = mapped_column(Float, nullable=True)
    rf_tire_wear_right: Mapped[float | None] = mapped_column(Float, nullable=True)
    rf_brake_pressure: Mapped[float | None] = mapped_column(Float, nullable=True)

    # Left Rear
    lr_tire_temp_left: Mapped[float | None] = mapped_column(Float, nullable=True)
    lr_tire_temp_middle: Mapped[float | None] = mapped_column(Float, nullable=True)
    lr_tire_temp_right: Mapped[float | None] = mapped_column(Float, nullable=True)
    lr_tire_wear_left: Mapped[float | None] = mapped_column(Float, nullable=True)
    lr_tire_wear_middle: Mapped[float | None] = mapped_column(Float, nullable=True)
    lr_tire_wear_right: Mapped[float | None] = mapped_column(Float, nullable=True)
    lr_brake_pressure: Mapped[float | None] = mapped_column(Float, nullable=True)

    # Right Rear
    rr_tire_temp_left: Mapped[float | None] = mapped_column(Float, nullable=True)
    rr_tire_temp_middle: Mapped[float | None] = mapped_column(Float, nullable=True)
    rr_tire_temp_right: Mapped[float | None] = mapped_column(Float, nullable=True)
    rr_tire_wear_left: Mapped[float | None] = mapped_column(Float, nullable=True)
    rr_tire_wear_middle: Mapped[float | None] = mapped_column(Float, nullable=True)
    rr_tire_wear_right: Mapped[float | None] = mapped_column(Float, nullable=True)
    rr_brake_pressure: Mapped[float | None] = mapped_column(Float, nullable=True)

    # Track conditions
    track_temp: Mapped[float | None] = mapped_column(Float, nullable=True)
    track_wetness: Mapped[int | None] = mapped_column(Integer, nullable=True)
    air_temp: Mapped[float | None] = mapped_column(Float, nullable=True)

    # Session state
    session_flags: Mapped[int | None] = mapped_column(Integer, nullable=True)
    track_surface: Mapped[int | None] = mapped_column(Integer, nullable=True)
    on_pit_road: Mapped[bool | None] = mapped_column(Boolean, nullable=True)

    id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), primary_key=True, default_factory=uuid.uuid4
    )

    # Relationships
    track_session: Mapped[TrackSession] = relationship(
        "TrackSession", back_populates="telemetry_frames", init=False
    )
    lap: Mapped[Lap] = relationship("Lap", back_populates="telemetry_frames", init=False)

    # Indexes for efficient time-series queries
    __table_args__ = (
        Index("idx_telemetry_lap_id", "lap_id"),
        Index("idx_telemetry_track_session_id", "track_session_id"),
        Index("idx_telemetry_timestamp", "timestamp"),
        Index("idx_session_time", "session_time"),
    )

    @classmethod
    def from_telemetry_frame(
        cls,
        frame: TelemetryFrame,
        track_session_id: uuid.UUID,
        lap_id: uuid.UUID,
    ) -> Self:
        """Create a Telemetry database model from a TelemetryFrame."""
        return cls(
            track_session_id=track_session_id,
            lap_id=lap_id,
            # Time fields
            timestamp=frame.timestamp,
            session_time=frame.session_time,
            # Lap information
            lap_number=frame.lap_number,
            lap_distance_pct=frame.lap_distance_pct,
            lap_distance=frame.lap_distance,
            current_lap_time=frame.current_lap_time,
            last_lap_time=frame.last_lap_time if frame.last_lap_time > 0 else None,
            best_lap_time=frame.best_lap_time if frame.best_lap_time > 0 else None,
            # Vehicle state
            speed=frame.speed,
            rpm=frame.rpm,
            gear=frame.gear,
            # Driver inputs
            throttle=frame.throttle,
            brake=frame.brake,
            clutch=frame.clutch,
            steering_angle=frame.steering_angle,
            # Vehicle dynamics
            lateral_acceleration=frame.lateral_acceleration,
            longitudinal_acceleration=frame.longitudinal_acceleration,
            vertical_acceleration=frame.vertical_acceleration,
            yaw_rate=frame.yaw_rate,
            roll_rate=frame.roll_rate,
            pitch_rate=frame.pitch_rate,
            # Vehicle velocity
            velocity_x=frame.velocity_x,
            velocity_y=frame.velocity_y,
            velocity_z=frame.velocity_z,
            # Vehicle orientation
            yaw=frame.yaw,
            pitch=frame.pitch,
            roll=frame.roll,
            # GPS position
            latitude=frame.latitude,
            longitude=frame.longitude,
            altitude=frame.altitude,
            # Tire temps - Left Front
            lf_tire_temp_left=frame.tire_temps.get("LF", {}).get("left"),
            lf_tire_temp_middle=frame.tire_temps.get("LF", {}).get("middle"),
            lf_tire_temp_right=frame.tire_temps.get("LF", {}).get("right"),
            # Tire temps - Right Front
            rf_tire_temp_left=frame.tire_temps.get("RF", {}).get("left"),
            rf_tire_temp_middle=frame.tire_temps.get("RF", {}).get("middle"),
            rf_tire_temp_right=frame.tire_temps.get("RF", {}).get("right"),
            # Tire temps - Left Rear
            lr_tire_temp_left=frame.tire_temps.get("LR", {}).get("left"),
            lr_tire_temp_middle=frame.tire_temps.get("LR", {}).get("middle"),
            lr_tire_temp_right=frame.tire_temps.get("LR", {}).get("right"),
            # Tire temps - Right Rear
            rr_tire_temp_left=frame.tire_temps.get("RR", {}).get("left"),
            rr_tire_temp_middle=frame.tire_temps.get("RR", {}).get("middle"),
            rr_tire_temp_right=frame.tire_temps.get("RR", {}).get("right"),
            # Tire wear - Left Front
            lf_tire_wear_left=frame.tire_wear.get("LF", {}).get("left"),
            lf_tire_wear_middle=frame.tire_wear.get("LF", {}).get("middle"),
            lf_tire_wear_right=frame.tire_wear.get("LF", {}).get("right"),
            # Tire wear - Right Front
            rf_tire_wear_left=frame.tire_wear.get("RF", {}).get("left"),
            rf_tire_wear_middle=frame.tire_wear.get("RF", {}).get("middle"),
            rf_tire_wear_right=frame.tire_wear.get("RF", {}).get("right"),
            # Tire wear - Left Rear
            lr_tire_wear_left=frame.tire_wear.get("LR", {}).get("left"),
            lr_tire_wear_middle=frame.tire_wear.get("LR", {}).get("middle"),
            lr_tire_wear_right=frame.tire_wear.get("LR", {}).get("right"),
            # Tire wear - Right Rear
            rr_tire_wear_left=frame.tire_wear.get("RR", {}).get("left"),
            rr_tire_wear_middle=frame.tire_wear.get("RR", {}).get("middle"),
            rr_tire_wear_right=frame.tire_wear.get("RR", {}).get("right"),
            # Brake pressure
            lf_brake_pressure=frame.brake_line_pressure.get("LF"),
            rf_brake_pressure=frame.brake_line_pressure.get("RF"),
            lr_brake_pressure=frame.brake_line_pressure.get("LR"),
            rr_brake_pressure=frame.brake_line_pressure.get("RR"),
            # Track conditions
            track_temp=frame.track_temp,
            track_wetness=frame.track_wetness,
            air_temp=frame.air_temp,
            # Session state
            session_flags=frame.session_flags,
            track_surface=frame.track_surface,
            on_pit_road=frame.on_pit_road,
        )
