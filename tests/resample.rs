use rubato::audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft, FixedSync, Resampler};
use wasm_audio_engine::resample::{ResampleError, StreamingResampler};

#[test]
fn chunked_resampling_matches_rubato_whole_buffer_reference() {
    let input: Vec<f32> = (0..4800).map(|i| ((i as f32) * 0.05).sin() * 0.5).collect();
    let adapter = InterleavedSlice::new(input.as_slice(), 1, input.len()).unwrap();
    let mut reference = Fft::<f32>::new(48_000, 16_000, 1024, 1, FixedSync::Input).unwrap();
    let expected = reference
        .process_all(&adapter, input.len(), None)
        .unwrap()
        .take_data();
    let mut streaming = StreamingResampler::new(48_000, 16_000, 1, 1024).unwrap();
    let mut actual = Vec::new();
    for chunk in input.chunks(333) {
        actual.extend(streaming.process(chunk).unwrap());
    }
    actual.extend(streaming.finish().unwrap());
    assert_eq!(actual.len(), expected.len());
    assert!(actual
        .iter()
        .zip(&expected)
        .all(|(a, b)| (a - b).abs() < 1e-6));
    assert!(matches!(
        streaming.finish(),
        Err(ResampleError::AlreadyFinished)
    ));
}

#[test]
fn handles_empty_short_and_invalid_inputs() {
    let mut empty = StreamingResampler::new(48_000, 16_000, 1, 1024).unwrap();
    assert!(empty.finish().unwrap().is_empty());
    let mut short = StreamingResampler::new(48_000, 16_000, 1, 1024).unwrap();
    assert!(short.process(&[0.0; 3]).unwrap().is_empty());
    assert_eq!(short.finish().unwrap().len(), 1);
    assert!(matches!(
        StreamingResampler::new(0, 16_000, 1, 1024),
        Err(ResampleError::InvalidConfiguration)
    ));
}

#[test]
fn stereo_and_rate_boundaries_match_whole_buffer_processing() {
    for (input_rate, output_rate) in [
        (44_100, 48_000),
        (16_000, 48_000),
        (48_000, 48_000),
        (48_000, 8_000),
    ] {
        for frames in [1, 1023, 1024, 1025, 5001] {
            let input: Vec<f32> = (0..frames * 2)
                .map(|i| (i as f32 * 0.031).sin() * 0.4)
                .collect();
            let adapter = InterleavedSlice::new(input.as_slice(), 2, frames).unwrap();
            let mut reference =
                Fft::<f32>::new(input_rate, output_rate, 1024, 2, FixedSync::Input).unwrap();
            let expected = reference
                .process_all(&adapter, frames, None)
                .unwrap()
                .take_data();
            let mut streaming =
                StreamingResampler::new(input_rate as u32, output_rate as u32, 2, 1024).unwrap();
            let mut actual = Vec::new();
            for part in input.chunks(2 * 137) {
                actual.extend(streaming.process(part).unwrap());
            }
            actual.extend(streaming.finish().unwrap());
            assert_eq!(
                actual.len(),
                expected.len(),
                "{input_rate} -> {output_rate}, {frames} frames"
            );
            assert!(
                actual
                    .iter()
                    .zip(&expected)
                    .all(|(a, b)| (a - b).abs() < 1e-6),
                "{input_rate} -> {output_rate}, {frames} frames"
            );
        }
    }
}

#[test]
fn downsampling_retains_passband_and_suppresses_above_nyquist() {
    fn output_rms(frequency: f32) -> f32 {
        let input: Vec<f32> = (0..48_000)
            .map(|i| (std::f32::consts::TAU * frequency * i as f32 / 48_000.0).sin() * 0.5)
            .collect();
        let mut resampler = StreamingResampler::new(48_000, 16_000, 1, 1024).unwrap();
        let mut output = Vec::new();
        for chunk in input.chunks(1000) {
            output.extend(resampler.process(chunk).unwrap());
        }
        output.extend(resampler.finish().unwrap());
        let interior = &output[512..output.len() - 512];
        (interior.iter().map(|sample| sample * sample).sum::<f32>() / interior.len() as f32).sqrt()
    }
    assert!((output_rms(1000.0) - 0.35355).abs() < 0.02);
    assert!(output_rms(12_000.0) < 0.01);
}

#[test]
fn preserves_impulse_timestamp_and_ceil_duration_across_chunks() {
    let frames = 48_001usize;
    let mut input = vec![0.0; frames];
    input[12_000] = 0.5;
    let mut resampler = StreamingResampler::new(48_000, 16_000, 1, 1024).unwrap();
    let mut output = Vec::new();
    for block in input.chunks(137) {
        output.extend(resampler.process(block).unwrap());
    }
    output.extend(resampler.finish().unwrap());
    assert_eq!(output.len(), 16_001);
    let peak = output
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
        .unwrap()
        .0;
    assert_eq!(peak, 4_000);
    assert_eq!(12_000.0 / 48_000.0, peak as f64 / 16_000.0);
    assert!(output[0].abs() < 1e-6);
    assert!(output[output.len() - 1].abs() < 1e-6);
    let duration_error = output.len() as f64 / 16_000.0 - frames as f64 / 48_000.0;
    assert!((0.0..1.0 / 16_000.0).contains(&duration_error));
}
