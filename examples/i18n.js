// English source messages are the canonical keys; translations are data only.
export const messages = {
  "Local processing · Experimental": [
    "本地处理 · 实验版",
    "Pemprosesan setempat · Eksperimen",
  ],
  "WASM Audio Engine / Test workspace": [
    "WASM Audio Engine / 测试工作区",
    "WASM Audio Engine / Ruang ujian",
  ],
  "Listen closer.": ["听清每一处细节。", "Dengar dengan lebih jelas."],
  "Try the audio engine in your browser. Prepare a WAV, inspect its waveform, and listen to the result — your audio stays on this device.":
    [
      "在浏览器中体验音频引擎：处理 WAV、查看波形并试听结果。音频始终保留在本设备。",
      "Cuba enjin audio dalam pelayar. Proses WAV, semak bentuk gelombang dan dengar hasilnya — audio kekal pada peranti ini.",
    ],
  "01 Choose audio": ["01 选择音频", "01 Pilih audio"],
  "Select a classic PCM16 WAV file to begin.": [
    "选择标准 PCM16 WAV 文件以开始。",
    "Pilih fail WAV PCM16 standard untuk bermula.",
  ],
  "Choose a WAV file": ["选择 WAV 文件", "Pilih fail WAV"],
  "Mono or stereo · 8–192 kHz · up to 8 MiB": [
    "单声道或立体声 · 8–192 kHz · 最大 8 MiB",
    "Mono atau stereo · 8–192 kHz · sehingga 8 MiB",
  ],
  "Try a generated sample": ["试用生成的示例音频", "Cuba sampel yang dijana"],
  "MP3, M4A, AAC and Opus are not supported by this test page. The size limit exists because this page keeps the output in memory.":
    [
      "本测试页面暂不支持 MP3、M4A、AAC 和 Opus。由于输出保存在内存中，页面设有文件大小限制。",
      "Halaman ujian ini tidak menyokong MP3, M4A, AAC dan Opus. Saiz fail dihadkan kerana output disimpan dalam memori.",
    ],
  "02 Processing settings": ["02 处理设置", "02 Tetapan pemprosesan"],
  "Adjust the output and detection settings before processing.": [
    "处理前可调整输出和检测设置。",
    "Laraskan tetapan output dan pengesanan sebelum memproses.",
  ],
  "Output sample rate": ["输出采样率", "Kadar pensampelan output"],
  "Output channels": ["输出声道", "Saluran output"],
  Mono: ["单声道", "Mono"],
  Stereo: ["立体声", "Stereo"],
  "Silence threshold": ["静音阈值", "Ambang senyap"],
  "Minimum silence (source frames)": [
    "最短静音长度（原音频帧）",
    "Tempoh senyap minimum (bingkai sumber)",
  ],
  "Normalize peak to 80%": [
    "将峰值标准化至 80%",
    "Normalkan puncak kepada 80%",
  ],
  "Process audio": ["处理音频", "Proses audio"],
  Cancel: ["取消", "Batal"],
  "Loading audio engine…": ["正在加载音频引擎…", "Memuatkan enjin audio…"],
  "03 Analysis": ["03 分析", "03 Analisis"],
  "Waveform and quiet intervals use the original recording timeline.": [
    "波形和静音区间均使用原始录音的时间轴。",
    "Bentuk gelombang dan selang senyap menggunakan garis masa rakaman asal.",
  ],
  "INPUT DURATION": ["输入时长", "TEMPOH INPUT"],
  "QUIET INTERVALS": ["静音区间", "SELANG SENYAP"],
  "APPLIED GAIN": ["应用增益", "GAIN DIGUNAKAN"],
  "Low energy is not voice detection. Quiet speech and background noise can affect the result.":
    [
      "低能量检测不等于语音检测。轻声讲话和背景噪声可能影响结果。",
      "Pengesanan tenaga rendah bukan pengesanan pertuturan. Pertuturan perlahan dan bunyi latar boleh mempengaruhi hasil.",
    ],
  "04 Listen & export": ["04 试听与导出", "04 Dengar dan eksport"],
  "Your processed WAV will appear here.": [
    "处理后的 WAV 将显示在此处。",
    "WAV yang telah diproses akan dipaparkan di sini.",
  ],
  "Download WAV": ["下载 WAV", "Muat turun WAV"],
  "05 More engine tests": ["05 更多功能测试", "05 Ujian enjin tambahan"],
  "Process the source above first, then try excerpts, detected quiet interval removal, chunks, merging, or a frequency analysis.":
    [
      "请先处理上方的原音频，再测试片段提取、删除检测到的静音区间、分块、合并或频谱分析。",
      "Proses audio sumber di atas dahulu, kemudian cuba pengekstrakan petikan, pembuangan selang senyap, pembahagian, penggabungan atau analisis frekuensi.",
    ],
  "Excerpt start (seconds)": ["片段起点（秒）", "Mula petikan (saat)"],
  "Excerpt end (seconds)": ["片段终点（秒）", "Akhir petikan (saat)"],
  "Chunk length (seconds)": ["分块时长（秒）", "Tempoh segmen (saat)"],
  "Merge with another PCM16 WAV": [
    "与另一个 PCM16 WAV 合并",
    "Gabungkan dengan WAV PCM16 lain",
  ],
  "Extract excerpt": ["提取片段", "Ekstrak petikan"],
  "Remove detected quiet intervals": [
    "删除检测到的静音区间",
    "Buang selang senyap yang dikesan",
  ],
  "Make chunks": ["生成分块", "Bahagikan audio"],
  "Merge WAVs": ["合并 WAV", "Gabungkan WAV"],
  "Analyze frequency": ["分析频谱", "Analisis frekuensi"],
  "Removing quiet intervals can remove speech. Merging requires matching sample rates and channel counts. Chunk output is limited to 50 files in this test page.":
    [
      "删除静音区间可能删掉讲话。合并要求采样率和声道数一致。本页面最多生成 50 个分块文件。",
      "Pembuangan selang senyap mungkin membuang pertuturan. Penggabungan memerlukan kadar pensampelan dan bilangan saluran yang sepadan. Halaman ini mengehadkan output kepada 50 fail segmen.",
    ],
  "Process a WAV to enable these tests.": [
    "处理一个 WAV 后即可启用这些测试。",
    "Proses WAV untuk mengaktifkan ujian ini.",
  ],
  "Ready to test additional operations.": [
    "可以测试其他操作了。",
    "Sedia untuk menguji operasi tambahan.",
  ],
  "Working locally…": ["正在本地处理…", "Memproses secara setempat…"],
  "Ready to process locally.": [
    "已准备好在本地处理。",
    "Sedia untuk pemprosesan setempat.",
  ],
  "Choose a WAV file to begin.": [
    "选择 WAV 文件以开始。",
    "Pilih fail WAV untuk bermula.",
  ],
  "Choose a WAV file first.": ["请先选择 WAV 文件。", "Pilih fail WAV dahulu."],
  "This test page accepts files up to 8 MiB.": [
    "本测试页面接受最大 8 MiB 的文件。",
    "Halaman ujian ini menerima fail sehingga 8 MiB.",
  ],
  "Enter a threshold from 0 to 1 and a positive whole number of source frames.":
    [
      "请输入 0 到 1 之间的阈值，以及正整数的原音频帧数。",
      "Masukkan ambang antara 0 hingga 1 dan bilangan bingkai sumber dalam nombor bulat positif.",
    ],
  "Output exceeds this page's 32 MiB limit.": [
    "输出超过本页面的 32 MiB 限制。",
    "Output melebihi had 32 MiB halaman ini.",
  ],
  "Processing complete. Your audio stayed on this device.": [
    "处理完成。音频始终保留在本设备。",
    "Pemprosesan selesai. Audio kekal pada peranti ini.",
  ],
  "Processing stopped: {reason}": [
    "处理已停止：{reason}",
    "Pemprosesan dihentikan: {reason}",
  ],
  "Operation failed: {reason}": [
    "操作失败：{reason}",
    "Operasi gagal: {reason}",
  ],
  "Engine failed to load. Build the SDK and serve the repository over localhost.":
    [
      "引擎加载失败。请检查网络；本地开发时请构建 SDK 并通过 localhost 提供页面。",
      "Enjin gagal dimuatkan. Semak sambungan; untuk pembangunan setempat, bina SDK dan hidangkan halaman melalui localhost.",
    ],
  Analyzing: ["分析中", "Menganalisis"],
  Processing: ["处理中", "Memproses"],
  "{phase}: {completed} / {total} frames": [
    "{phase}：{completed} / {total} 帧",
    "{phase}: {completed} / {total} bingkai",
  ],
  "{rate} Hz · {channels} · {frames} frames": [
    "{rate} Hz · {channels} · {frames} 帧",
    "{rate} Hz · {channels} · {frames} bingkai",
  ],
  "{count} saturated samples": ["{count} 个饱和样本", "{count} sampel tepu"],
  "{seconds}s": ["{seconds} 秒", "{seconds} saat"],
  "{start}s – {end}s": ["{start} 秒 – {end} 秒", "{start} saat – {end} saat"],
  "Showing 100 of {count} intervals.": [
    "显示 {count} 个区间中的前 100 个。",
    "Memaparkan 100 daripada {count} selang.",
  ],
  "Download excerpt": ["下载片段", "Muat turun petikan"],
  "Download trimmed WAV": ["下载裁剪后的 WAV", "Muat turun WAV yang dipangkas"],
  "Download merged WAV": ["下载合并后的 WAV", "Muat turun WAV gabungan"],
  "Chunk {count}": ["分块 {count}", "Segmen {count}"],
  "Extracted source frames {start}–{end}.": [
    "已提取原音频的第 {start}–{end} 帧。",
    "Bingkai sumber {start}–{end} telah diekstrak.",
  ],
  "Removed {count} quiet intervals; {frames} frames remain.": [
    "已删除 {count} 个静音区间；剩余 {frames} 帧。",
    "{count} selang senyap dibuang; {frames} bingkai berbaki.",
  ],
  "Created {count} source-format chunks.": [
    "已生成 {count} 个保留原格式的分块。",
    "{count} segmen dalam format sumber telah dijana.",
  ],
  "Merged {frames} frames at {rate} Hz.": [
    "已合并 {frames} 帧，采样率为 {rate} Hz。",
    "{frames} bingkai digabungkan pada {rate} Hz.",
  ],
  "First {frames} decoded frames: strongest bin {frequency} Hz, amplitude {amplitude}. Browser decoding may resample to {rate} Hz.":
    [
      "前 {frames} 个解码帧：最强频率为 {frequency} Hz，幅度为 {amplitude}。浏览器解码可能重采样至 {rate} Hz。",
      "{frames} bingkai pertama yang dinyahkod: bin terkuat {frequency} Hz, amplitud {amplitude}. Pelayar mungkin mensampel semula kepada {rate} Hz.",
    ],
  "Enter a positive whole number of seconds.": [
    "请输入正整数秒数。",
    "Masukkan bilangan saat dalam nombor bulat positif.",
  ],
  "More than 50 chunks; choose a longer chunk length.": [
    "分块超过 50 个；请增加分块时长。",
    "Lebih daripada 50 segmen; pilih tempoh segmen yang lebih panjang.",
  ],
  "Choose a second PCM16 WAV up to 8 MiB.": [
    "请选择另一个不超过 8 MiB 的 PCM16 WAV。",
    "Pilih WAV PCM16 kedua yang tidak melebihi 8 MiB.",
  ],
  "At least four audio frames are required.": [
    "至少需要四个音频帧。",
    "Sekurang-kurangnya empat bingkai audio diperlukan.",
  ],
  "The operation was cancelled.": ["操作已取消。", "Operasi dibatalkan."],
  "Invalid audio settings or range.": [
    "音频设置或范围无效。",
    "Tetapan atau julat audio tidak sah.",
  ],
  "The WAV file is malformed or incomplete.": [
    "WAV 文件格式错误或不完整。",
    "Fail WAV rosak atau tidak lengkap.",
  ],
  "Unsupported audio format or incompatible WAV inputs.": [
    "音频格式不受支持，或 WAV 输入格式不一致。",
    "Format audio tidak disokong atau input WAV tidak serasi.",
  ],
  "The operation exceeds a supported size limit.": [
    "操作超过支持的大小限制。",
    "Operasi melebihi had saiz yang disokong.",
  ],
  "Audio processing failed.": ["音频处理失败。", "Pemprosesan audio gagal."],
  "The audio engine could not initialize.": [
    "音频引擎无法初始化。",
    "Enjin audio tidak dapat dimulakan.",
  ],
  "The operation could not be completed. Check the input and try again.": [
    "操作无法完成。请检查输入后重试。",
    "Operasi tidak dapat diselesaikan. Semak input dan cuba lagi.",
  ],
  Language: ["语言", "Bahasa"],
  "Processing progress": ["处理进度", "Kemajuan pemprosesan"],
  "Source audio waveform": ["原音频波形", "Bentuk gelombang audio sumber"],
  "Source audio playback": ["原音频播放", "Main balik audio sumber"],
  "Processed audio playback": ["处理后音频播放", "Main balik audio diproses"],
  "Install app": ["安装应用", "Pasang aplikasi"],
  "Install help": ["安装说明", "Panduan pemasangan"],
  "Use your browser's Install app menu. On iPhone or iPad, choose Share, then Add to Home Screen.":
    [
      "使用浏览器菜单中的“安装应用”。在 iPhone 或 iPad 上，选择“分享”，然后选择“添加到主屏幕”。",
      "Gunakan menu Pasang aplikasi dalam pelayar. Pada iPhone atau iPad, pilih Kongsi, kemudian Tambah ke Skrin Utama.",
    ],
  "Preparing offline resources…": [
    "正在准备离线资源…",
    "Menyediakan sumber luar talian…",
  ],
  "Ready for offline use": ["已可离线使用", "Sedia untuk kegunaan luar talian"],
  "Offline · app resources cached": [
    "离线 · 应用资源已缓存",
    "Luar talian · sumber aplikasi dicache",
  ],
  "Offline setup unavailable. Audio processing still works while this page is loaded.":
    [
      "离线设置暂不可用。页面保持打开时仍可处理音频。",
      "Persediaan luar talian tidak tersedia. Audio masih boleh diproses selagi halaman ini dibuka.",
    ],
  "Offline installation is available in a built or deployed site.": [
    "离线安装可在构建后或已部署的网站中使用。",
    "Pemasangan luar talian tersedia pada laman yang telah dibina atau diterbitkan.",
  ],
  "Update {version} is ready. Reloading clears current audio results.": [
    "更新 {version} 已就绪。重新加载会清除当前音频结果。",
    "Kemas kini {version} tersedia. Muat semula akan memadamkan hasil audio semasa.",
  ],
  "Update and reload": ["更新并重新加载", "Kemas kini dan muat semula"],
  Later: ["稍后", "Kemudian"],
  "Review update": ["查看更新", "Semak kemas kini"],
  "Audio results are temporary. Only app resources and your language preference are saved.":
    [
      "音频结果为临时数据。仅保存应用资源和语言偏好。",
      "Hasil audio bersifat sementara. Hanya sumber aplikasi dan pilihan bahasa disimpan.",
    ],
};

