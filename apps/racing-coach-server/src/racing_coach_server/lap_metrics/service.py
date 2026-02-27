"""Service for lap metrics management."""

import logging
from uuid import UUID

from racing_coach_core.algs.events import LapMetrics as LapMetricsDataclass
from sqlalchemy import delete, select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from racing_coach_server.lap_metrics.models import BrakingMetricsDB, CornerMetricsDB, LapMetricsDB
from racing_coach_server.track_sessions.laps.exceptions import LapNotFoundError
from racing_coach_server.track_sessions.laps.models import Lap

logger = logging.getLogger(__name__)


class LapMetricsService:
    """Service for lap metrics operations."""

    def __init__(self, db: AsyncSession) -> None:
        self.db = db

    async def add_or_update_lap_metrics(
        self,
        lap_metrics: LapMetricsDataclass,
        lap_id: UUID,
    ) -> LapMetricsDB:
        """
        Add or update metrics for a lap (upsert pattern).

        If metrics already exist for this lap, they are deleted and replaced.

        Raises:
            LapNotFoundError: If the lap does not exist
        """
        # Verify lap exists
        stmt = select(Lap).where(Lap.id == lap_id)
        result = await self.db.execute(stmt)
        lap = result.scalar_one_or_none()

        if not lap:
            raise LapNotFoundError(str(lap_id))

        # Delete existing metrics if they exist (upsert pattern)
        delete_stmt = delete(LapMetricsDB).where(LapMetricsDB.lap_id == lap_id)
        await self.db.execute(delete_stmt)
        await self.db.flush()

        # Create new metrics record
        db_lap_metrics = LapMetricsDB(
            lap_id=lap_id,
            lap_time=lap_metrics.lap_time,
            total_corners=lap_metrics.total_corners,
            total_braking_zones=lap_metrics.total_braking_zones,
            average_corner_speed=lap_metrics.average_corner_speed,
            max_speed=lap_metrics.max_speed,
            min_speed=lap_metrics.min_speed,
        )
        self.db.add(db_lap_metrics)
        await self.db.flush()

        # Create braking zone records
        braking_zones: list[BrakingMetricsDB] = []
        for i, braking in enumerate(lap_metrics.braking_zones, start=1):
            db_braking = BrakingMetricsDB(
                lap_metrics_id=db_lap_metrics.id,
                zone_number=i,
                braking_point_distance=braking.braking_point_distance,
                braking_point_speed=braking.braking_point_speed,
                end_distance=braking.end_distance,
                max_brake_pressure=braking.max_brake_pressure,
                braking_duration=braking.braking_duration,
                minimum_speed=braking.minimum_speed,
                initial_deceleration=braking.initial_deceleration,
                average_deceleration=braking.average_deceleration,
                braking_efficiency=braking.braking_efficiency,
                has_trail_braking=braking.has_trail_braking,
                trail_brake_distance=braking.trail_brake_distance,
                trail_brake_percentage=braking.trail_brake_percentage,
            )
            braking_zones.append(db_braking)

        # Create corner records
        corners: list[CornerMetricsDB] = []
        for i, corner in enumerate(lap_metrics.corners, start=1):
            db_corner = CornerMetricsDB(
                lap_metrics_id=db_lap_metrics.id,
                corner_number=i,
                turn_in_distance=corner.turn_in_distance,
                apex_distance=corner.apex_distance,
                exit_distance=corner.exit_distance,
                throttle_application_distance=corner.throttle_application_distance,
                turn_in_speed=corner.turn_in_speed,
                apex_speed=corner.apex_speed,
                exit_speed=corner.exit_speed,
                throttle_application_speed=corner.throttle_application_speed,
                max_lateral_g=corner.max_lateral_g,
                time_in_corner=corner.time_in_corner,
                corner_distance=corner.corner_distance,
                max_steering_angle=corner.max_steering_angle,
                speed_loss=corner.speed_loss,
                speed_gain=corner.speed_gain,
            )
            corners.append(db_corner)

        # Batch insert
        self.db.add_all(braking_zones)
        self.db.add_all(corners)

        logger.info(
            f"Added/updated metrics for lap {lap_id}: "
            f"{len(braking_zones)} braking zones, {len(corners)} corners"
        )

        return db_lap_metrics

    async def get_lap_metrics(self, lap_id: UUID) -> LapMetricsDB | None:
        """Get metrics for a specific lap with relationships eagerly loaded."""
        stmt = (
            select(LapMetricsDB)
            .where(LapMetricsDB.lap_id == lap_id)
            .options(
                selectinload(LapMetricsDB.braking_zones),
                selectinload(LapMetricsDB.corners),
            )
        )
        result = await self.db.execute(stmt)
        metrics = result.scalar_one_or_none()

        if metrics:
            logger.debug(f"Found metrics for lap {lap_id}")
        else:
            logger.debug(f"No metrics found for lap {lap_id}")

        return metrics
