/* YSH — Service Worker (PWA + offline support)
   App shell precached; navigation is network-first with offline fallback;
   static assets (pkg, styles, icons) are network-first so deploys nuevos
   SIEMPRE cargan el build actual (evita 401 por WASM viejo en cache). */
const CACHE = 'ysh-v7';
const SHELL = ['/', '/index.html', '/style.css', '/favicon.svg', '/manifest.webmanifest'];

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE).then((cache) => cache.addAll(SHELL))
  );
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(keys.filter((k) => k !== CACHE).map((k) => caches.delete(k)))
    ).then(() => self.clients.claim())
    // Tras tomar control, recarga todas las pestañas abiertas para que
    // apliquen la version nueva de la app sin hard-reload manual.
    .then(() => self.clients.matchAll({ type: 'window' }))
    .then((clients) => {
      clients.forEach((client) => client.navigate(client.url));
    })
  );
});

self.addEventListener('fetch', (event) => {
  const { request } = event;
  if (request.method !== 'GET') return;

  const url = new URL(request.url);
  if (url.origin !== self.location.origin) return;

  // SPA navigations: network-first, cache fallback for offline.
  if (request.mode === 'navigate') {
    event.respondWith(
      fetch(request)
        .then((res) => {
          const copy = res.clone();
          caches.open(CACHE).then((cache) => cache.put('/index.html', copy));
          return res;
        })
        .catch(() => caches.match('/index.html').then((c) => c || Response.error()))
    );
    return;
  }

  // Static assets: NETWORK-FIRST (siempre intenta la red primero; cachea la
  // copia nueva) con fallback a cache solo si no hay red. Asi el build nuevo
  // de /pkg se carga SIEMPRE y nunca se sirve el WASM viejo que causaba 401.
  if (url.pathname.startsWith('/pkg/') || SHELL.includes(url.pathname) || url.pathname.startsWith('/icon-')) {
    event.respondWith(
      fetch(request)
        .then((res) => {
          if (res.ok) {
            const copy = res.clone();
            caches.open(CACHE).then((cache) => cache.put(request, copy));
          }
          return res;
        })
        .catch(() => caches.match(request))
    );
  }
});