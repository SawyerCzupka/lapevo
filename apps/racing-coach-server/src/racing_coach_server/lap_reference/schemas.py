"""Pydantic schemas for the lap_reference API."""

from pydantic import BaseModel
from racing_coach_core.algs.events import (
    BrakingMetrics,
    CornerMetrics,
)


class ReferenceLapResponse(BaseModel):
    """Response model for the best reference lap for a track+car combo."""

    lap_id: str
    lap_time: float
    total_corners: int
    total_braking_zones: int
    average_corner_speed: float
    max_speed: float
    min_speed: float
    braking_zones: list[BrakingMetrics]
    corners: list[CornerMetrics]
