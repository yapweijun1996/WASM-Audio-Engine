import { message } from "./i18n.js";

export function initPwa() {
  const $ = (id) => document.getElementById(id);
  let installEvent,
    waiting,
    busy = false,
    reloadRequested = false;
  const showOffline = () =>
    message(
      $("offline-status"),
      navigator.onLine
        ? "Ready for offline use"
        : "Offline · app resources cached",
    );
  document.addEventListener("audio-busy", (event) => {
    busy = event.detail;
    $("apply-update").disabled = busy;
  });
  window.addEventListener("beforeinstallprompt", (event) => {
    event.preventDefault();
    installEvent = event;
    $("install").hidden = false;
  });
  window.addEventListener("appinstalled", () => {
    installEvent = undefined;
    $("install").hidden = true;
  });
  $("install").addEventListener("click", async () => {
    if (!installEvent) return;
    const event = installEvent;
    installEvent = undefined;
    $("install").hidden = true;
    try {
      await event.prompt();
    } catch {
      message(
        $("offline-status"),
        "Use your browser's Install app menu. On iPhone or iPad, choose Share, then Add to Home Screen.",
      );
    }
  });
  $("later-update").addEventListener("click", () => {
    $("pwa-update").hidden = true;
  });
  $("review-update").addEventListener("click", () => {
    if (waiting) $("pwa-update").hidden = false;
  });
  $("apply-update").addEventListener("click", () => {
    if (busy || !waiting) return;
    reloadRequested = true;
    waiting.postMessage({ type: "APPLY_UPDATE" });
  });
  if (
    !("serviceWorker" in navigator) ||
    !document.querySelector('meta[name="audio-engine-build"]')
  ) {
    message(
      $("offline-status"),
      "Offline installation is available in a built or deployed site.",
    );
    return;
  }
  navigator.serviceWorker.addEventListener("controllerchange", () => {
    if (reloadRequested) window.location.reload();
  });
  const offerUpdate = (worker) => {
    waiting = worker;
    worker.addEventListener("statechange", () => {
      if (worker.state === "activated" || worker.state === "redundant") {
        waiting = undefined;
        $("review-update").hidden = true;
        $("pwa-update").hidden = true;
      }
    });
    const channel = new MessageChannel();
    channel.port1.onmessage = (event) => {
      message(
        $("update-message"),
        "Update {version} is ready. Reloading clears current audio results.",
        { version: event.data.version },
      );
      $("pwa-update").hidden = false;
      $("review-update").hidden = false;
      channel.port1.close();
    };
    worker.postMessage({ type: "GET_VERSION" }, [channel.port2]);
  };
  navigator.serviceWorker
    .register(new URL("../service-worker.js", import.meta.url), {
      updateViaCache: "none",
    })
    .then((registration) => {
      if (registration.waiting) offerUpdate(registration.waiting);
      registration.addEventListener("updatefound", () => {
        const worker = registration.installing;
        worker.addEventListener("statechange", () => {
          if (
            worker.state === "installed" &&
            registration.waiting &&
            registration.active
          )
            offerUpdate(registration.waiting);
          if (worker.state === "redundant")
            message(
              $("offline-status"),
              "Offline setup unavailable. Audio processing still works while this page is loaded.",
            );
        });
      });
      navigator.serviceWorker.ready.then(() => {
        showOffline();
        window.addEventListener("online", showOffline);
        window.addEventListener("offline", showOffline);
      });
    })
    .catch(() =>
      message(
        $("offline-status"),
        "Offline setup unavailable. Audio processing still works while this page is loaded.",
      ),
    );
}
