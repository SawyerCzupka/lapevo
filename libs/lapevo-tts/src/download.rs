use std::path::Path;

use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;
use tracing::info;

use crate::{Result, TtsError};

/// Download a file if it doesn't already exist.
pub(crate) async fn ensure_file(url: &str, path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let filename = path.file_name().unwrap().to_string_lossy();
    info!("Downloading {filename}...");

    let tmp_path = path.with_extension("part");
    let response = reqwest::get(url).await?.error_for_status()?;
    let total = response.content_length();
    let mut stream = response.bytes_stream();
    let mut file = tokio::fs::File::create(&tmp_path).await?;
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        if let Some(total) = total {
            let pct = (downloaded as f64 / total as f64 * 100.0) as u32;
            if pct.is_multiple_of(10)
                && (downloaded - chunk.len() as u64) * 100 / total < pct as u64
            {
                info!("  {filename}: {pct}% ({downloaded} / {total} bytes)");
            }
        }
    }

    tokio::fs::rename(&tmp_path, path).await?;
    info!("  {filename}: download complete");
    Ok(())
}

/// Download and extract a tar.bz2 archive if the sentinel file doesn't exist.
///
/// - `url`: URL to the `.tar.bz2` archive
/// - `target_dir`: directory where the archive contents will be extracted
/// - `sentinel`: a file inside `target_dir` whose presence means the model is already extracted
pub(crate) async fn ensure_model_dir(
    url: &str,
    target_dir: &Path,
    sentinel: &Path,
) -> Result<()> {
    if sentinel.exists() {
        return Ok(());
    }

    tokio::fs::create_dir_all(target_dir).await?;

    let archive_path = target_dir.with_extension("tar.bz2");

    // Download the archive
    ensure_file(url, &archive_path).await?;

    // Extract in a blocking task
    let archive_path_clone = archive_path.clone();
    let target_dir = target_dir.to_path_buf();
    tokio::task::spawn_blocking(move || -> std::result::Result<(), TtsError> {
        info!("Extracting model archive...");
        let file =
            std::fs::File::open(&archive_path_clone).map_err(|e| TtsError::Archive(e.to_string()))?;
        let decoder =
            bzip2::read::BzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);
        archive
            .unpack(&target_dir)
            .map_err(|e| TtsError::Archive(e.to_string()))?;
        info!("Model extraction complete");
        Ok(())
    })
    .await
    .map_err(|e| TtsError::Archive(e.to_string()))??;

    // Clean up the archive
    if archive_path.exists() {
        tokio::fs::remove_file(&archive_path).await?;
    }

    Ok(())
}
