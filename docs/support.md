# Experimental Support and Component Assessment

## Tested Environment

- Rust 1.99.1 on Ubuntu under Windows Subsystem for Linux: native tests, Clippy, and `wasm32-unknown-unknown` builds.
- Node.js 25.2.1 and TypeScript 5.9.3 on Windows: wrapper typecheck and build.
- Chrome 157.0.0.0 on Windows 11 Home 10.0.26200, Intel Core i5-1235U: local PCM16 WAV smoke tests described in [validation.md](validation.md).

The experimental 0.1.0 support baseline is Rust 1.99.1, wasm-bindgen CLI 0.2.130, Node.js 25.2.1 for development, and Chrome 157 on Windows for browser execution. Use the committed lockfiles and match the wasm-bindgen CLI to Cargo.lock. `.node-version` records the development Node version. Other versions are outside the declared experimental baseline until verified; this is not a production compatibility guarantee. Android, iOS, desktop targets other than the tested Linux C ABI, other browsers, and other Rust/Node versions have not been verified. Native Windows Rust tests also require a C/C++ linker that was not present in the initial environment.

## FFT Candidates

Rubato already depends on RustFFT for its default FFT resampler. RustFFT also provides the mono spectrum primitive currently being evaluated in this repository. The tpt-dsp project documents FFT, windowing, and spectrum-analysis primitives, as well as a browser WASM demonstration; its scope overlaps these needs.

For this initial implementation, reuse RustFFT rather than adding a second FFT implementation solely for spectrum analysis. This is a dependency-minimization decision, not a measured comparison of quality, performance, or maintenance. Reconsider tpt-dsp if a concrete DSP requirement is not met, after reviewing its exact version, features, transitive licenses, target compatibility, and test evidence. Sources: [RustFFT](https://github.com/ejmahler/RustFFT), [Rubato](https://github.com/HEnquist/rubato), and [tpt-dsp](https://github.com/tpt-solutions/tpt-dsp).
