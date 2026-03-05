mod download;
mod error;
mod kokoro;
mod player;

pub use error::{Result, TtsError};

pub mod backend {
    pub use crate::kokoro::KokoroBackend;
}

use async_trait::async_trait;
use player::AudioPlayer;

#[derive(Debug, Clone)]
pub struct AudioClip {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

#[async_trait]
pub trait TtsBackend: Send + Sync {
    async fn synth(&self, text: &str) -> Result<AudioClip>;
}

pub struct Tts {
    backend: Box<dyn TtsBackend>,
    player: AudioPlayer,
}

impl Tts {
    /// Create a TTS instance with the Kokoro backend (sherpa-rs) and platform cache dir.
    pub async fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .ok_or(TtsError::CacheDir)?
            .join("lapevo")
            .join("tts");
        let backend = backend::KokoroBackend::new(cache_dir).await?;
        Self::with_backend(backend)
    }

    /// Create a TTS instance with a custom backend.
    pub fn with_backend(backend: impl TtsBackend + 'static) -> Result<Self> {
        let player = AudioPlayer::new()?;
        Ok(Self {
            backend: Box::new(backend),
            player,
        })
    }

    pub async fn synth(&self, text: &str) -> Result<AudioClip> {
        self.backend.synth(text).await
    }

    pub fn play(&self, clip: &AudioClip) -> Result<()> {
        self.player.play(clip)
    }

    pub async fn speak(&self, text: &str) -> Result<()> {
        let clip = self.synth(text).await?;
        self.play(&clip)
    }
}
