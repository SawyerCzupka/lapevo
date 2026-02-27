"""Polyfactory factories for creating test data for racing-coach-server tests."""

from datetime import datetime, timedelta, timezone
from typing import Any

from polyfactory.factories.pydantic_factory import ModelFactory
from polyfactory.factories.sqlalchemy_factory import SQLAlchemyFactory
from polyfactory.fields import Use
from racing_coach_core.schemas.telemetry import (
    LapTelemetry,
    SessionFrame,
    TelemetryFrame,
    TelemetrySequence,
)
from racing_coach_server.auth.models import DeviceAuthorization, DeviceToken, User, UserSession
from racing_coach_server.auth.utils import hash_password, hash_token
from racing_coach_server.lap_metrics.models import BrakingMetricsDB, CornerMetricsDB, LapMetricsDB
from racing_coach_server.telemetry.models import Telemetry
from racing_coach_server.track_sessions.laps.models import Lap
from racing_coach_server.track_sessions.models import TrackSession

# ============================================================================
# Pydantic Schema Factories
# ============================================================================


class SessionFrameFactory(ModelFactory[SessionFrame]): ...


class TelemetryFrameFactory(ModelFactory[TelemetryFrame]): ...


class LapTelemetryFactory(ModelFactory[LapTelemetry]): ...


class TelemetrySequenceFactory(ModelFactory[TelemetrySequence]): ...


# ============================================================================
# SQLAlchemy Model Factories - Telemetry
# ============================================================================


class TrackSessionFactory(SQLAlchemyFactory[TrackSession]):
    """Factory for creating TrackSession database model instances."""

    __set_relationships__ = False

    track_id = Use(lambda: __import__("random").randint(1, 500))
    track_name = Use(lambda: __import__("faker").Faker().company())
    track_config_name = Use(lambda: __import__("faker").Faker().word())
    track_type = Use(lambda: "road course")
    car_id = Use(lambda: __import__("random").randint(1, 200))
    car_name = Use(lambda: __import__("faker").Faker().company())
    car_class_id = Use(lambda: __import__("random").randint(1, 50))
    series_id = Use(lambda: __import__("random").randint(1, 100))
    session_type = Use(lambda: __import__("faker").Faker().word())

    @classmethod
    def build(cls, **kwargs: Any) -> TrackSession:
        """Build with post-construction timestamp assignment."""
        instance = super().build(**kwargs)
        now = datetime.now(timezone.utc)
        instance.created_at = now
        instance.updated_at = now
        return instance


class LapFactory(SQLAlchemyFactory[Lap]):
    """Factory for creating Lap database model instances."""

    __set_relationships__ = False

    lap_number = Use(lambda: __import__("random").randint(1, 50))
    lap_time = Use(lambda: __import__("random").uniform(60.0, 180.0))
    is_valid = Use(lambda: True)

    @classmethod
    def build(cls, **kwargs: Any) -> Lap:
        """Build with post-construction timestamp assignment."""
        instance = super().build(**kwargs)
        now = datetime.now(timezone.utc)
        instance.created_at = now
        instance.updated_at = now
        return instance


