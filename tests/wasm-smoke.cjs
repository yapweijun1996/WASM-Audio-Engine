// Pass the path to wasm-bindgen's nodejs output as the first argument.
const assert = require('node:assert/strict');
const engine = require(process.argv[2]);

const samples = new Float32Array([0.25, -0.5, 0]);
assert.equal(engine.normalize_pcm_peak(samples, 1), 2);
assert.deepEqual(Array.from(samples), [0.5, -1, 0]);

const silence = new Float32Array([0, 0]);
assert.equal(engine.normalize_pcm_peak(silence, 0.5), 1);
assert.deepEqual(Array.from(silence), [0, 0]);

assert.throws(() => engine.normalize_pcm_peak(samples, 0));
console.log('WASM normalization smoke test passed');
