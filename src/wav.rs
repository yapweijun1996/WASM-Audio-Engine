//! Bounded reads and writes for uncompressed 16-bit integer PCM WAV files.

use std::io::{Read, Seek, Write};

/// WAV input or output failure.
#[derive(Debug)]
pub enum WavError {
    /// The WAV codec or sample representation is not supported.
    UnsupportedFormat,
    /// The file metadata or requested chunk size is invalid.
    InvalidParameter,
    /// The supplied interleaved samples do not contain complete frames.
    IncompleteFrame,
    /// An output sample is non-finite or outside the normalized PCM range.
    InvalidSample,
    /// The underlying WAV operation failed.
    Codec(hound::Error),
}

impl From<hound::Error> for WavError {
    fn from(error: hound::Error) -> Self {
        Self::Codec(error)
    }
}

/// A streaming reader for 16-bit PCM WAV data.
pub struct Pcm16Reader<R: Read> {
    reader: hound::WavReader<R>,
    spec: hound::WavSpec,
}

impl<R: Read> Pcm16Reader<R> {
    /// Parse the WAV header and validate the supported format.
    pub fn new(input: R) -> Result<Self, WavError> {
        let reader = hound::WavReader::new(input)?;
        let spec = reader.spec();
        if spec.sample_format != hound::SampleFormat::Int || spec.bits_per_sample != 16 {
            return Err(WavError::UnsupportedFormat);
        }
        if spec.channels == 0 || spec.sample_rate == 0 {
            return Err(WavError::InvalidParameter);
        }
        Ok(Self { reader, spec })
    }

    /// Return the original WAV parameters.
    pub fn spec(&self) -> hound::WavSpec {
        self.spec
    }

    /// Read at most `max_frames` interleaved frames. Empty output means EOF.
    pub fn read_chunk(&mut self, max_frames: usize) -> Result<Vec<f32>, WavError> {
        if max_frames == 0 {
            return Err(WavError::InvalidParameter);
        }
        let channels = usize::from(self.spec.channels);
        let capacity = max_frames
            .checked_mul(channels)
            .ok_or(WavError::InvalidParameter)?;
        let mut result = Vec::new();
        result
            .try_reserve(capacity)
            .map_err(|_| WavError::InvalidParameter)?;
        for sample in self.reader.samples::<i16>().take(capacity) {
            result.push(f32::from(sample?) / 32768.0);
        }
        if !result.len().is_multiple_of(channels) {
            return Err(WavError::IncompleteFrame);
        }
        Ok(result)
    }
}

/// A streaming writer for 16-bit PCM WAV data.
pub struct Pcm16Writer<W: Write + Seek> {
    writer: hound::WavWriter<W>,
    channels: usize,
}

impl<W: Write + Seek> Pcm16Writer<W> {
    /// Create a WAV writer with the requested sample rate and channel count.
    pub fn new(output: W, sample_rate: u32, channels: u16) -> Result<Self, WavError> {
        if sample_rate == 0 || channels == 0 {
            return Err(WavError::InvalidParameter);
        }
        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        Ok(Self {
            writer: hound::WavWriter::new(output, spec)?,
            channels: usize::from(channels),
        })
    }

    /// Write complete interleaved frames. Invalid samples are rejected before writing.
    pub fn write_chunk(&mut self, samples: &[f32]) -> Result<(), WavError> {
        if !samples.len().is_multiple_of(self.channels) {
            return Err(WavError::IncompleteFrame);
        }
        if samples
            .iter()
            .any(|sample| !sample.is_finite() || *sample < -1.0 || *sample > 1.0)
        {
            return Err(WavError::InvalidSample);
        }
        for sample in samples {
            let quantized = (sample * 32768.0).round().clamp(-32768.0, 32767.0) as i16;
            self.writer.write_sample(quantized)?;
        }
        Ok(())
    }

    /// Finish the WAV header and return any output error.
    pub fn finish(self) -> Result<(), WavError> {
        self.writer.finalize()?;
        Ok(())
    }
}
