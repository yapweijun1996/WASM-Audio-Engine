# Repository Guidelines

## Project Status and Scope

WASM-Audio-Engine is an early-stage plan for a lightweight, offline-first audio processing SDK. The intended core is Rust, with WebAssembly and TypeScript integration for browsers and future native integrations. No audio processing API, supported format, platform integration, or release is established yet. Keep proposals clearly labeled until implemented and verified.

## Project Structure

The repository currently contains only `.gitattributes` and Git metadata. Source, tests, examples, and build configuration have not yet been added. When introducing them, document their locations here and keep the DSP core separate from platform bindings and codec adapters. Avoid committing generated build artifacts or large audio fixtures without a clear need.

## Build, Test, and Development

No build or test commands are configured yet. Add reproducible commands and required toolchain versions when the first implementation lands; do not assume `cargo`, npm, or WASM packaging commands already work in this repository. Changes to build configuration should explain how to run the relevant checks locally.

## Coding Style and Design

Write code, identifiers, comments, and technical documentation in English. Follow the formatter and linter configuration introduced with each language; use `rustfmt` conventions for Rust and consistent TypeScript formatting once configured. Prefer small, clear modules and explicit ownership of audio buffers, sample rates, channel layouts, and errors. Preserve public APIs once released; discuss breaking changes before introducing them.

## Testing Guidelines

Add focused tests alongside each feature, covering normal inputs, boundaries, invalid data, and relevant audio properties. Use deterministic synthetic samples where practical. Verify platform bindings and codec support on their actual targets; a native unit test alone does not prove browser, Android, or iOS behavior. Document the test commands and any fixture licenses as the suite develops.

## Commits and Pull Requests

The only existing Git subject is `Initial commit`; no established commit convention can be inferred. Use concise, descriptive English subjects. Pull requests should explain the problem and change, identify affected platforms or formats, link relevant issues when available, and list checks performed and limitations.

## Research and Dependencies

Evaluate existing projects before adding dependencies: [Symphonia](https://github.com/pdeljanov/Symphonia) for decoding and [Rubato](https://github.com/HEnquist/rubato) for resampling are useful starting points. Verify target compatibility, maintenance, codec coverage, and license terms for the intended use. WASM does not supply codecs automatically; avoid claiming offline speech recognition or FFmpeg-equivalent coverage.