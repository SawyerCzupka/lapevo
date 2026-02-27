"""Pydantic schemas for the laps sub-feature API."""

from datetime import datetime

from pydantic import BaseModel, Field


class LapDetailResponse(BaseModel):
    """Response model for lap detail endpoint."""

    lap_id: str = Field(description="UUID of the lap")
    session_id: str = Field(description="UUID of the session")
    lap_number: int = Field(description="Lap number in the session")
    lap_time: float | None = Field(description="Lap time in seconds")
    is_valid: bool = Field(description="Whether the lap is valid")
    track_name: str = Field(description="Name of the track")
    track_config_name: str | None = Field(description="Track configuration name")
    car_name: str = Field(description="Name of the car")
    has_metrics: bool = Field(description="Whether metrics have been computed")
    created_at: datetime = Field(description="When the lap was recorded")


class TelemetryFrameResponse(BaseModel):
    """Response model for a single telemetry frame."""

    timestamp: datetime
    session_time: float
    lap_number: int
    lap_distance_pct: float
    lap_distance: float
    current_lap_time: float

    # Vehicle state
    speed: float
    rpm: float
    gear: int

    # Driver inputs
    throttle: float
    brake: float
    clutch: float
    steering_angle: float

    # Vehicle dynamics
    lateral_acceleration: float
    longitudinal_acceleration: float
    vertical_acceleration: float
    yaw_rate: float
    roll_rate: float
    pitch_rate: float

    # Vehicle velocity
    velocity_x: float
    velocity_y: float
    velocity_z: float

    # Vehicle orientation
    yaw: float
    pitch: float
    roll: float

    # GPS position
    latitude: float
    longitude: float
    altitude: float

    # Track conditions (optional)
    track_temp: float | None = None
    air_temp: float | None = None


class LapTelemetryResponse(BaseModel):
    """Response model for lap telemetry endpoint."""

    lap_id: str = Field(description="UUID of the lap")
    session_id: str = Field(description="UUID of the session")
    lap_number: int = Field(description="Lap number")
    frame_count: int = Field(description="Number of telemetry frames")
    frames: list[TelemetryFrameResponse] = Field(description="Telemetry frames")
