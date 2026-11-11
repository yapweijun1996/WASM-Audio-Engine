//! Local audio processing primitives.
//!
//! This crate is under development. It includes a low-level WebAssembly
//! DSP bindings, streamed PCM16 WAV I/O, resampling, silence and waveform
//! analysis, spectrum analysis, and frame-based audio operations.

pub mod chunk;
pub mod pcm;
pub mod pipeline;
pub mod resample;
pub mod silence;
pub mod spectrum;
pub mod timeline;
pub mod wav;
pub mod waveform;

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(not(target_arch = "wasm32"))]
mod c_api;

/// An invalid sample or processing parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioError {
    /// A sample is not finite.
    NonFiniteSample,
    /// A target peak is not finite or is outside the interval (0, 1].
    InvalidTargetPeak,
    /// The nonzero gain cannot be represented as an f32.
    GainOutOfRange,
}

/// Normalize a PCM buffer to a target peak, returning the applied gain.
///
/// Silent or empty input is unchanged and returns a gain of 1.0. All samples
/// are checked before mutation, so invalid input does not change the buffer.
pub fn normalize_peak(samples: &mut [f32], target_peak: f32) -> Result<f32, AudioError> {
    if !target_peak.is_finite() || target_peak <= 0.0 || target_peak > 1.0 {
        return Err(AudioError::InvalidTargetPeak);
    }

    let mut peak = 0.0_f32;
    for &sample in samples.iter() {
        if !sample.is_finite() {
            return Err(AudioError::NonFiniteSample);
        }
        peak = peak.max(sample.abs());
    }
    if peak == 0.0 {
        return Ok(1.0);
    }

    // Divide before multiplying to avoid an overflowing intermediate gain.
    let gain = target_peak / peak;
    if !gain.is_finite() || gain == 0.0 {
        // Extreme amplitudes may produce a gain that f32 cannot represent.
        return Err(AudioError::GainOutOfRange);
    }
    for sample in samples {
        *sample = (*sample / peak) * target_peak;
    }
    Ok(gain)
}

/// Normalize a JavaScript Float32Array in place and return the applied gain.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn normalize_pcm_peak(
    samples: &mut [f32],
    target_peak: f32,
) -> Result<f32, wasm_bindgen::JsValue> {
    normalize_peak(samples, target_peak)
        .map_err(|error| wasm_bindgen::JsValue::from_str(&format!("{error:?}")))
}

#[cfg(test)]
mod tests {
    use super::{normalize_peak, AudioError};

    #[test]
    fn normalizes_peak_without_changing_polarity() {
        let mut samples = [0.25, -0.5, 0.0];
        assert_eq!(normalize_peak(&mut samples, 1.0), Ok(2.0));
        assert_eq!(samples, [0.5, -1.0, 0.0]);
    }

    #[test]
    fn silent_and_empty_inputs_are_unchanged() {
        let mut silence = [0.0, 0.0];
        assert_eq!(normalize_peak(&mut silence, 0.8), Ok(1.0));
        assert_eq!(normalize_peak(&mut [], 0.8), Ok(1.0));
        assert_eq!(silence, [0.0, 0.0]);
    }

    #[test]
    fn rejects_invalid_input_before_mutating() {
        let mut samples = [0.2, f32::NAN];
        assert_eq!(
            normalize_peak(&mut samples, 1.0),
            Err(AudioError::NonFiniteSample)
        );
        assert_eq!(samples[0], 0.2);
        assert!(samples[1].is_nan());
        assert_eq!(
            normalize_peak(&mut [0.2], 0.0),
            Err(AudioError::InvalidTargetPeak)
        );
    }

    #[test]
    fn reduces_peaks_above_full_scale() {
        let mut samples = [2.0, -1.0];
        assert_eq!(normalize_peak(&mut samples, 1.0), Ok(0.5));
        assert_eq!(samples, [1.0, -0.5]);
    }

    #[test]
    fn rejects_unrepresentable_gain_without_changing_samples() {
        let tiny = f32::from_bits(1);
        let mut samples = [tiny];
        assert_eq!(
            normalize_peak(&mut samples, 1.0),
            Err(AudioError::GainOutOfRange)
        );
        assert_eq!(samples, [tiny]);
        let mut large = [f32::MAX];
        assert_eq!(
            normalize_peak(&mut large, tiny),
            Err(AudioError::GainOutOfRange)
        );
        assert_eq!(large, [f32::MAX]);
    }
}
