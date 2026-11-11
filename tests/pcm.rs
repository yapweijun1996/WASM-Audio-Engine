use wasm_audio_engine::pcm::{convert_channels, extract_frames, PcmError};

#[test]
fn extracts_stereo_frame_ranges() {
    let samples = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6];
    assert_eq!(
        extract_frames(&samples, 2, 1, 3).unwrap(),
        &[0.3, 0.4, 0.5, 0.6]
    );
    assert_eq!(extract_frames(&samples, 2, 3, 3).unwrap(), &[]);
    assert_eq!(
        extract_frames(&samples, 2, 2, 4),
        Err(PcmError::InvalidRange)
    );
}

#[test]
fn converts_mono_and_stereo_with_explicit_averaging() {
    assert_eq!(
        convert_channels(&[0.5, -0.5], 1, 2).unwrap(),
        [0.5, 0.5, -0.5, -0.5]
    );
    assert_eq!(
        convert_channels(&[1.0, -1.0, 0.5, 0.5], 2, 1).unwrap(),
        [0.0, 0.5]
    );
    assert_eq!(
        convert_channels(&[0.1], 2, 1),
        Err(PcmError::IncompleteFrame)
    );
    assert_eq!(
        convert_channels(&[f32::NAN], 1, 1),
        Err(PcmError::NonFiniteSample)
    );
    assert_eq!(
        convert_channels(&[], 3, 1),
        Err(PcmError::UnsupportedChannels)
    );
}
