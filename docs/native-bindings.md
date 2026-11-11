# Native Binding Design

**Status: partial.** An experimental C ABI for PCM peak normalization and streaming resampling is implemented in `include/wasm_audio_engine.h`. A Linux C caller smoke test under WSL checked normalization, invalid arguments, a 48 kHz to 16 kHz stream, output length, and handle disposal. Platform wrappers and native-device validation are not implemented. The Rust core remains the DSP owner. WebAssembly and JavaScript are the browser integration, not a prerequisite for native applications.

## Boundary

Build a Rust native library exposing a narrow, versioned C ABI. Android and iOS wrappers own platform packaging, I/O, lifecycle, and scheduling; the core owns PCM processing and frame semantics. Desktop applications can use the Rust API directly or the same C ABI. Do not expose Rust layout, references, panics, or allocator ownership directly across the boundary.

The experimental resampler uses an opaque handle with create, push, read, finish, and destroy operations; destroying the handle abandons processing. A future stable ABI needs explicit versioning and cancellation semantics. Configuration structs include an ABI version and size so newer fields can be added without silently changing older callers. Validate sample rates, channel counts, frame lengths, options, and state transitions before processing. Return documented status codes and copyable diagnostic text; callers must supply valid pointers and buffer lengths, since an ABI cannot prove that arbitrary foreign pointers are safe. Where unwinding is supported, prevent Rust panics from crossing the C boundary; aborting builds and foreign-memory violations are not recoverable guarantees.

For streaming PCM, the caller supplies interleaved buffers and capacities. Each operation reports input frames consumed and output frames produced; insufficient output capacity is distinct from an error or end of stream. Do not retain caller buffers after a call. Expose an explicit final flush for resampler delay and a matching cancellation path. One owner uses a handle at a time unless a later contract explicitly adds synchronization. Keep callbacks or output queues bounded and never invoke platform UI from the core.

Normalization requires a source-peak analysis pass or a caller-supplied, validated peak before the processing pass. Silence and waveform results retain original source-frame positions; resampled output has its own frame rate. PCM16 WAV I/O requires clear ownership of seekable input and output, partial-output rollback, and size limits. No recognition, network upload, or speaker identity is performed by the core.

## Platform Wrappers

- **Android:** package a JNI bridge with a Kotlin API and native libraries for the explicitly supported ABIs. Keep processing off the UI thread, close handles on cancellation or lifecycle teardown, and avoid copying a complete recording into a Java array. Define file/content-URI permissions and temporary-output cleanup in the app wrapper.
- **iOS:** package a C-compatible library in an XCFramework with a Swift-facing wrapper for selected device and simulator architectures. Run long work away from the main actor; define file access, cancellation, lifecycle teardown, and background-execution limitations explicitly.
- **Desktop:** provide platform-specific native libraries or a direct Rust crate. Bindings for any desktop framework are separate packages with the same ownership and error contract.

The exact target list, minimum OS versions, artifact names, build toolchains, and Java/Swift async APIs remain decisions to validate against real consumer requirements. Platform builds must not claim support solely because the Rust core compiles for another target.

## Acceptance Before Support Claims

1. Build the exact native artifacts for each declared architecture with pinned toolchains and include the applicable third-party notices.
2. Exercise handle creation, invalid inputs, output-capacity boundaries, cancellation, repeated disposal, and memory ownership from the actual Kotlin, Swift, or desktop caller.
3. Compare sample counts, timestamps, resampling, clipping, and chunk boundaries with the same reference fixtures used by the Rust core.
4. Measure long-recording memory, elapsed time, app lifecycle interruptions, and recovery on named physical devices. Record unsupported versions and failure behavior; simulator success is not physical-device evidence.
5. Verify local-only behavior and consumer packaging. Assess codec licenses and patents separately before adding non-PCM formats.


## Verified Linux C Package

On the tested WSL Linux host, build with `cargo build --release --locked`, compile `tests/native/c_smoke.c` against `include/` and `target/release`, and run with `LD_LIBRARY_PATH=target/release`. The smoke checks normalization, invalid pointers/arguments permitted by the contract, streaming resampling, frame counts, and disposal; arbitrary foreign-pointer validity remains the caller's responsibility.

Package an existing build with `python scripts/package_native.py --library target/release/libwasm_audio_engine.so --target x86_64-unknown-linux-gnu --output target/wasm-audio-engine-linux-0.1.0.zip`. The script creates a new archive exclusively and verifies SHA-256 for the library, header, project license, and dependency notices. It does not build, sign, install, or infer platform support. The current Linux archive passed this read-back check. Android/iOS packages and other desktop targets remain unbuilt and unverified. The user confirmed no physical devices/Mac environment is currently available.
