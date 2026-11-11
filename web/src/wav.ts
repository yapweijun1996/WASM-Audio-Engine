import { AudioEngineError, guardDsp } from "./errors.js";
import type { WasmAudioBindings } from "./index.js";

export interface WavChunkOptions {
  chunkFrames: number;
  overlapFrames?: number;
  signal?: AbortSignal;
}

export interface WavExtractOptions {
  startFrame: number;
  endFrame: number;
  signal?: AbortSignal;
}

export interface WavExtractSecondsOptions {
  startSeconds: number;
  endSeconds: number;
  signal?: AbortSignal;
}

export interface WavMergeOptions {
  signal?: AbortSignal;
  onProgress?: (completedBytes: number, totalBytes: number) => void;
}

export interface WavMergeResult {
  outputFrames: number;
  sampleRate: number;
  channels: 1 | 2;
}

export interface WavTrimInterval {
  startFrame: number;
  endFrame: number;
}
export interface WavKeptSpan {
  originalStartFrame: number;
  originalEndFrame: number;
  outputStartFrame: number;
  outputEndFrame: number;
}
export interface WavTrimResult extends WavMergeResult {
  mapping: WavKeptSpan[];
}

export interface WavChunk {
  startFrame: number;
  endFrame: number;
  sampleRate: number;
  channels: 1 | 2;
  audio: Blob;
}

export interface WavProcessOptions {
  outputSampleRate: number;
  outputChannels: 1 | 2;
  normalizePeak?: number;
  silenceThreshold: number;
  minSilenceFrames: number;
  waveformWindowFrames: number;
  inputChunkFrames?: number;
  signal?: AbortSignal;
  onProgress?: (
    phase: "analyze" | "process",
    completedFrames: number,
    totalFrames: number,
  ) => void;
  onSilence?: (startFrame: number, endFrame: number) => void;
  onWaveform?: (
    startFrame: number,
    endFrame: number,
    min: number,
    max: number,
  ) => void;
}

export interface WavProcessResult {
  inputFrames: number;
  outputFrames: number;
  inputSampleRate: number;
  inputChannels: 1 | 2;
  outputSampleRate: number;
  silenceThreshold: number;
  minSilenceFrames: number;
  appliedGain: number;
  clippedSamples: number;
}

interface WavInfo {
  channels: 1 | 2;
  sampleRate: number;
  dataOffset: number;
  frames: number;
}

/** Yield source-format WAV slices, limiting each audio payload to 64 MiB. */
export async function* chunkWav(
  input: Blob,
  options: WavChunkOptions,
): AsyncGenerator<WavChunk> {
  const overlap = options.overlapFrames ?? 0;
  if (
    !Number.isSafeInteger(options.chunkFrames) ||
    options.chunkFrames < 1 ||
    !Number.isSafeInteger(overlap) ||
    overlap < 0 ||
    overlap >= options.chunkFrames
  ) {
    throw new AudioEngineError("INVALID_ARGUMENT", "Invalid WAV chunk options");
  }
  const info = await inspectWav(input, options.signal);
  if (options.chunkFrames * info.channels * 2 > 64 * 1024 * 1024) {
    throw new AudioEngineError(
      "LIMIT_EXCEEDED",
      "WAV chunk exceeds the 64 MiB payload limit",
    );
  }
  for (let start = 0; start < info.frames;) {
    abortIfNeeded(options.signal);
    const end = Math.min(info.frames, start + options.chunkFrames);
    yield createWavSlice(input, info, start, end);
    if (end === info.frames) break;
    start = end - overlap;
  }
}

/** Extract a half-open source-frame range as a WAV blob. */
export async function extractWav(
  input: Blob,
  options: WavExtractOptions,
): Promise<WavChunk> {
  const info = await inspectWav(input, options.signal);
  const { startFrame, endFrame } = options;
  if (
    !Number.isSafeInteger(startFrame) ||
    !Number.isSafeInteger(endFrame) ||
    startFrame < 0 ||
    endFrame < startFrame ||
    endFrame > info.frames
  ) {
    throw new AudioEngineError("INVALID_ARGUMENT", "Invalid WAV frame range");
  }
  if ((endFrame - startFrame) * info.channels * 2 > 64 * 1024 * 1024)
    throw new AudioEngineError(
      "LIMIT_EXCEEDED",
      "WAV excerpt exceeds the 64 MiB payload limit",
    );
  abortIfNeeded(options.signal);
  return createWavSlice(input, info, startFrame, endFrame);
}

