# WASM Audio Engine

> **Status: initial development.** Experimental Rust and browser paths support PCM16 WAV preparation, normalization, low-energy detection, waveform summaries, mono/stereo conversion, resampling, chunking, extraction, interval trimming, WAV merging, and mono spectrum analysis. Supported formats and tested environments remain limited; no package has been published.

WASM Audio Engine is an experimental lightweight, offline-first audio processing SDK. Its Rust core provides reusable audio processing through a TypeScript API and WebAssembly bindings for the web. Native Android, iOS, and desktop integrations are future goals and require their own bindings and verification.

The project aims to combine suitable existing libraries behind a consistent interface rather than reimplement every DSP algorithm. It is not intended to replace all of FFmpeg.

## Planned capabilities

- Detect low-energy intervals and generate waveform data for audio navigation.
- Trim and extract segments, merge compatible audio, and normalize peak volume.
- Process long recordings in chunks and resample PCM audio.
- Convert channels and export supported audio formats, starting with WAV as an MVP candidate.
- Analyze the frequency spectrum of a bounded mono PCM window.

Silence detection is not voice activity detection (VAD). The engine does not transcribe speech or identify speakers. Transcript synchronization and speaker comparison are application workflows built on timestamps, waveform data, and extracted clips.

## Intended API

The following is an illustrative proposal. Input, output, processing order, and options remain to be designed and tested.

```ts
const result = await audioEngine.process(audioFile, {
  normalize: true,
  trimSilence: true,
  sampleRate: 16000,
});
```

The experimental implemented browser API is `createAudioEngine(bindings)` followed by `engine.processWav(inputBlob, outputStream, options)`. It supports classic RIFF WAV with format code 1, mono/stereo integer PCM16 at 8–192 kHz, and writes to a caller-provided `WritableStream`; its analysis callbacks report source-frame positions. Cancellation may leave partial output, and the caller owns the destination and rollback. The source remains local unless the caller deliberately supplies a network-backed destination.

```ts
import * as bindings from './generated/wasm_audio_engine.js';
import { createAudioEngine } from './dist/index.js';

const engine = await createAudioEngine(bindings);
const result = await engine.processWav(inputFile, outputStream, {
  outputSampleRate: 16000,
  outputChannels: 1,
  normalizePeak: 0.8,
  silenceThreshold: 0.01,
  minSilenceFrames: 4800,
  waveformWindowFrames: 4800,
  onSilence: (startFrame, endFrame) => console.log(startFrame, endFrame),
});
```

Generate the bindings with `npm run build:wasm` using the matching `wasm-bindgen` CLI, then run `npm run build`. For a locally packed SDK, import the wrapper from `wasm-audio-engine` and bindings from `wasm-audio-engine/wasm`. `inputFile` and `outputStream` are supplied by the application. The thresholds and callback positions above use input frames; divide positions by the input sample rate to obtain seconds. Normalization uses the input peak and performs a separate read pass. The result reports how many output samples were saturated when encoding PCM16. `processWav` does not automatically remove silence or produce transcript timestamps.

`engine.chunkWav(inputFile, { chunkFrames, overlapFrames })` yields WAV `Blob` chunks with source-frame start and end positions. Each chunk is capped at a 64 MiB audio payload; callers own upload, retry, transcript alignment, and storage behavior.

`engine.extractWav` accepts a half-open frame range; `engine.extractWavSeconds` accepts seconds and rounds non-empty ranges outward to whole source frames. Both return a source-format WAV excerpt, limited to a 64 MiB audio payload.

`engine.mergeWav(inputs, outputStream)` streams compatible PCM16 WAV inputs in order without re-encoding. Inputs must have the same sample rate and channel count; overlapping input ranges are not deduplicated. The caller owns the output destination and any rollback of partial output.

`engine.spectrum(samples, sampleRate)` returns one-sided frequency and peak-amplitude bins for a mono PCM window of 4–65,536 normalized samples. It uses a Hann window; it is not a power spectral density measurement.

`engine.trimWav(input, intervals, outputStream)` explicitly removes sorted, non-overlapping source-frame intervals and returns retained spans mapping original to output frames at the original sample rate. It accepts at most 100,000 intervals and retains mapping metadata in memory. Silence detection is not VAD; callers decide whether detected intervals are safe to remove.

See [the versioned experimental API contract](docs/api.md) for exported types, error codes, cancellation, and timeline semantics. Run `npm test` for SDK error-contract checks.

## Development and contributions

Rust primitives and the native pipeline are in `src/`; the TypeScript browser SDK is in `web/src/`; focused tests are in `tests/`; the bounded browser example is in `examples/`; and design and validation notes are in `docs/`. `dist/` and `target/` are generated build outputs.

With a Rust toolchain and native linker installed, run `cargo test --locked` for the native tests, `cargo clippy --all-targets --locked -- -D warnings` for static checks, and `cargo build --locked --target wasm32-unknown-unknown` to build the core for WebAssembly. The WASM target must be installed separately. Run `npm ci`, then `npm run format:check`, `npm run typecheck`, and `npm run build` for the TypeScript wrapper. For the experimental browser smoke test, use a `wasm-bindgen` CLI matching the version in `Cargo.lock`, run `wasm-bindgen --target web --out-dir target/browser target/wasm32-unknown-unknown/debug/wasm_audio_engine.wasm`, serve the repository root on localhost, and open `/tests/browser-smoke.html`. See [DESIGN.md](DESIGN.md) for proposed scope and [TASK.md](TASK.md) for progress.

After `npm run build:wasm` and `npm run build`, serve the repository root locally and open `/examples/`. The local test page includes a generated sample and supports playback, normalization, resampling, waveform and quiet-interval inspection, extraction, interval removal, chunking, merging, and spectrum analysis. It limits each input to 8 MiB and buffered output to 32 MiB; use `processWav` with a streaming destination for larger files. Run `python -m http.server 18786 --bind 127.0.0.1` from the repository root and open `http://127.0.0.1:18786/examples/`.

## Licensing

The project is licensed under the [MIT License](LICENSE), which permits commercial use subject to its terms. See [DEPENDENCIES.md](DEPENDENCIES.md) and the generated [third-party license inventory](THIRD_PARTY_NOTICES.txt). Before release, review exact dependency versions, transitive dependencies, required license notices, and applicable codec patent considerations. Codec support is not supplied automatically by WebAssembly.
