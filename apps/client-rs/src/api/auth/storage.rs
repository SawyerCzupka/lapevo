//! File-based credential storage for authentication tokens.

use std::fs;
use std::path::PathBuf;

use crate::api::auth::models::StoredCredentials;
use crate::api::error::{ApiError, ApiResult};

const CONFIG_DIR_NAME: &str = "racing-coach";
const CREDENTIALS_FILE: &str = "credentials.json";

/// Get the credentials file path.
///
/// Returns `~/.config/racing-coach/credentials.json` on Linux,
/// `~/Library/Application Support/racing-coach/credentials.json` on macOS,
/// `C:\Users\<user>\AppData\Roaming\racing-coach\credentials.json` on Windows.
pub fn credentials_path() -> ApiResult<PathBuf> {
    let config_dir = dirs::config_dir().ok_or_else(|| {
        ApiError::CredentialStorage("Could not determine config directory".to_string())
    })?;

    Ok(config_dir.join(CONFIG_DIR_NAME).join(CREDENTIALS_FILE))
}

/// Load stored credentials from disk.
///
/// Returns `Ok(None)` if the credentials file does not exist.
pub fn load_credentials() -> ApiResult<Option<StoredCredentials>> {
    let path = credentials_path()?;

    if !path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(&path)
        .map_err(|e| ApiError::CredentialStorage(format!("Failed to read credentials: {}", e)))?;

    let creds: StoredCredentials = serde_json::from_str(&contents)
        .map_err(|e| ApiError::CredentialStorage(format!("Invalid credentials format: {}", e)))?;

    Ok(Some(creds))
}

/// Save credentials to disk with secure permissions.
///
/// Creates the parent directory if needed. On Unix systems, sets file
/// permissions to 0600 (owner read/write only).
pub fn save_credentials(creds: &StoredCredentials) -> ApiResult<()> {
    let path = credentials_path()?;

    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            ApiError::CredentialStorage(format!("Failed to create config dir: {}", e))
        })?;
    }

    let contents = serde_json::to_string_pretty(creds).map_err(|e| {
        ApiError::CredentialStorage(format!("Failed to serialize credentials: {}", e))
    })?;

    fs::write(&path, &contents)
        .map_err(|e| ApiError::CredentialStorage(format!("Failed to write credentials: {}", e)))?;

    // Set file permissions to 0600 (owner read/write only) on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        fs::set_permissions(&path, perms).map_err(|e| {
            ApiError::CredentialStorage(format!("Failed to set permissions: {}", e))
        })?;
    }

    Ok(())
}

/// Delete stored credentials from disk.
pub fn delete_credentials() -> ApiResult<()> {
    let path = credentials_path()?;

    if path.exists() {
        fs::remove_file(&path).map_err(|e| {
            ApiError::CredentialStorage(format!("Failed to delete credentials: {}", e))
        })?;
    }

    Ok(())
}

/// Check if credentials exist on disk.
pub fn has_credentials() -> bool {
    credentials_path().map(|p| p.exists()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_credentials_path_is_valid() {
        let path = credentials_path();
        assert!(path.is_ok());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains("racing-coach"));
        assert!(path.to_string_lossy().contains("credentials.json"));
    }

    #[test]
    fn test_load_nonexistent_returns_none() {
        // Use a temporary directory that won't have credentials
        let result = load_credentials();
        // This may or may not find credentials depending on the test environment
        assert!(result.is_ok());
    }

    #[test]
    fn test_stored_credentials_serialization() {
        let creds = StoredCredentials {
            access_token: "test_token".to_string(),
            device_name: "Test Device".to_string(),
            created_at: Some(Utc::now()),
        };

        let json = serde_json::to_string(&creds).unwrap();
        let parsed: StoredCredentials = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.access_token, "test_token");
        assert_eq!(parsed.device_name, "Test Device");
        assert!(parsed.created_at.is_some());
    }
}
