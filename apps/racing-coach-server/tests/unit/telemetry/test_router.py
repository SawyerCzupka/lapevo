"""Unit tests for telemetry router endpoints."""

from contextlib import asynccontextmanager
from typing import Any
from unittest.mock import AsyncMock, patch
from uuid import uuid4

import pytest
from httpx import ASGITransport, AsyncClient
from racing_coach_server.app import app
from racing_coach_server.track_sessions.laps.models import Lap
from racing_coach_server.track_sessions.models import TrackSession
from sqlalchemy.ext.asyncio import AsyncSession

from tests.polyfactories import (
    LapTelemetryFactory,
    SessionFrameFactory,
    TelemetryFrameFactory,
    TrackSessionFactory,
)


@pytest.mark.unit
class TestTelemetryRouter:
    """Unit tests for telemetry API endpoints."""

    async def test_upload_lap_success(
        self,
        telemetry_frame_factory: TelemetryFrameFactory,
        session_frame_factory: SessionFrameFactory,
        lap_telemetry_factory: LapTelemetryFactory,
        track_session_factory: TrackSessionFactory,
    ):
        """Test successful lap upload."""
        # Arrange
        frames = [telemetry_frame_factory.build(lap_number=1) for _ in range(100)]
        # lap_telemetry = lap_telemetry_factory.build(frames=frames)
        session_frame = session_frame_factory.build()

        mock_track_session = track_session_factory.build(
            id=session_frame.session_id,
            track_id=session_frame.track_id,
            track_name=session_frame.track_name,
            track_config_name=session_frame.track_config_name,
            track_type=session_frame.track_type,
            car_id=session_frame.car_id,
            car_name=session_frame.car_name,
            car_class_id=session_frame.car_class_id,
            series_id=session_frame.series_id,
            session_type=session_frame.session_type,
        )

        lap_id = uuid4()
        mock_lap = Lap(
            id=lap_id,
            track_session_id=session_frame.session_id,
            lap_number=1,
            lap_time=None,
            is_valid=False,
        )

        # Arrange service mocks
        mock_session_service = AsyncMock()
        mock_session_service.add_or_get_session.return_value = mock_track_session
        mock_session_service.add_lap.return_value = mock_lap

        mock_telemetry_service = AsyncMock()
        mock_telemetry_service.add_telemetry_sequence.return_value = None

        mock_db = AsyncMock()

        async def mock_session_service_dep():
            return mock_session_service

        async def mock_telemetry_service_dep():
            return mock_telemetry_service

        async def mock_db_dep():
            return mock_db

        @asynccontextmanager
        async def mock_transaction(session: AsyncSession):
            yield session

        # Use FastAPI dependency overrides
        from racing_coach_server.database.engine import get_async_session
        from racing_coach_server.dependencies import (
            get_lap_service,
            get_telemetry_service,
            get_track_session_service,
        )

        app.dependency_overrides[get_track_session_service] = mock_session_service_dep
        app.dependency_overrides[get_lap_service] = mock_session_service_dep
        app.dependency_overrides[get_telemetry_service] = mock_telemetry_service_dep
        app.dependency_overrides[get_async_session] = mock_db_dep

        with patch("racing_coach_server.telemetry.router.transactional_session") as mock_txn:
            mock_txn.side_effect = mock_transaction

            async with AsyncClient(
                transport=ASGITransport(app=app), base_url="http://test"
            ) as client:
                # Act

                data: dict[str, Any] = {
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

                response = await client.post("/api/v1/telemetry/lap", json=data)

        # Clean up override
        app.dependency_overrides.clear()

        # Assert
        assert response.status_code == 200
        data: dict[str, Any] = response.json()
        assert data["status"] == "success"
        assert data["lap_id"] == str(lap_id)
        mock_session_service.add_or_get_session.assert_called_once()
        mock_session_service.add_lap.assert_called_once()
        mock_telemetry_service.add_telemetry_sequence.assert_called_once()

    async def test_get_latest_session_success(
        self,
        track_session_factory: TrackSessionFactory,
    ) -> None:
        """Test retrieving the latest session."""
        # Arrange
        mock_session: TrackSession = track_session_factory.build()

        mock_session_service = AsyncMock()
        mock_session_service.get_latest_session.return_value = mock_session

        async def mock_session_service_dep():
            return mock_session_service

        # Use FastAPI dependency overrides
        from racing_coach_server.dependencies import get_track_session_service

        app.dependency_overrides[get_track_session_service] = mock_session_service_dep

        async with AsyncClient(transport=ASGITransport(app=app), base_url="http://test") as client:
            # Act
            response = await client.get("/api/v1/track_sessions/latest")

        # Clean up override
        app.dependency_overrides.clear()

        # Assert
        assert response.status_code == 200
        data: dict[str, Any] = response.json()
        assert data["track_id"] == mock_session.track_id
        assert data["track_name"] == mock_session.track_name
        assert data["car_id"] == mock_session.car_id

    async def test_get_latest_session_not_found(self) -> None:
        """Test retrieving latest session when none exists."""
        # Arrange
        mock_session_service = AsyncMock()
        mock_session_service.get_latest_session.return_value = None

        async def mock_session_service_dep():
            return mock_session_service

        # Use FastAPI dependency overrides
        from racing_coach_server.dependencies import get_track_session_service

        app.dependency_overrides[get_track_session_service] = mock_session_service_dep

        async with AsyncClient(transport=ASGITransport(app=app), base_url="http://test") as client:
            # Act
            response = await client.get("/api/v1/track_sessions/latest")

        # Clean up override
        app.dependency_overrides.clear()

        # Assert
        assert response.status_code == 404
        assert "No sessions found" in response.json()["detail"]
