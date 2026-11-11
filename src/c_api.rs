//! Experimental C ABI for PCM peak normalization and streaming resampling.

use crate::resample::{ResampleError, StreamingResampler};
use crate::{normalize_peak, AudioError};

const OK: i32 = 0;
const INVALID_ARGUMENT: i32 = 1;
const INVALID_TARGET: i32 = 2;
const NON_FINITE_SAMPLE: i32 = 3;
const GAIN_OUT_OF_RANGE: i32 = 4;
const PANIC: i32 = 5;
const OUTPUT_PENDING: i32 = 6;
const PROCESSING_ERROR: i32 = 7;

/// Opaque state owned by the C ABI.
pub struct AeResampler {
    inner: StreamingResampler,
    channels: usize,
    pending: Vec<f32>,
    read_offset: usize,
    finished: bool,
}

fn aligned<T>(pointer: *const T) -> bool {
    !pointer.is_null() && (pointer as usize).is_multiple_of(std::mem::align_of::<T>())
}

fn code_for_resample_error(error: ResampleError) -> i32 {
    match error {
        ResampleError::InvalidConfiguration
        | ResampleError::IncompleteFrame
        | ResampleError::NonFiniteSample => INVALID_ARGUMENT,
        ResampleError::AlreadyFinished
        | ResampleError::TimelineOverflow
        | ResampleError::Backend(_) => PROCESSING_ERROR,
    }
}

/// Create a streaming resampler. The caller must follow the C header contract.
///
/// # Safety
/// The caller must uphold the C header pointer, lifetime, and non-aliasing contract.
#[no_mangle]
pub unsafe extern "C" fn ae_resampler_create(
    input_rate: u32,
    output_rate: u32,
    channels: u32,
    chunk_frames: u32,
    output_handle: *mut *mut AeResampler,
) -> i32 {
    if !aligned(output_handle) {
        return INVALID_ARGUMENT;
    }
    let result = std::panic::catch_unwind(|| {
        StreamingResampler::new(
            input_rate,
            output_rate,
            channels as usize,
            chunk_frames as usize,
        )
    });
    match result {
        Ok(Ok(inner)) => {
            let handle = Box::new(AeResampler {
                inner,
                channels: channels as usize,
                pending: Vec::new(),
                read_offset: 0,
                finished: false,
            });
            unsafe { output_handle.write(Box::into_raw(handle)) };
            OK
        }
        Ok(Err(error)) => code_for_resample_error(error),
        Err(_) => PANIC,
    }
}

/// Push a bounded number of complete interleaved frames.
///
/// # Safety
/// The caller must uphold the C header pointer, lifetime, and non-aliasing contract.
#[no_mangle]
pub unsafe extern "C" fn ae_resampler_push(
    handle: *mut AeResampler,
    samples: *const f32,
    frames: usize,
) -> i32 {
    if !aligned(handle) || frames > 8192 {
        return INVALID_ARGUMENT;
    }
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let state = unsafe { &mut *handle };
        if state.finished {
            return PROCESSING_ERROR;
        }
        if state.read_offset < state.pending.len() {
            return OUTPUT_PENDING;
        }
        let Some(count) = frames.checked_mul(state.channels) else {
            return INVALID_ARGUMENT;
        };
        if count > isize::MAX as usize / std::mem::size_of::<f32>()
            || (count != 0 && !aligned(samples))
        {
            return INVALID_ARGUMENT;
        }
        let values = if count == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(samples, count) }
        };
        match state.inner.process(values) {
            Ok(output) => {
                if output.iter().any(|sample| !sample.is_finite()) {
                    state.finished = true;
                    return PROCESSING_ERROR;
                }
                state.pending = output;
                state.read_offset = 0;
                OK
            }
            Err(error) => {
                let code = code_for_resample_error(error);
                if code == PROCESSING_ERROR {
                    state.finished = true;
                }
                code
            }
        }
    }));
    match result {
        Ok(code) => code,
        Err(_) => {
            unsafe { (*handle).finished = true };
            PANIC
        }
    }
}

