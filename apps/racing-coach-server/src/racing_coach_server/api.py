from fastapi import APIRouter

from racing_coach_server.auth.router import router as auth_router
from racing_coach_server.core_test.router import router as ct_router
from racing_coach_server.health.router import router as health_router
from racing_coach_server.lap_comparison.router import router as lap_comparison_router
from racing_coach_server.lap_metrics.router import router as lap_metrics_router
from racing_coach_server.lap_reference.router import router as lap_reference_router
from racing_coach_server.telemetry.router import router as telemetry_router
from racing_coach_server.track_sessions.laps.router import router as laps_router
from racing_coach_server.track_sessions.router import router as track_sessions_router
from racing_coach_server.tracks.router import router as tracks_router

api_router = APIRouter()

api_router.include_router(health_router, prefix="")
api_router.include_router(auth_router, prefix="/auth")
api_router.include_router(telemetry_router, prefix="/telemetry")
api_router.include_router(track_sessions_router, prefix="/track_sessions")
api_router.include_router(laps_router, prefix="/track_sessions/{session_id}/laps")
api_router.include_router(lap_metrics_router, prefix="/lap_metrics")
api_router.include_router(lap_comparison_router, prefix="/lap_comparison")
api_router.include_router(lap_reference_router, prefix="/lap_reference")
api_router.include_router(tracks_router, prefix="/tracks")
api_router.include_router(ct_router)
