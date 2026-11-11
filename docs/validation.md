# Validation Notes

These records describe the specific builds tested at the time. Later source changes can invalidate earlier evidence; rerun the relevant checks against the exact release candidate.

## Initial synthetic long-recording smoke test

- Date: 2026-11-11; environment: Ubuntu under Windows Subsystem for Linux, Rust 1.99.1, optimized test build.
- Input: generated 60-minute, 16 kHz, mono, 16-bit PCM WAV silence; 8,192-frame input blocks. The source generates zero bytes on demand and does not read a real file.
- Path: WAV reader, low-energy detection, one-second waveform summaries, non-overlapping one-second PCM chunks, and 16 kHz to 8 kHz resampling. The test verifies frame counts and intervals.
- Result: passed. One separately timed run reported 1.66 s wall time and 3,840 KiB maximum RSS from `/usr/bin/time -v` for the test process. An earlier version of the test without resampling reported 1.15 s and 2,688 KiB; the two paths are different and are not a controlled comparison.
- Reproduce: `cargo test --release --locked --test long_recording -- --ignored`. For memory, run the resulting test executable with `/usr/bin/time -v` and the same test selection.
- Limits: one measurement per path, synthetic silence, no real file I/O, normalization, encoding, TypeScript, browser, or mobile device. This is not a benchmark for real meeting recordings or evidence of end-to-end bounded browser memory.

## Browser synthetic long-recording smoke test

- Date: 2026-11-11; Chrome 157.0.0.0 on Windows, optimized WASM build, localhost test page `tests/browser-long.html`.
- Input: generated 60-minute, 16 kHz, mono, 16-bit PCM silent WAV file, 115,200,044 bytes. The temporary file was deleted after the test.
- Path: two-pass normalization analysis, source-timeline silence and waveform analysis, 16 kHz to 8 kHz resampling, and WAV encoding to a caller-supplied sink that discards bytes. No audio was uploaded; observed page resource requests were local test assets.
- Result: one completed run passed in 23.14 s, producing 28,800,000 frames and 57,600,044 output bytes. Sampled `performance.memory.usedJSHeapSize` peaked at 70,068,346 bytes; WASM linear memory at completion was 1,245,184 bytes. The JS heap metric is browser-specific and sampled; neither metric measures total browser process memory.
- A prior run using a debug WASM build was stopped before completion and produced no measurement. It is not included in the result above.
- Reproduce: generate a fixture with `python tests/create-long-fixture.py <new-temp-path.wav>`, build the TypeScript wrapper and release WASM, generate web bindings with the matching `wasm-bindgen` CLI, serve the repository root on localhost, open `tests/browser-long.html`, and select the fixture. Delete the generated fixture when finished.
- The generated release web assets measured 365,460 bytes for WASM, 14,033 bytes for wasm-bindgen JavaScript, and 10,266 bytes for the unminified TypeScript wrapper output. These sizes describe this build and its exported paths, not every candidate dependency or a compressed distribution.
- Limits: one synthetic silent file, one browser and host, a discard sink, and no mobile device, real speech, transcript, or speech provider. It does not prove total browser peak memory, quality for all input, or device-wide support.

## Browser user-provided long-recording check

- Date: 2026-11-11; Chrome 157.0.0.0 on Windows; optimized WASM build. The user supplied a roughly 59-minute stereo interview MP3. FFmpeg was used only to create a temporary 48 kHz PCM16 WAV input; it is not part of the engine path. The temporary copy was deleted after the run.
- Path: two-pass input-peak normalization, source-frame silence and waveform analysis, stereo-to-mono conversion, 48 kHz to 16 kHz resampling, and PCM16 WAV output to a discard sink. Observed page resource requests were local test assets.
- Result: one run passed in 140.13 s. It processed 170,220,544 input frames and produced 56,740,182 output frames and 113,480,408 output bytes; the output reported zero saturated samples. Sampled JS heap reached 72,839,740 bytes and WASM linear memory was 1,310,720 bytes at completion.
- A separate 30-second excerpt from the same source passed a signal-presence check: the encoded output had a peak of approximately 0.787 and RMS of 0.084, with zero reported saturated samples. Its temporary WAV was also deleted.
- A further 30-second PCM16 excerpt, with normalization disabled, was compared against Chrome's `OfflineAudioContext.decodeAudioData` resampled to 16 kHz and averaged to mono. Excluding 512 output frames at each edge, correlation was 0.99954, output/reference RMS ratio 1.00078, and RMSE 0.00285. The preselected smoke thresholds were correlation > 0.9 and RMS ratio between 0.8 and 1.25; both passed. This is one implementation comparison, not listening evaluation or codec certification. The temporary excerpt was deleted.
- A separate Chrome chunking check generated 61 WAV Blob slices at 60 seconds with 1-second overlap; reported offsets ended at the original 170,220,544 frames. It checked each chunk's metadata and sampled 16 overlapping bytes at each boundary. The 0.165 s run did not read or upload the complete chunk contents and is not a throughput measurement. A short synthetic test compares a full overlap.
- Limits: the source was an interview, not a representative multi-speaker meeting corpus. Output was discarded, not listened to or compared with a reference decoder. Silence labels, transcript alignment, mobile behavior, and total browser process peak memory were not assessed. These checks do not establish general accuracy or capacity.

