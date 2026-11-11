import { writeFile } from "node:fs/promises";
const [port, inputPath, outputPath] = process.argv.slice(2);
const base = `http://127.0.0.1:${port}`;
const pages = await (await fetch(base + "/json/list")).json();
const page = pages.find((p) => p.type === "page");
if (!page) throw new Error("No isolated Chrome page");
const socket = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((resolve, reject) => {
  socket.addEventListener("open", resolve, { once: true });
  socket.addEventListener("error", reject, { once: true });
});
let next = 0;
const pending = new Map();
socket.addEventListener("message", (event) => {
  const data = JSON.parse(event.data);
  const entry = pending.get(data.id);
  if (entry) {
    pending.delete(data.id);
    data.error
      ? entry.reject(new Error(JSON.stringify(data.error)))
      : entry.resolve(data.result);
  }
});
const send = (method, params = {}) =>
  new Promise((resolve, reject) => {
    const id = ++next;
    pending.set(id, { resolve, reject });
    socket.send(JSON.stringify({ id, method, params }));
  });
const evaluate = async (expression) => {
  const r = await send("Runtime.evaluate", {
    expression,
    returnByValue: true,
    awaitPromise: true,
  });
  if (r.exceptionDetails) throw new Error(JSON.stringify(r.exceptionDetails));
  return r.result.value;
};
try {
  await send("Page.navigate", {
    url: "http://127.0.0.1:18786/tests/browser-real.html",
  });
  let loaded = false;
  for (let i = 0; i < 100; i++) {
    await new Promise((r) => setTimeout(r, 100));
    if (
      await evaluate(
        "document.readyState === 'complete' && !!document.querySelector('#file')",
      )
    ) {
      loaded = true;
      break;
    }
  }
  if (!loaded) throw new Error("Test page did not load");
  const doc = await send("DOM.getDocument");
  const node = await send("DOM.querySelector", {
    nodeId: doc.root.nodeId,
    selector: "#file",
  });
  await send("DOM.setFileInputFiles", {
    nodeId: node.nodeId,
    files: [inputPath],
  });
  // CDP file selection fires the normal input/change path, matching the user entry point.
  let result;
  for (let i = 0; i < 1200; i++) {
    await new Promise((r) => setTimeout(r, 500));
    const text = await evaluate(
      "document.querySelector('#result').textContent",
    );
    if (text.startsWith("FAIL:")) throw new Error(text);
    if (text.startsWith("{")) {
      result = JSON.parse(text);
      break;
    }
  }
  if (!result) throw new Error("Long-recording test timed out");
  result.resourceOrigins = await evaluate(
    "[...new Set(performance.getEntriesByType('resource').map(e=>new URL(e.name).origin))]",
  );
  await writeFile(outputPath, JSON.stringify(result, null, 2) + "\n");
} finally {
  await send("Browser.close").catch(() => {});
  socket.close();
}
