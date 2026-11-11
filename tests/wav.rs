use std::io::Cursor;
use wasm_audio_engine::wav::{Pcm16Reader, Pcm16Writer, WavError};

#[test]
fn writes_and_reads_stereo_in_bounded_chunks() {
    let mut output = Cursor::new(Vec::new());
    {
        let mut writer = Pcm16Writer::new(&mut output, 48_000, 2).unwrap();
        writer.write_chunk(&[0.5, -0.5, 0.0, 0.25]).unwrap();
        writer.write_chunk(&[-1.0, 0.0]).unwrap();
        writer.finish().unwrap();
    }
    let mut reader = Pcm16Reader::new(Cursor::new(output.into_inner())).unwrap();
    assert_eq!(reader.spec().sample_rate, 48_000);
    assert_eq!(reader.spec().channels, 2);
    assert_eq!(reader.read_chunk(2).unwrap(), [0.5, -0.5, 0.0, 0.25]);
    assert_eq!(reader.read_chunk(2).unwrap(), [-1.0, 0.0]);
    assert!(reader.read_chunk(2).unwrap().is_empty());
    assert!(matches!(
        reader.read_chunk(0),
        Err(WavError::InvalidParameter)
    ));
}

#[test]
fn rejects_invalid_samples_and_incomplete_frames_before_writing() {
    let mut output = Cursor::new(Vec::new());
    {
        let mut writer = Pcm16Writer::new(&mut output, 16_000, 2).unwrap();
        assert!(matches!(
            writer.write_chunk(&[0.5]),
            Err(WavError::IncompleteFrame)
        ));
        assert!(matches!(
            writer.write_chunk(&[0.0, f32::NAN]),
            Err(WavError::InvalidSample)
        ));
        assert!(matches!(
            writer.write_chunk(&[0.0, 1.1]),
            Err(WavError::InvalidSample)
        ));
        writer.write_chunk(&[0.0, 0.5]).unwrap();
        writer.finish().unwrap();
    }
    let mut reader = Pcm16Reader::new(Cursor::new(output.into_inner())).unwrap();
    assert_eq!(reader.read_chunk(2).unwrap(), [0.0, 0.5]);
}

#[test]
fn rejects_other_wav_sample_formats() {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16_000,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut output = Cursor::new(Vec::new());
    hound::WavWriter::new(&mut output, spec)
        .unwrap()
        .finalize()
        .unwrap();
    assert!(matches!(
        Pcm16Reader::new(Cursor::new(output.into_inner())),
        Err(WavError::UnsupportedFormat)
    ));
}
