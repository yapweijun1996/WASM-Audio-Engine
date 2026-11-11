# Tasks

**Status:** Planning. All tasks below are proposed and incomplete. Order and scope may change after technical validation.

## 1. Project Foundation

- [ ] Select and publish a project license that permits free commercial use.
- [ ] Initialize the Rust project and TypeScript SDK structure.
- [ ] Configure local formatting, static checks, builds, and tests.
- [ ] Define supported Rust, browser, and toolchain versions.
- [ ] Establish dependency inventory and license review procedures.

## 2. Technical Validation and Design

- [ ] Validate Hound, Rubato, and wasm-bindgen on the intended WASM target.
- [ ] Measure candidate bundle size, processing time, and peak memory.
- [ ] Evaluate RustFFT and tpt-dsp for feature overlap and actual needs.
- [ ] Define PCM representation, channel layout, sample rate, and time units.
- [ ] Define the public API, processing order, errors, cancellation, and resource limits.
- [ ] Design bounded, end-to-end chunk processing without requiring a full decoded recording in memory.

## 3. Audio Processing MVP

- [ ] Read and validate supported WAV variants.
- [ ] Export WAV and verify the output format.
- [ ] Implement peak normalization, including silent and clipped input behavior.
- [ ] Detect low-energy intervals and return thresholds and timestamps; do not treat this as VAD.
- [ ] Resample PCM with appropriate filtering and documented delay handling.
- [ ] Generate time-indexed waveform amplitude summaries.
- [ ] Trim and extract audio clips by time range.
- [ ] Convert channels using documented rules.
- [ ] Output audio chunks with original offsets and configurable overlap.
- [ ] Provide a mapping to the original timeline when silence is removed.

## 4. TypeScript and Browser SDK

- [ ] Wrap WASM initialization and expose a TypeScript API.
- [ ] Define audio, analysis, metadata, and error result types.
- [ ] Specify local processing, progress reporting, and cancellation behavior.
- [ ] Create a browser example demonstrating processing and results.
- [ ] Verify that local processing does not upload audio by default.

## 5. Quality and Acceptance

- [ ] Test empty, silent, clipped, malformed, and unsupported inputs.
- [ ] Test short and long recordings, multiple channels, and different sample rates.
- [ ] Check resampling output length, timestamps, first and last frames, and chunk boundaries.
- [ ] Compare chunked and continuous processing using suitable reference signals.
- [ ] Measure browser time and peak memory using a representative one-hour recording.
- [ ] Record tested browsers and devices and identify unverified environments.
- [ ] Review exact direct and transitive dependencies, license notices, and applicable codec patents before release.

## 6. MeetingMemo Integration

These are application tasks that depend on suitable engine capabilities; they are not engine implementation claims.

- [ ] Use waveform data and existing transcript timestamps for navigation.
- [ ] Use extracted clips for speaker A/B listening comparisons.
- [ ] Select channel, sample rate, and export format for the speech-to-text provider.
- [ ] Manage chunk uploads, retries, and transcript result stitching.
- [ ] Handle overlapping chunks and words crossing boundaries.

## 7. Later Capabilities

- [ ] Define and implement merging of compatible audio clips.
- [ ] Evaluate and implement FFT spectrum analysis.
- [ ] Evaluate MP3, AAC, and Opus codecs and their commercial-use requirements.
- [ ] Design Android, iOS, and desktop native bindings.
- [ ] Verify functionality, performance, and memory on actual target devices.

## Suggested First Milestone

Select the project license and validate Hound and Rubato in a browser before stabilizing the MVP API. Record the tested versions, inputs, results, and remaining limitations.
