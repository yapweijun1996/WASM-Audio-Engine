import assert from "node:assert/strict";
import test from "node:test";
import {
  AudioEngineError,
  API_VERSION,
  ERROR_CONTRACT_VERSION,
  createAudioEngine,
} from "../dist/index.js";
const bindings = {
  default: async () => {},
  normalize_pcm_peak() {
    throw "bad sample";
  },
  analyze_spectrum() {
    throw "bad window";
  },
};
const engine = await createAudioEngine(bindings);
const sink = () => new WritableStream({ write() {} });
test("versioned DSP failure retains its cause", () => {
  assert.equal(API_VERSION, "0.1.0");
  assert.equal(ERROR_CONTRACT_VERSION, 1);
  assert.throws(
    () => engine.normalizePeak(new Float32Array([NaN]), 1),
    (e) =>
      e instanceof AudioEngineError &&
      e.code === "DSP_ERROR" &&
      e.cause === "bad sample" &&
      e.contractVersion === 1,
  );
});
test("initialization rejects with typed failure and original cause", async () => {
  const cause = new Error("load failure");
  await assert.rejects(
    createAudioEngine({
      ...bindings,
      default: async () => {
        throw cause;
      },
    }),
    (e) => e.code === "INITIALIZATION_FAILED" && e.cause === cause,
  );
});
test("malformed audio has a stable code", async () => {
  await assert.rejects(
    engine.extractWav(new Blob(["bad"]), { startFrame: 0, endFrame: 1 }),
    (e) => e.code === "INVALID_AUDIO",
  );
});
test("empty merge input has a stable code", async () => {
  await assert.rejects(
    engine.mergeWav([], sink()),
    (e) => e.code === "INVALID_ARGUMENT",
  );
});
test("chunk iterator reports malformed audio through iteration", async () => {
  await assert.rejects(
    async () => {
      for await (const chunk of engine.chunkWav(new Blob(["bad"]), {
        chunkFrames: 1024,
      }))
        void chunk;
    },
    (e) => e.code === "INVALID_AUDIO",
  );
});
test("caller abort reason propagates unchanged", async () => {
  const reason = new Error("caller cancellation");
  const controller = new AbortController();
  controller.abort(reason);
  await assert.rejects(
    engine.extractWav(new Blob(["bad"]), {
      startFrame: 0,
      endFrame: 1,
      signal: controller.signal,
    }),
    (e) => e === reason,
  );
});
