# Tasks

**Status:** Experimental Web/WASM test handoff verified in Chrome; see `docs/validation.md`. Unchecked tasks remain incomplete. Order and scope may change after technical validation.

## 1. Project Foundation

- [x] Select and publish the MIT project license. LICENSE was committed and pushed to GitHub main as `ed0f69a88418b871008ca4274ff89df5c4488975`; the published file was read back and matched local LICENSE. Third-party obligations remain documented separately.
- [x] Initialize the Rust project and TypeScript SDK structure. The SDK remains experimental and includes streamed PCM16 WAV processing and analysis.
- [x] Configure local formatting, static checks, builds, and tests. Rust and TypeScript commands and browser smoke-test instructions are documented in `README.md`.
- [x] Define the experimental supported baseline in `docs/support.md`: Rust 1.99.1, wasm-bindgen CLI 0.2.130, Node.js 25.2.1, and Chrome 157 on Windows. Other targets remain unverified.
- [x] Establish dependency inventory and license review procedures (see `DEPENDENCIES.md`). Release-specific review and notices remain open.

## 2. Technical Validation and Design

- [x] Validate Hound, Rubato, and wasm-bindgen on `wasm32-unknown-unknown`. Hound PCM16 read/write passed Chrome in a dedicated `validation` feature build; Rubato and the normal bindings passed browser regressions. See `tests/hound-wasm.html` and `docs/validation.md`.
- [x] Measure the current candidate runtime assets, processing time, and sampled browser process-tree memory. Runtime assets total 407,250 bytes (sum of individual gzip sizes: 135,458 bytes). The supplied near-hour recording passed the isolated Chrome measurement; details and sampling limits are in `docs/validation.md`.
- [x] Evaluate RustFFT and tpt-dsp for feature overlap and actual needs. The initial, unbenchmarked assessment is in `docs/support.md`.
- [x] Define PCM representation, channel layout, sample rate, and time units for the experimental PCM16 path in `DESIGN.md`.
- [x] Define the experimental public API, processing order, errors, cancellation, and resource limits in `DESIGN.md`; stable error codes and broader formats remain future work.
- [x] Design bounded, end-to-end PCM16 processing without requiring a full decoded recording in memory. The caller controls Blob backing, output buffering, and callback storage; total browser process memory is not yet measured.

## 3. Audio Processing MVP

- [x] Read and validate the supported WAV subset. Native PCM16 reads and browser classic RIFF PCM16 validation have focused tests; other WAV variants are not claimed for the browser.
- [x] Export PCM16 WAV and verify its format. Native round-trip and Chrome output-header, length, and signal-presence checks pass.
- [x] Implement peak normalization, including silent and clipped input behavior. Rust and Chrome checks cover silent, over-range, and clipped inputs; normalization does not repair clipping.
- [x] Detect low-energy intervals and return thresholds and source-frame positions; this is not VAD. Native chunk-boundary and Chrome tests pass; the result includes the input rate for time conversion.
- [x] Resample PCM with filtering and startup-delay handling. The streaming Rust path matches Rubato's whole-buffer reference on tested rates and boundaries, passes signal checks, and ran in Chrome on a roughly 59-minute recording.
- [x] Generate time-indexed waveform amplitude summaries. Rust and Chrome paths report source-frame windows; tested limits are documented.
- [x] Trim and extract PCM16 WAV clips by frame or time range. Chrome tests check byte fidelity, bounds, and time rounding; excerpts are capped at 64 MiB.
- [x] Convert mono/stereo channels using documented duplication and averaging rules. Native and Chrome checks pass; other channel layouts are unsupported.
- [x] Output audio chunks with original offsets and configurable overlap. Rust PCM and browser WAV chunkers pass boundary tests; Chrome also checked offsets and sampled overlap on a roughly 59-minute recording. Integrating the SDK into a particular application is outside this repository's scope.
- [x] Provide a mapping to the original timeline when silence is removed. Rust mapping and Chrome PCM16 WAV trim tests cover retained spans, invalid overlaps, and fully removed audio.

## 4. TypeScript and Browser SDK

