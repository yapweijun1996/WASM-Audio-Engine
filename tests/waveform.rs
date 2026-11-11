use wasm_audio_engine::waveform::{WaveformBuilder, WaveformError, WaveformPoint};

#[test]
fn summarizes_stereo_windows_across_chunks_and_flushes_the_tail() {
    let mut waveform = WaveformBuilder::new(2, 2).unwrap();
    assert!(waveform.process(&[0.2, -0.1]).unwrap().is_empty());
    assert_eq!(
        waveform.process(&[0.3, -0.4, 0.5, 0.6]).unwrap(),
        [WaveformPoint {
            start_frame: 0,
            end_frame: 2,
            min: -0.4,
            max: 0.3
        }]
    );
    assert_eq!(
        waveform.finish().unwrap(),
        Some(WaveformPoint {
            start_frame: 2,
            end_frame: 3,
            min: 0.5,
            max: 0.6
        })
    );
    assert_eq!(waveform.finish(), Err(WaveformError::AlreadyFinished));
}

#[test]
fn invalid_data_does_not_advance_the_timeline() {
    let mut waveform = WaveformBuilder::new(2, 1).unwrap();
    assert_eq!(
        waveform.process(&[0.0]),
        Err(WaveformError::IncompleteFrame)
    );
    assert_eq!(
        waveform.process(&[f32::NAN, 0.0]),
        Err(WaveformError::NonFiniteSample)
    );
    assert_eq!(
        waveform.process(&[-0.2, 0.3]).unwrap(),
        [WaveformPoint {
            start_frame: 0,
            end_frame: 1,
            min: -0.2,
            max: 0.3
        }]
    );
    assert_eq!(waveform.finish().unwrap(), None);
}
