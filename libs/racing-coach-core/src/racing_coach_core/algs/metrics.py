"""Comprehensive metrics extraction for racing telemetry data."""

import logging
from dataclasses import dataclass
from enum import Enum

from racing_coach_core.schemas.telemetry import TelemetryFrame, TelemetrySequence
from racing_coach_core.utils.track import normalize_lap_distance_delta

from .events import (
    BrakingMetrics,
    CornerMetrics,
    CornerSegmentInput,
    LapMetrics,
    TrailBrakingInfo,
)

logger = logging.getLogger(__name__)


class CornerDetectionMode(Enum):
    """Controls how corners are detected during metric extraction."""

    AUTO = "auto"  # Auto-detect from steering threshold (current behavior)
    SEGMENTS = "segments"  # Use provided segments only, return empty if none
    SEGMENTS_WITH_FALLBACK = "segments_with_fallback"  # Use segments if available, else auto


# Thresholds for detection
BRAKE_THRESHOLD = 0.05  # 5% brake application
STEERING_THRESHOLD = 0.15  # 15% steering angle (absolute value)
THROTTLE_THRESHOLD = 0.05  # 5% throttle application


@dataclass
class FrameRangeStats:
    """Statistics computed over a range of telemetry frames."""

    min_speed: float
    max_lateral_g: float
    max_steering_angle: float
    throttle_application_idx: int


def _compute_frame_range_stats(
    frames: list[TelemetryFrame],
    start_idx: int,
    end_idx: int,
    throttle_threshold: float,
) -> FrameRangeStats:
    """Compute corner statistics in a single pass over the frame range."""
    min_speed = float("inf")
    max_lateral_g = 0.0
    max_steering = 0.0
    throttle_idx = start_idx
    throttle_found = False

    for i in range(start_idx, end_idx + 1):
        frame = frames[i]
        min_speed = min(min_speed, frame.speed)
        max_lateral_g = max(max_lateral_g, abs(frame.lateral_acceleration))
        max_steering = max(max_steering, abs(frame.steering_angle))
        if not throttle_found and frame.throttle > throttle_threshold:
            throttle_idx = i
            throttle_found = True

    return FrameRangeStats(min_speed, max_lateral_g, max_steering, throttle_idx)


def _find_corner_exit(
    frames: list[TelemetryFrame],
    start_idx: int,
    steering_threshold: float,
) -> int:
    """Find the exit index of a corner starting at start_idx."""
    for i in range(start_idx, len(frames)):
        if abs(frames[i].steering_angle) < steering_threshold:
            return i
    return len(frames) - 1


def _find_max_lateral_g_idx(
    frames: list[TelemetryFrame],
    start_idx: int,
    end_idx: int,
) -> int:
    """Find index of maximum lateral G in range."""
    return max(range(start_idx, end_idx + 1), key=lambda i: abs(frames[i].lateral_acceleration))


