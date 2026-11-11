use std::io::Cursor;
use wasm_audio_engine::pipeline::{process_wav, PipelineError, PipelineOptions};
use wasm_audio_engine::wav::{Pcm16Reader, Pcm16Writer};

fn options() -> PipelineOptions {
    PipelineOptions {
        output_rate: 16_000,
        output_channels: 1,
        normalize_peak: Some(0.8),
        silence_threshold: 0.01,
        min_silence_frames: 100,
        waveform_window_frames: 1000,
        input_chunk_frames: 333,
    }
}

#[test]
fn prepares_stereo_wav_with_original_timeline_analysis() {
    let mut input = Cursor::new(Vec::new());
    {
        let mut writer = Pcm16Writer::new(&mut input, 48_000, 2).unwrap();
        let mut samples = vec![0.0_f32; 4800 * 2];
        for frame in 1200..3600 {
            let value = (frame as f32 * 0.08).sin() * 0.25;
            samples[frame * 2] = value;
            samples[frame * 2 + 1] = value;
        }
        writer.write_chunk(&samples).unwrap();
        writer.finish().unwrap();
    }
    input.set_position(0);
    let mut output = Cursor::new(Vec::new());
    let mut intervals = Vec::new();
    let mut points = Vec::new();
    let stats = process_wav(&mut input, &mut output, &options(), |silence, waveform| {
        intervals.extend_from_slice(silence);
        points.extend_from_slice(waveform);
    })
    .unwrap();
    assert_eq!(stats.input_frames, 4800);
    assert_eq!(stats.output_frames, 1600);
    assert!(stats.applied_gain > 3.1 && stats.applied_gain < 3.3);
    assert_eq!(points.first().unwrap().start_frame, 0);
    assert_eq!(points.last().unwrap().end_frame, 4800);
    assert!(intervals.first().unwrap().start_frame == 0);
    assert!(intervals.last().unwrap().end_frame == 4800);
    let mut reader = Pcm16Reader::new(Cursor::new(output.into_inner())).unwrap();
    assert_eq!(reader.spec().sample_rate, 16_000);
    assert_eq!(reader.spec().channels, 1);
    assert_eq!(reader.read_chunk(2000).unwrap().len(), 1600);
}

#[test]
fn rejects_invalid_options_before_writing_output() {
    let mut input = Cursor::new(Vec::new());
    let mut output = Cursor::new(Vec::new());
    let mut invalid = options();
    invalid.input_chunk_frames = 0;
    assert!(matches!(
        process_wav(&mut input, &mut output, &invalid, |_, _| {}),
        Err(PipelineError::InvalidOptions)
    ));
    assert!(output.into_inner().is_empty());
}
