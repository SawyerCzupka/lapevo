"""Integration tests for API endpoints with real database."""

from typing import Any

import pytest
from httpx import AsyncClient, Response
from racing_coach_core.schemas.telemetry import SessionFrame, TelemetryFrame
from racing_coach_server.telemetry.models import Telemetry
from racing_coach_server.track_sessions.laps.models import Lap
from racing_coach_server.track_sessions.models import TrackSession
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from tests.polyfactories import SessionFrameFactory, TelemetryFrameFactory, TrackSessionFactory


@pytest.mark.integration
@pytest.mark.slow
class TestHealthEndpoint:
    """Integration tests for health check endpoint."""

    async def test_health_check_with_database(self, test_client: AsyncClient) -> None:
        """Test health check endpoint returns healthy with real database."""
        # Act
        response: Response = await test_client.get("/api/v1/health")

        # Assert
        assert response.status_code == 200
        data: dict[str, Any] = response.json()
        assert data["status"] == "healthy"
        assert data["database_status"] == "healthy"
        assert "successful" in data["database_message"].lower()


@pytest.mark.integration
@pytest.mark.slow
class TestTelemetryEndpoints:
    """Integration tests for telemetry API endpoints."""

    async def test_upload_lap_creates_session_and_data(
        self,
        test_client: AsyncClient,
        db_session: AsyncSession,
        telemetry_frame_factory: TelemetryFrameFactory,
        session_frame_factory: SessionFrameFactory,
    ) -> None:
        """Test uploading a lap creates session, lap, and telemetry data."""
        # Arrange
        session_frame: SessionFrame = session_frame_factory.build()
        frames: list[TelemetryFrame] = [
            telemetry_frame_factory.build(lap_number=1) for _ in range(10)
        ]

        # Act
        data = {
            "lap": {
                "frames": [
                    {
                        **frame.model_dump(),
                        "timestamp": frame.timestamp.isoformat(),
                    }
                    for frame in frames
                ],
                "lap_time": 90.5,
            },
            "session": {
                **session_frame.model_dump(),
                "timestamp": session_frame.timestamp.isoformat(),
                "session_id": str(session_frame.session_id),
            },
        }
        response: Response = await test_client.post(
            "/api/v1/telemetry/lap",
            json=data,
        )

        # Assert
        assert response.status_code == 200
        data: dict[str, Any] = response.json()
        assert data["status"] == "success"
        assert "lap_id" in data

        # Verify session was created
        stmt = select(TrackSession).where(TrackSession.id == session_frame.session_id)
        result = await db_session.execute(stmt)
        track_session: TrackSession | None = result.scalar_one_or_none()
        assert track_session is not None
        assert track_session.track_id == session_frame.track_id
        assert track_session.car_id == session_frame.car_id

        # Verify lap was created
        stmt = select(Lap).where(Lap.track_session_id == session_frame.session_id)
        result = await db_session.execute(stmt)
        lap: Lap | None = result.scalar_one_or_none()
        assert lap is not None
        assert lap.lap_number == 1

        # Verify telemetry was created
        stmt = select(Telemetry).where(Telemetry.lap_id == lap.id)
        result = await db_session.execute(stmt)
        telemetry_records: list[Telemetry] = list(result.scalars().all())
        assert len(telemetry_records) == 10

    async def test_upload_lap_idempotent_session_creation(
        self,
        test_client: AsyncClient,
        db_session: AsyncSession,
        telemetry_frame_factory: TelemetryFrameFactory,
        session_frame_factory: SessionFrameFactory,
    ) -> None:
        """Test uploading multiple laps for same session doesn't duplicate session."""
        # Arrange
        session_frame: SessionFrame = session_frame_factory.build()

        # Upload first lap
        frames1 = [telemetry_frame_factory.build(lap_number=1) for _ in range(5)]
        data: dict[str, Any] = {
            "lap": {
                "frames": [
                    {**f.model_dump(), "timestamp": f.timestamp.isoformat()} for f in frames1
                ],
                "lap_time": 90.5,
            },
            "session": {
                **session_frame.model_dump(),
                "timestamp": session_frame.timestamp.isoformat(),
                "session_id": str(session_frame.session_id),
            },
        }
        await test_client.post(
            "/api/v1/telemetry/lap",
            json=data,
        )

        # Act - Upload second lap with same session
        frames2 = [telemetry_frame_factory.build(lap_number=2) for _ in range(5)]
        data: dict[str, Any] = {
            "lap": {
                "frames": [
                    {**f.model_dump(), "timestamp": f.timestamp.isoformat()} for f in frames2
                ],
                "lap_time": 88.3,
            },
            "session": {
                **session_frame.model_dump(),
                "timestamp": session_frame.timestamp.isoformat(),
                "session_id": str(session_frame.session_id),
            },
        }
        response = await test_client.post(
            "/api/v1/telemetry/lap",
            json=data,
        )

        # Assert
        assert response.status_code == 200

        # Verify only one session was created
        stmt = select(TrackSession).where(TrackSession.id == session_frame.session_id)
        result = await db_session.execute(stmt)
        sessions = result.scalars().all()
        assert len(sessions) == 1

        # Verify two laps were created
        stmt = select(Lap).where(Lap.track_session_id == session_frame.session_id)
        result = await db_session.execute(stmt)
        laps = result.scalars().all()
        assert len(laps) == 2

    async def test_get_latest_session(
        self,
        test_client: AsyncClient,
        db_session: AsyncSession,
        track_session_factory: TrackSessionFactory,
    ) -> None:
        """Test retrieving the latest session."""
        # Arrange - Create two sessions
        session1 = track_session_factory.build()
        session2 = track_session_factory.build()

        db_session.add(session1)
        await db_session.flush()

        # Add small delay to ensure different timestamps
        import asyncio

        await asyncio.sleep(0.01)

        db_session.add(session2)
        await db_session.flush()
        await db_session.commit()

        # Act
        response = await test_client.get("/api/v1/track_sessions/latest")

        # Assert
        assert response.status_code == 200
        data = response.json()
        # Should return session2 (most recent)
        assert data["session_id"] == str(session2.id)
        assert data["track_id"] == session2.track_id

    async def test_get_latest_session_not_found(
        self,
        test_client: AsyncClient,
    ) -> None:
        """Test retrieving latest session when none exists."""
        # Act
        response = await test_client.get("/api/v1/track_sessions/latest")

        # Assert
        assert response.status_code == 404
        assert "No sessions found" in response.json()["detail"]