def extract_lap_metrics(
    sequence: TelemetrySequence,
    lap_number: int | None = None,
    brake_threshold: float = BRAKE_THRESHOLD,
    steering_threshold: float = STEERING_THRESHOLD,
    throttle_threshold: float = THROTTLE_THRESHOLD,
    corner_segments: list[CornerSegmentInput] | None = None,
    lateral_positions: list[float] | None = None,
    corner_mode: CornerDetectionMode = CornerDetectionMode.SEGMENTS_WITH_FALLBACK,
) -> LapMetrics:
    """
    Extract comprehensive metrics from a lap's telemetry sequence.

    Args:
        sequence: The telemetry sequence for the lap
        lap_number: The lap number (optional, extracted from telemetry if not provided)
        brake_threshold: Minimum brake application to consider as braking
        steering_threshold: Minimum steering angle to consider as turning
        throttle_threshold: Minimum throttle application to consider as acceleration
        corner_segments: Optional predefined corner boundaries for segment-based extraction
        lateral_positions: Optional lateral positions for apex detection (-1=left, +1=right)
        track_length: Track length in meters (required when using corner_segments)
        corner_mode: How to detect corners (AUTO, SEGMENTS, or SEGMENTS_WITH_FALLBACK)

    Returns:
        LapMetrics containing all extracted performance data
    """
    frames = sequence.frames

    if not frames:
        raise ValueError("Cannot extract metrics from empty telemetry sequence")

    # Extract lap number from first frame if not provided
    if lap_number is None:
        lap_number = frames[0].lap_number

    # Get lap time if available (some sequence types have this attribute)
    lap_time: float | None = getattr(sequence, "lap_time", None)

    # Extract braking zones
    braking_zones = _extract_braking_zones(frames, brake_threshold, steering_threshold)

    # Determine corner extraction method based on mode and available data
    can_use_segments = (
        corner_mode != CornerDetectionMode.AUTO
        and corner_segments is not None
        and len(corner_segments) > 0
    )

    if can_use_segments and corner_segments is not None:
        corners = _extract_corners_from_segments(
            frames, corner_segments, lateral_positions, throttle_threshold
        )
        logger.debug(f"Extracted {len(corners)} corners from {len(corner_segments)} segments")
    elif corner_mode == CornerDetectionMode.SEGMENTS:
        # SEGMENTS mode but no segments available - return empty
        corners = []
        logger.debug("SEGMENTS mode with no segments available - returning empty corners list")
    else:
        # AUTO mode or SEGMENTS_WITH_FALLBACK with no segments - use auto-detection
        corners = _extract_corners(frames, steering_threshold, throttle_threshold)

    # Calculate lap-wide statistics
    speeds = [frame.speed for frame in frames]
    corner_speeds = [corner.apex_speed for corner in corners]

    lap_metrics = LapMetrics(
        lap_number=lap_number,
        lap_time=lap_time,
        braking_zones=braking_zones,
        corners=corners,
        total_corners=len(corners),
        total_braking_zones=len(braking_zones),
        average_corner_speed=sum(corner_speeds) / len(corner_speeds) if corner_speeds else 0.0,
        max_speed=max(speeds) if speeds else 0.0,
        min_speed=min(speeds) if speeds else 0.0,
    )

    logger.info(
        f"Extracted metrics for lap {lap_number}: "
        f"{len(braking_zones)} braking zones, {len(corners)} corners"
    )

    return lap_metrics


def _extract_braking_zones(
    frames: list[TelemetryFrame],
    brake_threshold: float,
    steering_threshold: float,
) -> list[BrakingMetrics]:
    """Extract detailed braking metrics from telemetry frames."""
    braking_zones: list[BrakingMetrics] = []

    i = 0
    while i < len(frames):
        # Find start of braking zone
        if frames[i].brake <= brake_threshold:
            i += 1
            continue

        # Braking zone detected
        start_idx = i
        start_frame = frames[start_idx]

        # Track max pressure and minimum speed during braking
        max_pressure = start_frame.brake
        min_speed = start_frame.speed
        end_idx = start_idx

        # Find end of braking zone
        while i < len(frames) and frames[i].brake > brake_threshold:
            max_pressure = max(max_pressure, frames[i].brake)
            min_speed = min(min_speed, frames[i].speed)
            end_idx = i
            i += 1

        end_frame = frames[end_idx]

        # Calculate braking duration
        duration = (end_frame.timestamp - start_frame.timestamp).total_seconds()

        # Calculate deceleration metrics
        initial_decel = _calculate_deceleration(frames, start_idx, min(start_idx + 5, end_idx))
        avg_decel = _calculate_deceleration(frames, start_idx, end_idx)

        # Braking efficiency: deceleration per unit of brake pressure
        efficiency = abs(avg_decel) / max_pressure if max_pressure > 0 else 0.0

        # Detect trail braking (braking while turning)
        trail_braking_info = _detect_trail_braking(
            frames, start_idx, end_idx, steering_threshold, brake_threshold
        )

        braking_metrics = BrakingMetrics(
            braking_point_distance=start_frame.lap_distance,
            braking_point_speed=start_frame.speed,
            end_distance=end_frame.lap_distance,
            max_brake_pressure=max_pressure,
            braking_duration=duration,
            minimum_speed=min_speed,
            initial_deceleration=initial_decel,
            average_deceleration=avg_decel,
            braking_efficiency=efficiency,
            has_trail_braking=trail_braking_info.has_trail_braking,
            trail_brake_distance=trail_braking_info.distance,
            trail_brake_percentage=trail_braking_info.percentage,
        )

        braking_zones.append(braking_metrics)

    return braking_zones


