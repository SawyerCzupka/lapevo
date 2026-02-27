"""Integration tests for metrics API endpoints."""

from typing import Any
from uuid import uuid4

import pytest
from httpx import AsyncClient
from racing_coach_core.algs.events import BrakingMetrics, CornerMetrics, LapMetrics
from racing_coach_server.lap_metrics.models import LapMetricsDB
from racing_coach_server.track_sessions.laps.models import Lap
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from tests.polyfactories import (
    BrakingMetricsDBFactory,
    CornerMetricsDBFactory,
    LapMetricsDBFactory,
    TrackSessionFactory,
)


@pytest.mark.integration
class TestMetricsAPI:
    """Integration tests for metrics upload and retrieval."""

    async def test_upload_lap_metrics_success(
        self,
        test_client: AsyncClient,
        db_session: AsyncSession,
        track_session_factory: TrackSessionFactory,
    ) -> None:
        """Test successful upload of lap metrics."""
        # Create a track session and lap first
        track_session = track_session_factory.build()
        db_session.add(track_session)

        lap = Lap(
            id=uuid4(),
            track_session_id=track_session.id,
            lap_number=1,
            lap_time=90.5,
            is_valid=True,
        )
        db_session.add(lap)
        await db_session.commit()

        # Create metrics to upload
        lap_metrics = LapMetrics(
            lap_number=1,
            lap_time=90.5,
            braking_zones=[
                BrakingMetrics(
                    braking_point_distance=0.25,
                    braking_point_speed=75.0,
                    end_distance=0.28,
                    max_brake_pressure=0.9,
                    braking_duration=1.5,
                    minimum_speed=45.0,
                    initial_deceleration=-12.0,
                    average_deceleration=-10.5,
                    braking_efficiency=11.7,
                    has_trail_braking=True,
                    trail_brake_distance=0.02,
                    trail_brake_percentage=0.6,
                ),
                BrakingMetrics(
                    braking_point_distance=0.75,
                    braking_point_speed=80.0,
                    end_distance=0.78,
                    max_brake_pressure=0.85,
                    braking_duration=1.2,
                    minimum_speed=50.0,
                    initial_deceleration=-11.0,
                    average_deceleration=-9.5,
                    braking_efficiency=11.2,
                    has_trail_braking=False,
                    trail_brake_distance=0.0,
                    trail_brake_percentage=0.0,
                ),
            ],
            corners=[
                CornerMetrics(
                    turn_in_distance=0.28,
                    apex_distance=0.30,
                    exit_distance=0.33,
                    throttle_application_distance=0.31,
                    turn_in_speed=45.0,
                    apex_speed=40.0,
                    exit_speed=55.0,
                    throttle_application_speed=42.0,
                    max_lateral_g=2.5,
                    time_in_corner=2.0,
                    corner_distance=0.05,
                    max_steering_angle=0.8,
                    speed_loss=5.0,
                    speed_gain=15.0,
                )
            ],
            total_corners=1,
            total_braking_zones=2,
            average_corner_speed=40.0,
            max_speed=80.0,
            min_speed=40.0,
        )

        # Upload metrics
        data: dict[str, Any] = {
            "lap_id": str(lap.id),
            "lap_metrics": {
                "lap_number": lap_metrics.lap_number,
                "lap_time": lap_metrics.lap_time,
                "braking_zones": [
                    {
                        "braking_point_distance": b.braking_point_distance,
                        "braking_point_speed": b.braking_point_speed,
                        "end_distance": b.end_distance,
                        "max_brake_pressure": b.max_brake_pressure,
                        "braking_duration": b.braking_duration,
                        "minimum_speed": b.minimum_speed,
                        "initial_deceleration": b.initial_deceleration,
                        "average_deceleration": b.average_deceleration,
                        "braking_efficiency": b.braking_efficiency,
                        "has_trail_braking": b.has_trail_braking,
                        "trail_brake_distance": b.trail_brake_distance,
                        "trail_brake_percentage": b.trail_brake_percentage,
                    }
                    for b in lap_metrics.braking_zones
                ],
                "corners": [
                    {
                        "turn_in_distance": c.turn_in_distance,
                        "apex_distance": c.apex_distance,
                        "exit_distance": c.exit_distance,
                        "throttle_application_distance": c.throttle_application_distance,
                        "turn_in_speed": c.turn_in_speed,
                        "apex_speed": c.apex_speed,
                        "exit_speed": c.exit_speed,
                        "throttle_application_speed": c.throttle_application_speed,
                        "max_lateral_g": c.max_lateral_g,
                        "time_in_corner": c.time_in_corner,
                        "corner_distance": c.corner_distance,
                        "max_steering_angle": c.max_steering_angle,
                        "speed_loss": c.speed_loss,
                        "speed_gain": c.speed_gain,
                    }
                    for c in lap_metrics.corners
                ],
                "total_corners": lap_metrics.total_corners,
                "total_braking_zones": lap_metrics.total_braking_zones,
                "average_corner_speed": lap_metrics.average_corner_speed,
                "max_speed": lap_metrics.max_speed,
                "min_speed": lap_metrics.min_speed,
            },
        }
        response = await test_client.post(
            "/api/v1/lap_metrics/lap",
            json=data,
        )

        # Assert
        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "success"
        assert "lap_metrics_id" in data

        # Verify metrics were stored in database
        result = await db_session.execute(
            select(LapMetricsDB)
            .where(LapMetricsDB.lap_id == lap.id)
            .options(
                selectinload(LapMetricsDB.braking_zones),
                selectinload(LapMetricsDB.corners),
            )
        )
        stored_metrics = result.scalar_one_or_none()
        assert stored_metrics is not None
        assert stored_metrics.total_corners == 1
        assert stored_metrics.total_braking_zones == 2
        assert len(stored_metrics.braking_zones) == 2
        assert len(stored_metrics.corners) == 1

    async def test_get_lap_metrics_success(
        self,
        test_client: AsyncClient,
        db_session: AsyncSession,
        track_session_factory: TrackSessionFactory,
        lap_metrics_db_factory: LapMetricsDBFactory,
        braking_metrics_db_factory: BrakingMetricsDBFactory,
        corner_metrics_db_factory: CornerMetricsDBFactory,
    ) -> None:
        """Test successful retrieval of lap metrics."""
        # Create test data
        track_session = track_session_factory.build()
        db_session.add(track_session)

        lap = Lap(
            id=uuid4(),
            track_session_id=track_session.id,
            lap_number=1,
            lap_time=90.5,
            is_valid=True,
        )
        db_session.add(lap)
        await db_session.flush()

        # Create metrics
        metrics = lap_metrics_db_factory.build(
            lap_id=lap.id,
            lap_time=90.5,
            total_corners=2,
            total_braking_zones=2,
        )
        db_session.add(metrics)
        await db_session.flush()

        # Add braking zones
        braking1 = braking_metrics_db_factory.build(lap_metrics_id=metrics.id, zone_number=1)
        braking2 = braking_metrics_db_factory.build(lap_metrics_id=metrics.id, zone_number=2)
        db_session.add_all([braking1, braking2])

        # Add corners
        corner1 = corner_metrics_db_factory.build(lap_metrics_id=metrics.id, corner_number=1)
        corner2 = corner_metrics_db_factory.build(lap_metrics_id=metrics.id, corner_number=2)
        db_session.add_all([corner1, corner2])

        await db_session.commit()

        # Retrieve metrics
        response = await test_client.get(f"/api/v1/lap_metrics/lap/{lap.id}")

        # Assert
        assert response.status_code == 200
        data = response.json()
        assert data["lap_id"] == str(lap.id)
        assert data["lap_time"] == 90.5
        assert data["total_corners"] == 2
        assert data["total_braking_zones"] == 2
        assert len(data["braking_zones"]) == 2
        assert len(data["corners"]) == 2

    async def test_get_lap_metrics_not_found(
        self,
        test_client: AsyncClient,
    ) -> None:
        """Test retrieval of metrics for non-existent lap."""
        fake_lap_id = uuid4()
        response = await test_client.get(f"/api/v1/lap_metrics/lap/{fake_lap_id}")

        assert response.status_code == 404
        assert "not found" in response.json()["detail"].lower()

    async def test_upload_metrics_for_non_existent_lap(
        self,
        test_client: AsyncClient,
    ) -> None:
        """Test uploading metrics for a lap that doesn't exist."""
        fake_lap_id = uuid4()

        response = await test_client.post(
            "/api/v1/lap_metrics/lap",
            json={
                "lap_id": str(fake_lap_id),
                "lap_metrics": {
                    "lap_number": 1,
                    "lap_time": 90.0,
                    "braking_zones": [],
                    "corners": [],
                    "total_corners": 0,
                    "total_braking_zones": 0,
                    "average_corner_speed": 0.0,
                    "max_speed": 100.0,
                    "min_speed": 50.0,
                },
            },
        )

        assert response.status_code == 404
        assert "not found" in response.json()["detail"].lower()

    async def test_upload_metrics_upsert_pattern(
        self,
        test_client: AsyncClient,
        db_session: AsyncSession,
        track_session_factory: TrackSessionFactory,
    ) -> None:
        """Test that uploading metrics twice uses upsert (replace old metrics)."""
        # Create a track session and lap
        track_session = track_session_factory.build()
        db_session.add(track_session)

        lap = Lap(
            id=uuid4(),
            track_session_id=track_session.id,
            lap_number=1,
            lap_time=90.5,
            is_valid=True,
        )
        db_session.add(lap)
        await db_session.commit()

        # Upload metrics first time
        metrics_v1: dict[str, Any] = {
            "lap_id": str(lap.id),
            "lap_metrics": {
                "lap_number": 1,
                "lap_time": 90.5,
                "braking_zones": [],
                "corners": [],
                "total_corners": 0,
                "total_braking_zones": 0,
                "average_corner_speed": 40.0,
                "max_speed": 80.0,
                "min_speed": 40.0,
            },
        }
        response1 = await test_client.post("/api/v1/lap_metrics/lap", json=metrics_v1)
        assert response1.status_code == 200

        # Upload metrics second time with different values
        metrics_v2: dict[str, Any] = {
            "lap_id": str(lap.id),
            "lap_metrics": {
                "lap_number": 1,
                "lap_time": 90.5,
                "braking_zones": [],
                "corners": [],
                "total_corners": 5,  # Changed
                "total_braking_zones": 3,  # Changed
                "average_corner_speed": 45.0,  # Changed
                "max_speed": 85.0,  # Changed
                "min_speed": 35.0,  # Changed
            },
        }
        response2 = await test_client.post("/api/v1/lap_metrics/lap", json=metrics_v2)
        assert response2.status_code == 200

        # Verify only one metrics record exists with updated values
        result = await db_session.execute(select(LapMetricsDB).where(LapMetricsDB.lap_id == lap.id))
        all_metrics = result.scalars().all()
        assert len(all_metrics) == 1

        stored_metrics = all_metrics[0]
        assert stored_metrics.total_corners == 5
        assert stored_metrics.total_braking_zones == 3
        assert stored_metrics.average_corner_speed == 45.0

    async def test_compare_laps_success(
        self,
        test_client: AsyncClient,
        db_session: AsyncSession,
        track_session_factory: TrackSessionFactory,
        lap_metrics_db_factory: LapMetricsDBFactory,
        braking_metrics_db_factory: BrakingMetricsDBFactory,
        corner_metrics_db_factory: CornerMetricsDBFactory,
    ) -> None:
        """Test successful comparison of two laps."""
        # Create track session
        track_session = track_session_factory.build()
        db_session.add(track_session)

        # Create two laps
        lap1 = Lap(
            id=uuid4(),
            track_session_id=track_session.id,
            lap_number=1,
            lap_time=92.0,
            is_valid=True,
        )
        lap2 = Lap(
            id=uuid4(),
            track_session_id=track_session.id,
            lap_number=2,
            lap_time=90.0,
            is_valid=True,
        )
        db_session.add_all([lap1, lap2])
        await db_session.flush()

        # Create metrics for lap 1 (baseline)
        metrics1 = lap_metrics_db_factory.build(
            lap_id=lap1.id,
            lap_time=92.0,
            max_speed=95.0,
            average_corner_speed=45.0,
        )
        db_session.add(metrics1)
        await db_session.flush()

        braking1 = braking_metrics_db_factory.build(
            lap_metrics_id=metrics1.id,
            zone_number=1,
            braking_point_distance=0.25,
            braking_point_speed=70.0,
        )
        corner1 = corner_metrics_db_factory.build(
            lap_metrics_id=metrics1.id,
            corner_number=1,
            apex_distance=0.30,
            apex_speed=45.0,
        )
        db_session.add_all([braking1, corner1])

        # Create metrics for lap 2 (comparison - faster)
        metrics2 = lap_metrics_db_factory.build(
            lap_id=lap2.id,
            lap_time=90.0,
            max_speed=98.0,
            average_corner_speed=48.0,
        )
        db_session.add(metrics2)
        await db_session.flush()

        braking2 = braking_metrics_db_factory.build(
            lap_metrics_id=metrics2.id,
            zone_number=1,
            braking_point_distance=0.25,
            braking_point_speed=75.0,
        )
        corner2 = corner_metrics_db_factory.build(
            lap_metrics_id=metrics2.id,
            corner_number=1,
            apex_distance=0.30,
            apex_speed=48.0,
        )
        db_session.add_all([braking2, corner2])

        await db_session.commit()

        # Compare laps
        response = await test_client.get(
            f"/api/v1/lap_comparison?lap_id_1={lap1.id}&lap_id_2={lap2.id}"
        )

        # Assert
        assert response.status_code == 200
        data = response.json()

        # Check summary
        assert data["summary"]["baseline_lap_id"] == str(lap1.id)
        assert data["summary"]["comparison_lap_id"] == str(lap2.id)
        assert data["summary"]["lap_time_delta"] == -2.0  # Lap 2 is 2s faster
        assert data["summary"]["max_speed_delta"] == 3.0
        assert data["summary"]["matched_braking_zones"] == 1
        assert data["summary"]["matched_corners"] == 1

        # Check braking zone comparison
        assert len(data["braking_zone_comparisons"]) == 1
        assert data["braking_zone_comparisons"][0]["braking_point_speed_delta"] == 5.0

        # Check corner comparison
        assert len(data["corner_comparisons"]) == 1
        assert data["corner_comparisons"][0]["apex_speed_delta"] == 3.0

    async def test_compare_laps_not_found(
        self,
        test_client: AsyncClient,
    ) -> None:
        """Test comparison with non-existent lap."""
        fake_lap_id_1 = uuid4()
        fake_lap_id_2 = uuid4()

        response = await test_client.get(
            f"/api/v1/lap_comparison?lap_id_1={fake_lap_id_1}&lap_id_2={fake_lap_id_2}"
        )

        assert response.status_code == 404
