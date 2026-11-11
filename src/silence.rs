//! Streaming low-amplitude interval detection. This is not voice detection.

/// A half-open interval measured in frames on the original timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SilenceInterval {
    pub start_frame: u64,
    pub end_frame: u64,
}

/// Invalid detector input or state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SilenceError {
    InvalidConfiguration,
    IncompleteFrame,
    NonFiniteSample,
    TimelineOverflow,
    AlreadyFinished,
}

/// Detect consecutive frames whose every channel is at or below a threshold.
pub struct SilenceDetector {
    channels: usize,
    threshold: f32,
    min_frames: u64,
    position: u64,
    start: Option<u64>,
    finished: bool,
}

impl SilenceDetector {
    /// Create a detector using a linear amplitude threshold in [0, 1].
    pub fn new(channels: usize, threshold: f32, min_frames: u64) -> Result<Self, SilenceError> {
        if channels == 0
            || !threshold.is_finite()
            || !(0.0..=1.0).contains(&threshold)
            || min_frames == 0
        {
            return Err(SilenceError::InvalidConfiguration);
        }
        Ok(Self {
            channels,
            threshold,
            min_frames,
            position: 0,
            start: None,
            finished: false,
        })
    }

    /// Process complete interleaved frames and return newly completed intervals.
    /// Invalid input is rejected before the detector state is changed.
    pub fn process(&mut self, samples: &[f32]) -> Result<Vec<SilenceInterval>, SilenceError> {
        if self.finished {
            return Err(SilenceError::AlreadyFinished);
        }
        if !samples.len().is_multiple_of(self.channels) {
            return Err(SilenceError::IncompleteFrame);
        }
        if samples.iter().any(|sample| !sample.is_finite()) {
            return Err(SilenceError::NonFiniteSample);
        }
        let frames = u64::try_from(samples.len() / self.channels)
            .map_err(|_| SilenceError::TimelineOverflow)?;
        self.position
            .checked_add(frames)
            .ok_or(SilenceError::TimelineOverflow)?;
        let mut intervals = Vec::new();
        for frame in samples.chunks_exact(self.channels) {
            if frame.iter().all(|sample| sample.abs() <= self.threshold) {
                self.start.get_or_insert(self.position);
            } else if let Some(start_frame) = self.start.take() {
                if self.position - start_frame >= self.min_frames {
                    intervals.push(SilenceInterval {
                        start_frame,
                        end_frame: self.position,
                    });
                }
            }
            self.position += 1;
        }
        Ok(intervals)
    }

    /// Finish the stream and return its trailing interval, if long enough.
    pub fn finish(&mut self) -> Result<Option<SilenceInterval>, SilenceError> {
        if self.finished {
            return Err(SilenceError::AlreadyFinished);
        }
        self.finished = true;
        Ok(self
            .start
            .take()
            .filter(|start| self.position - *start >= self.min_frames)
            .map(|start_frame| SilenceInterval {
                start_frame,
                end_frame: self.position,
            }))
    }
}
