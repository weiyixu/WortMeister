// sw.js - service worker for offline use.
//
// Caches the app shell (HTML/CSS/JS/WASM/CSV/icons) so WortMeister works with
// no network after the first load - which is what makes it feel like a real
// installed app on iPhone. Bump CACHE_VERSION whenever you deploy new assets
// so clients pick up the update.

const CACHE_VERSION = "wortmeister-v2";


// Paths are relative so the app also works when hosted under a sub-path like
// https://user.github.io/Rust/ (GitHub Pages project sites).
const ASSETS = [
  "./",
  "./index.html",
  "./styles.css",
  "./main.js",
  "./manifest.webmanifest",
  "./vocabulary.csv",
  "./pkg/wortmeister_wasm.js",
  "./pkg/wortmeister_wasm_bg.wasm",
  "./icons/icon-192.png",
  "./icons/icon-512.png",
  "./icons/icon-180.png",
];

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(CACHE_VERSION).then((cache) =>
      // Don't fail install if an optional asset (e.g. an icon) is missing.
      Promise.allSettled(ASSETS.map((url) => cache.add(url)))
    )
  );
  self.skipWaiting();
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(
        keys.filter((k) => k !== CACHE_VERSION).map((k) => caches.delete(k))
      )
    )
  );
  self.clients.claim();
});

// Cache-first: serve from cache, fall back to network, and cache new GETs.
self.addEventListener("fetch", (event) => {
  if (event.request.method !== "GET") return;
  event.respondWith(
    caches.match(event.request).then((cached) => {
      if (cached) return cached;
      return fetch(event.request)
        .then((res) => {
          const copy = res.clone();
          caches.open(CACHE_VERSION).then((c) => c.put(event.request, copy));
          return res;
        })
        .catch(() => cached);
    })
  );
});
