# Codec Research

**Status: candidate evaluation complete for the PCM16-only 0.1.0 scope, 2026-11-11.** None of these codecs is implemented by this SDK. A software license, a patent license, and a verified implementation on each target are separate questions. No candidate below is approved for release.

| Format | Candidates and evidence | Commercial-use questions |
| --- | --- | --- |
| MP3 | [Symphonia](https://github.com/pdeljanov/Symphonia) documents an MP3 decoder under MPL-2.0. [minimp3](https://github.com/lieff/minimp3) is a C decoder under CC0, with third-party bindings listed by the project. Neither provides an MP3 encoder by itself. | Symphonia has file-level license obligations. CC0 explicitly does not waive patents; verify the exact implementation, transitive code, notices, jurisdictions, and patent position. |
| AAC / M4A | Symphonia documents AAC-LC decoding and MP4 demuxing; it is not a general AAC encoder. Codec features and containers must both be tested. | [Via Licensing Alliance](https://www.via-la.com/licensing-programs/aac/) states that its AAC patent program applies to end-user encoder and decoder products. Applicability, exemptions, costs, and other rights require a release-specific assessment; a permissive source license alone is insufficient. |
| Opus | The [Opus reference implementation](https://opus-codec.org/license/) is BSD-3-Clause and provides an encoder and decoder. Symphonia's current status table lists its own Opus decoder as unfinished. | The Opus project describes royalty-free patent grants with conditions, including litigation-related terms. Review the exact grants, notices, chosen implementation, bindings, and distribution targets before making a commercial-use claim. |

The current browser path handles classic PCM16 WAV only. Browser-native decoding may vary by format and platform, and `decodeAudioData` consumes complete encoded file data; it is not evidence of a bounded-memory streaming decoder. No blanket patent-free or zero-obligation claim is made.


## Decision for 0.1.0

Ship classic PCM16 WAV only. Do not add a compressed-codec dependency or claim codec conversion beyond the implemented WAV operations. This meets the project's no-fee commercial-use design goal without introducing an unverified AAC or compressed-codec implementation.

- Opus is the preferred future compressed-codec candidate: its official license page permits commercial reference-code use without fees, subject to BSD notices and the stated patent-grant conditions. Actual bindings, container handling, build targets, resource use, and error behavior must be independently validated before adoption.
- MP3 is deferred. Fraunhofer's [official MP3 page](https://www.iis.fraunhofer.de/en/ff/amm/consumer-electronics/mp3.html) says the Technicolor/Fraunhofer licensing program for certain patents/software ended on April 23, 2017; this does not license every implementation or establish every jurisdiction's patent position. Symphonia remains a decoder candidate, not an encoder or an adopted dependency.
- AAC is excluded from the current no-fee baseline. Via LA's AAC program documents licensing terms; do not promise unconditional royalty-free commercial distribution. The program page timed out during a direct fetch; its indexed official content was available. No fee schedule or exemption has been assumed.

This evaluation closes the candidate-selection task, not implementation or legal approval for a future codec release. The current package contains none of these compressed codecs.
