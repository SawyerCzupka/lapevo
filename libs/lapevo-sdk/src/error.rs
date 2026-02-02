//! Error types for the Racing Coach API client.

use crate::auth::DeviceAuthErrorCode;
use thiserror::Error;

/// Result type alias for API operations.
pub type ApiResult<T> = Result<T, ApiError>;

/// Errors that can occur during API operations.
#[derive(Debug, Error)]
pub enum ApiError {
    /// HTTP request failed (network error, timeout, etc.)
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    /// Server returned an error response.
    #[error("Server error (status {status}): {message}")]
    ServerError {
        /// HTTP status code.
        status: u16,
        /// Error message from the server.
        message: String,
    },

    /// Failed to serialize request body.
    #[error("Failed to serialize request: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Invalid server URL configuration.
    #[error("Invalid server URL: {0}")]
    InvalidUrl(String),

    /// Server returned 401 Unauthorized.
    #[error("Authentication required")]
    Unauthorized,

    /// Device authorization flow failed.
    #[error("Device authorization failed: {0}")]
    DeviceAuthFailed(DeviceAuthErrorCode),

    /// Device authorization code expired before user authorized.
    #[error("Device authorization expired")]
    DeviceAuthExpired,

    /// User denied the device authorization request.
    #[error("Device authorization denied by user")]
    DeviceAuthDenied,

    /// Credential storage error (file I/O).
    #[error("Credential storage error: {0}")]
    CredentialStorage(String),

    /// Client is not authenticated.
    #[error("Not authenticated - call authenticate() first")]
    NotAuthenticated,
}

impl ApiError {
    /// Check if this error indicates the client needs to re-authenticate.
    pub fn requires_reauth(&self) -> bool {
        matches!(self, ApiError::Unauthorized | ApiError::NotAuthenticated)
    }
}
