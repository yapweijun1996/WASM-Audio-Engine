# Design and MVP Plan

**Status: proposed.** Rust PCM processing primitives and experimental native and browser PCM16 WAV preparation paths are under development. The wider design, real-recording performance, and platform support remain unverified. Decisions and acceptance targets must be verified against actual code and runtime evidence as the project develops.

## Goals and boundaries

Provide a simple, consistent SDK for local audio preparation and navigation, particularly long meeting recordings. The core should support use without uploading audio; an application may separately upload audio to a speech-to-text provider when the user chooses that workflow.

The engine processes audio. It does not perform transcription, infer speaker identity, or promise that low-energy regions contain no speech. Applications own transcript timestamps, navigation UI, upload and retry policy, and user decisions about speakers. Integrating with a specific application is outside this repository?s scope. The project is a focused audio SDK, not a complete multimedia framework.

## Proposed architecture

1. **Input and decoding:** inspect the container and codec, validate input, and decode supported formats into PCM. Unsupported formats return a clear error.
2. **PCM processing:** represent sample rate, channel layout, sample format, and timeline explicitly. Apply requested operations in a documented order.
3. **Output:** encode supported formats and return audio, analysis data, metadata, or chunks with original timeline offsets.
4. **Bindings:** expose the Rust core to a TypeScript web SDK through WebAssembly; investigate native bindings separately for Android, iOS, and desktop.

## Experimental PCM and Browser Contract

PCM is represented as interleaved `f32` samples with an explicit channel count and sample rate. PCM16 decoding divides signed samples by 32,768. The browser WAV path accepts classic RIFF format-code-1 PCM16 with one or two channels and an 8–192 kHz sample rate. Mono-to-stereo duplicates samples; stereo-to-mono averages the two channels. Analysis, resampling, and channel conversion reject non-finite inputs. The browser and native preparation pipelines saturate output outside [-1, 1] and report saturated samples; the lower-level Rust PCM16 writer rejects out-of-range samples.

Frame ranges are non-negative, half-open intervals on the original input timeline. Convert frames to seconds using the input sample rate. Waveform min/max values combine the input channels. Silence detection considers a frame silent only when every channel is at or below the configured linear-amplitude threshold for the minimum number of consecutive frames; it is not voice detection.

For `processWav`, the browser validates options and the input header, optionally scans the source peak, then reads bounded PCM blocks. Source-timeline silence and waveform analysis happen before channel conversion, gain, resampling, and PCM16 encoding. The result records rates, channel count, silence settings, frame counts, gain, and saturation count. The path does not automatically remove silence. `trimWav` separately removes explicitly supplied source-frame intervals and returns retained-span mapping at the input rate.

The browser defaults to 8,192 input frames per block and accepts 1–65,536. The input is a caller-provided `Blob`, and encoded output is written with backpressure to a caller-provided `WritableStream`; the SDK does not itself upload audio. The caller controls whether the Blob or output destination buffers data or transmits it. Normalization reads the input twice. Extraction and individual chunks are limited to 64 MiB of audio payload; trim mapping is limited to 100,000 intervals; output must fit the 32-bit RIFF size. WAV header scanning permits at most 1,024 chunks.

The experimental 0.1.0 API uses versioned SDK error codes through `AudioEngineError`; `docs/api.md` defines the authoritative error and caller-failure contract. Cancellation uses `AbortSignal` between reads and writes; it cannot interrupt a synchronous WASM call or force a stalled destination write to finish. Output and callbacks may already have side effects when a failure or cancellation occurs; the caller owns transactional storage and rollback. These contracts are experimental and may change before a stable release.

Streaming and bounded memory are design goals across the full decode-process-encode path, not consequences of merely splitting a decoded buffer. The experimental native `process_wav` path handles PCM16 WAV with bounded input blocks, performs a second pass when normalizing, reports analysis in source frames, and records output samples saturated to PCM16 range. It requires seekable input and a distinct empty seekable output; callers must handle partial output and callback effects on error. It does not trim silence. Browser storage and processing must remain local unless an application explicitly chooses otherwise.

The experimental fixed-rate resampler uses Rubato's FFT resampler, discards the reported startup delay, and flushes a final partial block with silence. It returns `ceil(input_frames * output_rate / input_rate)` frames after finalization. Each caller must bound input blocks and consume outputs rather than collect an entire long recording in memory. Tests compare several chunk boundaries and rates with Rubato's whole-buffer path and check a passband tone and an above-Nyquist tone; these are not exhaustive audio-quality certification.

