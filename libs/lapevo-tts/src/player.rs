use rodio::OutputStream;
use rodio::OutputStreamBuilder;
use rodio::buffer::SamplesBuffer;

use crate::{AudioClip, Result, TtsError};

pub(crate) struct AudioPlayer {
    stream: OutputStream,
}

impl AudioPlayer {
    pub(crate) fn new() -> Result<Self> {
        let stream = OutputStreamBuilder::open_default_stream()
            .map_err(|e| TtsError::Playback(e.to_string()))?;
        Ok(Self { stream })
    }

    pub(crate) fn play(&self, clip: &AudioClip) -> Result<()> {
        let source = SamplesBuffer::new(clip.channels, clip.sample_rate, clip.samples.clone());
        self.stream.mixer().add(source);
        Ok(())
    }
}
