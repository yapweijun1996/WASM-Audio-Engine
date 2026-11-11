//! Split interleaved PCM into bounded, optionally overlapping chunks.

/// PCM data and its half-open frame range on the input timeline.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioChunk {
    pub start_frame: u64,
    pub end_frame: u64,
    pub samples: Vec<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkError {
    InvalidConfiguration,
    IncompleteFrame,
    NonFiniteSample,
    TimelineOverflow,
    AlreadyFinished,
}

/// Emits fixed-size chunks and an optional final partial chunk.
pub struct Chunker {
    channels: usize,
    chunk_frames: usize,
    overlap_frames: usize,
    position: u64,
    start: u64,
    last_emitted_end: u64,
    buffer: Vec<f32>,
    finished: bool,
}

impl Chunker {
    pub fn new(
        channels: usize,
        chunk_frames: usize,
        overlap_frames: usize,
    ) -> Result<Self, ChunkError> {
        if channels == 0
            || chunk_frames == 0
            || overlap_frames >= chunk_frames
            || chunk_frames.checked_mul(channels).is_none()
        {
            return Err(ChunkError::InvalidConfiguration);
        }
        Ok(Self {
            channels,
            chunk_frames,
            overlap_frames,
            position: 0,
            start: 0,
            last_emitted_end: 0,
            buffer: Vec::new(),
            finished: false,
        })
    }

    /// Process complete interleaved frames; invalid input leaves state unchanged.
    pub fn process(&mut self, samples: &[f32]) -> Result<Vec<AudioChunk>, ChunkError> {
        if self.finished {
            return Err(ChunkError::AlreadyFinished);
        }
        if !samples.len().is_multiple_of(self.channels) {
            return Err(ChunkError::IncompleteFrame);
        }
        if samples.iter().any(|sample| !sample.is_finite()) {
            return Err(ChunkError::NonFiniteSample);
        }
        let frames = u64::try_from(samples.len() / self.channels)
            .map_err(|_| ChunkError::TimelineOverflow)?;
        self.position
            .checked_add(frames)
            .ok_or(ChunkError::TimelineOverflow)?;
        let mut output = Vec::new();
        for frame in samples.chunks_exact(self.channels) {
            self.buffer.extend_from_slice(frame);
            self.position += 1;
            if self.buffer.len() / self.channels == self.chunk_frames {
                output.push(AudioChunk {
                    start_frame: self.start,
                    end_frame: self.position,
                    samples: self.buffer.clone(),
                });
                self.last_emitted_end = self.position;
                let drop_samples = (self.chunk_frames - self.overlap_frames) * self.channels;
                self.buffer.drain(..drop_samples);
                self.start = self.position - self.overlap_frames as u64;
            }
        }
        Ok(output)
    }

    /// Finish the stream without emitting a duplicate overlap-only tail.
    pub fn finish(&mut self) -> Result<Option<AudioChunk>, ChunkError> {
        if self.finished {
            return Err(ChunkError::AlreadyFinished);
        }
        self.finished = true;
        if self.position <= self.last_emitted_end {
            return Ok(None);
        }
        Ok(Some(AudioChunk {
            start_frame: self.start,
            end_frame: self.position,
            samples: std::mem::take(&mut self.buffer),
        }))
    }
}
