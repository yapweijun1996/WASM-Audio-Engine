//! Mapping between original and silence-trimmed frame timelines.

use crate::silence::SilenceInterval;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeptSpan {
    pub original_start: u64,
    pub original_end: u64,
    pub output_start: u64,
    pub output_end: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineError {
    InvalidInterval,
}

/// Build half-open kept spans from sorted, non-overlapping intervals to remove.
pub fn build_trim_map(
    total_frames: u64,
    removed: &[SilenceInterval],
) -> Result<Vec<KeptSpan>, TimelineError> {
    let mut original = 0_u64;
    let mut output = 0_u64;
    let mut spans = Vec::new();
    for interval in removed {
        if interval.start_frame < original
            || interval.start_frame >= interval.end_frame
            || interval.end_frame > total_frames
        {
            return Err(TimelineError::InvalidInterval);
        }
        if interval.start_frame > original {
            let length = interval.start_frame - original;
            spans.push(KeptSpan {
                original_start: original,
                original_end: interval.start_frame,
                output_start: output,
                output_end: output + length,
            });
            output += length;
        }
        original = interval.end_frame;
    }
    if original < total_frames {
        let length = total_frames - original;
        spans.push(KeptSpan {
            original_start: original,
            original_end: total_frames,
            output_start: output,
            output_end: output + length,
        });
    }
    Ok(spans)
}

/// Map an output frame to its original frame. The output endpoint is excluded.
pub fn output_to_original(spans: &[KeptSpan], frame: u64) -> Option<u64> {
    spans
        .iter()
        .find(|span| frame >= span.output_start && frame < span.output_end)
        .and_then(|span| span.original_start.checked_add(frame - span.output_start))
}

/// Map a retained original frame to output; removed frames return None.
pub fn original_to_output(spans: &[KeptSpan], frame: u64) -> Option<u64> {
    spans
        .iter()
        .find(|span| frame >= span.original_start && frame < span.original_end)
        .and_then(|span| span.output_start.checked_add(frame - span.original_start))
}
