# Experimental API Contract

Version: `API_VERSION = "0.1.0"`. This freezes the currently documented experimental interface; it is not a production stability or cross-browser guarantee. Changes to signatures, frame semantics, supported formats, or error codes require an explicit version change and updated contract tests.

## Entry point and types

`createAudioEngine(bindings)` initializes local wasm-bindgen web bindings and returns an `AudioEngine`. Import the wrapper from `wasm-audio-engine` and generated bindings from `wasm-audio-engine/wasm`. Public option/result types are exported from the package root, with declarations generated in `dist/`.

- `normalizePeak(samples, targetPeak)` mutates interleaved Float32 PCM and returns linear gain. Silence returns 1; target peak is in (0, 1].
- `spectrum(samples, sampleRate)` analyzes a bounded mono window and returns `{ frequencyHz, amplitude }` bins. It applies a Hann window.
- `processWav(input, output, options)` reads classic PCM16 RIFF WAV, analyzes original source frames, optionally normalizes, converts mono/stereo, resamples, and writes PCM16 WAV. The result includes source/output rates and frame counts, applied gain, and saturated sample count.
- `chunkWav(input, options)` is an async iterator of `{ startFrame, endFrame, sampleRate, channels, audio }` chunks. Overlap is explicit. Each payload is limited to 64 MiB.
- `extractWav` uses a half-open source-frame range. `extractWavSeconds` rounds a non-empty seconds range outward to source frames. Excerpts preserve source format, with a 64 MiB payload cap.
- `mergeWav(inputs, output, options)` joins matching sample rates and channel counts. Overlap is not deduplicated.
- `trimWav(input, intervals, output, options)` removes sorted, non-overlapping source-frame intervals and returns retained source/output mapping spans.

`WavProcessOptions`, `WavProcessResult`, chunk/excerpt options, merge options/results, and trim interval/mapping types in `web/src/wav.ts` are the authoritative field definitions. Callbacks use original source-frame positions; divide by input sample rate for seconds. Output timestamps use output frames divided by output rate. Resampling emits ceil(inputFrames * outputRate / inputRate) frames; duration rounding is less than one output frame.

## Error contract version 1

SDK validation and DSP failures use `AudioEngineError`, with `code`, `contractVersion`, `message`, and optional `cause`:

| Code | Meaning |
| --- | --- |
| `INVALID_ARGUMENT` | Invalid processing configuration or range |
| `INVALID_AUDIO` | Missing, truncated, or inconsistent WAV structure |
| `UNSUPPORTED_FORMAT` | Unsupported PCM format or incompatible merge inputs |
| `LIMIT_EXCEEDED` | WAV size, excerpt, chunk, or metadata limit exceeded |
| `DSP_ERROR` | Failure at the WASM DSP boundary or invalid DSP output |
| `INITIALIZATION_FAILED` | Loading or initializing supplied bindings failed |

Branch on `code`, never on message text or the internal WASM cause. Caller stream, file-read, and callback errors propagate unchanged. Cancellation propagates `AbortSignal.reason`, or AbortError if no reason is supplied. Checks happen between blocks; blocked output writes cannot always be interrupted. Errors can leave partial destination output; callers own rollback. No automatic upload, paid request, or retry is performed.
