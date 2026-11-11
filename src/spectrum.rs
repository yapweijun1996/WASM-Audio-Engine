//! One-sided amplitude spectra for normalized mono PCM.

use rustfft::{num_complex::Complex, FftPlanner};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpectrumBin {
    pub frequency_hz: f32,
    pub amplitude: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpectrumError {
    InvalidSampleRate,
    InvalidLength,
    InvalidSample,
}

/// Analyze one mono window using a Hann window and coherent-gain correction.
///
/// Returns one-sided peak amplitudes, not power spectral density. Input samples
/// must be finite and within [-1, 1]; windows contain 4 to 65,536 samples.
pub fn spectrum(samples: &[f32], sample_rate: u32) -> Result<Vec<SpectrumBin>, SpectrumError> {
    if sample_rate == 0 {
        return Err(SpectrumError::InvalidSampleRate);
    }
    if !(4..=65_536).contains(&samples.len()) {
        return Err(SpectrumError::InvalidLength);
    }
    if samples
        .iter()
        .any(|sample| !sample.is_finite() || !(-1.0..=1.0).contains(sample))
    {
        return Err(SpectrumError::InvalidSample);
    }
    let length = samples.len();
    let mut window_sum = 0.0_f32;
    let mut data: Vec<_> = samples
        .iter()
        .enumerate()
        .map(|(index, sample)| {
            let window =
                0.5 - 0.5 * (std::f32::consts::TAU * index as f32 / (length - 1) as f32).cos();
            window_sum += window;
            Complex::new(sample * window, 0.0)
        })
        .collect();
    let mut planner = FftPlanner::<f32>::new();
    planner.plan_fft_forward(length).process(&mut data);
    Ok(data
        .iter()
        .take(length / 2 + 1)
        .enumerate()
        .map(|(index, bin)| {
            let is_dc_or_nyquist = index == 0 || (length.is_multiple_of(2) && index == length / 2);
            let factor = if is_dc_or_nyquist { 1.0 } else { 2.0 };
            SpectrumBin {
                frequency_hz: index as f32 * sample_rate as f32 / length as f32,
                amplitude: bin.norm() * factor / window_sum,
            }
        })
        .collect())
}
