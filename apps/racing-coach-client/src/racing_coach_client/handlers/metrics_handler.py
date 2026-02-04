"""Metrics extraction handler for racing coach client.

This handler listens to LAP_TELEMETRY_SEQUENCE events and extracts comprehensive
performance metrics including braking points, corner analysis, and lap statistics.
"""

from __future__ import annotations

import logging
import time
from typing import TYPE_CHECKING

from racing_coach_core.algs.boundary import compute_lateral_positions
from racing_coach_core.algs.metrics import CornerDetectionMode, extract_lap_metrics
from racing_coach_core.events import (
    Event,
    EventBus,
    HandlerContext,
    SystemEvents,
)
from racing_coach_core.events.checking import method_handles
from racing_coach_core.schemas.events import LapAndSession, MetricsAndSession

if TYPE_CHECKING:
    from racing_coach_client.services.track_service import TrackService

logger = logging.getLogger(__name__)


class MetricsHandler:
    """Handler for extracting performance metrics from lap telemetry."""

    def __init__(
        self,
        event_bus: EventBus,
        track_service: TrackService | None = None,
        corner_mode: CornerDetectionMode = CornerDetectionMode.SEGMENTS_WITH_FALLBACK,
    ):
        """Initialize the metrics handler.

        Args:
            event_bus: The event bus for publishing extracted metrics
            track_service: Optional service for fetching corner segments and track boundaries.
                If provided, enables segment-based corner extraction.
            corner_mode: How to detect corners (AUTO, SEGMENTS, or SEGMENTS_WITH_FALLBACK).
                Only used when track_service is provided.
        """
        self.event_bus = event_bus
        self.track_service = track_service
        self.corner_mode = corner_mode

    @method_handles(SystemEvents.LAP_TELEMETRY_SEQUENCE)
    def handle_lap_telemetry(self, context: HandlerContext[LapAndSession]):
        """Extract metrics from completed lap telemetry.

        Args:
            context: Handler context containing the lap telemetry event
        """
        data = context.event.data
        lap_telemetry = data.LapTelemetry
        session_frame = data.SessionFrame

        try:
            # Fetch track data if service available
            corner_segments = None
            lateral_positions = None

            if self.track_service:
                track_id = session_frame.track_id
                track_config = session_frame.track_config_name

                corner_segments = self.track_service.get_corner_segments(track_id, track_config)

                if corner_segments:
                    # Fetch track boundary for lateral position computation
                    boundary = self.track_service.get_track_boundary(track_id, track_config)
                    if boundary:
                        augmented = compute_lateral_positions(boundary, lap_telemetry)
                        lateral_positions = augmented.lateral_positions
                        logger.debug(
                            f"Using {len(corner_segments)} corner segments with lateral positions"
                        )
                    else:
                        # Have segments but no boundary - can still use segments with lateral G apex
                        logger.debug(
                            f"Have {len(corner_segments)} segments but no boundary - "
                            "will use lateral G for apex detection"
                        )

            # Extract comprehensive metrics from the lap
            start_time = time.time()
            lap_metrics = extract_lap_metrics(
                sequence=lap_telemetry,
                lap_number=lap_telemetry.frames[0].lap_number if lap_telemetry.frames else None,
                corner_segments=corner_segments,
                lateral_positions=lateral_positions,
                corner_mode=self.corner_mode,
            )
            extraction_time = time.time() - start_time

            logger.info(
                f"Extracted metrics for lap {lap_metrics.lap_number}: "
                f"{lap_metrics.total_braking_zones} braking zones, "
                f"{lap_metrics.total_corners} corners, "
                f"avg corner speed: {lap_metrics.average_corner_speed:.1f} m/s "
                f"({extraction_time * 1000:.1f}ms)"
            )

            # Log detailed braking metrics
            for i, braking in enumerate(lap_metrics.braking_zones):
                logger.debug(
                    f"  Braking zone {i + 1}: "
                    f"distance={braking.braking_point_distance:.3f}, "
                    f"speed={braking.braking_point_speed:.1f} m/s, "
                    f"max_pressure={braking.max_brake_pressure:.2f}, "
                    f"trail_braking={braking.has_trail_braking}"
                )

            # Log detailed corner metrics
            for i, corner in enumerate(lap_metrics.corners):
                logger.debug(
                    f"  Corner {i + 1}: "
                    f"turn_in={corner.turn_in_distance:.3f}, "
                    f"apex={corner.apex_distance:.3f}, "
                    f"exit={corner.exit_distance:.3f}, "
                    f"apex_speed={corner.apex_speed:.1f} m/s, "
                    f"max_lateral_g={corner.max_lateral_g:.2f}"
                )

            # Publish the extracted metrics with lap_id passed through from input event
            self.event_bus.thread_safe_publish(
                Event(
                    type=SystemEvents.LAP_METRICS_EXTRACTED,
                    data=MetricsAndSession(
                        LapMetrics=lap_metrics,
                        SessionFrame=session_frame,
                        lap_id=data.lap_id,
                    ),
                )
            )

            logger.info(f"Published metrics for lap {lap_metrics.lap_number}")

        except Exception as e:
            logger.error(f"Failed to extract metrics from lap telemetry: {e}", exc_info=True)
