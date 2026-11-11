# Dependency and License Review

The project license is MIT. Dependencies retain their own licenses; inclusion in a lockfile does not establish that a release meets their obligations or that every codec is patent-free.

## Inventory

- `Cargo.lock` records the resolved Rust dependency versions. Run `cargo metadata --locked --format-version 1` to inspect package versions, sources, and declared SPDX license expressions.
- `package-lock.json` records the resolved npm development dependency versions. Run `npm ls --all` and inspect each package's license metadata and license text.
- Current direct Rust dependencies are Hound, Rubato, RustFFT, and wasm-bindgen. RustFFT is also used transitively by Rubato. npm development dependencies are TypeScript and Prettier.
- The Rust metadata observed during initial development also includes `unicode-ident` with `(MIT OR Apache-2.0) AND Unicode-3.0`; review both applicable portions and notices. A metadata expression is a lead, not a substitute for the actual license text.
- `THIRD_PARTY_NOTICES.txt` inventories license texts from all locked Rust packages, including build-time packages. In a repository checkout, run `python scripts/generate_notices.py` to check it or add `--write` to regenerate it after a dependency change. The script requires Python 3.10+, Cargo, and the locked packages to be available. Review the generated contents; the inventory does not establish which terms apply to an artifact.

## Before Adding or Updating a Dependency

1. Record its purpose, exact version, source, target support, and enabled features. Prefer a maintained dependency that solves a demonstrated need.
2. Inspect the complete resolved direct and transitive dependency trees, actual license texts, and required copyright or notice files. Recheck after feature or version changes.
3. Check the intended distribution and commercial-use terms, including any applicable codec patent considerations. Escalate unresolved legal questions before release; do not equate a free download with unconditional use.
4. Run the relevant build, tests, and target-specific checks. Record known limitations and update the lockfile and this document if the inventory or procedure changes.

## Before Distribution

Prepare and verify the notices required by the exact shipped artifacts and licenses. Review build-time and runtime dependencies separately, and review codec implementations and formats separately from their package licenses. The generated inventory is a starting point, not a legal approval. The project has not yet completed this release review.


## Current PCM16 candidate audit (2026-11-11)

The locked Rust inventory contains MIT/Apache-2.0 alternatives, Apache-2.0, MIT, 0BSD/Apache-2.0 alternatives, Zlib/MIT/Apache-2.0 alternatives, and the additional Unicode-3.0 requirement on unicode-ident. The distribution policy chooses MIT where offered, 0BSD where offered without MIT, and Apache-2.0 otherwise; Unicode-3.0 notices are retained additionally. Full available license texts are retained in THIRD_PARTY_NOTICES.txt rather than stripping alternative grants. No FFmpeg, GPL/LGPL codec, MP3, AAC, or Opus implementation is present in this locked artifact.

The npm development tools are TypeScript 5.9.3 (Apache-2.0) and Prettier 3.9.10 (MIT). They are not bundled into the browser SDK. The npm package ships generated JavaScript/WASM, declarations, project MIT LICENSE, and the Rust notices inventory; packaging must verify those files are included. Native packages also include LICENSE, notices, and the C header. Review and regenerate the inventory after every dependency/feature change; future compressed codecs require the separate review in docs/codec-research.md.

Reviewed lockfile SHA-256 values: Cargo.lock `7cc352afd4c56565b2c2ac684556f178684e56a1aa97f22dddf66fad37e559ca`; package-lock.json `362a88ac272b3508f2a0291b5083b94881fabc891f987c48ce200d54288103e0`. This is a technical license/notice audit of the current candidate, not a guarantee about all patents, future dependencies, or unbuilt platforms. Public release remains pending.