def _extract_corners(
    frames: list[TelemetryFrame],
    steering_threshold: float,
    throttle_threshold: float,
) -> list[CornerMetrics]:
    """Extract detailed corner metrics from telemetry frames using auto-detection."""
    corners: list[CornerMetrics] = []

    i = 0
    while i < len(frames):
        # Find turn-in point (steering angle exceeds threshold)
        if abs(frames[i].steering_angle) <= steering_threshold:
            i += 1
            continue

        # Corner detected - find key points
        turn_in_idx = i
        exit_idx = _find_corner_exit(frames, turn_in_idx, steering_threshold)
        apex_idx = _find_max_lateral_g_idx(frames, turn_in_idx, exit_idx)

        # Get all stats in single pass
        stats = _compute_frame_range_stats(frames, turn_in_idx, exit_idx, throttle_threshold)

        # Get frames at key points
        turn_in_frame = frames[turn_in_idx]
        apex_frame = frames[apex_idx]
        exit_frame = frames[exit_idx]
        throttle_frame = frames[stats.throttle_application_idx]

        # Calculate time and distance
        time_in_corner = (exit_frame.timestamp - turn_in_frame.timestamp).total_seconds()
        corner_distance = normalize_lap_distance_delta(
            exit_frame.lap_distance - turn_in_frame.lap_distance
        )

        # Calculate speed deltas
        speed_loss = turn_in_frame.speed - stats.min_speed
        speed_gain = exit_frame.speed - stats.min_speed

        corner_metrics = CornerMetrics(
            turn_in_distance=turn_in_frame.lap_distance,
            apex_distance=apex_frame.lap_distance,
            exit_distance=exit_frame.lap_distance,
            throttle_application_distance=throttle_frame.lap_distance,
            turn_in_speed=turn_in_frame.speed,
            apex_speed=stats.min_speed,
            exit_speed=exit_frame.speed,
            throttle_application_speed=throttle_frame.speed,
            max_lateral_g=stats.max_lateral_g,
            time_in_corner=time_in_corner,
            corner_distance=corner_distance,
            max_steering_angle=stats.max_steering_angle,
            speed_loss=speed_loss,
            speed_gain=speed_gain,
        )

        corners.append(corner_metrics)
        i = exit_idx + 1

    return corners


def _calculate_deceleration(
    frames: list[TelemetryFrame],
    start_idx: int,
    end_idx: int,
) -> float:
    """
    Calculate average deceleration between two frame indices.

    Returns negative value for deceleration, positive for acceleration.
    """
    if start_idx >= end_idx or end_idx >= len(frames):
        return 0.0

    start_frame = frames[start_idx]
    end_frame = frames[end_idx]

    # Speed delta (m/s)
    speed_delta = end_frame.speed - start_frame.speed

    # Time delta (seconds)
    time_delta = (end_frame.timestamp - start_frame.timestamp).total_seconds()

    if time_delta <= 0:
        return 0.0

    # Acceleration/deceleration in m/s²
    return speed_delta / time_delta


def _detect_trail_braking(
    frames: list[TelemetryFrame],
    brake_start_idx: int,
    brake_end_idx: int,
    steering_threshold: float,
    brake_threshold: float,
) -> TrailBrakingInfo:
    """
    Detect if trail braking was used (braking while turning).

    Returns:
        TrailBrakingInfo with detection results
    """
    trail_brake_distance = 0.0
    trail_brake_pressure_sum = 0.0
    trail_brake_frames = 0
    has_trail_braking = False

    for i in range(brake_start_idx, min(brake_end_idx + 1, len(frames))):
        frame = frames[i]

        # Check if both braking and steering are happening
        if frame.brake > brake_threshold and abs(frame.steering_angle) > steering_threshold:
            has_trail_braking = True
            trail_brake_frames += 1
            trail_brake_pressure_sum += frame.brake

            # Calculate distance
            if i + 1 < len(frames):
                next_frame = frames[i + 1]
                distance_delta = next_frame.lap_distance - frame.lap_distance
                trail_brake_distance += normalize_lap_distance_delta(distance_delta)

    # Calculate average trail brake percentage
    trail_brake_percentage = (
        trail_brake_pressure_sum / trail_brake_frames if trail_brake_frames > 0 else 0.0
    )

    return TrailBrakingInfo(
        has_trail_braking=has_trail_braking,
        distance=trail_brake_distance,
        percentage=trail_brake_percentage,
    )


