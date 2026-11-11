#include "wasm_audio_engine.h"
#include <math.h>

int main(void) {
    float samples[] = {0.25f, -0.5f};
    float gain = -1.0f;
    if (ae_normalize_pcm_peak(samples, 2, 1.0f, &gain) != AE_OK) return 1;
    if (gain != 2.0f || samples[0] != 0.5f || samples[1] != -1.0f) return 2;
    if (ae_normalize_pcm_peak(NULL, 1, 1.0f, &gain) != AE_INVALID_ARGUMENT) return 3;
    if (ae_normalize_pcm_peak(NULL, 0, 1.0f, &gain) != AE_OK || gain != 1.0f) return 4;
    if (ae_normalize_pcm_peak(samples, 2, 1.0f, NULL) != AE_INVALID_ARGUMENT) return 7;
    float invalid[] = {0.25f, NAN};
    gain = -1.0f;
    if (ae_normalize_pcm_peak(invalid, 2, 1.0f, &gain) != AE_NON_FINITE_SAMPLE) return 5;
    if (invalid[0] != 0.25f || gain != -1.0f) return 6;
    AeResampler *resampler = NULL;
    if (ae_resampler_create(48000, 16000, 1, 1024, &resampler) != AE_OK) return 8;
    float input[1024] = {0};
    float output[1024];
    size_t total = 0, written = 0;
    for (size_t offset = 0; offset < 4800; offset += 1024) {
        size_t frames = 4800 - offset < 1024 ? 4800 - offset : 1024;
        if (ae_resampler_push(resampler, input, frames) != AE_OK) return 9;
        do {
            if (ae_resampler_read(resampler, output, 1024, &written) != AE_OK) return 10;
            total += written;
        } while (written != 0);
    }
    if (ae_resampler_finish(resampler) != AE_OK) return 11;
    do {
        if (ae_resampler_read(resampler, output, 1024, &written) != AE_OK) return 12;
        total += written;
    } while (written != 0);
    if (total != 1600) return 13;
    if (ae_resampler_push(resampler, input, 1) != AE_PROCESSING_ERROR) return 14;
    if (ae_resampler_destroy(resampler) != AE_OK) return 15;
    return 0;
}
