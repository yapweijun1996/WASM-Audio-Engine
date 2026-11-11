# Design and MVP Plan

**Status: proposed.** Nothing in this document asserts implemented functionality, measured performance, platform support, or a selected dependency. Decisions and acceptance targets must be verified against actual code and runtime evidence as the project develops.

## Goals and boundaries

Provide a simple, consistent SDK for local audio preparation and navigation, particularly long meeting recordings. The core should support use without uploading audio; an application may separately upload audio to a speech-to-text provider when the user chooses that workflow.

The engine processes audio. It does not perform transcription, infer speaker identity, or promise that low-energy regions contain no speech. MeetingMemo owns transcript timestamps, navigation UI, upload and retry policy, and user decisions about speakers. The project is a focused audio SDK, not a complete multimedia framework.

## Proposed architecture

1. **Input and decoding:** inspect the container and codec, validate input, and decode supported formats into PCM. Unsupported formats return a clear error.
2. **PCM processing:** represent sample rate, channel layout, sample format, and timeline explicitly. Apply requested operations in a documented order.
3. **Output:** encode supported formats and return audio, analysis data, metadata, or chunks with original timeline offsets.
4. **Bindings:** expose the Rust core to a TypeScript web SDK through WebAssembly; investigate native bindings separately for Android, iOS, and desktop.

Streaming and bounded memory are design goals across the full decode-process-encode path, not consequences of merely splitting a decoded buffer. Operations requiring global information may need a second pass or bounded temporary storage. Browser storage and processing must remain local unless an application explicitly chooses otherwise.

## Functional requirements

| Capability | Engine responsibility | Important boundary |
| --- | --- | --- |
| Silence detection | Return low-energy intervals with thresholds and timestamps. | Background noise and quiet speech can cause errors; this is not VAD. Removing intervals requires an explicit mapping to the original timeline. |
| Normalization | Apply a documented gain, initially considering peak normalization. | It does not repair clipping or noise or equalize speakers. All-silent input must be handled safely. |
| Chunking | Produce bounded audio chunks with original offsets and configurable overlap. | Applications own uploads, retries, transcript stitching, and words crossing boundaries. |
| Resampling | Convert PCM sample rates with appropriate filtering and documented delay handling. | Lower sample rates do not inherently improve quality or reduce compressed file size. |
| Waveform | Produce time-indexed amplitude summaries. | Transcript synchronization requires timestamps supplied by the application. |
| Clip extraction | Extract audio for review, preserving time and channel semantics. | Playback and speaker identity decisions belong to the application. |
| Export | Validate format, convert channels and rate, and encode supported output. | WAV is an MVP candidate; MP3, AAC, and Opus require separate codec support and review. |
| Merge and FFT | Merge compatible clips and produce frequency-domain analysis. | Detailed behavior and initial release scope remain open. Waveform summaries are distinct from FFT spectra. |

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
- Audit exact direct and transitive dependency versions, license obligations, notices, and codec patent considerations before distribution. Choose and publish the project's own license before claiming an open-source release.

## Open decisions

Public API and processing order; supported input/output formats and WAV variants; mono conversion rules; silence thresholds and timeline mapping; chunk overlap and maximum memory targets; browser and mobile support matrix; project license; and which candidate dependencies satisfy measured requirements.