class TelemetryDBFactory(SQLAlchemyFactory[Telemetry]):
    """Factory for creating Telemetry database model instances."""

    __set_relationships__ = False

    timestamp = Use(lambda: datetime.now(timezone.utc))
    session_time = Use(lambda: __import__("random").uniform(0.0, 3600.0))
    lap_number = Use(lambda: __import__("random").randint(1, 50))
    lap_distance_pct = Use(lambda: __import__("random").uniform(0.0, 1.0))
    lap_distance = Use(lambda: __import__("random").uniform(0.0, 5000.0))
    current_lap_time = Use(lambda: __import__("random").uniform(60.0, 180.0))
    last_lap_time = Use(lambda: __import__("random").uniform(60.0, 180.0))
    best_lap_time = Use(lambda: __import__("random").uniform(60.0, 180.0))
    speed = Use(lambda: __import__("random").uniform(0.0, 100.0))
    rpm = Use(lambda: __import__("random").uniform(1000.0, 8000.0))
    gear = Use(lambda: __import__("random").randint(1, 6))
    throttle = Use(lambda: __import__("random").uniform(0.0, 1.0))
    brake = Use(lambda: __import__("random").uniform(0.0, 1.0))
    clutch = Use(lambda: __import__("random").uniform(0.0, 1.0))
    steering_angle = Use(lambda: __import__("random").uniform(-1.57, 1.57))
    lateral_acceleration = Use(lambda: __import__("random").uniform(-20.0, 20.0))
    longitudinal_acceleration = Use(lambda: __import__("random").uniform(-20.0, 20.0))
    vertical_acceleration = Use(lambda: __import__("random").uniform(-10.0, 10.0))
    yaw_rate = Use(lambda: __import__("random").uniform(-1.0, 1.0))
    roll_rate = Use(lambda: __import__("random").uniform(-1.0, 1.0))
    pitch_rate = Use(lambda: __import__("random").uniform(-1.0, 1.0))
    velocity_x = Use(lambda: __import__("random").uniform(-100.0, 100.0))
    velocity_y = Use(lambda: __import__("random").uniform(-100.0, 100.0))
    velocity_z = Use(lambda: __import__("random").uniform(-100.0, 100.0))
    yaw = Use(lambda: __import__("random").uniform(-3.14, 3.14))
    pitch = Use(lambda: __import__("random").uniform(-3.14, 3.14))
    roll = Use(lambda: __import__("random").uniform(-3.14, 3.14))
    latitude = Use(lambda: __import__("random").uniform(-90.0, 90.0))
    longitude = Use(lambda: __import__("random").uniform(-180.0, 180.0))
    altitude = Use(lambda: __import__("random").uniform(0.0, 3000.0))

    # Tire temperatures
    lf_tire_temp_left = Use(lambda: __import__("random").uniform(70.0, 90.0))
    lf_tire_temp_middle = Use(lambda: __import__("random").uniform(70.0, 90.0))
    lf_tire_temp_right = Use(lambda: __import__("random").uniform(70.0, 90.0))
    rf_tire_temp_left = Use(lambda: __import__("random").uniform(70.0, 90.0))
    rf_tire_temp_middle = Use(lambda: __import__("random").uniform(70.0, 90.0))
    rf_tire_temp_right = Use(lambda: __import__("random").uniform(70.0, 90.0))
    lr_tire_temp_left = Use(lambda: __import__("random").uniform(70.0, 90.0))
    lr_tire_temp_middle = Use(lambda: __import__("random").uniform(70.0, 90.0))
    lr_tire_temp_right = Use(lambda: __import__("random").uniform(70.0, 90.0))
    rr_tire_temp_left = Use(lambda: __import__("random").uniform(70.0, 90.0))
    rr_tire_temp_middle = Use(lambda: __import__("random").uniform(70.0, 90.0))
    rr_tire_temp_right = Use(lambda: __import__("random").uniform(70.0, 90.0))

    # Tire wear
    lf_tire_wear_left = Use(lambda: __import__("random").uniform(0.9, 1.0))
    lf_tire_wear_middle = Use(lambda: __import__("random").uniform(0.9, 1.0))
    lf_tire_wear_right = Use(lambda: __import__("random").uniform(0.9, 1.0))
    rf_tire_wear_left = Use(lambda: __import__("random").uniform(0.9, 1.0))
    rf_tire_wear_middle = Use(lambda: __import__("random").uniform(0.9, 1.0))
    rf_tire_wear_right = Use(lambda: __import__("random").uniform(0.9, 1.0))
    lr_tire_wear_left = Use(lambda: __import__("random").uniform(0.9, 1.0))
    lr_tire_wear_middle = Use(lambda: __import__("random").uniform(0.9, 1.0))
    lr_tire_wear_right = Use(lambda: __import__("random").uniform(0.9, 1.0))
    rr_tire_wear_left = Use(lambda: __import__("random").uniform(0.9, 1.0))
    rr_tire_wear_middle = Use(lambda: __import__("random").uniform(0.9, 1.0))
    rr_tire_wear_right = Use(lambda: __import__("random").uniform(0.9, 1.0))

    # Brake pressure
    lf_brake_pressure = Use(lambda: __import__("random").uniform(1.5, 3.5))
    rf_brake_pressure = Use(lambda: __import__("random").uniform(1.5, 3.5))
    lr_brake_pressure = Use(lambda: __import__("random").uniform(1.5, 3.5))
    rr_brake_pressure = Use(lambda: __import__("random").uniform(1.5, 3.5))

    # Track conditions
    track_temp = Use(lambda: __import__("random").uniform(15.0, 50.0))
    track_wetness = Use(lambda: __import__("random").randint(0, 3))
    air_temp = Use(lambda: __import__("random").uniform(10.0, 40.0))

    # Session state
    session_flags = Use(lambda: __import__("random").randint(0, 65535))
    track_surface = Use(lambda: __import__("random").randint(0, 3))
    on_pit_road = Use(lambda: False)


