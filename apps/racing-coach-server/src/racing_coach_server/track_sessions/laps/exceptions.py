"""Feature-specific exceptions for the laps sub-domain."""

from racing_coach_server.exceptions import NotFoundError, RacingCoachException


class LapsException(RacingCoachException):
    """Base exception for laps domain errors."""


class LapNotFoundError(NotFoundError, LapsException):
    """Raised when a lap cannot be found."""

    def __init__(self, lap_id: str | None = None) -> None:
        message = f"Lap {lap_id} not found" if lap_id else "Lap not found"
        super().__init__(message)