## Local package smoke test

- An experimental npm archive was built and installed into an isolated temporary consumer. Node.js 25.2.1 loaded both exported module paths, initialized the packaged WASM, normalized PCM, and processed a short generated mono PCM16 WAV into a local stream; the check passed.
- This checks package consumption, not release readiness. Third-party notices, cross-browser compatibility, mobile bindings, and production publication remain unverified.

## Web test handoff acceptance

Date: 2026-11-11. Scope: the experimental PCM16 WAV SDK and local test page in Chrome on the tested Windows host, not a production release or native platform acceptance.

- Rebuilt the release `wasm32-unknown-unknown` core and regenerated both `dist/wasm` and `target/browser` with wasm-bindgen 0.2.130. The final WASM is 371,597 bytes; SHA-256: `d478799ee44ff7990067a0fa94b7266d8740188a18fcf4f2915f7461f682f88c`. Earlier long-recording measurements above describe their earlier builds.
- Rust: 30 tests passed; the explicitly ignored synthetic-hour test was not rerun. Strict Clippy passed. TypeScript typecheck, build, formatting, and Git whitespace checks passed.
- Final browser SDK smoke page returned `PASS` after regenerating bindings. It covers normalization, spectrum, WAV processing, channel conversion, extraction, interval trimming and mapping, merging, chunk overlap, malformed inputs, and cancellation.
- The test UI's generated three-second sample loaded and processed successfully. Extraction returned frames 0-16,000; quiet-interval trimming retained 31,999 frames; chunking created one chunk; merging the sample with itself returned 96,000 frames. Frequency analysis completed using the browser-decoded first window (the browser may resample before the engine's FFT).
- Downloaded WAV headers and lengths matched three-second outputs: 16 kHz mono / 48,000 frames; 44.1 kHz stereo / 132,300 frames with normalization disabled; 48 kHz mono / 144,000 frames. Audio elements reached readyState 4; this is decoder readiness, not a subjective listening evaluation.
- UI rejected an out-of-range threshold and a malformed RIFF file. Immediate cancellation reported AbortError and hid the output link; processing afterward succeeded. No stale download remained after failure. Chrome console had no warnings/errors. A 390-pixel mobile viewport had no horizontal overflow; this is layout emulation, not phone validation.
- Repacked and reinstalled the final generated SDK into the isolated consumer; base processing, chunking, and trimming consumer checks passed.

To test manually, serve the repository root with `python -m http.server 18786 --bind 127.0.0.1`, open `/examples/`, choose **Try a generated sample**, and select **Process audio**. Additional operations become available after processing. Each input is limited to 8 MiB, buffered output to 32 MiB, and chunk downloads to 50. Supported input is classic mono/stereo PCM16 WAV. MP3/M4A codecs, cross-browser support, total process memory, and native platform packaging remain outside this handoff's verified coverage.


## Task completion pass: typed contract and component validation

- The experimental support baseline is defined in `support.md`; `api.md` freezes API 0.1.0 and SDK error contract version 1. Caller stream/callback failures and cancellation reasons are preserved, rather than translated into an unrelated DSP failure.
- Six Node error-contract tests passed. Final browser regressions passed with actual WASM DSP error-code checks and caller sink/callback error-identity checks. The test UI's sample, processing, excerpt, trim, chunk, merge, and spectrum operations were rechecked after the TypeScript changes.
- Hound PCM16 writer/reader execution passed in Chrome using the dedicated `validation` feature build. It round-trips six interleaved stereo samples at 16 kHz and checks EOF. This optional validation export is absent from the normal build. Reproduce with `cargo build --release --locked --target wasm32-unknown-unknown --features validation`, generate matching web bindings into `target/validation`, and open `tests/hound-wasm.html` on the local server. Rebuild without the feature for distribution.
- Rust tests: 31 passed, with the synthetic-hour test left explicitly ignored in the default suite. The added impulse test confirms frame 12,000 at 48 kHz maps to frame 4,000 at 16 kHz, 48,001 input frames produce 16,001 output frames, endpoints remain silent, and duration rounding is less than one output frame. All-feature native Clippy passed with warnings denied. Typecheck, formatting, and whitespace checks passed.
- The license inventory check matched Cargo.lock. The final npm archive contained LICENSE, THIRD_PARTY_NOTICES.txt, error module/declarations, and WASM; its isolated consumer processing/chunk/trim checks passed. Default WASM hash and both lockfile hashes remain as recorded above.
- The Linux shared library passed the C caller smoke test, then `scripts/package_native.py` created and read-back verified `target/wasm-audio-engine-linux-0.1.0.zip` with header, licenses, and SHA-256 manifest. This is Linux packaging evidence only.
- Remaining acceptance gaps are recorded in TASK.md. The user confirmed no physical devices/Mac environment is currently available. No SDK package publication or additional target-device claim has been made. The project MIT license was subsequently published independently as described below.


