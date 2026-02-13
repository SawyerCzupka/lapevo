use std::path::{Path, PathBuf};

use async_trait::async_trait;
use futures_util::StreamExt;
use kokoro_tts::{KokoroTts, Voice};
use tokio::io::AsyncWriteExt;
use tracing::info;

use crate::{AudioClip, Result, TtsBackend, TtsError};

const KOKORO_MODEL_URL: &str =
    "https://github.com/mzdk100/kokoro/releases/download/V1.0/kokoro-v1.0.int8.onnx";
const KOKORO_VOICES_URL: &str =
    "https://github.com/mzdk100/kokoro/releases/download/V1.0/voices.bin";

const KOKORO_SAMPLE_RATE: u32 = 24000;

pub struct KokoroBackend {
    tts: KokoroTts,
    voice: Voice,
}

impl KokoroBackend {
    pub async fn new(models_dir: impl Into<PathBuf>, voice: Voice) -> Result<Self> {
        let models_dir = models_dir.into();
        let model_path = models_dir.join("kokoro-v1.0.int8.onnx");
        let voices_path = models_dir.join("voices.bin");

        ensure_file(KOKORO_MODEL_URL, &model_path).await?;
        ensure_file(KOKORO_VOICES_URL, &voices_path).await?;

        let tts = KokoroTts::new(&model_path, &voices_path)
            .await
            .map_err(|e| TtsError::ModelInit(e.to_string()))?;

        info!("Kokoro TTS model loaded");

        Ok(Self { tts, voice })
    }
}

#[async_trait]
impl TtsBackend for KokoroBackend {
    async fn synth(&self, text: &str) -> Result<AudioClip> {
        let (samples, duration) = self
            .tts
            .synth(text, self.voice)
            .await
            .map_err(|e| TtsError::Synthesis(e.to_string()))?;

        info!("TTS synthesis took {:?}", duration);

        Ok(AudioClip {
            samples,
            sample_rate: KOKORO_SAMPLE_RATE,
            channels: 1,
        })
    }
}

async fn ensure_file(url: &str, path: &Path) -> Result<()> {
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
