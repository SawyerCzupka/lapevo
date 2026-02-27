"""Feature-specific exceptions for the lap_metrics domain."""

from racing_coach_server.exceptions import NotFoundError, RacingCoachException


class LapMetricsException(RacingCoachException):
    """Base exception for lap_metrics domain errors."""


class LapMetricsNotFoundError(NotFoundError, LapMetricsException):
    """Raised when metrics cannot be found for a lap."""

    def __init__(self, lap_id: str | None = None) -> None:
        message = f"Metrics not found for lap {lap_id}" if lap_id else "Metrics not found"
        super().__init__(message)
