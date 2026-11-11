// Exercise the built app through its real DOM and service worker in isolated Chrome.
import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import { EventEmitter } from "node:events";
import { readFile, writeFile, mkdtemp } from "node:fs/promises";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { resolve, join, extname, sep } from "node:path";

const liveArgument = process.argv[2] === "--live" ? process.argv[3] : null;
const liveBase = liveArgument
  ? new URL(liveArgument).href.replace(/\/?$/, "/")
  : null;
if (process.argv[2] === "--live" && !liveBase)
  throw new Error("Provide the live site URL");
const initialSite = resolve(liveBase ? "_site" : process.argv[2] || "_site");
const scratch = await mkdtemp(join(tmpdir(), "wasm-audio-pwa-e2e-"));
let site = initialSite;
const prefix = "/WASM-Audio-Engine/";
const mime = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".wasm": "application/wasm",
  ".json": "application/json",
  ".webmanifest": "application/manifest+json",
  ".png": "image/png",
  ".svg": "image/svg+xml",
};
const server = createServer(async (request, response) => {
  try {
    const path = decodeURIComponent(
      new URL(request.url, "http://localhost").pathname,
    );
    if (request.method !== "GET" || !path.startsWith(prefix)) {
      response.writeHead(404).end();
      return;
    }
    const name = resolve(
      site,
      path.slice(prefix.length) + (path.endsWith("/") ? "index.html" : ""),
    );
    if (!name.startsWith(site + sep)) {
      response.writeHead(403).end();
      return;
    }
    const bytes = await readFile(name);
    response
      .writeHead(200, {
        "Content-Type": mime[extname(name)] || "text/plain",
        "Cache-Control": "no-store",
      })
      .end(bytes);
  } catch {
    response.writeHead(404).end();
  }
});
await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
const base = liveBase || `http://127.0.0.1:${server.address().port}${prefix}`;
const chromePath =
  process.env.CHROME_PATH ||
  (process.platform === "win32"
    ? "C:/Program Files/Google/Chrome/Application/chrome.exe"
    : "/usr/bin/google-chrome");
const child = spawn(
  chromePath,
  [
    "--headless=new",
    "--no-first-run",
    "--disable-extensions",
    "--disable-background-networking",
    "--remote-debugging-port=0",
    `--user-data-dir=${join(scratch, "profile")}`,
    "about:blank",
  ],
  { windowsHide: true, stdio: "ignore" },
);
let socket;
const results = [];
const consoleErrors = [];
const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

