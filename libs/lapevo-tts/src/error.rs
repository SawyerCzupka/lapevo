use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, TtsError>;

#[derive(Debug, Error)]
pub enum TtsError {
    #[error("failed to download model file: {0}")]
    Download(#[from] reqwest::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to initialize TTS model: {0}")]
    ModelInit(String),

    #[error("synthesis failed: {0}")]
    Synthesis(String),

    #[error("playback failed: {0}")]
    Playback(String),

    #[error("archive extraction failed: {0}")]
    Archive(String),

    #[error("model file not found: {0}")]
    ModelNotFound(PathBuf),

    #[error("could not determine platform cache directory")]
    CacheDir,
}
