use wasm_audio_engine::spectrum::{spectrum, SpectrumError};

#[test]
fn locates_a_sine_tone_and_estimates_its_amplitude() {
    let samples: Vec<f32> = (0..1024)
        .map(|i| (std::f32::consts::TAU * 1000.0 * i as f32 / 16_000.0).sin() * 0.5)
        .collect();
    let bins = spectrum(&samples, 16_000).unwrap();
    let peak = bins
        .iter()
        .max_by(|a, b| a.amplitude.total_cmp(&b.amplitude))
        .unwrap();
    assert_eq!(peak.frequency_hz, 1000.0);
    assert!((peak.amplitude - 0.5).abs() < 0.002);
}

#[test]
fn rejects_invalid_windows_and_samples() {
    assert_eq!(
        spectrum(&[0.0; 3], 16_000),
        Err(SpectrumError::InvalidLength)
    );
    assert_eq!(
        spectrum(&[0.0; 4], 0),
        Err(SpectrumError::InvalidSampleRate)
    );
    assert_eq!(
        spectrum(&[0.0, 0.0, f32::NAN, 0.0], 16_000),
        Err(SpectrumError::InvalidSample)
    );
}