# ============================================================================
# SQLAlchemy Model Factories - Metrics
# ============================================================================


class LapMetricsDBFactory(SQLAlchemyFactory[LapMetricsDB]):
    """Factory for creating LapMetricsDB database model instances."""

    __set_relationships__ = False

    lap_time = Use(lambda: __import__("random").uniform(60.0, 180.0))
    total_corners = Use(lambda: __import__("random").randint(3, 15))
    total_braking_zones = Use(lambda: __import__("random").randint(3, 15))
    average_corner_speed = Use(lambda: __import__("random").uniform(25.0, 45.0))
    max_speed = Use(lambda: __import__("random").uniform(70.0, 100.0))
    min_speed = Use(lambda: __import__("random").uniform(15.0, 30.0))

    @classmethod
    def build(cls, **kwargs: Any) -> LapMetricsDB:
        """Build with post-construction timestamp assignment."""
        instance = super().build(**kwargs)
        now = datetime.now(timezone.utc)
        instance.created_at = now
        instance.updated_at = now
        return instance


class BrakingMetricsDBFactory(SQLAlchemyFactory[BrakingMetricsDB]):
    """Factory for creating BrakingMetricsDB database model instances."""

    __set_relationships__ = False

    zone_number = Use(lambda: __import__("random").randint(1, 10))
    braking_point_distance = Use(lambda: __import__("random").uniform(0.0, 1.0))
    braking_point_speed = Use(lambda: __import__("random").uniform(30.0, 80.0))
    end_distance = Use(lambda: __import__("random").uniform(0.0, 1.0))
    max_brake_pressure = Use(lambda: __import__("random").uniform(0.5, 1.0))
    braking_duration = Use(lambda: __import__("random").uniform(0.5, 3.0))
    minimum_speed = Use(lambda: __import__("random").uniform(10.0, 50.0))
    initial_deceleration = Use(lambda: __import__("random").uniform(-15.0, -5.0))
    average_deceleration = Use(lambda: __import__("random").uniform(-12.0, -4.0))
    braking_efficiency = Use(lambda: __import__("random").uniform(5.0, 15.0))
    has_trail_braking = Use(lambda: __import__("random").choice([True, False]))
    trail_brake_distance = Use(lambda: __import__("random").uniform(0.0, 0.05))
    trail_brake_percentage = Use(lambda: __import__("random").uniform(0.0, 0.8))


