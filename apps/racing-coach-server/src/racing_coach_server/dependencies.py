"""Dependency injection functions for FastAPI route handlers."""

from typing import Annotated

from fastapi import Depends

from racing_coach_server.auth.dependencies import (
    AdminUserDep,
    AuthServiceDep,
    CurrentUserDep,
    OptionalUserDep,
    get_auth_service,
    get_current_user,
    get_current_user_optional,
    require_admin,
)
from racing_coach_server.auth.service import AuthService
from racing_coach_server.database.dependencies import AsyncSessionDep
from racing_coach_server.lap_metrics.service import LapMetricsService
from racing_coach_server.lap_reference.service import LapReferenceService
from racing_coach_server.telemetry.service import TelemetryService
from racing_coach_server.track_sessions.laps.service import LapService
from racing_coach_server.track_sessions.service import TrackSessionService


# Track session service
async def get_track_session_service(db: AsyncSessionDep) -> TrackSessionService:
    """Provide TrackSessionService with injected AsyncSession."""
    return TrackSessionService(db)


TrackSessionServiceDep = Annotated[TrackSessionService, Depends(get_track_session_service)]


# Lap service
async def get_lap_service(db: AsyncSessionDep) -> LapService:
    """Provide LapService with injected AsyncSession."""
    return LapService(db)


LapServiceDep = Annotated[LapService, Depends(get_lap_service)]


# Lap metrics service
async def get_lap_metrics_service(db: AsyncSessionDep) -> LapMetricsService:
    """Provide LapMetricsService with injected AsyncSession."""
    return LapMetricsService(db)


LapMetricsServiceDep = Annotated[LapMetricsService, Depends(get_lap_metrics_service)]


# Lap reference service
async def get_lap_reference_service(db: AsyncSessionDep) -> LapReferenceService:
    """Provide LapReferenceService with injected AsyncSession."""
    return LapReferenceService(db)


LapReferenceServiceDep = Annotated[LapReferenceService, Depends(get_lap_reference_service)]


# Telemetry service
async def get_telemetry_service(db: AsyncSessionDep) -> TelemetryService:
    """Provide TelemetryService with injected AsyncSession."""
    return TelemetryService(db)


TelemetryServiceDep = Annotated[TelemetryService, Depends(get_telemetry_service)]


__all__ = [
    # Database
    "AsyncSessionDep",
    # Track sessions
    "get_track_session_service",
    "TrackSessionServiceDep",
    # Laps
    "get_lap_service",
    "LapServiceDep",
    # Lap metrics
    "get_lap_metrics_service",
    "LapMetricsServiceDep",
    # Lap reference
    "get_lap_reference_service",
    "LapReferenceServiceDep",
    # Telemetry
    "get_telemetry_service",
    "TelemetryServiceDep",
    # Auth
    "get_auth_service",
    "AuthService",
    "AuthServiceDep",
    "get_current_user",
    "get_current_user_optional",
    "require_admin",
    "CurrentUserDep",
    "OptionalUserDep",
    "AdminUserDep",
]
