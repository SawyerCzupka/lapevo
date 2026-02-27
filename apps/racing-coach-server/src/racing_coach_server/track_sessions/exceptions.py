"""Feature-specific exceptions for the track_sessions domain."""

from racing_coach_server.exceptions import NotFoundError, RacingCoachException


class TrackSessionsException(RacingCoachException):
    """Base exception for track_sessions domain errors."""


class SessionNotFoundError(NotFoundError, TrackSessionsException):
    """Raised when a session cannot be found."""

    def __init__(self, session_id: str | None = None) -> None:
        message = f"Session {session_id} not found" if session_id else "Session not found"
        super().__init__(message)