@pytest.mark.integration
@pytest.mark.slow
class TestTransactionManagement:
    """Integration tests for database transaction management."""

    async def test_lap_upload_transaction_commits_on_success(
        self,
        test_client: AsyncClient,
        db_session: AsyncSession,
        telemetry_frame_factory: TelemetryFrameFactory,
        session_frame_factory: SessionFrameFactory,
    ) -> None:
        """Test that successful lap upload commits all changes."""
        # Arrange
        session_frame: SessionFrame = session_frame_factory.build()
        frames = [telemetry_frame_factory.build(lap_number=1) for _ in range(5)]

        # Act
        data: dict[str, Any] = {
            "lap": {
                "frames": [
                    {**f.model_dump(), "timestamp": f.timestamp.isoformat()} for f in frames
                ],
                "lap_time": 90.5,
            },
            "session": {
                **session_frame.model_dump(),
                "timestamp": session_frame.timestamp.isoformat(),
                "session_id": str(session_frame.session_id),
            },
        }
        response = await test_client.post(
            "/api/v1/telemetry/lap",
            json=data,
        )

        # Assert
        assert response.status_code == 200

        # Verify data persisted (in a new session to confirm commit)
        stmt = select(TrackSession).where(TrackSession.id == session_frame.session_id)
        result = await db_session.execute(stmt)
        session = result.scalar_one_or_none()
        assert session is not None

        stmt = select(Lap).where(Lap.track_session_id == session_frame.session_id)
        result = await db_session.execute(stmt)
        laps = result.scalars().all()
        assert len(laps) == 1

        stmt = select(Telemetry).where(Telemetry.track_session_id == session_frame.session_id)
        result = await db_session.execute(stmt)
        telemetry = result.scalars().all()
        assert len(telemetry) == 5


@pytest.mark.integration
@pytest.mark.slow
class TestErrorHandling:
    """Integration tests for error handling."""

    @pytest.mark.parametrize(
        "invalid_payload,expected_status",
        [
            # Missing required field
            ({"lap": {"frames": []}, "session": {}}, 422),
            # Invalid data types
            (
                {
                    "lap": {"frames": "not-a-list", "lap_time": 90.5},
                    "session": {"session_id": "not-a-uuid"},
                },
                422,
            ),
        ],
    )
    async def test_upload_lap_validation_errors(
        self,
        test_client: AsyncClient,
        invalid_payload: dict[str, Any],
        expected_status: int,
    ) -> None:
        """Test that invalid payloads return appropriate error codes."""
        # Act
        response: Response = await test_client.post(
            "/api/v1/telemetry/lap",
            json=invalid_payload,
        )

        # Assert
        assert response.status_code == expected_status

    async def test_duplicate_lap_number_constraint(
        self,
        test_client: AsyncClient,
        db_session: AsyncSession,
        telemetry_frame_factory: TelemetryFrameFactory,
        session_frame_factory: SessionFrameFactory,
    ) -> None:
        """Test that uploading same lap number twice fails."""
        # Arrange
        session_frame: SessionFrame = session_frame_factory.build()
        frames = [telemetry_frame_factory.build(lap_number=1) for _ in range(3)]

        # Upload first lap
        data: dict[str, Any] = {
            "lap": {
                "frames": [
                    {**f.model_dump(), "timestamp": f.timestamp.isoformat()} for f in frames
                ],
                "lap_time": 90.5,
            },
            "session": {
                **session_frame.model_dump(),
                "timestamp": session_frame.timestamp.isoformat(),
                "session_id": str(session_frame.session_id),
            },
        }
        await test_client.post(
            "/api/v1/telemetry/lap",
            json=data,
        )

        # Act - Try to upload same lap number again
        data: dict[str, Any] = {
            "lap": {
                "frames": [
                    {**f.model_dump(), "timestamp": f.timestamp.isoformat()} for f in frames
                ],
                "lap_time": 91.2,
            },
            "session": {
                **session_frame.model_dump(),
                "timestamp": session_frame.timestamp.isoformat(),
                "session_id": str(session_frame.session_id),
            },
        }
        response = await test_client.post(
            "/api/v1/telemetry/lap",
            json=data,
        )

        # Assert
        assert response.status_code == 500
        assert "error" in response.json()["detail"].lower()
