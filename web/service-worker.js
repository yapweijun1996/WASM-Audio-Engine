// Build tokens are replaced by scripts/build_site.py in the deployment artifact.
const REVISION = "__BUILD_REVISION__";
const VERSION = "__APP_VERSION__+" + REVISION.slice(0, 7);
const BASE = self.registration.scope;
const PREFIX = "wasm-audio-lab:" + new URL(BASE).pathname + ":";
const CACHE = PREFIX + REVISION;
const FILES = __PRECACHE_FILES__;
const urls = new Set(FILES.map((path) => new URL(path, BASE).href));
urls.add(new URL("build.json", BASE).href);

self.addEventListener("install", (event) => {
  event.waitUntil(
    (async () => {
      const response = await fetch(new URL("build.json", BASE), {
        cache: "no-store",
      });
      if (!response.ok) throw new Error("Offline manifest unavailable");
      const manifest = await response.clone().json();
      if (manifest.revision !== REVISION)
        throw new Error("Offline revision mismatch");
      const cache = await caches.open(CACHE);
      try {
        for (const path of FILES) {
          const url = new URL(path, BASE).href;
          const resource = await fetch(url, { cache: "reload" });
          if (!resource.ok) throw new Error("Offline asset unavailable");
          const hash = await crypto.subtle.digest(
            "SHA-256",
            await resource.clone().arrayBuffer(),
          );
          const hex = [...new Uint8Array(hash)]
            .map((n) => n.toString(16).padStart(2, "0"))
            .join("");
          if (hex !== manifest.assets[path])
            throw new Error("Offline asset integrity mismatch");
          await cache.put(url, resource);
        }
        await cache.put(new URL("build.json", BASE), response);
      } catch (error) {
        await caches.delete(CACHE);
        throw error;
      }
      // Updates remain waiting until the user explicitly chooses to reload.
    })(),
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    (async () => {
      for (const key of await caches.keys()) {
        if (key.startsWith(PREFIX) && key !== CACHE) await caches.delete(key);
      }
      await self.clients.claim();
    })(),
  );
});

self.addEventListener("message", (event) => {
  if (event.data?.type === "GET_VERSION")
    event.ports[0]?.postMessage({ version: VERSION });
  if (event.data?.type === "APPLY_UPDATE") event.waitUntil(self.skipWaiting());
});

self.addEventListener("fetch", (event) => {
  const request = event.request;
  if (request.method !== "GET") return;
  const url = new URL(request.url);
  if (url.origin !== self.location.origin) return;
  url.search = "";
  // Canonical navigation aliases share the same cached page.
  if (url.href === BASE) url.pathname += "index.html";
  if (url.href === new URL("examples/", BASE).href)
    url.pathname += "index.html";
  if (!urls.has(url.href)) return;
  event.respondWith(
    (async () => {
      const cached = await (await caches.open(CACHE)).match(url.href);
      return cached || fetch(request);
    })(),
  );
});
