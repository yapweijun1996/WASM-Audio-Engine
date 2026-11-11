export function setupTools(engine) {
  const $ = (id) => document.getElementById(id);
  const ids = [
    "extract-test",
    "trim-test",
    "chunk-test",
    "merge-test",
    "spectrum-test",
  ];
  let source,
    info,
    quiet = [],
    urls = [],
    busy = false;
  const clear = () => {
    for (const url of urls) URL.revokeObjectURL(url);
    urls = [];
    $("tools-output").replaceChildren();
  };
  const reset = () => {
    source = undefined;
    info = undefined;
    quiet = [];
    clear();
    for (const id of ids) $(id).disabled = true;
    $("tools-status").textContent = "Process a WAV to enable these tests.";
  };
  document.addEventListener("audio-source-change", reset);
  const setSource = (input, result, intervals) => {
    source = input;
    info = result;
    quiet = intervals;
    for (const id of ids) $(id).disabled = false;
    $("tools-status").textContent = "Ready to test additional operations.";
  };
  const link = (blob, name, label) => {
    const url = URL.createObjectURL(blob);
    urls.push(url);
    const a = document.createElement("a");
    a.href = url;
    a.download = name;
    a.textContent = label;
    a.className = "download";
    $("tools-output").append(a);
  };
  const run = (id, action) =>
    $(id).addEventListener("click", async () => {
      if (!source || busy) return;
      busy = true;
      $("file").disabled = true;
      $("sample").disabled = true;
      $("prepare").disabled = true;
      for (const item of ids) $(item).disabled = true;
      clear();
      $("tools-status").textContent = "Working locally…";
      try {
        await action();
      } catch (error) {
        clear();
        $("tools-status").textContent = `Operation failed: ${String(error)}`;
      } finally {
        busy = false;
        $("file").disabled = false;
        $("sample").disabled = false;
        $("prepare").disabled = false;
        if (source) for (const item of ids) $(item).disabled = false;
      }
    });
  const sink = () => {
    let bytes = 0;
    const parts = [];
    return {
      parts,
      stream: new WritableStream({
        write(data) {
          bytes += data.length;
          if (bytes > 32 * 1024 * 1024)
            throw new Error("Output exceeds this page's 32 MiB limit.");
          parts.push(data);
        },
      }),
    };
  };
  run("extract-test", async () => {
    const clip = await engine.extractWavSeconds(source, {
      startSeconds: Number($("extract-start").value),
      endSeconds: Number($("extract-end").value),
    });
    link(clip.audio, "excerpt.wav", "Download excerpt");
    $("tools-status").textContent =
      `Extracted source frames ${clip.startFrame}–${clip.endFrame}.`;
  });
  run("trim-test", async () => {
    const output = sink();
    const result = await engine.trimWav(
      source,
      quiet.map(([startFrame, endFrame]) => ({ startFrame, endFrame })),
      output.stream,
    );
    link(
      new Blob(output.parts, { type: "audio/wav" }),
      "trimmed.wav",
      "Download trimmed WAV",
    );
    $("tools-status").textContent =
      `Removed ${quiet.length} quiet intervals; ${result.outputFrames} frames remain.`;
  });
  run("chunk-test", async () => {
    const seconds = Number($("chunk-length").value);
    if (
      !Number.isSafeInteger(seconds) ||
      seconds < 1 ||
      seconds * info.inputSampleRate > Number.MAX_SAFE_INTEGER
    )
      throw new Error("Enter a positive whole number of seconds.");
    const length = seconds * info.inputSampleRate;
    if (Math.ceil(info.inputFrames / length) > 50)
      throw new Error("More than 50 chunks; choose a longer chunk length.");
    let count = 0;
    for await (const chunk of engine.chunkWav(source, {
      chunkFrames: length,
    })) {
      count++;
      link(chunk.audio, `chunk-${count}.wav`, `Chunk ${count}`);
    }
    $("tools-status").textContent = `Created ${count} source-format chunks.`;
  });
  run("merge-test", async () => {
    const second = $("merge-file").files[0];
    if (!second || second.size > 8 * 1024 * 1024)
      throw new Error("Choose a second PCM16 WAV up to 8 MiB.");
    const output = sink();
    const result = await engine.mergeWav([source, second], output.stream);
    link(
      new Blob(output.parts, { type: "audio/wav" }),
      "merged.wav",
      "Download merged WAV",
    );
    $("tools-status").textContent =
      `Merged ${result.outputFrames} frames at ${result.sampleRate} Hz.`;
  });
  run("spectrum-test", async () => {
    const context = new AudioContext();
    try {
      const decoded = await context.decodeAudioData(await source.arrayBuffer());
      const n = Math.min(4096, decoded.length);
      if (n < 4) throw new Error("At least four audio frames are required.");
      const samples = new Float32Array(n);
      for (let c = 0; c < decoded.numberOfChannels; c++) {
        const data = decoded.getChannelData(c);
        for (let i = 0; i < n; i++)
          samples[i] += data[i] / decoded.numberOfChannels;
      }
      const bins = engine.spectrum(samples, decoded.sampleRate);
      const peak = bins.reduce((a, b) => (b.amplitude > a.amplitude ? b : a));
      $("tools-status").textContent =
        `First ${n} decoded frames: strongest bin ${peak.frequencyHz.toFixed(1)} Hz, amplitude ${peak.amplitude.toFixed(4)}. Browser decoding may resample to ${decoded.sampleRate} Hz.`;
    } finally {
      await context.close();
    }
  });
  return { reset, setSource };
}