try {
  let port;
  for (let i = 0; i < 200; i++) {
    try {
      port = (
        await readFile(join(scratch, "profile/DevToolsActivePort"), "utf8")
      ).split("\n")[0];
      break;
    } catch {
      await delay(100);
    }
  }
  assert.ok(port, "Isolated Chrome did not start");
  const pages = await (
    await fetch(`http://127.0.0.1:${port}/json/list`)
  ).json();
  socket = new WebSocket(
    pages.find((page) => page.type === "page").webSocketDebuggerUrl,
  );
  await new Promise((resolve, reject) => {
    socket.addEventListener("open", resolve, { once: true });
    socket.addEventListener("error", reject, { once: true });
  });
  const events = new EventEmitter(),
    pending = new Map();
  let next = 0;
  socket.addEventListener("message", (event) => {
    const data = JSON.parse(event.data);
    if (data.id && pending.has(data.id)) {
      const entry = pending.get(data.id);
      pending.delete(data.id);
      clearTimeout(entry.timer);
      data.error
        ? entry.reject(new Error(JSON.stringify(data.error)))
        : entry.resolve(data.result);
    }
    if (data.method) events.emit(data.method, data.params);
  });
  const send = (method, params = {}) =>
    new Promise((resolve, reject) => {
      const id = ++next;
      const timer = setTimeout(() => {
        pending.delete(id);
        reject(new Error("CDP timeout: " + method));
      }, 15000);
      pending.set(id, { resolve, reject, timer });
      socket.send(JSON.stringify({ id, method, params }));
    });
  const evaluate = async (expression) => {
    const result = await send("Runtime.evaluate", {
      expression,
      returnByValue: true,
      awaitPromise: true,
    });
    if (result.exceptionDetails)
      throw new Error(JSON.stringify(result.exceptionDetails));
    return result.result.value;
  };
  const wait = async (expression, label) => {
    for (let i = 0; i < 300; i++) {
      if (await evaluate(expression)) return;
      await delay(50);
    }
    throw new Error("Timed out: " + label);
  };
  const load = () =>
    new Promise((resolve, reject) => {
      const timer = setTimeout(
        () => reject(new Error("Navigation timeout")),
        15000,
      );
      events.once("Page.loadEventFired", () => {
        clearTimeout(timer);
        resolve();
      });
    });
  const navigate = async (url) => {
    const loaded = load();
    await send("Page.navigate", { url });
    await loaded;
    await wait(
      "!!document.getElementById('prepare') && !document.getElementById('prepare').disabled",
      "engine ready",
    );
  };
  const reload = async () => {
    const loaded = load();
    await send("Page.reload", { ignoreCache: true });
    await loaded;
    await wait(
      "!!document.getElementById('prepare') && !document.getElementById('prepare').disabled",
      "engine after reload",
    );
  };
  const language = async (value) =>
    evaluate(
      `document.getElementById('language').value=${JSON.stringify(value)};document.getElementById('language').dispatchEvent(new Event('change'));`,
    );
  const click = async (id) =>
    evaluate(
      `{const element=document.getElementById(${JSON.stringify(id)});if(!element || element.disabled || !element.getClientRects().length)throw new Error('Control is not interactive: '+${JSON.stringify(id)});element.click();}`,
    );
  const processSample = async () => {
    await click("sample");
    await click("prepare");
    await wait("!document.getElementById('prepare').disabled", "processing");
    assert.equal(
      await evaluate("document.getElementById('download').hidden"),
      false,
    );
  };
  const step = async (name, fn) => {
    await fn();
    results.push(name);
    console.log("PASS", name);
  };
  events.on("Runtime.exceptionThrown", (data) =>
    consoleErrors.push(data.exceptionDetails.text),
  );
  events.on("Log.entryAdded", (data) => {
    if (data.entry.level === "error") consoleErrors.push(data.entry.text);
  });
  await send("Page.enable");
  await send("Runtime.enable");
  await send("Network.enable");
  await send("Log.enable");
  await send("Emulation.setLocaleOverride", { locale: "zh-CN" });
  await navigate(base + "examples/");

  await step(
    "English default, Chrome installability, and active scoped service worker",
    async () => {
      assert.equal(await evaluate("document.documentElement.lang"), "en");
      assert.equal(
        await evaluate("document.getElementById('prepare').textContent.trim()"),
        "Process audio",
      );
      await wait(
        "!!navigator.serviceWorker.controller",
        "service worker control",
      );
      const manifest = await send("Page.getAppManifest");
      assert.ok(manifest.url.endsWith("/manifest.webmanifest"));
      assert.equal(manifest.errors.length, 0);
      const install = await send("Page.getInstallabilityErrors");
      assert.deepEqual(install.installabilityErrors, []);
      const scope = await evaluate(
        "navigator.serviceWorker.getRegistration().then(r=>r.scope)",
      );
      assert.equal(scope, base);
    },
  );
  await step(
    "All three languages process, extract, trim, chunk, merge and analyze",
    async () => {
      for (const [locale, label] of [
        ["en", "Process audio"],
        ["zh-Hans", "处理音频"],
        ["ms", "Proses audio"],
      ]) {
        await language(locale);
        assert.equal(
          await evaluate(
            "document.getElementById('prepare').textContent.trim()",
          ),
          label,
        );
        await processSample();
        await evaluate(
          "{const dt=new DataTransfer();dt.items.add(document.getElementById('file').files[0]);document.getElementById('merge-file').files=dt.files;}",
        );
        for (const operation of [
          "extract",
          "trim",
          "chunk",
          "merge",
          "spectrum",
        ]) {
          await click(operation + "-test");
          await wait(
            `!document.getElementById('${operation}-test').disabled`,
            operation,
          );
          assert.equal(
            await evaluate(
              "document.getElementById('tools-status').dataset.i18n === 'Operation failed: {reason}'",
            ),
            false,
          );
        }
      }
    },
  );
  await step(
    "Language switching preserves results and translates validation",
    async () => {
      const before = await evaluate("document.getElementById('download').href");
      await language("zh-Hans");
      assert.equal(
        await evaluate("document.getElementById('download').href"),
        before,
      );
      assert.match(
        await evaluate("document.getElementById('output-note').textContent"),
        /单声道/,
      );
      await evaluate("document.getElementById('threshold').value='2'");
      await click("prepare");
      assert.match(
        await evaluate("document.getElementById('status').textContent"),
        /请输入/,
      );
      await evaluate("document.getElementById('threshold').value='0.01'");
      await language("ms");
      assert.match(
        await evaluate("document.getElementById('status').textContent"),
        /Masukkan/,
      );
      await reload();
      assert.equal(await evaluate("document.documentElement.lang"), "ms");
    },
  );
  await step("Malformed audio and cancellation are localized", async () => {
    await language("zh-Hans");
    await evaluate(
      "{const dt=new DataTransfer();dt.items.add(new File(['bad'],'bad.wav'));document.getElementById('file').files=dt.files;document.getElementById('file').dispatchEvent(new Event('change'));}",
    );
    await click("prepare");
    await wait(
      "!document.getElementById('prepare').disabled",
      "malformed audio",
    );
    assert.match(
      await evaluate("document.getElementById('status').textContent"),
      /格式错误或不完整/,
    );
    assert.equal(
      await evaluate("document.getElementById('download').hidden"),
      true,
    );
    await language("ms");
    assert.match(
      await evaluate("document.getElementById('status').textContent"),
      /rosak atau tidak lengkap/,
    );
    await evaluate(
      "document.getElementById('sample').click();document.getElementById('prepare').click();document.getElementById('cancel').click()",
    );
    await wait(
      "!document.getElementById('prepare').disabled",
      "cancelled processing",
    );
    assert.match(
      await evaluate("document.getElementById('status').textContent"),
      /dibatalkan/,
    );
    await processSample();
  });
  await step("Phone layout fits in each language", async () => {
    await send("Emulation.setDeviceMetricsOverride", {
      width: 390,
      height: 844,
      deviceScaleFactor: 1,
      mobile: true,
    });
    for (const locale of ["en", "zh-Hans", "ms"]) {
      await language(locale);
      assert.equal(
        await evaluate("document.documentElement.scrollWidth <= innerWidth"),
        true,
        locale,
      );
    }
    await send("Emulation.clearDeviceMetricsOverride");
  });
  await step(
    "Offline reload processes audio and caches no recordings",
    async () => {
      await send("Network.emulateNetworkConditions", {
        offline: true,
        latency: 0,
        downloadThroughput: 0,
        uploadThroughput: 0,
      });
      await reload();
      await processSample();
      const keys = await evaluate(
        "caches.keys().then(async names=>(await Promise.all(names.map(async n=>(await (await caches.open(n)).keys()).map(r=>r.url)))).flat())",
      );
      assert.ok(keys.some((url) => url.endsWith(".wasm")));
      assert.ok(
        keys.every(
          (url) => url.startsWith(base) && !/\.(wav|mp3|m4a)(?:$|\?)/.test(url),
        ),
      );
      await send("Network.emulateNetworkConditions", {
        offline: false,
        latency: 0,
        downloadThroughput: -1,
        uploadThroughput: -1,
      });
    },
  );
  if (!liveBase)
    await step(
      "Updates wait; Later preserves results; explicit update preserves language and other caches",
      async () => {
        const initial = JSON.parse(
          await readFile(join(initialSite, "build.json"), "utf8"),
        );
        const revision =
          initial.revision === "a".repeat(40) ? "b".repeat(40) : "a".repeat(40);
        const updated = join(scratch, "updated-site");
        execFileSync(
          process.platform === "win32" ? "python" : "python3",
          [
            "scripts/build_site.py",
            "--output",
            updated,
            "--revision",
            revision,
          ],
          { cwd: resolve("."), stdio: "pipe" },
        );
        await evaluate("caches.open('unrelated-app-cache')");
        site = updated;
        await evaluate(
          "navigator.serviceWorker.getRegistration().then(r=>r.update())",
        );
        await wait(
          "!document.getElementById('pwa-update').hidden",
          "waiting update prompt",
        );
        assert.equal(
          await evaluate(
            "fetch('../build.json').then(r=>r.json()).then(m=>m.revision)",
          ),
          initial.revision,
        );
        await click("later-update");
        assert.equal(
          await evaluate("document.getElementById('pwa-update').hidden"),
          true,
        );
        assert.equal(
          await evaluate("document.getElementById('download').hidden"),
          false,
        );
        // Reopen through the visible UI, without hidden actions or forced activation.
        await click("review-update");
        assert.equal(
          await evaluate("document.getElementById('pwa-update').hidden"),
          false,
        );
        await evaluate(
          "document.getElementById('prepare').click();globalThis.updateDisabledWhileProcessing=document.getElementById('apply-update').disabled",
        );
        assert.equal(
          await evaluate("globalThis.updateDisabledWhileProcessing"),
          true,
        );
        await wait(
          "!document.getElementById('prepare').disabled",
          "processing before update",
        );
        const loaded = load();
        await click("apply-update");
        await loaded;
        await wait(
          "!!document.getElementById('prepare') && !document.getElementById('prepare').disabled",
          "updated engine",
        );
        assert.equal(
          await evaluate(
            "fetch('../build.json').then(r=>r.json()).then(m=>m.revision)",
          ),
          revision,
        );
        assert.equal(await evaluate("document.documentElement.lang"), "ms");
        assert.equal(
          await evaluate("document.getElementById('download').hidden"),
          true,
        );
        const names = await evaluate("caches.keys()");
        assert.ok(names.includes("unrelated-app-cache"));
        assert.ok(!names.some((name) => name.endsWith(initial.revision)));
        await processSample();
      },
    );
  if (liveBase)
    console.log(
      "SKIP update fixture: live mode cannot replace the deployed server's files",
    );
  assert.deepEqual(consoleErrors, [], "Browser errors");
  await writeFile(
    join(scratch, "results.json"),
    JSON.stringify({ results, consoleErrors }, null, 2),
  );
  console.log(
    `PWA E2E: ${results.length} scenarios passed; artifacts: ${scratch}`,
  );
  await send("Browser.close").catch(() => {});
} catch (error) {
  console.error(error);
  await writeFile(
    join(scratch, "failure.json"),
    JSON.stringify({ results, consoleErrors, error: String(error) }, null, 2),
  );
  process.exitCode = 1;
} finally {
  socket?.close();
  if (child.exitCode === null) child.kill();
  await new Promise((resolve) => server.close(resolve));
}
