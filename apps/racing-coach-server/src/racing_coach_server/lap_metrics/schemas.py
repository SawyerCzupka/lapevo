"""Pydantic schemas for the lap_metrics API."""

from pydantic import BaseModel
from racing_coach_core.algs.events import (
    BrakingMetrics,
    CornerMetrics,
    LapMetrics,
)


class LapMetricsUploadRequest(BaseModel):
    """Request model for uploading lap metrics."""

    lap_metrics: LapMetrics
    lap_id: str  # UUID as string from client


class LapMetricsUploadResponse(BaseModel):
    """Response model for metrics upload."""

    status: str
    message: str
    lap_metrics_id: str  # UUID as string


class LapMetricsResponse(BaseModel):
    """Response model for retrieving lap metrics."""

    lap_id: str
    lap_time: float | None
    total_corners: int
    total_braking_zones: int
    average_corner_speed: float
    max_speed: float
    min_speed: float
    braking_zones: list[BrakingMetrics]
    corners: list[CornerMetrics]
