import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { messages, t, setLanguage, errorInfo } from "../examples/i18n.js";

const placeholders = (text) =>
  [...text.matchAll(/\{(\w+)\}/g)].map((match) => match[1]).sort();

test("all Mandarin and Malay messages preserve source placeholders", () => {
  for (const [key, translations] of Object.entries(messages)) {
    assert.equal(translations.length, 2, key);
    for (const text of translations) {
      assert.ok(text.trim(), key);
      assert.deepEqual(placeholders(text), placeholders(key), key);
    }
    assert.match(translations[0], /[\u3400-\u9fff]/, key);
  }
});

test("English default, explicit locales, fallback, and nested error messages", () => {
  assert.equal(t("Process audio"), "Process audio");
  setLanguage("zh-Hans", false);
  assert.equal(t("Process audio"), "处理音频");
  assert.equal(
    t("Processing stopped: {reason}", {
      reason: errorInfo({ name: "AbortError" }),
    }),
    "处理已停止：操作已取消。",
  );
  setLanguage("ms", false);
  assert.equal(t("Process audio"), "Proses audio");
  assert.equal(t("{count} saturated samples", { count: 2 }), "2 sampel tepu");
  setLanguage("unsupported", false);
  assert.equal(t("Process audio"), "Process audio");
});

test("static page and literal runtime messages have translations", async () => {
  const html = await readFile(
    new URL("../examples/index.html", import.meta.url),
    "utf8",
  );
  const decode = (s) =>
    s
      .replaceAll("&amp;", "&")
      .replaceAll("&quot;", '"')
      .replaceAll("&#x27;", "'");
  for (const match of html.matchAll(/data-i18n(?:-aria)?="([^"]+)"/g))
    assert.ok(messages[decode(match[1])], match[1]);
  for (const file of [
    "../examples/index.html",
    "../examples/tools.js",
    "../examples/pwa.js",
  ]) {
    const code = await readFile(new URL(file, import.meta.url), "utf8");
    for (const match of code.matchAll(
      /(?:message\([^,]+,\s*|uiError\()"([^"\n]+)"/g,
    ))
      assert.ok(messages[match[1]], match[1]);
  }
});