/** Extract a time range, including frames touched by its endpoints. */
export async function extractWavSeconds(
  input: Blob,
  options: WavExtractSecondsOptions,
): Promise<WavChunk> {
  const info = await inspectWav(input, options.signal);
  const { startSeconds, endSeconds } = options;
  if (
    !Number.isFinite(startSeconds) ||
    !Number.isFinite(endSeconds) ||
    startSeconds < 0 ||
    endSeconds < startSeconds ||
    endSeconds > info.frames / info.sampleRate
  ) {
    throw new AudioEngineError("INVALID_ARGUMENT", "Invalid WAV time range");
  }
  const startFrame =
    startSeconds === info.frames / info.sampleRate
      ? info.frames
      : Math.floor(startSeconds * info.sampleRate);
  const endFrame =
    startSeconds === endSeconds
      ? startFrame
      : Math.min(info.frames, Math.ceil(endSeconds * info.sampleRate));
  if ((endFrame - startFrame) * info.channels * 2 > 64 * 1024 * 1024)
    throw new AudioEngineError(
      "LIMIT_EXCEEDED",
      "WAV excerpt exceeds the 64 MiB payload limit",
    );
  abortIfNeeded(options.signal);
  return createWavSlice(input, info, startFrame, endFrame);
}

function createWavSlice(
  input: Blob,
  info: WavInfo,
  startFrame: number,
  endFrame: number,
): WavChunk {
  const firstByte = info.dataOffset + startFrame * info.channels * 2;
  const lastByte = info.dataOffset + endFrame * info.channels * 2;
  const audio = new Blob(
    [
      wavHeader(info.sampleRate, info.channels, endFrame - startFrame),
      input.slice(firstByte, lastByte),
    ],
    { type: "audio/wav" },
  );
  return {
    startFrame,
    endFrame,
    sampleRate: info.sampleRate,
    channels: info.channels,
    audio,
  };
}

/** Merge compatible source-format WAV data into a caller-provided stream. */
export async function mergeWav(
  inputs: readonly Blob[],
  output: WritableStream<Uint8Array>,
  options: WavMergeOptions = {},
): Promise<WavMergeResult> {
  if (inputs.length === 0)
    throw new AudioEngineError(
      "INVALID_ARGUMENT",
      "At least one WAV input is required",
    );
  const metadata: WavInfo[] = [];
  let totalFrames = 0;
  for (const input of inputs) {
    const info = await inspectWav(input, options.signal);
    if (
      metadata.length &&
      (info.sampleRate !== metadata[0].sampleRate ||
        info.channels !== metadata[0].channels)
    ) {
      throw new AudioEngineError(
        "UNSUPPORTED_FORMAT",
        "WAV inputs must have the same sample rate and channel count",
      );
    }
    totalFrames += info.frames;
    if (!Number.isSafeInteger(totalFrames))
      throw new AudioEngineError(
        "LIMIT_EXCEEDED",
        "Merged frame count is too large",
      );
    metadata.push(info);
  }
  const { sampleRate, channels } = metadata[0];
  const header = wavHeader(sampleRate, channels, totalFrames);
  const totalBytes = totalFrames * channels * 2;
  const writer = output.getWriter();
  let completedBytes = 0;
  try {
    abortIfNeeded(options.signal);
    await writer.write(header);
    for (let index = 0; index < inputs.length; index++) {
      const info = metadata[index];
      const bytes = info.frames * channels * 2;
      for (let offset = 0; offset < bytes; offset += 1024 * 1024) {
        abortIfNeeded(options.signal);
        const block = new Uint8Array(
          await inputs[index]
            .slice(
              info.dataOffset + offset,
              info.dataOffset + Math.min(bytes, offset + 1024 * 1024),
            )
            .arrayBuffer(),
        );
        abortIfNeeded(options.signal);
        if (block.length !== Math.min(1024 * 1024, bytes - offset))
          throw new AudioEngineError("INVALID_AUDIO", "Truncated WAV data");
        await writer.write(block);
        completedBytes += block.length;
        options.onProgress?.(completedBytes, totalBytes);
      }
    }
    abortIfNeeded(options.signal);
    await writer.close();
    return { outputFrames: totalFrames, sampleRate, channels };
  } catch (error) {
    await writer.abort(error).catch(() => undefined);
    throw error;
  } finally {
    writer.releaseLock();
  }
}