export const languages = ["en", "zh-Hans", "ms"];
const storageKey = "wasm-audio-engine.language";
let locale = "en";

export function t(key, values = {}) {
  const index = locale === "zh-Hans" ? 0 : locale === "ms" ? 1 : -1;
  const template = index < 0 ? key : (messages[key]?.[index] ?? key);
  return template.replace(/\{(\w+)\}/g, (_, name) => {
    const value = values[name];
    if (value && typeof value === "object" && "key" in value)
      return t(value.key, value.values);
    if (typeof value === "number")
      return new Intl.NumberFormat(locale).format(value);
    return String(value ?? "");
  });
}

function render(node) {
  if (node.dataset.i18n)
    node.textContent = t(
      node.dataset.i18n,
      JSON.parse(node.dataset.i18nValues || "{}"),
    );
  if (node.dataset.i18nAria)
    node.setAttribute("aria-label", t(node.dataset.i18nAria));
}

export function message(node, key, values = {}) {
  node.dataset.i18n = key;
  node.dataset.i18nValues = JSON.stringify(values);
  render(node);
}

export function setLanguage(value, persist = true) {
  locale = languages.includes(value) ? value : "en";
  if (typeof document !== "undefined") {
    document.documentElement.lang = locale;
    document.querySelectorAll("[data-i18n], [data-i18n-aria]").forEach(render);
    const selector = document.getElementById("language");
    if (selector) selector.value = locale;
    document.dispatchEvent(new Event("languagechange"));
  }
  if (persist) {
    try {
      localStorage.setItem(storageKey, locale);
    } catch {
      /* Storage can be unavailable in private or restricted contexts. */
    }
  }
}

export function initLanguage() {
  let saved;
  try {
    saved = localStorage.getItem(storageKey);
  } catch {
    /* Keep the English default. */
  }
  setLanguage(saved, false);
  document
    .getElementById("language")
    .addEventListener("change", (event) => setLanguage(event.target.value));
}

export function uiError(key) {
  const error = new Error(key);
  error.uiKey = key;
  return error;
}
export function errorInfo(error) {
  const codes = {
    INVALID_ARGUMENT: "Invalid audio settings or range.",
    INVALID_AUDIO: "The WAV file is malformed or incomplete.",
    UNSUPPORTED_FORMAT: "Unsupported audio format or incompatible WAV inputs.",
    LIMIT_EXCEEDED: "The operation exceeds a supported size limit.",
    DSP_ERROR: "Audio processing failed.",
    INITIALIZATION_FAILED: "The audio engine could not initialize.",
  };
  return {
    key:
      error?.name === "AbortError"
        ? "The operation was cancelled."
        : error?.uiKey ||
          codes[error?.code] ||
          "The operation could not be completed. Check the input and try again.",
  };
}
