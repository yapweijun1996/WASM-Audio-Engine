# WASM Audio Engine

> **Status: planning.** This repository does not yet contain an audio engine, SDK, build configuration, or published package. APIs and capabilities below are design goals, not implemented features.

WASM Audio Engine is a proposed lightweight, offline-first audio processing SDK. Its planned Rust core would provide reusable audio processing, with a TypeScript API and WebAssembly bindings for the web. Native Android, iOS, and desktop integrations are future goals and require their own bindings and verification.

The project aims to combine suitable existing libraries behind a consistent interface rather than reimplement every DSP algorithm. It is not intended to replace all of FFmpeg.

## Planned capabilities

- Detect low-energy intervals and generate waveform data for audio navigation.
- Trim and extract segments, merge compatible audio, and normalize peak volume.
- Process long recordings in chunks and resample PCM audio.
- Convert channels and export supported audio formats, starting with WAV as an MVP candidate.
- Provide frequency analysis in a later or separately scoped release.

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

## Development and contributions

There are currently no build, test, installation, or release commands. See [DESIGN.md](DESIGN.md) for proposed scope, architecture, component research, and open decisions, and [AGENTS.md](AGENTS.md) for contributor guidance.

## Licensing

The project intends to permit free commercial use, but its own license has not yet been selected. Before release, review the exact dependency versions, transitive dependencies, required license notices, and applicable codec patent considerations. Codec support is not supplied automatically by WebAssembly.