/** Remove sorted, non-overlapping source-frame intervals and return retained-span mapping. */
export async function trimWav(
  input: Blob,
  removed: readonly WavTrimInterval[],
  output: WritableStream<Uint8Array>,
  options: WavMergeOptions = {},
): Promise<WavTrimResult> {
  if (removed.length > 100_000)
    throw new AudioEngineError("LIMIT_EXCEEDED", "Too many trim intervals");
  const info = await inspectWav(input, options.signal);
  const mapping: WavKeptSpan[] = [];
  const pieces: Blob[] = [];
  let original = 0;
  let outputFrame = 0;
  const keep = (start: number, end: number) => {
    if (end === start) return;
    const next = outputFrame + end - start;
    mapping.push({
      originalStartFrame: start,
      originalEndFrame: end,
      outputStartFrame: outputFrame,
      outputEndFrame: next,
    });
    pieces.push(createWavSlice(input, info, start, end).audio);
    outputFrame = next;
  };
  for (const interval of removed) {
    if (
      !Number.isSafeInteger(interval.startFrame) ||
      !Number.isSafeInteger(interval.endFrame) ||
      interval.startFrame < original ||
      interval.endFrame <= interval.startFrame ||
      interval.endFrame > info.frames
    ) {
      throw new AudioEngineError(
        "INVALID_ARGUMENT",
        "Invalid or overlapping trim interval",
      );
    }
    keep(original, interval.startFrame);
    original = interval.endFrame;
  }
  keep(original, info.frames);
  if (pieces.length === 0) pieces.push(createWavSlice(input, info, 0, 0).audio);
  const merged = await mergeWav(pieces, output, options);
  return { ...merged, mapping };
}

function abortIfNeeded(signal?: AbortSignal): void {
  if (signal?.aborted)
    throw signal.reason ?? new DOMException("Aborted", "AbortError");
}

function tag(bytes: Uint8Array, offset: number): string {
  return String.fromCharCode(...bytes.subarray(offset, offset + 4));
}

async function inspectWav(input: Blob, signal?: AbortSignal): Promise<WavInfo> {
  abortIfNeeded(signal);
  const header = new Uint8Array(await input.slice(0, 12).arrayBuffer());
  if (
    header.length !== 12 ||
    tag(header, 0) !== "RIFF" ||
    tag(header, 8) !== "WAVE"
  )
    throw new AudioEngineError("INVALID_AUDIO", "Expected a RIFF/WAVE file");
  const declaredEnd = new DataView(header.buffer).getUint32(4, true) + 8;
  if (declaredEnd > input.size || declaredEnd < 12)
    throw new AudioEngineError("INVALID_AUDIO", "Invalid RIFF length");
  let format:
    { channels: 1 | 2; sampleRate: number; blockAlign: number } | undefined;
  let data: { offset: number; bytes: number } | undefined;
  let offset = 12;
  for (let chunks = 0; offset < declaredEnd && chunks < 1024; chunks++) {
    abortIfNeeded(signal);
    const raw = new Uint8Array(
      await input.slice(offset, offset + 8).arrayBuffer(),
    );
    if (raw.length !== 8)
      throw new AudioEngineError("INVALID_AUDIO", "Truncated WAV chunk header");
    const size = new DataView(raw.buffer).getUint32(4, true);
    const body = offset + 8;
    const next = body + size + (size % 2);
    if (next > declaredEnd)
      throw new AudioEngineError("INVALID_AUDIO", "Invalid WAV chunk length");
    const id = tag(raw, 0);
    if (id === "fmt " && !format) {
      if (size < 16)
        throw new AudioEngineError("INVALID_AUDIO", "Invalid WAV format chunk");
      const fmt = new DataView(
        await input.slice(body, body + 16).arrayBuffer(),
      );
      const code = fmt.getUint16(0, true);
      const channels = fmt.getUint16(2, true);
      const sampleRate = fmt.getUint32(4, true);
      const byteRate = fmt.getUint32(8, true);
      const blockAlign = fmt.getUint16(12, true);
      const bits = fmt.getUint16(14, true);
      if (
        code !== 1 ||
        (channels !== 1 && channels !== 2) ||
        bits !== 16 ||
        blockAlign !== channels * 2 ||
        byteRate !== sampleRate * blockAlign ||
        sampleRate < 8000 ||
        sampleRate > 192000
      )
        throw new AudioEngineError(
          "UNSUPPORTED_FORMAT",
          "Only mono/stereo 16-bit PCM WAV at 8–192 kHz is supported",
        );
      format = { channels, sampleRate, blockAlign };
    }
    if (id === "data" && !data) data = { offset: body, bytes: size };
    if (format && data) {
      if (data.bytes % format.blockAlign !== 0)
        throw new AudioEngineError(
          "INVALID_AUDIO",
          "WAV data contains an incomplete frame",
        );
      return {
        channels: format.channels,
        sampleRate: format.sampleRate,
        dataOffset: data.offset,
        frames: data.bytes / format.blockAlign,
      };
    }
    offset = next;
  }
  throw new AudioEngineError(
    "INVALID_AUDIO",
    "WAV format or data chunk not found",
  );
}