## Isolated Chrome process-tree memory measurement

- Date: 2026-11-11. Current default release WASM, versioned TypeScript wrapper, Chrome headless in a separate local profile on Windows. The input was the same user-supplied roughly 59-minute recording, temporarily converted to 48 kHz stereo PCM16 WAV using the local FFmpeg executable. That temporary WAV was deleted after measurement; original supplied files were preserved. FFmpeg remains outside the SDK dependency/runtime path.
- The normal file-selection handler in `tests/browser-real.html` ran the two-pass normalization, analysis, mono conversion, resampling, and PCM16 output-discard workflow. It returned PASS in 100,543.7 ms, with 170,220,544 input frames, 56,740,182 output frames, 113,480,408 output bytes, and zero saturated output samples. Resampling can produce intersample peaks above the normalized source peak; observed output peak was 0.8201 for a source-peak target of 0.8.
- 368 process-tree samples were collected at a nominal 0.25-second interval (process inspection and scheduling add overhead). Maximum sampled private committed bytes: 419,966,976. Maximum sampled sum of working sets: 746,004,480. JavaScript heap high-water mark: 61,969,680 bytes; final WASM linear memory: 1,310,720 bytes.
- Metrics include the isolated Chrome parent and descendants only. The two process metrics have different meanings: private bytes are committed private memory, not resident RAM; summed working sets can double-count shared pages. These are sampled high-water marks, not a continuously observed exact peak, and exclude the Python/Node instrumentation processes and OS file cache. They cannot be compared directly with the earlier JS-only measurements or interpreted as the SDK's incremental memory cost.
- Observed page resource origins contained only `http://127.0.0.1:18786`. The browser was closed at completion. Its isolated generated profile and raw JSON under `target/` remain for local audit; no recursive cleanup was attempted.
- Runtime asset sizes: errors.js 745 bytes, index.js 1,756, wav.js 18,486, generated bindings 14,666, and WASM 371,597: 407,250 bytes total. Sum of independently gzipped asset sizes is 135,458 bytes; this is an offline size calculation, not observed HTTP compression.
- Reproduction on this Windows host requires existing Chrome, Node.js, Python, and psutil 7.1.3. Serve the repository on 127.0.0.1:18786, prepare a PCM16 WAV in a temporary location, and run `python scripts/measure_browser_memory.py --input <file.wav> --output target/browser-memory-acceptance.json`. The runner uses a fresh headless Chrome profile and selects the input through CDP; it does not upload the audio. Remove only the explicitly generated temporary WAV afterward.
- Coverage is one supplied near-hour interview and one machine/browser. Representative meeting speech labels, subjective listening quality, physical devices, other browsers, and a universal memory bound remain unverified. The headless result and the earlier interactive-browser result are separate runs, not a controlled speed comparison.


## Project license publication

LICENSE alone was committed and pushed to GitHub main as `ed0f69a88418b871008ca4274ff89df5c4488975`. `git ls-remote` confirmed the exact remote SHA, and the raw LICENSE at that immutable SHA was read back and matched the local file after line-ending normalization. No hosted CI/deployment workflow was present in the checked remote tree. SDK implementation and updated documentation remain local working-tree changes; this license publication is not an SDK package release.


## GitHub Pages deployment candidate

The user explicitly authorized GitHub Actions deployment and selected GitHub Actions as the Pages source. `.github/workflows/pages.yml` builds on pushes to main or manual dispatch. Build steps have read-only repository permissions; only the separate deploy job has Pages/OIDC write permissions. Official GitHub Actions revisions are pinned. Main deployments share one concurrency group.

Local candidate checks passed: workflow YAML structure, TypeScript formatting and error-contract tests, a whitelisted static artifact, its revision/hashes, rejection of an extra file, and rejection of a mismatched source revision. The artifact root redirects to examples/; a Chrome run served below `/_site/` successfully loaded all relative JS/WASM assets, processed the generated sample, and exposed the download. Only runtime files and licenses are copied, with no source tree, private recordings, tests, native builds, or node_modules. Live workflow and Pages acceptance must be recorded after publication; these local checks alone do not establish a deployed site.
