"""SQLAlchemy models for the auth feature."""

import uuid
from datetime import datetime

from sqlalchemy import Boolean, DateTime, ForeignKey, Index, Integer, String, func
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from racing_coach_server.database.base import Base
from racing_coach_server.database.mixins import TimestampMixin


class User(TimestampMixin, Base):
    """Model representing a user account."""

    __tablename__ = "user"

    # Required fields (no defaults) come first for MappedAsDataclass
    email: Mapped[str] = mapped_column(String(255), nullable=False, unique=True)
    password_hash: Mapped[str] = mapped_column(String(255), nullable=False)

    # Optional fields with defaults
    display_name: Mapped[str | None] = mapped_column(String(100), nullable=True, default=None)
    email_verified_at: Mapped[datetime | None] = mapped_column(
        DateTime(timezone=True), nullable=True, default=None
    )
    is_active: Mapped[bool] = mapped_column(Boolean, default=True)
    is_admin: Mapped[bool] = mapped_column(Boolean, default=False)

    # Primary key with default_factory
    id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), primary_key=True, default_factory=uuid.uuid4
    )

    # Relationships
    sessions: Mapped[list["UserSession"]] = relationship(
        "UserSession", back_populates="user", cascade="all, delete-orphan", init=False
    )
    device_tokens: Mapped[list["DeviceToken"]] = relationship(
        "DeviceToken", back_populates="user", cascade="all, delete-orphan", init=False
    )

    __table_args__ = (
        Index("idx_user_email", "email"),
        Index("idx_user_is_active", "is_active"),
        Index("idx_user_is_admin", "is_admin"),
    )


class UserSession(Base):
    """Model representing a web browser session."""

    __tablename__ = "user_session"

    # Required fields (no defaults) come first
    user_id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), ForeignKey("user.id", ondelete="CASCADE"), nullable=False
    )
    token_hash: Mapped[str] = mapped_column(String(64), nullable=False, unique=True)
    expires_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False)

    # Optional fields with defaults
    user_agent: Mapped[str | None] = mapped_column(String(500), nullable=True, default=None)
    ip_address: Mapped[str | None] = mapped_column(
        String(45), nullable=True, default=None
    )  # IPv6 max length
    revoked_at: Mapped[datetime | None] = mapped_column(
        DateTime(timezone=True), nullable=True, default=None
    )

    # Primary key with default_factory
    id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), primary_key=True, default_factory=uuid.uuid4
    )

    # Server-defaulted timestamps
    last_active_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), server_default=func.now(), init=False
    )
    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), server_default=func.now(), init=False
    )

    # Relationships
    user: Mapped["User"] = relationship("User", back_populates="sessions", init=False)

    __table_args__ = (
        Index("idx_session_user_id", "user_id"),
        Index("idx_session_token_hash", "token_hash"),
        Index("idx_session_expires_at", "expires_at"),
    )


class DeviceToken(Base):
    """Model representing a device token for desktop client authentication."""

    __tablename__ = "device_token"

    # Required fields (no defaults) come first
    user_id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), ForeignKey("user.id", ondelete="CASCADE"), nullable=False
    )
    token_hash: Mapped[str] = mapped_column(String(64), nullable=False, unique=True)
    device_name: Mapped[str] = mapped_column(String(100), nullable=False)

    # Optional fields with defaults
    device_id: Mapped[str | None] = mapped_column(
        String(255), nullable=True, default=None
    )  # Machine identifier
    expires_at: Mapped[datetime | None] = mapped_column(
        DateTime(timezone=True), nullable=True, default=None
    )
    revoked_at: Mapped[datetime | None] = mapped_column(
        DateTime(timezone=True), nullable=True, default=None
    )
    last_used_at: Mapped[datetime | None] = mapped_column(
        DateTime(timezone=True), nullable=True, default=None
    )

    # Primary key with default_factory
    id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), primary_key=True, default_factory=uuid.uuid4
    )

    # Server-defaulted timestamp
    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), server_default=func.now(), init=False
    )

    # Relationships
    user: Mapped["User"] = relationship("User", back_populates="device_tokens", init=False)

    __table_args__ = (
        Index("idx_device_token_user_id", "user_id"),
        Index("idx_device_token_hash", "token_hash"),
    )


class DeviceAuthorization(Base):
    """Model for OAuth-style device authorization flow (RFC 8628)."""

    __tablename__ = "device_authorization"

    # Required fields (no defaults) come first
    device_code: Mapped[str] = mapped_column(String(64), nullable=False, unique=True)
    user_code: Mapped[str] = mapped_column(String(8), nullable=False, unique=True)
    device_name: Mapped[str] = mapped_column(String(100), nullable=False)
    expires_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False)

    # Optional/defaulted fields
    user_id: Mapped[uuid.UUID | None] = mapped_column(
        UUID(as_uuid=True), ForeignKey("user.id", ondelete="CASCADE"), nullable=True, default=None
    )
    status: Mapped[str] = mapped_column(
        String(20), nullable=False, default="pending"
    )  # pending, authorized, denied, expired, consumed
    interval: Mapped[int] = mapped_column(Integer, default=5)  # Polling interval in seconds
    authorized_at: Mapped[datetime | None] = mapped_column(
        DateTime(timezone=True), nullable=True, default=None
    )

    # Primary key with default_factory
    id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), primary_key=True, default_factory=uuid.uuid4, init=False
    )

    # Server-defaulted timestamp
    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), server_default=func.now(), init=False
    )

    __table_args__ = (
        Index("idx_device_auth_device_code", "device_code"),
        Index("idx_device_auth_user_code", "user_code"),
        Index("idx_device_auth_status", "status"),
    )
