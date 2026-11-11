//! Chunked, fixed-rate PCM resampling using Rubato.

use rubato::audioadapter::Adapter;
use rubato::audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft, FixedSync, Indexing, Resampler};

#[derive(Debug)]
pub enum ResampleError {
    InvalidConfiguration,
    IncompleteFrame,
    NonFiniteSample,
    TimelineOverflow,
    AlreadyFinished,
    Backend(String),
}

/// A streaming resampler. Returned samples are interleaved and start at frame zero.
pub struct StreamingResampler {
    inner: Fft<f32>,
    channels: usize,
    input_rate: u32,
    output_rate: u32,
    input: Vec<f32>,
    pending: Vec<f32>,
    input_frames: u64,
    emitted_frames: u64,
    trim_frames: usize,
    finished: bool,
}

impl StreamingResampler {
    /// Construct a fixed-rate resampler. Rates, channels, and chunk size are bounded.
    pub fn new(
        input_rate: u32,
        output_rate: u32,
        channels: usize,
        chunk_frames: usize,
    ) -> Result<Self, ResampleError> {
        if !(8_000..=192_000).contains(&input_rate)
            || !(8_000..=192_000).contains(&output_rate)
            || !(1..=8).contains(&channels)
            || !(256..=8192).contains(&chunk_frames)
        {
            return Err(ResampleError::InvalidConfiguration);
        }
        let inner = Fft::new(
            input_rate as usize,
            output_rate as usize,
            chunk_frames,
            channels,
            FixedSync::Input,
        )
        .map_err(|error| ResampleError::Backend(error.to_string()))?;
        let trim_frames = inner.output_delay();
        Ok(Self {
            inner,
            channels,
            input_rate,
            output_rate,
            input: Vec::new(),
            pending: Vec::new(),
            input_frames: 0,
            emitted_frames: 0,
            trim_frames,
            finished: false,
        })
    }

    /// Process complete input frames; returned output need not match the input length.
    pub fn process(&mut self, samples: &[f32]) -> Result<Vec<f32>, ResampleError> {
        if self.finished {
            return Err(ResampleError::AlreadyFinished);
        }
        if !samples.len().is_multiple_of(self.channels) {
            return Err(ResampleError::IncompleteFrame);
        }
        if samples.iter().any(|sample| !sample.is_finite()) {
            return Err(ResampleError::NonFiniteSample);
        }
        let frames = u64::try_from(samples.len() / self.channels)
            .map_err(|_| ResampleError::TimelineOverflow)?;
        self.input_frames = self
            .input_frames
            .checked_add(frames)
            .ok_or(ResampleError::TimelineOverflow)?;
        let mut remaining = samples;
        while !remaining.is_empty() {
            let required = self
                .inner
                .input_frames_next()
                .checked_mul(self.channels)
                .ok_or(ResampleError::TimelineOverflow)?;
            let take = (required - self.input.len()).min(remaining.len());
            self.input.extend_from_slice(&remaining[..take]);
            remaining = &remaining[take..];
            if self.input.len() == required {
                if let Err(error) = self.process_buffer(None) {
                    self.finished = true;
                    return Err(error);
                }
            }
        }
        self.take_available(false)
    }

    /// Flush the final partial block and filter delay, returning the remaining samples.
    pub fn finish(&mut self) -> Result<Vec<f32>, ResampleError> {
        if self.finished {
            return Err(ResampleError::AlreadyFinished);
        }
        self.finished = true;
        let remaining_frames = self.input.len() / self.channels;
        if remaining_frames > 0 {
            self.process_buffer(Some(remaining_frames))?;
        }
        let expected = self.expected_frames(true)?;
        while self
            .emitted_frames
            .checked_add((self.pending.len() / self.channels) as u64)
            .ok_or(ResampleError::TimelineOverflow)?
            < expected
        {
            let previous = self.pending.len();
            self.process_buffer(Some(0))?;
            if self.pending.len() == previous && self.trim_frames == 0 {
                return Err(ResampleError::Backend(
                    "resampler produced no output while flushing".into(),
                ));
            }
        }
        self.take_available(true)
    }

    fn process_buffer(&mut self, partial: Option<usize>) -> Result<(), ResampleError> {
        let frames = self.input.len() / self.channels;
        let adapter = InterleavedSlice::new(self.input.as_slice(), self.channels, frames)
            .map_err(|error| ResampleError::Backend(error.to_string()))?;
        let indexing = partial.map(|length| Indexing::new().partial_len(length));
        let result = self
            .inner
            .process(&adapter, indexing.as_ref())
            .map_err(|error| ResampleError::Backend(error.to_string()))?;
        let count = result.frames();
        let skip = self.trim_frames.min(count);
        self.trim_frames -= skip;
        let data = result.take_data();
        self.pending
            .extend_from_slice(&data[skip * self.channels..]);
        self.input.clear();
        Ok(())
    }

    fn expected_frames(&self, round_up: bool) -> Result<u64, ResampleError> {
        let numerator = u128::from(self.input_frames) * u128::from(self.output_rate);
        let denominator = u128::from(self.input_rate);
        let count = if round_up {
            numerator.div_ceil(denominator)
        } else {
            numerator / denominator
        };
        u64::try_from(count).map_err(|_| ResampleError::TimelineOverflow)
    }

    fn take_available(&mut self, final_output: bool) -> Result<Vec<f32>, ResampleError> {
        let allowed = self
            .expected_frames(final_output)?
            .saturating_sub(self.emitted_frames);
        let available = (self.pending.len() / self.channels) as u64;
        let take_frames = allowed.min(available) as usize;
        let count = take_frames
            .checked_mul(self.channels)
            .ok_or(ResampleError::TimelineOverflow)?;
        let output: Vec<f32> = self.pending.drain(..count).collect();
        self.emitted_frames += take_frames as u64;
        if final_output {
            self.pending.clear();
        }
        Ok(output)
    }
}
