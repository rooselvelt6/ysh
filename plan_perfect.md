# Plan Perfect — Arreglar los errores de "nada funciona" en YSH

> Plan de corrección de errores del frontend + despliegue SPA.

## Diagnóstico honesto de POR QUÉ se dijo "sin errores" y no era cierto

Error doble en la iteración anterior:
1. Se validó solo con curl (API) — la API devolvía 200 en todo, pero el navegador crashea
   en la navegación del SPA. Eso solo se ve renderizando la app real, no con curl.
2. Fallo de proceso: se debió hacer el test en navegador antes de afirmar que todo andaba.

## Los 3 errores reales encontrados

### ERROR 1 — `api::go()` crashea al navegar (el más grave)
- **Síntoma:** tras el login la app no navega a la home; consola:
  `panicked at src/api.rs: "You cannot call use_navigate outside a <Router>"`.
- **Causa:** `api::go()` en `frontend/src/api.rs` usaba `use_navigate()`, hook de Leptos que
  solo funciona durante el render de un componente, pero se llama dentro de
  `spawn_local`/closures (después del login). Lanza un panic de Rust.
- **Fix (aplicado):** cambiar `go()` para usar `window.location.set_href(path)` en vez de
  `use_navigate()`. Funciona desde cualquier contexto y la recarga completa garantiza que
  `init_auth()` lea el token de localStorage.
- **Estado:** escrito en código, compilado y desplegado, sin commit.

### ERROR 2 — El fallback SPA devuelve 404 en vez de 200
- **Síntoma:** `/login`, `/wallet`, `/moments`... devuelven HTTP 404 (con el HTML del SPA
  como cuerpo). Confirmado con curl.
- **Causa:** en `src/server.rs`, `ServeFile::new(index_html)` como `not_found_service` de
  `ServeDir`. tower-http conserva el status 404 que provocó el fallback en vez de 200.
  Rompe navegación SPA por ruta directa / refresco en rutas como `/wallet`.
- **Fix (empezado):** manejador `spa_fallback` que: sirve assets reales del `static_dir`,
  devuelve `index.html` con status 200 para rutas SPA, y mantiene 404 JSON solo para
  APIs/WS/health inexistentes.
- **Estado:** escrito en código, sin compilar ni desplegar ni commitear.

### ERROR 3 — El Service Worker cachea la app vieja para siempre
- **Síntoma:** un usuario que ya cargó la app nunca recibe el wasm nuevo tras nuevos deploys.
- **Causa:** en `frontend/public/sw.js`: nombre de caché fijo `CACHE = 'ysh-v1'` y estrategia
  cache-first para `/pkg/*`. El caché nunca se invalida.
- **Fix:** versionar el caché (p.ej. `ysh-v2`) y/o hacer `/pkg/*` stale-while-revalidate.
  Aplicar en `public/sw.js` y copiar a `dist/sw.js`.

### Error menor / a revisar (opcional)
- El feed de momentos muestra `"comments":0` aunque tengan comentarios (bug de conteo en
  backend; no rompe la UI).

## Pasos de ejecución

**Fase A — Frontend (wasm)**
1. Revisar que `api::go()` en `frontend/src/api.rs` use `window.location` (ya está).
2. Reconstruir wasm: `wasm-pack build --target web --out-dir pkg` en `frontend/` y copiar a
   `frontend/dist/pkg/`.

**Fase B — Backend (fallback SPA)**
3. Revisar/completar el manejador `spa_fallback` en `src/server.rs` (ya escrito).
4. Verificar compilación: `cargo build`; ajustar imports si hace falta
   (p.ej. `tower::util::ServiceExt` para `.oneshot()`).

**Fase C — Service Worker**
5. Modificar `frontend/public/sw.js`: `CACHE` a `ysh-v2` y `/pkg/*` stale-while-revalidate.
   Copiar a `frontend/dist/sw.js`.

**Fase D — Verificación real (en navegador, no solo curl)**
6. Desplegar: `docker compose --profile monitoring up -d --build ysh`.
7. curl: `/`, `/login`, `/wallet` deben devolver 200; APIs inexistentes siguen 404 JSON.
8. Test en navegador (Chrome headless CDP, contenido en un solo comando sin procesos
   colgados): login → home → post → wallet. Confirmar cero panics (`use_navigate`) y
   navegación correcta.

**Fase E — Commit**
9. Commit de: `api.rs`, `lib.rs`, `server.rs`, `sw.js`, y binarios `dist/pkg/*` regenerados.
10. Este `plan_perfect.md` documenta el plan y el resultado final.