function wavHeader(
  sampleRate: number,
  channels: number,
  frames: number,
): Uint8Array<ArrayBuffer> {
  const bytes = frames * channels * 2;
  if (!Number.isSafeInteger(bytes) || bytes > 0xffffffff - 36)
    throw new AudioEngineError(
      "LIMIT_EXCEEDED",
      "Output exceeds RIFF/WAV size limit",
    );
  const out = new Uint8Array(44);
  const view = new DataView(out.buffer);
  for (const [offset, value] of [
    [0, "RIFF"],
    [8, "WAVE"],
    [12, "fmt "],
    [36, "data"],
  ] as const) {
    for (let i = 0; i < value.length; i++)
      out[offset + i] = value.charCodeAt(i);
  }
  view.setUint32(4, 36 + bytes, true);
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true);
  view.setUint16(22, channels, true);
  view.setUint32(24, sampleRate, true);
  view.setUint32(28, sampleRate * channels * 2, true);
  view.setUint16(32, channels * 2, true);
  view.setUint16(34, 16, true);
  view.setUint32(40, bytes, true);
  return out;
}

async function readFrames(
  input: Blob,
  info: WavInfo,
  start: number,
  count: number,
  signal?: AbortSignal,
): Promise<Float32Array> {
  abortIfNeeded(signal);
  const byteOffset = info.dataOffset + start * info.channels * 2;
  const bytes = await input
    .slice(byteOffset, byteOffset + count * info.channels * 2)
    .arrayBuffer();
  abortIfNeeded(signal);
  if (bytes.byteLength !== count * info.channels * 2)
    throw new AudioEngineError("INVALID_AUDIO", "Truncated WAV data");
  const view = new DataView(bytes);
  const samples = new Float32Array(count * info.channels);
  for (let i = 0; i < samples.length; i++)
    samples[i] = view.getInt16(i * 2, true) / 32768;
  return samples;
}

function channelConvert(
  samples: Float32Array,
  source: 1 | 2,
  target: 1 | 2,
  gain: number,
): Float32Array {
  const out = new Float32Array((samples.length / source) * target);
  for (let frame = 0; frame < samples.length / source; frame++) {
    if (source === 2 && target === 1)
      out[frame] =
        (samples[frame * 2] * 0.5 + samples[frame * 2 + 1] * 0.5) * gain;
    else if (source === 1 && target === 2)
      out[frame * 2] = out[frame * 2 + 1] = samples[frame] * gain;
    else
      for (let channel = 0; channel < source; channel++)
        out[frame * source + channel] =
          samples[frame * source + channel] * gain;
  }
  return out;
}

function validateOptions(options: WavProcessOptions): number {
  const chunk = options.inputChunkFrames ?? 8192;
  if (
    !Number.isInteger(options.outputSampleRate) ||
    options.outputSampleRate < 8000 ||
    options.outputSampleRate > 192000 ||
    (options.outputChannels !== 1 && options.outputChannels !== 2) ||
    !Number.isFinite(options.silenceThreshold) ||
    options.silenceThreshold < 0 ||
    options.silenceThreshold > 1 ||
    !Number.isSafeInteger(options.minSilenceFrames) ||
    options.minSilenceFrames < 1 ||
    !Number.isSafeInteger(options.waveformWindowFrames) ||
    options.waveformWindowFrames < 1 ||
    !Number.isInteger(chunk) ||
    chunk < 1 ||
    chunk > 65536 ||
    (options.normalizePeak !== undefined &&
      (!Number.isFinite(options.normalizePeak) ||
        options.normalizePeak <= 0 ||
        options.normalizePeak > 1))
  )
    throw new AudioEngineError(
      "INVALID_ARGUMENT",
      "Invalid WAV processing options",
    );
  return chunk;
}

