//! Native streaming WAV preparation. Browser file integration is not implemented.

use std::io::{Read, Seek, SeekFrom, Write};

use crate::pcm::{convert_channels, PcmError};
use crate::resample::{ResampleError, StreamingResampler};
use crate::silence::{SilenceDetector, SilenceError, SilenceInterval};
use crate::wav::{Pcm16Reader, Pcm16Writer, WavError};
use crate::waveform::{WaveformBuilder, WaveformError, WaveformPoint};

/// Options for a mono/stereo 16-bit PCM WAV preparation pass.
pub struct PipelineOptions {
    pub output_rate: u32,
    pub output_channels: u16,
    pub normalize_peak: Option<f32>,
    pub silence_threshold: f32,
    pub min_silence_frames: u64,
    pub waveform_window_frames: u64,
    pub input_chunk_frames: usize,
}

#[derive(Debug)]
pub enum PipelineError {
    InvalidOptions,
    InputIo(std::io::Error),
    Wav(WavError),
    Pcm(PcmError),
    Silence(SilenceError),
    Waveform(WaveformError),
    Resample(ResampleError),
    FrameOverflow,
}

/// Processing statistics. Clipping reports output samples saturated for PCM16.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PipelineStats {
    pub input_frames: u64,
    pub output_frames: u64,
    pub applied_gain: f32,
    pub clipped_samples: u64,
}

/// Process WAV in bounded input blocks, reporting analysis on the original timeline.
///
/// Normalization performs two passes. The input must be seekable. Analysis callbacks
/// may be called multiple times, including with empty slices. The caller owns output
/// rollback if an error occurs, because a partial WAV may already have been written.
/// Use an empty output distinct from the input; callbacks may have run before an error.
pub fn process_wav<R: Read + Seek, W: Write + Seek>(
    input: &mut R,
    output: &mut W,
    options: &PipelineOptions,
    mut analysis: impl FnMut(&[SilenceInterval], &[WaveformPoint]),
) -> Result<PipelineStats, PipelineError> {
    if options.input_chunk_frames == 0
        || options.input_chunk_frames > 65_536
        || !(1..=2).contains(&options.output_channels)
        || options
            .normalize_peak
            .is_some_and(|peak| !peak.is_finite() || peak <= 0.0 || peak > 1.0)
    {
        return Err(PipelineError::InvalidOptions);
    }
    let start = input.stream_position().map_err(PipelineError::InputIo)?;
    let (spec, peak) = {
        let mut scanner = Pcm16Reader::new(&mut *input).map_err(PipelineError::Wav)?;
        let spec = scanner.spec();
        if !(1..=2).contains(&spec.channels) {
            return Err(PipelineError::InvalidOptions);
        }
        let mut peak = 0.0_f32;
        if options.normalize_peak.is_some() {
            loop {
                let block = scanner
                    .read_chunk(options.input_chunk_frames)
                    .map_err(PipelineError::Wav)?;
                if block.is_empty() {
                    break;
                }
                for sample in block {
                    peak = peak.max(sample.abs());
                }
            }
        }
        (spec, peak)
    };
    input
        .seek(SeekFrom::Start(start))
        .map_err(PipelineError::InputIo)?;
    let mut reader = Pcm16Reader::new(&mut *input).map_err(PipelineError::Wav)?;
    let gain = match options.normalize_peak {
        Some(target) if peak > 0.0 => target / peak,
        _ => 1.0,
    };
    let mut silence = SilenceDetector::new(
        usize::from(spec.channels),
        options.silence_threshold,
        options.min_silence_frames,
    )
    .map_err(PipelineError::Silence)?;
    let mut waveform =
        WaveformBuilder::new(usize::from(spec.channels), options.waveform_window_frames)
            .map_err(PipelineError::Waveform)?;
    let mut resampler = StreamingResampler::new(
        spec.sample_rate,
        options.output_rate,
        usize::from(options.output_channels),
        1024,
    )
    .map_err(PipelineError::Resample)?;
    let mut writer = Pcm16Writer::new(output, options.output_rate, options.output_channels)
        .map_err(PipelineError::Wav)?;
    let mut stats = PipelineStats {
        input_frames: 0,
        output_frames: 0,
        applied_gain: gain,
        clipped_samples: 0,
    };
    loop {
        let block = reader
            .read_chunk(options.input_chunk_frames)
            .map_err(PipelineError::Wav)?;
        if block.is_empty() {
            break;
        }
        stats.input_frames = stats
            .input_frames
            .checked_add((block.len() / usize::from(spec.channels)) as u64)
            .ok_or(PipelineError::FrameOverflow)?;
        let intervals = silence.process(&block).map_err(PipelineError::Silence)?;
        let points = waveform.process(&block).map_err(PipelineError::Waveform)?;
        analysis(&intervals, &points);
        let mut converted = convert_channels(
            &block,
            usize::from(spec.channels),
            usize::from(options.output_channels),
        )
        .map_err(PipelineError::Pcm)?;
        if let Some(target) = options.normalize_peak.filter(|_| peak > 0.0) {
            for sample in &mut converted {
                *sample = (*sample / peak) * target;
            }
        }
        let resampled = resampler
            .process(&converted)
            .map_err(PipelineError::Resample)?;
        write_output(&mut writer, resampled, &mut stats, options.output_channels)?;
    }
    let intervals: Vec<_> = silence
        .finish()
        .map_err(PipelineError::Silence)?
        .into_iter()
        .collect();
    let points: Vec<_> = waveform
        .finish()
        .map_err(PipelineError::Waveform)?
        .into_iter()
        .collect();
    analysis(&intervals, &points);
    let tail = resampler.finish().map_err(PipelineError::Resample)?;
    write_output(&mut writer, tail, &mut stats, options.output_channels)?;
    writer.finish().map_err(PipelineError::Wav)?;
    Ok(stats)
}

fn write_output<W: Write + Seek>(
    writer: &mut Pcm16Writer<W>,
    mut samples: Vec<f32>,
    stats: &mut PipelineStats,
    channels: u16,
) -> Result<(), PipelineError> {
    for sample in &mut samples {
        if sample.is_finite() && !(-1.0..=1.0).contains(sample) {
            stats.clipped_samples = stats
                .clipped_samples
                .checked_add(1)
                .ok_or(PipelineError::FrameOverflow)?;
            *sample = sample.clamp(-1.0, 1.0);
        }
    }
    writer.write_chunk(&samples).map_err(PipelineError::Wav)?;
    stats.output_frames = stats
        .output_frames
        .checked_add((samples.len() / usize::from(channels)) as u64)
        .ok_or(PipelineError::FrameOverflow)?;
    Ok(())
}
