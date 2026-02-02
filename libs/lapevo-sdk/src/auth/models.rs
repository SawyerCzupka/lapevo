//! Authentication models for the device authorization flow.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Request to initiate device authorization flow.
#[derive(Debug, Clone, Serialize)]
pub struct DeviceAuthorizationRequest {
    pub device_name: String,
}

/// Response from device authorization initiation.
#[derive(Debug, Clone, Deserialize)]
pub struct DeviceAuthorizationResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    /// Seconds until the device code expires.
    pub expires_in: u64,
    /// Recommended polling interval in seconds.
    pub interval: u64,
}

/// Request to poll for device token.
#[derive(Debug, Clone, Serialize)]
pub struct DeviceTokenRequest {
    pub device_code: String,
}

/// Successful device token response.
#[derive(Debug, Clone, Deserialize)]
pub struct DeviceTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub device_name: String,
}

/// Device authorization error codes (RFC 8628).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceAuthErrorCode {
    /// User has not yet authorized the device.
    AuthorizationPending,
    /// User denied the authorization request.
    AccessDenied,
    /// Device code has expired.
    ExpiredToken,
    /// Client is polling too frequently.
    SlowDown,
    /// Invalid device code.
    InvalidGrant,
    /// Unknown error code.
    Unknown(String),
}

impl DeviceAuthErrorCode {
    /// Parse error code from server response string.
    pub fn from_str(s: &str) -> Self {
        match s {
            "authorization_pending" => Self::AuthorizationPending,
            "access_denied" => Self::AccessDenied,
            "expired_token" => Self::ExpiredToken,
            "slow_down" => Self::SlowDown,
            "invalid_grant" => Self::InvalidGrant,
            other => Self::Unknown(other.to_string()),
        }
    }
}

impl std::fmt::Display for DeviceAuthErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AuthorizationPending => write!(f, "authorization_pending"),
            Self::AccessDenied => write!(f, "access_denied"),
            Self::ExpiredToken => write!(f, "expired_token"),
            Self::SlowDown => write!(f, "slow_down"),
            Self::InvalidGrant => write!(f, "invalid_grant"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

/// Stored credentials structure for file persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCredentials {
    pub access_token: String,
    pub device_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}
