#ifndef WASM_AUDIO_ENGINE_H
#define WASM_AUDIO_ENGINE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Experimental ABI. Status values may change before a stable release. */
enum ae_status {
    AE_OK = 0,
    AE_INVALID_ARGUMENT = 1,
    AE_INVALID_TARGET = 2,
    AE_NON_FINITE_SAMPLE = 3,
    AE_GAIN_OUT_OF_RANGE = 4,
    AE_PANIC = 5,
    AE_OUTPUT_PENDING = 6,
    AE_PROCESSING_ERROR = 7
};

/* Normalize sample_count interleaved f32 samples in place and write linear gain.
 * samples may be NULL only when sample_count is zero. Otherwise the caller must
 * provide an aligned, writable allocation containing sample_count floats.
 * output_gain must point to a separate aligned, writable float and must not alias
 * samples. Both allocations must remain valid and exclusively accessible for the
 * call. The function validates input before mutation; output_gain is written only
 * on success. Invalid foreign pointers remain undefined behavior. */
int32_t ae_normalize_pcm_peak(float *samples, size_t sample_count,
                              float target_peak, float *output_gain);


/* Opaque, single-threaded streaming resampler; experimental ABI. All pointers
 * must refer to live, correctly aligned allocations, remain valid for each call,
 * and not alias the handle or other writable arguments. Invalid pointers cause
 * undefined behavior. Only NULL/misalignment and lengths can be checked.
 * A successfully created handle must be destroyed exactly once. After AE_PANIC
 * or AE_PROCESSING_ERROR, destroy it; do not continue processing. */
typedef struct AeResampler AeResampler;

/* Write a new handle only on success. Rates, channels, and chunk size must be
 * supported by the engine. Input and output are interleaved f32 PCM. */
int32_t ae_resampler_create(uint32_t input_rate, uint32_t output_rate,
                            uint32_t channels, uint32_t chunk_frames,
                            AeResampler **output_handle);
/* Push at most 8192 complete frames. samples may be NULL for zero frames.
 * Drain all output before another push or finish, or AE_OUTPUT_PENDING results. */
int32_t ae_resampler_push(AeResampler *handle, const float *samples, size_t frames);
int32_t ae_resampler_finish(AeResampler *handle);
/* output holds capacity_frames * channels floats; NULL is allowed for zero
 * capacity. written_frames is required and is written only on success. */
int32_t ae_resampler_read(AeResampler *handle, float *output,
                          size_t capacity_frames, size_t *written_frames);
int32_t ae_resampler_destroy(AeResampler *handle);

#ifdef __cplusplus
}
#endif
#endif
