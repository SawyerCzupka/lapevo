//! HTTP client for the Racing Coach API.

use std::sync::Arc;

use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

use crate::auth::{
    DeviceAuthErrorCode, DeviceAuthorizationRequest, DeviceAuthorizationResponse,
    DeviceTokenRequest, DeviceTokenResponse, StoredCredentials, delete_credentials,
    load_credentials, save_credentials,
};
use crate::error::{ApiError, ApiResult};
use crate::models::{
    CornerSegmentListResponse, LapMetrics, LapTelemetry, LapUploadRequest, LapUploadResponse,
    MetricsUploadRequest, MetricsUploadResponse, SessionFrame, TrackBoundaryListResponse,
    TrackBoundaryResponse, UserResponse,
};

const DEVICE_TOKEN_HEADER: &str = "X-Device-Token";

/// HTTP client for the Racing Coach API.
///
/// This client is designed to be cloned and shared across tasks.
/// The underlying [`reqwest::Client`] uses `Arc` internally for efficient cloning.
/// Authentication tokens are stored in an `Arc<RwLock>` for thread-safe access.
#[derive(Clone)]
pub struct ServerAPIClient {
    client: Client,
    base_url: String,
    /// Access token for authenticated requests.
    access_token: Arc<RwLock<Option<String>>>,
}