## Functional requirements

| Capability | Engine responsibility | Important boundary |
| --- | --- | --- |
| Silence detection | Return low-energy intervals with thresholds and timestamps. | Background noise and quiet speech can cause errors; this is not VAD. Removing intervals requires an explicit mapping to the original timeline. |
| Normalization | Apply a documented gain, initially considering peak normalization. | It does not repair clipping or noise or equalize speakers. All-silent input must be handled safely. |
| Chunking | Produce bounded audio chunks with original offsets and configurable overlap. | Applications own uploads, retries, transcript stitching, and words crossing boundaries. |
| Resampling | Convert PCM sample rates with appropriate filtering and documented delay handling. | The experimental Rust wrapper accepts 8–192 kHz, 1–8 channels, and 256–8,192-frame internal blocks; callers must bound each input call. Lower sample rates do not inherently improve quality or reduce compressed file size. |
| Waveform | Produce time-indexed amplitude summaries. | Transcript synchronization requires timestamps supplied by the application. |
| Clip extraction | Extract audio for review, preserving time and channel semantics. | Playback and speaker identity decisions belong to the application. |
| Export | Validate format, convert channels and rate, and encode supported output. | WAV is an MVP candidate; MP3, AAC, and Opus require separate codec support and review. |
| Merge and FFT | Merge compatible clips and produce frequency-domain analysis. | Compatible PCM16 merging and bounded mono FFT are implemented with the contracts in `docs/api.md`. Waveform summaries are distinct from FFT spectra. |

## Initial MVP proposal

Start with WAV input and output, peak normalization, silence interval detection, resampling, waveform summaries, and clip extraction. Validate a complete browser workflow with a representative one-hour recording before claiming long-recording support. Define processing order, channel conversion, time units, output format, error behavior, cancellation, and resource limits before stabilizing the public API. Merge, FFT, and native SDKs can follow as independently scoped milestones.

## Candidate components

| Candidate | Potential role | Published license | Evaluation needed |
| --- | --- | --- | --- |
| [wasm-bindgen](https://github.com/wasm-bindgen/wasm-bindgen) | Rust–JavaScript bindings | MIT OR Apache-2.0 | Browser packaging and error handling. |
| [Rubato](https://github.com/HEnquist/rubato) | Resampling | MIT OR Apache-2.0 | WASM size, quality, delay, and chunk boundaries. |
| [Hound](https://github.com/ruuda/hound) | WAV PCM I/O | Apache-2.0 | WASM compatibility, WAV variants, and streaming behavior. |
| [RustFFT](https://github.com/ejmahler/RustFFT) | FFT | MIT OR Apache-2.0 | Target compatibility, SIMD, size, and overlap with other candidates. |
| [tpt-dsp](https://github.com/tpt-solutions/tpt-dsp) | DSP primitives | MIT OR Apache-2.0 | Relevant modules, quality, compatibility, and dependency footprint. |

[Symphonia](https://github.com/pdeljanov/Symphonia) is a separate decoding candidate under MPL-2.0; its codec and target support must be reviewed for the selected version. [ffmpeg.wasm](https://github.com/ffmpegwasm/ffmpeg.wasm) is a broader comparison point, not a planned core dependency. No candidate is approved solely by inclusion in this table.

## Verification and release gates

- Test empty, silent, clipped, malformed, multi-channel, short, and long inputs; check sample counts, timestamp offsets, segment boundaries, clipping, and output format.
- Compare resampling and chunked output against appropriate reference signals, including the first and last frames and boundaries between chunks.
- Measure peak memory, elapsed time, and packaged size on named browsers and devices using reproducible inputs. Native tests alone do not establish web or mobile support.
- Verify that local processing performs no audio upload; separately test any host application network behavior.
- Audit exact direct and transitive dependency versions, license obligations, notices, and codec patent considerations before distribution. The project license is MIT; dependencies retain their own terms.

## Open decisions

Public API and processing order; supported input/output formats and WAV variants; mono conversion rules; silence thresholds and timeline mapping; chunk overlap and maximum memory targets; browser and mobile support matrix; and which candidate dependencies satisfy measured requirements.