def _compute_corner_metrics(
    frames: list[TelemetryFrame],
    turn_in_idx: int,
    apex_idx: int,
    exit_idx: int,
    throttle_threshold: float,
) -> CornerMetrics:
    """
    Compute CornerMetrics given key frame indices.

    Shared by both auto-detection and segment-based extraction (DRY).

    Args:
        frames: All telemetry frames for the lap
        turn_in_idx: Frame index where corner entry begins
        apex_idx: Frame index of the apex
        exit_idx: Frame index where corner exit ends
        throttle_threshold: Minimum throttle to consider as application

    Returns:
        CornerMetrics with all computed values
    """
    turn_in_frame = frames[turn_in_idx]
    apex_frame = frames[apex_idx]
    exit_frame = frames[exit_idx]

    # Get all stats in single pass
    stats = _compute_frame_range_stats(frames, turn_in_idx, exit_idx, throttle_threshold)
    throttle_frame = frames[stats.throttle_application_idx]

    # Calculate time and distance
    time_in_corner = (exit_frame.timestamp - turn_in_frame.timestamp).total_seconds()
    corner_distance = normalize_lap_distance_delta(
        exit_frame.lap_distance - turn_in_frame.lap_distance
    )

    # Calculate speed deltas
    speed_loss = turn_in_frame.speed - stats.min_speed
    speed_gain = exit_frame.speed - stats.min_speed

    return CornerMetrics(
        turn_in_distance=turn_in_frame.lap_distance,
        apex_distance=apex_frame.lap_distance,
        exit_distance=exit_frame.lap_distance,
        throttle_application_distance=throttle_frame.lap_distance,
        turn_in_speed=turn_in_frame.speed,
        apex_speed=stats.min_speed,
        exit_speed=exit_frame.speed,
        throttle_application_speed=throttle_frame.speed,
        max_lateral_g=stats.max_lateral_g,
        time_in_corner=time_in_corner,
        corner_distance=corner_distance,
        max_steering_angle=stats.max_steering_angle,
        speed_loss=speed_loss,
        speed_gain=speed_gain,
    )


def _find_apex_in_segment(
    frames: list[TelemetryFrame],
    segment_indices: list[int],
    lateral_positions: list[float] | None,
) -> int:
    """
    Find apex index within a segment.

    Uses lateral position if available (point closest to inside edge),
    otherwise falls back to max lateral G.

    Args:
        frames: All telemetry frames
        segment_indices: Indices of frames within this corner segment
        lateral_positions: Optional lateral positions (-1=left, +1=right)

    Returns:
        Frame index of the apex
    """
    if lateral_positions is None:
        # Fallback: use max lateral G
        return max(segment_indices, key=lambda i: abs(frames[i].lateral_acceleration))

    # Determine corner direction from average steering
    avg_steering = sum(frames[i].steering_angle for i in segment_indices) / len(segment_indices)
    is_left_corner = avg_steering < 0

    # Apex = point closest to inside edge
    # Left corner: inside edge is left boundary (lateral_position = -1)
    # Right corner: inside edge is right boundary (lateral_position = +1)
    if is_left_corner:
        return min(segment_indices, key=lambda i: lateral_positions[i])
    else:
        return max(segment_indices, key=lambda i: lateral_positions[i])


def _extract_corners_from_segments(
    frames: list[TelemetryFrame],
    segments: list[CornerSegmentInput],
    lateral_positions: list[float] | None,
    throttle_threshold: float,
) -> list[CornerMetrics]:
    """
    Extract corners using predefined segment boundaries.

    Args:
        frames: All telemetry frames for the lap
        segments: Corner segment definitions with start/end distances in meters
        track_length: Total track length in meters (for distance conversion)
        lateral_positions: Optional lateral positions for apex detection
        throttle_threshold: Minimum throttle to consider as application

    Returns:
        List of CornerMetrics for each segment with sufficient data
    """
    corners: list[CornerMetrics] = []

    for segment in sorted(segments, key=lambda s: s.corner_number):
        # Find frame indices within segment
        segment_indices = [
            i
            for i, f in enumerate(frames)
            if segment.start_distance <= f.lap_distance <= segment.end_distance
        ]

        if len(segment_indices) < 2:
            logger.warning(
                f"Corner {segment.corner_number}: insufficient frames "
                f"(found {len(segment_indices)}, need at least 2)"
            )
            continue

        turn_in_idx = segment_indices[0]
        exit_idx = segment_indices[-1]
        apex_idx = _find_apex_in_segment(frames, segment_indices, lateral_positions)

        corner = _compute_corner_metrics(
            frames, turn_in_idx, apex_idx, exit_idx, throttle_threshold
        )
        corners.append(corner)

    return corners
