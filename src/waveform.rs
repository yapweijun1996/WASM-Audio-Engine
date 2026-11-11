//! Time-indexed min/max amplitude summaries for interleaved PCM.

/// One half-open frame range and its amplitude extrema across all channels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaveformPoint {
    pub start_frame: u64,
    pub end_frame: u64,
    pub min: f32,
    pub max: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveformError {
    InvalidConfiguration,
    IncompleteFrame,
    NonFiniteSample,
    TimelineOverflow,
    AlreadyFinished,
}

/// Summarize fixed-size frame windows while preserving state across chunks.
pub struct WaveformBuilder {
    channels: usize,
    window_frames: u64,
    position: u64,
    window_start: u64,
    min: f32,
    max: f32,
    finished: bool,
}

impl WaveformBuilder {
    pub fn new(channels: usize, window_frames: u64) -> Result<Self, WaveformError> {
        if channels == 0 || window_frames == 0 {
            return Err(WaveformError::InvalidConfiguration);
        }
        Ok(Self {
            channels,
            window_frames,
            position: 0,
            window_start: 0,
            min: f32::INFINITY,
            max: f32::NEG_INFINITY,
            finished: false,
        })
    }

    /// Process complete frames; invalid input leaves the state unchanged.
    pub fn process(&mut self, samples: &[f32]) -> Result<Vec<WaveformPoint>, WaveformError> {
        if self.finished {
            return Err(WaveformError::AlreadyFinished);
        }
        if !samples.len().is_multiple_of(self.channels) {
            return Err(WaveformError::IncompleteFrame);
        }
        if samples.iter().any(|sample| !sample.is_finite()) {
            return Err(WaveformError::NonFiniteSample);
        }
        let frames = u64::try_from(samples.len() / self.channels)
            .map_err(|_| WaveformError::TimelineOverflow)?;
        self.position
            .checked_add(frames)
            .ok_or(WaveformError::TimelineOverflow)?;
        let mut points = Vec::new();
        for frame in samples.chunks_exact(self.channels) {
            for &sample in frame {
                self.min = self.min.min(sample);
                self.max = self.max.max(sample);
            }
            self.position += 1;
            if self.position - self.window_start == self.window_frames {
                points.push(self.take_point());
            }
        }
        Ok(points)
    }

    /// Return a final partial window, if one exists.
    pub fn finish(&mut self) -> Result<Option<WaveformPoint>, WaveformError> {
        if self.finished {
            return Err(WaveformError::AlreadyFinished);
        }
        self.finished = true;
        Ok((self.position != self.window_start).then(|| self.take_point()))
    }

    fn take_point(&mut self) -> WaveformPoint {
        let result = WaveformPoint {
            start_frame: self.window_start,
            end_frame: self.position,
            min: self.min,
            max: self.max,
        };
        self.window_start = self.position;
        self.min = f32::INFINITY;
        self.max = f32::NEG_INFINITY;
        result
    }
}
