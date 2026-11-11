use wasm_audio_engine::silence::SilenceInterval;
use wasm_audio_engine::timeline::{
    build_trim_map, original_to_output, output_to_original, TimelineError,
};

#[test]
fn maps_retained_frames_around_removed_silence() {
    let removed = [
        SilenceInterval {
            start_frame: 2,
            end_frame: 5,
        },
        SilenceInterval {
            start_frame: 8,
            end_frame: 9,
        },
    ];
    let map = build_trim_map(10, &removed).unwrap();
    assert_eq!(output_to_original(&map, 0), Some(0));
    assert_eq!(output_to_original(&map, 2), Some(5));
    assert_eq!(output_to_original(&map, 5), Some(9));
    assert_eq!(output_to_original(&map, 6), None);
    assert_eq!(original_to_output(&map, 3), None);
    assert_eq!(original_to_output(&map, 9), Some(5));
}

#[test]
fn rejects_overlapping_or_out_of_range_intervals() {
    let bad = [
        SilenceInterval {
            start_frame: 2,
            end_frame: 6,
        },
        SilenceInterval {
            start_frame: 5,
            end_frame: 7,
        },
    ];
    assert_eq!(
        build_trim_map(10, &bad),
        Err(TimelineError::InvalidInterval)
    );
    assert!(build_trim_map(
        5,
        &[SilenceInterval {
            start_frame: 0,
            end_frame: 5
        }]
    )
    .unwrap()
    .is_empty());
    assert!(build_trim_map(0, &[]).unwrap().is_empty());
}