impl ServerAPIClient {
    /// Create a new API client with default configuration.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The server base URL (e.g., "http://localhost:8000")
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidUrl`] if the URL is empty.
    /// Returns [`ApiError::Request`] if the HTTP client fails to build.
    pub fn new(base_url: impl Into<String>) -> ApiResult<Self> {
        let base_url = base_url.into();

        if base_url.is_empty() {
            return Err(ApiError::InvalidUrl("URL cannot be empty".into()));
        }

        // Remove trailing slash for consistent URL building
        let base_url = base_url.trim_end_matches('/').to_string();

        let client = Client::builder().build().map_err(ApiError::Request)?;

        Ok(Self {
            client,
            base_url,
            access_token: Arc::new(RwLock::new(None)),
        })
    }

    /// Create a client from an existing [`reqwest::Client`].
    ///
    /// Use this when you want to share a client across multiple API instances
    /// or when you need custom client configuration (timeouts, TLS, etc.).
    ///
    /// # Arguments
    ///
    /// * `client` - A pre-configured reqwest Client
    /// * `base_url` - The server base URL
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidUrl`] if the URL is empty.
    pub fn with_client(client: Client, base_url: impl Into<String>) -> ApiResult<Self> {
        let base_url = base_url.into();

        if base_url.is_empty() {
            return Err(ApiError::InvalidUrl("URL cannot be empty".into()));
        }

        let base_url = base_url.trim_end_matches('/').to_string();

        Ok(Self {
            client,
            base_url,
            access_token: Arc::new(RwLock::new(None)),
        })
    }

    /// Create a client and load stored credentials if available.
    ///
    /// This is a convenience constructor that automatically loads previously
    /// saved authentication tokens from disk.
    pub async fn new_with_stored_credentials(base_url: impl Into<String>) -> ApiResult<Self> {
        let client = Self::new(base_url)?;
        client.load_stored_token().await?;
        Ok(client)
    }

    /// Check if the client has an authentication token set.
    pub async fn is_authenticated(&self) -> bool {
        self.access_token.read().await.is_some()
    }

    /// Get the current access token (if any).
    pub async fn get_token(&self) -> Option<String> {
        self.access_token.read().await.clone()
    }

    /// Manually set the access token.
    pub async fn set_token(&self, token: Option<String>) {
        *self.access_token.write().await = token;
    }

    /// Load token from stored credentials file.
    ///
    /// Returns `true` if credentials were found and loaded.
    pub async fn load_stored_token(&self) -> ApiResult<bool> {
        match load_credentials()? {
            Some(creds) => {
                self.set_token(Some(creds.access_token)).await;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    /// Clear authentication (remove token from memory and disk).
    pub async fn logout(&self) -> ApiResult<()> {
        self.set_token(None).await;
        delete_credentials()?;
        Ok(())
    }

    /// Run the device authorization flow.
    ///
    /// This is a high-level method that handles the entire OAuth device flow:
    /// 1. Initiates device authorization
    /// 2. Prints the user code and verification URI to stdout
    /// 3. Polls for token until authorized, denied, or expired
    /// 4. Stores the token on success
    ///
    /// # Arguments
    ///
    /// * `device_name` - Human-readable name for this device
    ///
    /// # Example
    ///
    /// ```ignore
    /// client.authenticate("My Racing PC").await?;
    /// ```
    pub async fn authenticate(&self, device_name: impl Into<String>) -> ApiResult<()> {
        // Step 1: Initiate device authorization
        let auth_response = self.initiate_device_auth(device_name.into()).await?;

        // Step 2: Print auth instructions
        println!();
        println!("========================================");
        println!("        AUTHENTICATION REQUIRED");
        println!("========================================");
        println!();
        println!("  1. Open: {}", auth_response.verification_uri);
        println!("  2. Enter code: {}", auth_response.user_code);
        println!();
        println!("Waiting for authorization...");
        println!();

        // Step 3: Poll for token
        let token_response = self
            .poll_for_token(
                &auth_response.device_code,
                auth_response.interval,
                auth_response.expires_in,
            )
            .await?;

        println!("Authorization successful!");
        println!();

        // Step 4: Store token
        let creds = StoredCredentials {
            access_token: token_response.access_token.clone(),
            device_name: token_response.device_name,
            created_at: Some(Utc::now()),
        };
        save_credentials(&creds)?;

        // Step 5: Set token in client
        self.set_token(Some(token_response.access_token)).await;

        Ok(())
    }

    /// Initiate device authorization flow.
    async fn initiate_device_auth(
        &self,
        device_name: String,
    ) -> ApiResult<DeviceAuthorizationResponse> {
        let url = format!("{}/api/v1/auth/device/authorize", self.base_url);
        let request = DeviceAuthorizationRequest { device_name };

        debug!("Initiating device authorization at {}", url);

        let response = self.client.post(&url).json(&request).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(ApiError::ServerError { status, message })
        }
    }

    /// Poll for device token with automatic retry.
    async fn poll_for_token(
        &self,
        device_code: &str,
        interval_secs: u64,
        expires_in_secs: u64,
    ) -> ApiResult<DeviceTokenResponse> {
        let url = format!("{}/api/v1/auth/device/token", self.base_url);
        let request = DeviceTokenRequest {
            device_code: device_code.to_string(),
        };

        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(expires_in_secs);
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));

        loop {
            interval.tick().await;

            if start.elapsed() > timeout {
                return Err(ApiError::DeviceAuthExpired);
            }

            let response = self.client.post(&url).json(&request).send().await?;

            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            if status.is_success() {
                return serde_json::from_str(&body).map_err(ApiError::from);
            }

            if status == reqwest::StatusCode::BAD_REQUEST {
                let error_code = parse_device_auth_error(&body);

                match error_code {
                    DeviceAuthErrorCode::AuthorizationPending => {
                        // Continue polling
                        debug!("Authorization pending, continuing to poll...");
                        continue;
                    }
                    DeviceAuthErrorCode::SlowDown => {
                        // Increase interval (RFC 8628 recommendation)
                        warn!("Server requested slow down, waiting 5 extra seconds");
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        continue;
                    }
                    DeviceAuthErrorCode::AccessDenied => {
                        return Err(ApiError::DeviceAuthDenied);
                    }
                    DeviceAuthErrorCode::ExpiredToken => {
                        return Err(ApiError::DeviceAuthExpired);
                    }
                    _ => {
                        return Err(ApiError::DeviceAuthFailed(error_code));
                    }
                }
            }

            // Other error
            return Err(ApiError::ServerError {
                status: status.as_u16(),
                message: body,
            });
        }
    }

    /// Build a request with authentication header if token is present.
    async fn authenticated_request(
        &self,
        method: reqwest::Method,
        url: &str,
    ) -> reqwest::RequestBuilder {
        let mut builder = self.client.request(method, url);

        if let Some(token) = self.access_token.read().await.as_ref() {
            builder = builder.header(DEVICE_TOKEN_HEADER, token);
        }

        builder
    }

    /// Upload lap telemetry data.
    ///
    /// # Arguments
    ///
    /// * `lap` - The lap telemetry data containing all frames
    /// * `session` - The session frame with track/car metadata
    /// * `lap_id` - Optional client-provided UUID (server generates one if not provided)
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::Request`] on network failure.
    /// Returns [`ApiError::ServerError`] if the server returns an error status.
    /// Returns [`ApiError::Unauthorized`] if authentication is required.
    /// Returns [`ApiError::Serialization`] if request serialization fails.
    #[instrument(skip(self, lap, session), fields(frame_count = lap.frames.len()))]
    pub async fn upload_lap(
        &self,
        lap: &LapTelemetry,
        session: &SessionFrame,
        lap_id: Option<Uuid>,
        is_valid: bool,
    ) -> ApiResult<LapUploadResponse> {
        let url = format!("{}/api/v1/telemetry/lap", self.base_url);

        debug!("Uploading lap telemetry to {}", url);

        let request_body = LapUploadRequest {
            lap: lap.clone(),
            session: session.clone(),
        };

        let mut request = self
            .authenticated_request(reqwest::Method::POST, &url)
            .await
            .json(&request_body);

        request = request.query(&[("is_valid", is_valid)]);

        if let Some(id) = lap_id {
            request = request.query(&[("lap_id", id.to_string())]);
        }

        let response = request.send().await?;

        self.handle_response(response).await
    }

    /// Upload lap metrics.
    ///
    /// # Arguments
    ///
    /// * `lap_metrics` - The extracted lap metrics
    /// * `lap_id` - The UUID of the lap (must match a previously uploaded lap)
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::Request`] on network failure.
    /// Returns [`ApiError::ServerError`] if the server returns an error status (including 404 if lap not found).
    /// Returns [`ApiError::Unauthorized`] if authentication is required.
    /// Returns [`ApiError::Serialization`] if request serialization fails.
    #[instrument(skip(self, lap_metrics), fields(lap_id = %lap_id))]
    pub async fn upload_lap_metrics(
        &self,
        lap_metrics: &LapMetrics,
        lap_id: Uuid,
    ) -> ApiResult<MetricsUploadResponse> {
        let url = format!("{}/api/v1/metrics/lap", self.base_url);

        debug!("Uploading lap metrics to {}", url);

        let request_body = MetricsUploadRequest {
            lap_metrics: lap_metrics.clone(),
            lap_id: lap_id.to_string(),
        };

        let response = self
            .authenticated_request(reqwest::Method::POST, &url)
            .await
            .json(&request_body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Fetch list of all track boundaries (summaries only).
    #[instrument(skip(self))]
    pub async fn fetch_track_boundaries(&self) -> ApiResult<TrackBoundaryListResponse> {
        let url = format!("{}/api/v1/tracks", self.base_url);

        debug!("Fetching track boundaries from {}", url);

        let response = self
            .authenticated_request(reqwest::Method::GET, &url)
            .await
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Fetch detailed track boundary data including coordinate arrays.
    #[instrument(skip(self), fields(%boundary_id))]
    pub async fn fetch_track_boundary(
        &self,
        boundary_id: Uuid,
    ) -> ApiResult<TrackBoundaryResponse> {
        let url = format!("{}/api/v1/tracks/{}", self.base_url, boundary_id);

        debug!("Fetching track boundary detail from {}", url);

        let response = self
            .authenticated_request(reqwest::Method::GET, &url)
            .await
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Fetch corner segments for a track boundary.
    #[instrument(skip(self), fields(%boundary_id))]
    pub async fn fetch_corner_segments(
        &self,
        boundary_id: Uuid,
    ) -> ApiResult<CornerSegmentListResponse> {
        let url = format!("{}/api/v1/tracks/{}/corners", self.base_url, boundary_id);

        debug!("Fetching corner segments from {}", url);

        let response = self
            .authenticated_request(reqwest::Method::GET, &url)
            .await
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Get the current authenticated user's profile.
    pub async fn get_me(&self) -> ApiResult<UserResponse> {
        let url = format!("{}/api/v1/auth/me", self.base_url);

        let response = self
            .authenticated_request(reqwest::Method::GET, &url)
            .await
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Check if the server is reachable by hitting the health endpoint.
    ///
    /// Returns `true` if the server responds (any status), `false` on connection error.
    pub async fn check_server_reachable(&self) -> bool {
        let url = format!("{}/api/v1/health", self.base_url);
        self.client.get(&url).send().await.is_ok()
    }

    /// Validate stored credentials against the server.
    ///
    /// 1. Checks server reachability — if unreachable, logs a warning and returns `Ok(())`.
    /// 2. Calls `/auth/me` — if the token is invalid (401), deletes stored credentials
    ///    and triggers re-authentication via the device flow.
    ///
    /// # Arguments
    ///
    /// * `device_name` - Device name to use if re-authentication is needed
    pub async fn validate_credentials(&self, device_name: &str) -> ApiResult<()> {
        if !self.check_server_reachable().await {
            warn!(
                "Server `{}` is not reachable — skipping credential validation",
                self.base_url
            );
            return Ok(());
        }

        match self.get_me().await {
            Ok(user) => {
                info!(
                    "Authenticated as {} ({})",
                    user.display_name.as_deref().unwrap_or("unknown"),
                    user.email
                );
                Ok(())
            }
            Err(ApiError::Unauthorized) => {
                warn!("Stored credentials are invalid — re-authenticating");
                self.logout().await?;
                self.authenticate(device_name).await
            }
            Err(e) => {
                warn!("Failed to validate credentials: {e}");
                Ok(())
            }
        }
    }

    /// Handle HTTP response, converting error status codes to [`ApiError`].
    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> ApiResult<T> {
        let status = response.status();

        if status.is_success() {
            let body = response.json::<T>().await?;
            Ok(body)
        } else if status == reqwest::StatusCode::UNAUTHORIZED {
            Err(ApiError::Unauthorized)
        } else {
            // Try to extract error message from response body
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            Err(ApiError::ServerError {
                status: status.as_u16(),
                message,
            })
        }
    }
}

/// Parse device authorization error from server response body.
fn parse_device_auth_error(body: &str) -> DeviceAuthErrorCode {
    #[derive(Deserialize)]
    struct ErrorDetail {
        error: String,
    }
    #[derive(Deserialize)]
    struct ErrorResponse {
        detail: ErrorDetail,
    }

    serde_json::from_str::<ErrorResponse>(body)
        .map(|r| DeviceAuthErrorCode::from_str(&r.detail.error))
        .unwrap_or(DeviceAuthErrorCode::Unknown("parse_error".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_empty_url() {
        let result = ServerAPIClient::new("");
        assert!(matches!(result, Err(ApiError::InvalidUrl(_))));
    }

    #[test]
    fn test_new_strips_trailing_slash() {
        let client = ServerAPIClient::new("http://localhost:8000/").unwrap();
        assert_eq!(client.base_url, "http://localhost:8000");
    }

    #[test]
    fn test_with_client_validates_url() {
        let http_client = Client::new();
        let result = ServerAPIClient::with_client(http_client, "");
        assert!(matches!(result, Err(ApiError::InvalidUrl(_))));
    }
}