/// Flush the resampler once all pending output has been read.
///
/// # Safety
/// The caller must uphold the C header pointer, lifetime, and non-aliasing contract.
#[no_mangle]
pub unsafe extern "C" fn ae_resampler_finish(handle: *mut AeResampler) -> i32 {
    if !aligned(handle) {
        return INVALID_ARGUMENT;
    }
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let state = unsafe { &mut *handle };
        if state.finished {
            return PROCESSING_ERROR;
        }
        if state.read_offset < state.pending.len() {
            return OUTPUT_PENDING;
        }
        state.finished = true;
        match state.inner.finish() {
            Ok(output) => {
                if output.iter().any(|sample| !sample.is_finite()) {
                    return PROCESSING_ERROR;
                }
                state.pending = output;
                state.read_offset = 0;
                OK
            }
            Err(error) => code_for_resample_error(error),
        }
    }));
    match result {
        Ok(code) => code,
        Err(_) => {
            unsafe { (*handle).finished = true };
            PANIC
        }
    }
}

/// Read up to capacity_frames of interleaved output.
///
/// # Safety
/// The caller must uphold the C header pointer, lifetime, and non-aliasing contract.
#[no_mangle]
pub unsafe extern "C" fn ae_resampler_read(
    handle: *mut AeResampler,
    output: *mut f32,
    capacity_frames: usize,
    written_frames: *mut usize,
) -> i32 {
    if !aligned(handle) || !aligned(written_frames) {
        return INVALID_ARGUMENT;
    }
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let state = unsafe { &mut *handle };
        let Some(capacity) = capacity_frames.checked_mul(state.channels) else {
            return INVALID_ARGUMENT;
        };
        if capacity > isize::MAX as usize / std::mem::size_of::<f32>()
            || (capacity != 0 && !aligned(output))
        {
            return INVALID_ARGUMENT;
        }
        let count = capacity.min(state.pending.len() - state.read_offset);
        if count != 0 {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    state.pending.as_ptr().add(state.read_offset),
                    output,
                    count,
                )
            };
        }
        state.read_offset += count;
        unsafe { written_frames.write(count / state.channels) };
        OK
    }));
    match result {
        Ok(code) => code,
        Err(_) => {
            unsafe { (*handle).finished = true };
            PANIC
        }
    }
}

/// Destroy a handle exactly once; null or misaligned pointers are rejected.
///
/// # Safety
/// The caller must uphold the C header pointer, lifetime, and non-aliasing contract.
#[no_mangle]
pub unsafe extern "C" fn ae_resampler_destroy(handle: *mut AeResampler) -> i32 {
    if !aligned(handle) {
        return INVALID_ARGUMENT;
    }
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        drop(Box::from_raw(handle))
    }));
    match result {
        Ok(()) => OK,
        Err(_) => PANIC,
    }
}

/// Normalize an interleaved PCM buffer.
///
/// # Safety
/// The caller must uphold the C header's pointer, allocation, lifetime, and
/// non-aliasing requirements.
#[no_mangle]
pub unsafe extern "C" fn ae_normalize_pcm_peak(
    samples: *mut f32,
    sample_count: usize,
    target_peak: f32,
    output_gain: *mut f32,
) -> i32 {
    if output_gain.is_null()
        || !(output_gain as usize).is_multiple_of(std::mem::align_of::<f32>())
        || sample_count > isize::MAX as usize / std::mem::size_of::<f32>()
        || (sample_count != 0
            && (samples.is_null()
                || !(samples as usize).is_multiple_of(std::mem::align_of::<f32>())))
    {
        return INVALID_ARGUMENT;
    }
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let values: &mut [f32] = if sample_count == 0 {
            &mut []
        } else {
            // The foreign caller guarantees a writable, aligned allocation of
            // sample_count elements with no aliasing for the duration of this call.
            unsafe { std::slice::from_raw_parts_mut(samples, sample_count) }
        };
        normalize_peak(values, target_peak)
    }));
    match result {
        Ok(Ok(gain)) => {
            // The caller guarantees a valid, non-aliased output pointer.
            unsafe { output_gain.write(gain) };
            OK
        }
        Ok(Err(AudioError::InvalidTargetPeak)) => INVALID_TARGET,
        Ok(Err(AudioError::NonFiniteSample)) => NON_FINITE_SAMPLE,
        Ok(Err(AudioError::GainOutOfRange)) => GAIN_OUT_OF_RANGE,
        Err(_) => PANIC,
    }
}
