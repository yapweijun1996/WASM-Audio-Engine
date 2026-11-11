use wasm_audio_engine::chunk::{AudioChunk, ChunkError, Chunker};

#[test]
fn preserves_offsets_and_overlap_across_input_calls() {
    let mut chunker = Chunker::new(1, 4, 2).unwrap();
    assert!(chunker.process(&[0.0, 1.0, 2.0]).unwrap().is_empty());
    assert_eq!(
        chunker.process(&[3.0, 4.0, 5.0]).unwrap(),
        [
            AudioChunk {
                start_frame: 0,
                end_frame: 4,
                samples: vec![0.0, 1.0, 2.0, 3.0]
            },
            AudioChunk {
                start_frame: 2,
                end_frame: 6,
                samples: vec![2.0, 3.0, 4.0, 5.0]
            },
        ]
    );
    assert_eq!(chunker.finish().unwrap(), None);
    assert_eq!(chunker.finish(), Err(ChunkError::AlreadyFinished));
}

#[test]
fn emits_partial_tail_only_when_it_contains_new_frames() {
    let mut chunker = Chunker::new(2, 3, 1).unwrap();
    let full = chunker
        .process(&[0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0])
        .unwrap();
    assert_eq!((full[0].start_frame, full[0].end_frame), (0, 3));
    assert_eq!(
        chunker.finish().unwrap(),
        Some(AudioChunk {
            start_frame: 2,
            end_frame: 4,
            samples: vec![2.0, 2.0, 3.0, 3.0]
        })
    );
}

#[test]
fn rejects_bad_input_without_advancing() {
    assert!(matches!(
        Chunker::new(1, 4, 4),
        Err(ChunkError::InvalidConfiguration)
    ));
    let mut chunker = Chunker::new(2, 2, 0).unwrap();
    assert_eq!(chunker.process(&[0.0]), Err(ChunkError::IncompleteFrame));
    assert_eq!(
        chunker.process(&[f32::NAN, 0.0]),
        Err(ChunkError::NonFiniteSample)
    );
    assert!(chunker.process(&[0.0, 1.0]).unwrap().is_empty());
    assert_eq!(
        chunker.finish().unwrap(),
        Some(AudioChunk {
            start_frame: 0,
            end_frame: 1,
            samples: vec![0.0, 1.0]
        })
    );
}
