# Archived MeetingMemo Investigation Notes

**Status: archived investigation; application integration is outside this project's scope.** The MeetingMemo checkout was inspected on 2026-11-11 at `main` commit `1887b2d`. Recheck its current instructions, status, code, and dependencies before editing it. No MeetingMemo source files were changed during this investigation.

## Existing Responsibilities

- `src/audio/chunked-audio.ts` owns chunked playback and window transitions. The application's media element clock is authoritative for synchronized transcript and speaker review.
- `src/features/SpeakerReview.tsx` already offers timed sample playback and a manual comparison view. `src/features/usePlaybackClock.ts` and the transcript views use the existing audio clock; an SDK integration must preserve seeking, pause, overlapping speakers, and untimed segments.
- `src/audio/acoustic-audio.ts` currently decodes bounded windows and resamples them for speaker embeddings. Changing this resampling can change embeddings and matching behavior; the available real-recording voice-matching evidence is not labelled accuracy ground truth. Do not replace it without an appropriate baseline and regression evaluation.
- Existing long-recording import, storage, chunk transcription, and recovery distinguish completed, failed, and unknown paid-request outcomes. Integrating WAV chunks must preserve explicit confirmation and must not automatically retry unknown paid outcomes.
- Long-recording import currently accepts MP3 and MP4/M4A only. Its manifest and meeting schemas encode only `mp3` or `mp4`, and `ChunkedAudio` branches on those formats. Adding WAV is therefore a cross-boundary change involving import, storage schemas, playback, speech jobs, and recovery—not only a new splitter. A local static impact scan identified direct callers in `MeetingWorkspace.tsx` and `NewMeeting.tsx` but was partial because of depth and unresolved external observations; it is not a complete impact map.

Four existing local test files covering speaker review, playback boundaries, and chunked audio passed 37 tests on Node 22.23.3. This is baseline evidence for the app, not evidence of SDK integration or real speaker accuracy.

## Proposed Integration Boundaries

1. Establish a reproducible SDK dependency and asset-delivery method. The SDK is currently private, experimental, and uncommitted; a local archive was smoke-tested, but a package version or vendored artifact must have traceable source, third-party notices, and an update procedure. Do not reference an absolute temporary path.
2. For WAV-specific preparation, introduce an explicit local operation using the SDK's `processWav`, `chunkWav`, and source-frame metadata. Preserve original recordings, storage quotas, provider confirmation, and cancellation/partial-output recovery. Unsupported MP3/M4A inputs must continue through the app's existing paths.
3. Derive waveform navigation from source-frame positions and existing transcript timestamps. Keep the media element clock authoritative; do not invent timestamps or identify speakers from low-energy detection.
4. Preserve the current speaker A/B review and acoustic continuity. Any use of SDK excerpts must retain source offsets and must not substitute manual comparison for automatic meeting-scoped speaker matching.
5. Test changes with existing targeted unit and browser flows, malformed input and cancellation cases, and a real long recording where authorized. Provider calls that may incur charges, deployment, physical-device acceptance, and model-accuracy claims require their own applicable authorization and evidence.

## Open Dependencies

The SDK has not been released and its complete commercial-use review remains open. The supplied long recording was tested as a local PCM16 WAV conversion, but a labelled meeting corpus, independent speaker-matching accuracy evaluation, and physical Android/iOS environments are not available from current evidence. These gaps do not prevent unrelated local SDK work, but they limit integration and release claims.
