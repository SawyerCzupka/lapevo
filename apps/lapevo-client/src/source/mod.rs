use std::path::PathBuf;

/// Configuration for IBT file replay (used by interactive UI).
#[derive(Debug, Clone)]
pub struct ReplayConfig {
    pub file_path: PathBuf,
    pub speed: f64,
}
