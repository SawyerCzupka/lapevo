use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use sherpa_rs::tts::{KokoroTts as SherpaKokoroTts, KokoroTtsConfig};
use tracing::info;

use crate::download::ensure_model_dir;
use crate::{AudioClip, Result, TtsBackend, TtsError};

const MODEL_ARCHIVE_URL: &str =
    "https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/kokoro-en-v0_19.tar.bz2";

/// Subdirectory name inside the archive.
const MODEL_DIR_NAME: &str = "kokoro-en-v0_19";

const SAMPLE_RATE: u32 = 24000;

/// Default speaker ID (af_heart).
const DEFAULT_SPEAKER_ID: i32 = 0;
const DEFAULT_SPEED: f32 = 1.0;

pub struct KokoroBackend {
    tts: Arc<Mutex<SherpaKokoroTts>>,
    speaker_id: i32,
    speed: f32,
}

impl KokoroBackend {
    /// Create a new Kokoro backend with default settings.
    ///
    /// Downloads the model on first use (~340MB).
    pub async fn new(cache_dir: impl Into<PathBuf>) -> Result<Self> {
        Self::with_options(cache_dir, DEFAULT_SPEAKER_ID, DEFAULT_SPEED).await
    }

    /// Create a new Kokoro backend with custom speaker and speed.
    pub async fn with_options(
        cache_dir: impl Into<PathBuf>,
        speaker_id: i32,
        speed: f32,
    ) -> Result<Self> {
        let cache_dir = cache_dir.into();
        let model_dir = cache_dir.join(MODEL_DIR_NAME);
        let sentinel = model_dir.join("model.onnx");

        ensure_model_dir(MODEL_ARCHIVE_URL, &cache_dir, &sentinel).await?;

        let model = model_dir.join("model.onnx");
        let voices = model_dir.join("voices.bin");
        let tokens = model_dir.join("tokens.txt");
        let data_dir = model_dir.join("espeak-ng-data");

        let config = KokoroTtsConfig {
            model: model.to_string_lossy().into_owned(),
            voices: voices.to_string_lossy().into_owned(),
            tokens: tokens.to_string_lossy().into_owned(),
            data_dir: data_dir.to_string_lossy().into_owned(),
            length_scale: 1.0,
            ..Default::default()
        };

        let tts = SherpaKokoroTts::new(config);

        info!("Kokoro TTS model loaded (sherpa-rs)");

        Ok(Self {
            tts: Arc::new(Mutex::new(tts)),
            speaker_id,
            speed,
        })
    }
}

#[async_trait]
impl TtsBackend for KokoroBackend {
    async fn synth(&self, text: &str) -> Result<AudioClip> {
        let tts = Arc::clone(&self.tts);
        let text = text.to_owned();
        let sid = self.speaker_id;
        let speed = self.speed;

        let audio = tokio::task::spawn_blocking(move || {
            let mut tts = tts.lock().unwrap();
            tts.create(&text, sid, speed)
        })
        .await
        .map_err(|e| TtsError::Synthesis(e.to_string()))?
        .map_err(|e| TtsError::Synthesis(e.to_string()))?;

        info!("TTS synthesis complete ({}ms)", audio.duration);

        Ok(AudioClip {
            samples: audio.samples,
            sample_rate: SAMPLE_RATE,
            channels: 1,
        })
    }
}
