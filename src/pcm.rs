//! Operations on interleaved PCM frames.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PcmError {
    UnsupportedChannels,
    IncompleteFrame,
    InvalidRange,
    NonFiniteSample,
    SizeOverflow,
}

/// Borrow a half-open frame range without copying samples.
pub fn extract_frames(
    samples: &[f32],
    channels: usize,
    start: usize,
    end: usize,
) -> Result<&[f32], PcmError> {
    if channels == 0 {
        return Err(PcmError::UnsupportedChannels);
    }
    if !samples.len().is_multiple_of(channels) {
        return Err(PcmError::IncompleteFrame);
    }
    if start > end || end > samples.len() / channels {
        return Err(PcmError::InvalidRange);
    }
    let first = start.checked_mul(channels).ok_or(PcmError::SizeOverflow)?;
    let last = end.checked_mul(channels).ok_or(PcmError::SizeOverflow)?;
    Ok(&samples[first..last])
}

/// Convert mono/stereo channels, averaging left and right when downmixing.
pub fn convert_channels(
    samples: &[f32],
    source: usize,
    target: usize,
) -> Result<Vec<f32>, PcmError> {
    if !(1..=2).contains(&source) || !(1..=2).contains(&target) {
        return Err(PcmError::UnsupportedChannels);
    }
    if !samples.len().is_multiple_of(source) {
        return Err(PcmError::IncompleteFrame);
    }
    if samples.iter().any(|sample| !sample.is_finite()) {
        return Err(PcmError::NonFiniteSample);
    }
    let frames = samples.len() / source;
    let capacity = frames.checked_mul(target).ok_or(PcmError::SizeOverflow)?;
    let mut result = Vec::new();
    result
        .try_reserve(capacity)
        .map_err(|_| PcmError::SizeOverflow)?;
    for frame in samples.chunks_exact(source) {
        match (source, target) {
            (1, 1) => result.push(frame[0]),
            (1, 2) => result.extend_from_slice(&[frame[0], frame[0]]),
            (2, 1) => result.push(frame[0] * 0.5 + frame[1] * 0.5),
            (2, 2) => result.extend_from_slice(frame),
            _ => unreachable!(),
        }
    }
    Ok(result)
}