class CornerMetricsDBFactory(SQLAlchemyFactory[CornerMetricsDB]):
    """Factory for creating CornerMetricsDB database model instances."""

    __set_relationships__ = False

    corner_number = Use(lambda: __import__("random").randint(1, 12))
    turn_in_distance = Use(lambda: __import__("random").uniform(0.0, 1.0))
    apex_distance = Use(lambda: __import__("random").uniform(0.0, 1.0))
    exit_distance = Use(lambda: __import__("random").uniform(0.0, 1.0))
    throttle_application_distance = Use(lambda: __import__("random").uniform(0.0, 1.0))
    turn_in_speed = Use(lambda: __import__("random").uniform(20.0, 60.0))
    apex_speed = Use(lambda: __import__("random").uniform(15.0, 50.0))
    exit_speed = Use(lambda: __import__("random").uniform(20.0, 70.0))
    throttle_application_speed = Use(lambda: __import__("random").uniform(15.0, 55.0))
    max_lateral_g = Use(lambda: __import__("random").uniform(0.5, 3.0))
    time_in_corner = Use(lambda: __import__("random").uniform(1.0, 5.0))
    corner_distance = Use(lambda: __import__("random").uniform(0.02, 0.15))
    max_steering_angle = Use(lambda: __import__("random").uniform(0.2, 1.5))
    speed_loss = Use(lambda: __import__("random").uniform(5.0, 30.0))
    speed_gain = Use(lambda: __import__("random").uniform(5.0, 40.0))


# ============================================================================
# SQLAlchemy Model Factories - Auth
# ============================================================================


class UserFactory(SQLAlchemyFactory[User]):
    """Factory for creating User database model instances."""

    __set_relationships__ = False

    email = Use(lambda: __import__("faker").Faker().email())
    password_hash = Use(lambda: hash_password("testpassword123"))
    display_name = Use(lambda: __import__("faker").Faker().name())
    is_active = Use(lambda: True)

    @classmethod
    def build(cls, **kwargs: Any) -> User:
        """Build with post-construction timestamp assignment."""
        instance = super().build(**kwargs)
        now = datetime.now(timezone.utc)
        instance.created_at = now
        instance.updated_at = now
        return instance


class UserSessionFactory(SQLAlchemyFactory[UserSession]):
    """Factory for creating UserSession database model instances."""

    __set_relationships__ = False

    token_hash = Use(lambda: hash_token("test_session_token"))
    expires_at = Use(lambda: datetime.now(timezone.utc) + timedelta(days=30))

    @classmethod
    def build(cls, **kwargs: Any) -> UserSession:
        """Build with post-construction timestamp assignment."""
        instance = super().build(**kwargs)
        now = datetime.now(timezone.utc)
        instance.created_at = now
        instance.last_active_at = now
        return instance


class DeviceTokenFactory(SQLAlchemyFactory[DeviceToken]):
    """Factory for creating DeviceToken database model instances."""

    __set_relationships__ = False

    token_hash = Use(lambda: hash_token("test_device_token"))
    device_name = Use(lambda: __import__("faker").Faker().word())

    @classmethod
    def build(cls, **kwargs: Any) -> DeviceToken:
        """Build with post-construction timestamp assignment."""
        instance = super().build(**kwargs)
        instance.created_at = datetime.now(timezone.utc)
        return instance


class DeviceAuthorizationFactory(SQLAlchemyFactory[DeviceAuthorization]):
    """Factory for creating DeviceAuthorization database model instances."""

    __set_relationships__ = False

    device_code = Use(lambda: __import__("secrets").token_urlsafe(32))
    user_code = Use(lambda: __import__("faker").Faker().pystr(min_chars=8, max_chars=8))
    device_name = Use(lambda: __import__("faker").Faker().word())
    expires_at = Use(lambda: datetime.now(timezone.utc) + timedelta(minutes=15))
    status = Use(lambda: "pending")

    @classmethod
    def build(cls, **kwargs: Any) -> DeviceAuthorization:
        """Build with post-construction timestamp assignment."""
        instance = super().build(**kwargs)
        instance.created_at = datetime.now(timezone.utc)
        return instance