export async function processWav(
  bindings: WasmAudioBindings,
  input: Blob,
  output: WritableStream<Uint8Array>,
  options: WavProcessOptions,
): Promise<WavProcessResult> {
  const chunkFrames = validateOptions(options);
  const info = await inspectWav(input, options.signal);
  const expectedFrames = Number(
    (BigInt(info.frames) * BigInt(options.outputSampleRate) +
      BigInt(info.sampleRate) -
      1n) /
      BigInt(info.sampleRate),
  );
  const header = wavHeader(
    options.outputSampleRate,
    options.outputChannels,
    expectedFrames,
  );
  let peak = 0;
  if (options.normalizePeak !== undefined) {
    for (let start = 0; start < info.frames; start += chunkFrames) {
      const count = Math.min(chunkFrames, info.frames - start);
      const block = await readFrames(input, info, start, count, options.signal);
      for (const sample of block) peak = Math.max(peak, Math.abs(sample));
      options.onProgress?.("analyze", start + count, info.frames);
    }
  }
  const gain =
    options.normalizePeak !== undefined && peak > 0
      ? options.normalizePeak / peak
      : 1;
  const resampler = guardDsp(
    () =>
      new bindings.WebResampler(
        info.sampleRate,
        options.outputSampleRate,
        options.outputChannels,
        1024,
      ),
  );
  let silence:
    InstanceType<WasmAudioBindings["WebSilenceDetector"]> | undefined;
  let waveform:
    InstanceType<WasmAudioBindings["WebWaveformBuilder"]> | undefined;
  let writer: WritableStreamDefaultWriter<Uint8Array> | undefined;
  let clipped = 0;
  let writtenFrames = 0;
  try {
    silence = guardDsp(
      () =>
        new bindings.WebSilenceDetector(
          info.channels,
          options.silenceThreshold,
          BigInt(options.minSilenceFrames),
        ),
    );
    waveform = guardDsp(
      () =>
        new bindings.WebWaveformBuilder(
          info.channels,
          BigInt(options.waveformWindowFrames),
        ),
    );
    writer = output.getWriter();
    const emitSilence = (values: BigUint64Array) => {
      for (let i = 0; i < values.length; i += 2)
        options.onSilence?.(Number(values[i]), Number(values[i + 1]));
    };
    const emitWaveform = (values: Float64Array) => {
      for (let i = 0; i < values.length; i += 4)
        options.onWaveform?.(
          values[i],
          values[i + 1],
          values[i + 2],
          values[i + 3],
        );
    };
    const writeSamples = async (samples: Float32Array) => {
      const bytes = new Uint8Array(samples.length * 2);
      const view = new DataView(bytes.buffer);
      for (let i = 0; i < samples.length; i++) {
        if (!Number.isFinite(samples[i]))
          throw new AudioEngineError(
            "DSP_ERROR",
            "Non-finite resampled output",
          );
        if (samples[i] < -1 || samples[i] > 1) clipped++;
        const sample = Math.max(-1, Math.min(1, samples[i]));
        const scaled = sample * 32768;
        const rounded = scaled < 0 ? -Math.round(-scaled) : Math.round(scaled);
        view.setInt16(i * 2, Math.max(-32768, Math.min(32767, rounded)), true);
      }
      writtenFrames += samples.length / options.outputChannels;
      if (bytes.length) await writer!.write(bytes);
    };
    abortIfNeeded(options.signal);
    await writer.write(header);
    for (let start = 0; start < info.frames; start += chunkFrames) {
      const count = Math.min(chunkFrames, info.frames - start);
      const block = await readFrames(input, info, start, count, options.signal);
      emitSilence(guardDsp(() => silence!.process(block)));
      emitWaveform(guardDsp(() => waveform!.process(block)));
      await writeSamples(
        guardDsp(() =>
          resampler.process(
            channelConvert(block, info.channels, options.outputChannels, gain),
          ),
        ),
      );
      options.onProgress?.("process", start + count, info.frames);
    }
    abortIfNeeded(options.signal);
    emitSilence(guardDsp(() => silence!.finish()));
    emitWaveform(guardDsp(() => waveform!.finish()));
    await writeSamples(guardDsp(() => resampler.finish()));
    if (writtenFrames !== expectedFrames)
      throw new AudioEngineError(
        "DSP_ERROR",
        "Unexpected resampled frame count",
      );
    abortIfNeeded(options.signal);
    await writer.close();
    return {
      inputFrames: info.frames,
      outputFrames: writtenFrames,
      inputSampleRate: info.sampleRate,
      inputChannels: info.channels,
      outputSampleRate: options.outputSampleRate,
      silenceThreshold: options.silenceThreshold,
      minSilenceFrames: options.minSilenceFrames,
      appliedGain: gain,
      clippedSamples: clipped,
    };
  } catch (error) {
    if (writer) await writer.abort(error).catch(() => undefined);
    throw error;
  } finally {
    writer?.releaseLock();
    waveform?.free();
    silence?.free();
    resampler.free();
  }
}