- [x] Wrap WASM initialization and expose an experimental TypeScript API. Experimental PCM normalization, streamed PCM16 WAV processing, trimming, chunking, extraction, merging, and spectrum analysis passed Chrome smoke tests; final package consumer validation passed; stable API design remains open.
- [x] Define exported audio, analysis, metadata, and error types. `AudioEngineError` exposes contract version 1 and stable SDK error codes; caller I/O, callback failures, and cancellation reasons propagate unchanged.
- [x] Specify local processing, progress reporting, and cancellation behavior, including output rollback and stalled-write limitations, in `DESIGN.md`.
- [x] Create a browser example demonstrating processing and results. The bounded in-memory test page covers processing, playback, waveform, quiet intervals, extraction, trimming, chunking, merging, and spectrum analysis; browser validation is recorded separately.
- [x] Verify that local processing does not upload audio by default. The tested Chrome paths use local assets and a local output sink; a caller-supplied network destination can still transmit data.

- [x] Freeze the experimental 0.1.0 API and error contract version 1 in `docs/api.md`, with typed declarations and Node/browser contract checks. Production compatibility and additional platforms are separate acceptance requirements.

- [x] Add an explicitly authorized GitHub Actions Pages workflow that checks the SDK, builds release WASM, verifies a whitelisted static artifact, and deploys from main. Local packaging and project-subpath browser checks passed; live deployment acceptance is recorded in `docs/validation.md`.

## 5. Quality and Acceptance

- [x] Test empty, silent, clipped, malformed, and unsupported inputs on the current PCM16 paths. These checks do not establish support for other codecs or every malformed file.
- [x] Test short and long recordings, multiple channels, and different sample rates. Native reference checks and Chrome synthetic/real-recording results are documented; the real recording is an interview, not a meeting corpus.
- [x] Check resampling output length, timestamps, first and last frames, and chunk boundaries. The impulse test verifies source/output timestamp alignment, ceil frame counts, duration rounding, and silent endpoints; whole-buffer comparisons cover additional rates and boundaries.
- [x] Compare chunked and continuous resampling using reference signals across tested rates, channel counts, and block boundaries.
- [x] Measure browser time and sampled process-tree memory for the user-supplied near-hour recording: roughly 59 minutes, 100.54 seconds processing, 419,966,976 private bytes and 746,004,480 working-set bytes at sampled high-water marks. `docs/validation.md` states the sampling semantics and single-recording limits; this is not a multi-speaker meeting quality claim.
- [x] Record tested browsers and environments and identify unverified devices in `docs/support.md` and `docs/validation.md`.
- [x] Audit the current PCM16 candidate's exact direct/transitive dependency licenses, notices, and codec inclusion. The locked inventory matches; packaging retains licenses/notices. Scope, selected license options, lockfile hashes, and future-codec restrictions are in `DEPENDENCIES.md`. This is not blanket patent clearance or approval for changed dependencies.

## 6. Later Capabilities

- [x] Define and implement merging of compatible PCM16 WAV clips. Chrome checks byte-preserving output and rejects mismatched rates before writing; callers must handle overlap and partial-output rollback.
- [x] Evaluate and implement FFT spectrum analysis for mono PCM windows. Rust and Chrome tests cover tone frequency, amplitude, and invalid input; broader audio-quality certification remains open.
- [x] Evaluate MP3, AAC, and Opus candidates and commercial-use requirements. `docs/codec-research.md` records the decision to keep 0.1.0 PCM16-only, retain Opus as a future candidate, defer MP3, and exclude AAC from the current no-fee baseline. No compressed-codec implementation is claimed.
- [x] Design Android, iOS, and desktop native bindings. The proposed platform boundaries and experimental C ABI are in `docs/native-bindings.md`; platform wrappers remain unimplemented.
- [ ] Implement and package Android, iOS, and desktop native bindings against the validated ABI. The Linux C ABI passed a native C caller test and was packaged with checksummed manifest, header, MIT license, and dependency notices. Android/iOS and other desktop platform packaging remain unimplemented; Android tooling and a Mac/Xcode environment are unavailable.
- [ ] Verify functionality, performance, and memory on actual target devices. The user confirmed on 2026-11-11 that physical devices/Mac are not currently available; browser layout emulation and WSL checks do not satisfy this requirement.

## Suggested First Milestone

The experimental Web/WASM baseline, component validation, typed contract, and focused regressions are complete. Remaining gates are Android/iOS and other desktop platform packaging, plus physical-device acceptance. Sampled process-tree memory and the user-supplied near-hour input are now measured; other devices, recordings, and browser versions remain unverified. Unchecked tasks must remain open until their required evidence exists.
