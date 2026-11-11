//! Synthetic long-recording smoke test. Run explicitly with `cargo test --test long_recording -- --ignored`.

use std::io::{Cursor, Read};
use wasm_audio_engine::chunk::Chunker;
use wasm_audio_engine::resample::StreamingResampler;
use wasm_audio_engine::silence::SilenceDetector;
use wasm_audio_engine::wav::Pcm16Reader;
use wasm_audio_engine::waveform::WaveformBuilder;

#[test]
#[ignore = "processes a synthetic hour of PCM; run explicitly for long-recording evidence"]
fn processes_an_hour_of_synthetic_silence_in_bounded_input_blocks() {
    const RATE: u32 = 16_000;
    const FRAMES: u64 = RATE as u64 * 60 * 60;
    let data_bytes = (FRAMES * 2) as u32;
    let mut header = Vec::new();
    header.extend_from_slice(b"RIFF");
    header.extend_from_slice(&(36 + data_bytes).to_le_bytes());
    header.extend_from_slice(b"WAVEfmt ");
    header.extend_from_slice(&16_u32.to_le_bytes());
    header.extend_from_slice(&1_u16.to_le_bytes());
    header.extend_from_slice(&1_u16.to_le_bytes());
    header.extend_from_slice(&RATE.to_le_bytes());
    header.extend_from_slice(&(RATE * 2).to_le_bytes());
    header.extend_from_slice(&2_u16.to_le_bytes());
    header.extend_from_slice(&16_u16.to_le_bytes());
    header.extend_from_slice(b"data");
    header.extend_from_slice(&data_bytes.to_le_bytes());

    let source = Cursor::new(header).chain(std::io::repeat(0).take(data_bytes as u64));
    let mut reader = Pcm16Reader::new(source).unwrap();
    let mut silence = SilenceDetector::new(1, 0.0, 1).unwrap();
    let mut waveform = WaveformBuilder::new(1, RATE as u64).unwrap();
    let mut chunker = Chunker::new(1, RATE as usize, 0).unwrap();
    let mut resampler = StreamingResampler::new(RATE, 8_000, 1, 1024).unwrap();
    let mut chunks = 0;
    let mut points = 0;
    let mut resampled = 0;
    loop {
        let input = reader.read_chunk(8192).unwrap();
        if input.is_empty() {
            break;
        }
        assert!(silence.process(&input).unwrap().is_empty());
        points += waveform.process(&input).unwrap().len();
        chunks += chunker.process(&input).unwrap().len();
        resampled += resampler.process(&input).unwrap().len();
    }
    let interval = silence.finish().unwrap().unwrap();
    assert_eq!((interval.start_frame, interval.end_frame), (0, FRAMES));
    assert_eq!(points, 3600);
    assert_eq!(chunks, 3600);
    assert!(waveform.finish().unwrap().is_none());
    assert!(chunker.finish().unwrap().is_none());
    resampled += resampler.finish().unwrap().len();
    assert_eq!(resampled, FRAMES as usize / 2);
}
