use wasm_audio_engine::silence::{SilenceDetector, SilenceError, SilenceInterval};

#[test]
fn detects_intervals_across_chunk_boundaries() {
    let mut detector = SilenceDetector::new(2, 0.1, 2).unwrap();
    assert!(detector.process(&[0.8, 0.0, 0.0, 0.1]).unwrap().is_empty());
    assert_eq!(
        detector.process(&[0.0, 0.0, 0.2, 0.0]).unwrap(),
        [SilenceInterval {
            start_frame: 1,
            end_frame: 3
        }]
    );
    assert!(detector.process(&[0.0, 0.0, 0.0, 0.0]).unwrap().is_empty());
    assert_eq!(
        detector.finish().unwrap(),
        Some(SilenceInterval {
            start_frame: 4,
            end_frame: 6
        })
    );
    assert_eq!(detector.finish(), Err(SilenceError::AlreadyFinished));
}

#[test]
fn ignores_short_intervals_and_preserves_state_after_invalid_input() {
    let mut detector = SilenceDetector::new(1, 0.0, 2).unwrap();
    assert!(detector.process(&[0.0]).unwrap().is_empty());
    assert_eq!(
        detector.process(&[f32::NAN]),
        Err(SilenceError::NonFiniteSample)
    );
    assert_eq!(
        detector.process(&[1.0, 0.0, 0.0, 1.0]).unwrap(),
        [SilenceInterval {
            start_frame: 2,
            end_frame: 4
        }]
    );
    assert_eq!(detector.finish().unwrap(), None);
}

#[test]
fn rejects_invalid_configuration_and_partial_frames() {
    assert!(matches!(
        SilenceDetector::new(0, 0.1, 1),
        Err(SilenceError::InvalidConfiguration)
    ));
    assert!(matches!(
        SilenceDetector::new(1, f32::NAN, 1),
        Err(SilenceError::InvalidConfiguration)
    ));
    let mut detector = SilenceDetector::new(2, 0.1, 1).unwrap();
    assert_eq!(detector.process(&[0.0]), Err(SilenceError::IncompleteFrame));
    assert_eq!(detector.finish().unwrap(), None);
}
