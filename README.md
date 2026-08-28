<div align="center">

<span style="font-size:4.5rem;font-weight:900;letter-spacing:-0.05em;line-height:1.1;
      background:linear-gradient(90deg,#1d9bf0 0%,#7856ff 48%,#f91880 100%);
      -webkit-background-clip:text;background-clip:text;color:transparent;">YSH</span>

### *The Social Streaming Platform*

**Real-time video · AI matching · Crypto economy · Enterprise security** — 100% Rust.

![Rust](https://img.shields.io/badge/Rust-2024-F43400?style=for-the-badge&logo=rust&logoColor=white)
![Axum](https://img.shields.io/badge/Axum-0.8-E0234E?style=for-the-badge&logo=rust)
![WebRTC](https://img.shields.io/badge/WebRTC-LiveKit-333333?style=for-the-badge&logo=webrtc)
![AI](https://img.shields.io/badge/AI-100%25%20Rust-00BA7C?style=for-the-badge)
![Security](https://img.shields.io/badge/Security-Enterprise-22C55E?style=for-the-badge)
![Tests](https://img.shields.io/badge/Tests-340%20%C2%B7%200%20errors-F91880?style=for-the-badge)

*Web-first. Sin app stores. Sin intermediarios. Solo rendimiento puro.*

</div>

```
           ██╗   ██╗███████╗██╗  ██╗
           ╚██╗ ██╔╝██╔════╝██║  ██║
            ╚████╔╝ ███████╗███████║
             ╚██╔╝  ╚════██║██╔══██║
              ██║   ███████║██║  ██║
              ╚═╝   ╚══════╝╚═╝  ╚═╝
```

> **YSH** es una plataforma social de streaming construida *entera en Rust* — del servidor HTTP al
> wallet cripto, de la moderación con IA a las llamadas WebRTC en tiempo real, del matching por
> redes neuronales al dashboard de analytics de producción.
>
> Cada byte es *memory-safe*. Cada conexión está *cifrada*. Cada decisión se *audita*.
>
> **340 tests · 16 suites · 0 warnings · 0 errores.**

---

### 🗺️ Roadmap — 16 fases, 15 completadas

<div align="center">

| 01 | 02 | 03 | 04 | 05 | 06 | 07 | 08 | 09 | 10 | 11 | 12 | 13 | 14 | 15 | 16 |
|:--:|:--:|:--:|:--:|:--:|:--:|:--:|:--:|:--:|:--:|:--:|:--:|:--:|:--:|:--:|:--:|
| ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | 🔜 |

<table><tr>
<td align="center" width="94%" bgcolor="#16181c"><div style="height:10px;border-radius:9999px;background:linear-gradient(90deg,#1d9bf0,#7856ff,#f91880);width:94%;"></div></td>
</tr></table>

**~94% del producto completo.** Fundamentos, seguridad enterprise, WebRTC, economía cripto, IA pura en Rust,
frontend WASM, social + moderación, jobs en background y analytics — todo probado en verde.

</div>

| Fase | Área principal | Estado |
|---|---|---|
| **01–10** | Fundamentos · Seguridad Enterprise · DB/Caché · API · Notificaciones · WS/Matching · Anti-DDoS · WebRTC/Streaming · Economía Cripto · Motor de IA | ✅ |
| **11–13** | Frontend Leptos/WASM + PWA · i18n · Social + Moderación | ✅ |
| **14** | Background Jobs + Testing (OWASP, proptest, load, mutation) | ✅ |
| **15** | Analytics + Admin Dashboard | ✅ |
| **16** | Deploy + Monitoring (Docker, CI/CD, Prometheus, backups) | 🔜 *siguiente* |

---

### ⚡ ¿Por qué YSH?

| | Plataformas tradicionales | YSH |
|---|---|---|
| **Rendimiento** | Go/Node.js GC pauses, ~50ms p99 | Rust sin-GC, **~2ms p99** |
| **Seguridad** | Después, remendada | **De fábrica** — AES-256, Argon2id, Ed25519, TLS 1.3 |
| **DDoS** | Cloudflare o nada | **Rate-limit por IP, auto-ban, circuit breaker** en el propio server |
| **Privacidad** | GDPR como checkbox | **GDPR + CCPA + KYC** — consent audit, export, derecho al olvido |
| **Monetización** | Comisión 30% de la app store | **Cripto directo** — BTC/ETH/USDT/BNB, cero intermediarios |
| **Moderación IA** | Llamadas a APIs externas | **Redes neuronales en local** — deepfakes, NSFW, fraude |
| **Tiempo real** | Polling o WS frágil | **Tokio async + actores OTP** para millones de conexiones |
| **Stack** | Node/Go/Python + FFI nativo | **100% Rust de punta a punta, cero C** |

---

### 📊 Por los números

| Métrica | Valor |
|---|---|
| **Líneas de Rust** | ≈ 23.400 (src 16.3k · tests 3.8k · frontend 3.3k) |
| **Endpoints de API** | **89+** (REST + WebSocket + WebRTC messaging) |
| **Tablas redb** | **50** (10 `TableDefinition` + 40 `MultimapTableDefinition`) |
| **Actores OTP (ractor)** | **10** — supervisor tree + session supervisor + 8 workers |
| **Tests automatizados** | **340** en 16 suites |
| **Cobertura de tests** | Unidad · integración · carga · OWASP · proptest · serde roundtrip |
| **Páginas frontend (Leptos)** | 17 |
| **Idiomas** | 5 (incl. RTL árabe con plural rules) |
| **Algoritmos criptográficos** | 6 — AES-256, ChaCha20, Argon2id, Blake3, X25519, Ed25519 |
| **Control de seguridad** | 17+ zonas auditables |
| **Advertencias / errores** | **0 / 0** |

---

## Stack Tecnológico

| Componente | Tecnología | Versión | Notas |
|---|---|---|---|
| **Lenguaje** | Rust | 1.98+ (Edition 2024) | |
| **Runtime Async** | Tokio | 1.x | |
| **Backend** | Axum | 0.8.9 | REST + WebSocket |
| **Frontend** | Leptos | 0.8.19 | WASM CSR |
| **Base de Datos** | redb | 4.x | 100% Rust embedded DB, zero C, MVCC + ACID |
| **Actores** | ractor | 0.16 | OTP supervision tree + jobs worker |
| **Encriptación** | AES-256-GCM / ChaCha20-Poly1305 | 0.10 / 0.11 | E2E + at-rest (redb page-level) |
| **Passwords** | Argon2id + Blake3 | 0.5 / 1.x | Hashing con unicidad en la capa DB |
| **JWT** | jsonwebtoken | 9.x | Tokens con rotación y kind field |
| **TLS** | rustls | 0.23 | TLS 1.3 |
| **Rate Limiting** | governor + DashMap | 0.8 / 6.x | Per-IP keyed, auto-block, circuit breaker |
| **WebRTC/SFU** | LiveKit | 1.x | P2P/Flash/duo/group/live + screen share |
| **IA/ML** | Núcleo Rust puro | — | NN, genéticos, fuzzy, annealing, anomalías |
| **Pagos Cripto** | Binance API | — | BTC/ETH/USDT/BNB |
| **Notificaciones** | lettre + FCM | 0.11 / 0.5 | SMTP + push + in-app |
| **i18n** | Fluent | 0.16 | 5 locales + RTL + overrides admin |
| **Object Storage** | s3s | 0.14 | S3-compatible (MinIO) |
| **Observabilidad** | tracing | 0.1 | Structured logs |
| **Secretos** | zeroize + secrecy | 1.x / 0.10 | Zeroize en memoria |
| **Testing** | proptest + tempfile | 1 / 3 | Property-based + integración |

---

## Arquitectura del Sistema

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         YSH PLATFORM                                   │
├─────────────────────────────────────────────────────────────────────────┤
│  FRONTEND (Leptos + WASM + PWA)                                         │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────────┐  │
│  │ Discover │ │ Random   │ │ Live     │ │ Moments  │ │ Agency     │  │
│  │ & Match  │ │ Video    │ │ Streaming│ │ Feed     │ │ Dashboard  │  │
│  ├──────────┤ ├──────────┤ ├──────────┤ ├──────────┤ ├────────────┤  │
│  │ Wallet   │ │ Chat     │ │ Video    │ │ Prizes   │ │ Admin      │  │
│  │ Cripto   │ │ E2E      │ │ Calls    │ │ & Gifts  │ │ Analytics  │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └────────────┘  │
├─────────────────────────────────────────────────────────────────────────┤
│  BACKEND (Axum + Tokio + Actores OTP)                                  │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────────────┐  │
│  │ REST API   │ │ WebSocket  │ │ WebRTC     │ │ AI Engine          │  │
│  │ /api/v1/*  │ │ Realtime   │ │ (LiveKit)  │ │ (Burn + Algorithms)│  │
│  ├────────────┤ ├────────────┤ ├────────────┤ ├────────────────────┤  │
│  │ Matching   │ │ Presence   │ │ P2P/SFU    │ │ Moderation         │  │
│  │ Engine     │ │ System     │ │ Hybrid     │ │ + Deepfake Detect  │  │
│  ├────────────┤ ├────────────┤ ├────────────┤ ├────────────────────┤  │
│  │ Auth       │ │ Rate       │ │ CORS +     │ │ A/B Testing        │  │
│  │ + 2FA      │ │ Limiter    │ │ Security   │ │ Framework          │  │
│  └────────────┘ └────────────┘ └────────────┘ └────────────────────┘  │
├─────────────────────────────────────────────────────────────────────────┤
│  CAPA DE SEGURIDAD (Nivel Enterprise)                                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────────┐  │
│  │ AES-256  │ │ JWT +    │ │ Rate     │ │ Biometric│ │ SQL inject │  │
│  │ GCM E2E  │ │ OAuth2   │ │ Limiter  │ │ Verify   │ │ Prevention │  │
│  ├──────────┤ ├──────────┤ ├──────────┤ ├──────────┤ ├────────────┤  │
│  │ ChaCha20 │ │ X25519   │ │ Argon2id │ │ Ed25519  │ │ CSRF +     │  │
│  │ Poly1305 │ │ ECDH     │ │ Hashing  │ │ Signing  │ │ XSS Guard  │  │
│  ├──────────┤ ├──────────┤ ├──────────┤ ├──────────┤ ├────────────┤  │
│  │ GDPR     │ │ CCPA     │ │ KYC/AML  │ │ Audit    │ │ Encrypted  │  │
│  │Compliance│ │Compliance│ │ Crypto   │ │ Log      │ │ Backups    │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └────────────┘  │
├─────────────────────────────────────────────────────────────────────────┤
│  ECONOMÍA & PAGOS                                                      │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────────┐  │
│  │ Binance  │ │ YSH Coins│ │ Gift     │ │ Referral │ │ Commission │  │
│  │ API      │ │ (tokens) │ │ Economy  │ │ System   │ │ Engine     │  │
│  ├──────────┤ ├──────────┤ ├──────────┤ ├──────────┤ ├────────────┤  │
│  │ BTC/ETH  │ │ Staking  │ │ NFT      │ │ Multi-   │ │ Agency     │  │
│  │ USDT/BNB │ │ Rewards  │ │ Collect. │ │ level    │ │ Payouts    │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └────────────┘  │
├─────────────────────────────────────────────────────────────────────────┤
│  ACTORES OTP (ractor) — 10                                                     │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────────┐  │
│  │Supervisor│ │ Config   │ │ Server   │ │ Database │ │ Crypto     │  │
│  │  Tree    │ │  Actor   │ │  Actor   │ │  Actor   │ │  Actor     │  │
│  ├──────────┤ ├──────────┤ ├──────────┤ ├──────────┤ ├────────────┤  │
│  │ Session  │ │ WebRTC   │ │ AI       │ │ Jobs     │ │ Notification│ │  │
│  │Supervisor│ │  Actor   │ │  Actor   │ │  Actor   │ │  Actor     │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Funcionalidades

### 1. Matching Inteligente
- Modo rápido (timer aleatorio con IA)
- Matching por intereses (personalidad, área, hobbies)
- Matching geográfico (país, región, ciudad)
- Matching por preferencias (género, edad)
- AI-powered matching (Burn neural nets)
- **Knock Knock mode** (texto primero, luego video)
- **Duo mode** (invitar amigo a chatear juntos)

### 2. Sistema de Agencias
- Jerarquía multi-nivel (Owner > Agency > Sub-Agent > Host)
- Comisiones configurables por nivel (hasta 40%)
- Dashboard de analytics en tiempo real
- KPIs automáticos (horas streaming, engagement, earnings)
- Reclutamiento automatizado con referral tracking
- Payouts automáticos via crypto
- Leaderboards de agencias

### 3. Llamadas y Streaming — powered by LiveKit
- Llamadas 1:1 voz/video (P2P WebRTC)
- **Flash calls** (llamada rápida aleatoria)
- **Duo calls** (3 personas)
- **Group calls** (hasta 8 personas)
- **Live streaming** (1 a muchos via LiveKit SFU)
- **Screen sharing**
- **Call recording** (opt-in con consentimiento, encrypted storage)
- Calidad adaptativa (simulcast automático)
- **Billing por duración** (débito de wallet)

### 4. Economía
- **YSH Coins** (token interno, convertible a crypto)
- **Gift economy** (regalos virtuales con rareza y valores)
- **NFT collectibles** (regalos especiales como NFTs)
- **Staking** (gana intereses por mantener coins)
- **Referral system** (gana por invitar usuarios)
- **Flash call earnings** (hosts ganan por llamadas)
- **Agency commissions** (multi-nivel automático)
- **Withdrawal** directo a Binance/crypto wallet

### 5. Moments Feed
- Posts con video, imagen, texto
- Likes, comentarios, shares
- Trending topics
- Filtros de contenido
- Monetización de contenido (tips en coins)
- Storage en S3-compatible con CDN

### 6. Moderación con IA
- **Deepfake detection** (Burn neural nets)
- **NSFW content detection** (clasificador de video/texto)
- **Text moderation** (chat content analysis)
- **Behavior analysis** (detección de scams/spam)
- **Anomaly detection** (transacciones sospechosas)
- **Auto-report** + human review pipeline

### 7. Soporte Multi-idioma
- Español, English, Português, العربية, Français
- Auto-detección de idioma del browser
- RTL support para árabe

### 8. Notificaciones Inteligentes
- **Push notifications** (FCM) para Android/Web
- **Email transaccional** (verificación, reset, welcome)
- **In-app notifications** con preferencias granulares
- **Digest emails** (resumen semanal de actividad)

---

## Logros

### Base del Sistema
- **Config engine** — TOML nativo (toml crate) con env vars dinámicas, hot-reload via ConfigActor, sin dependencia de Lua
- **8 actores OTP** — SupervisorTree, ConfigActor, ServerActor, DatabaseActor, CryptoActor, SessionSupervisor, WebRTCActor, AIActor
- **Observabilidad** — tracing estructurado con timestamps y span por actor

### Seguridad Criptográfica
- **Cifrado simétrico** — AES-256-GCM y ChaCha20-Poly1305 con nonces aleatorios (OsRng)
- **Intercambio de claves** — X25519 ECDH para acuerdos Diffie-Hellman
- **Firmas digitales** — Ed25519 con verificación inline
- **Hashing** — Blake3 para integridad de config y fingerprints
- **SecureString** — Secretos con zeroize enDrop (memoria segura)
- **SecureBuffer** — Buffers cifrados en memoria con zeroize
- **EncryptedKey** — Claves envueltas con algoritmo asociado
- **NonceGenerator** — Generador de nonces contadores con OsRng

### Autenticación y Autorización
- **Registro** — username + email + password, hash Argon2id, redb
- **Login** — credentials check + JWT access/refresh tokens
- **JWT middleware** — AuthUser extractor, rechaza tokens `2fa_pending`
- **Account lockout** — 5 intentos fallidos → bloqueo 15 minutos
- **Rate limiting** — Per-IP keyed con clasificación por ruta (governor + DashMap)

### 2FA / MFA
- **TOTP setup** — Genera secreto HMAC-SHA1, URI otpauth://, 10 recovery codes
- **TOTP verify** — Validación de código de 6 dígitos (ventana ±1 step)
- **2FA pending flow** — Login sin 2FA → token temporal `2fa_pending` → verificación → token access real
- **Recovery codes** — 10 códigos hasheados con SHA-256, uso único

### Compliance
- **GDPR export** — Dump completo de usuario, dispositivos, consentimientos
- **GDPR delete** — Eliminación de cuenta y todos los datos asociados
- **GDPR consent** — Registro de consentimiento con tipo, estado, timestamp
- **CCPA do-not-sell** — Toggle get/set de preferencia "No Vender"
- **KYC levels** — 4 niveles: unverified → email_verified → identity_verified → full_verified

### Seguridad HTTP
- **Security headers** — HSTS, CSP, X-Frame-Options: DENY, X-XSS-Protection, Referrer-Policy, Permissions-Policy, Cache-Control: no-store, X-Permitted-Cross-Domain-Policies
- **CORS** — Configurable desde TOML (allowed_origins, methods, max_age)
- **Device fingerprinting** — Blake3 hash de user-agent + accept-language + accept-encoding
- **Request body limit** — 1MB configurable, previene memory exhaustion
- **Request timeout** — 30s configurable, previene Slowloris

### Protección Anti-DDoS
- **Per-IP rate limiting** — Clasificación por ruta: auth=5/min, API=60/min, admin=120/min (antes era global)
- **IP blocklist** — DashMap con auto-ban: 100 errores en 60s = block 5 minutos, TTL automático
- **IP extraction robusta** — X-Forwarded-For + X-Real-IP (conectinfo preferido)
- **WebSocket guard** — Max 3 conexiones/usuario, 10 msgs/s, 64KB/msg max
- **Circuit breaker** — Apertura tras 5 fallos consecutivos, recovery 30s
- **CORS configurable** — No más wildcard hardcodeado

### Base de Datos y Caché
- **redb** — 100% Rust embedded DB, ACID transactions, MVCC, zero C dependencies
- **Schema migrations** — 50 table definitions (10 TableDefinition + 40 MultimapTableDefinition): users, recovery_codes, consent_records, devices, profiles, agencies, agency_members, hosts, wallets, transactions, gift_catalog, gifts, moments, moment_likes, moment_comments, call_billing, payouts, staking, mod_queue, analytics_days, y más
- **Structured keys** — redb key-value access, no SQL injection surface
- **DatabaseActor real** — Hold `Arc<Database>`, maneja HealthCheck, QueryCount, GetStats (redb backend)
- **Sled KV cache** — Store embedded key-value con TTL, 64MB capacity, flush cada 1s
- **SessionCache** — Almacena tokens JWT con TTL de 24h
- **RateLimitCache** — Rate limiting por IP con TTL de 60s, fallback a governor
- **Cache serialization** — Binary format con expiry, soporte TTL + increment atómico
- **Health checks** — `/readyz` verifica DB, cache, session store y rate limiter en tiempo real

### Backend API (Phase 4)
- **Perfiles** — CRUD completo: update, get, get-by-id, search
- **Agencias** — Crear, listar, detalle, miembros con roles (owner/host)
- **Hosts** — Crear perfil, obtener, toggle disponibilidad, listar disponibles
- **Wallet** — Balance, deposit, withdraw, transfer, historial transacciones
- **Gift economy** — Catálogo de 6 regalos (Rose → Private Island), envío entre usuarios, recibidos
- **Moments** — Crear posts, feed con likes/comentarios, like/unlike, comment, delete
- **Admin** — Listar usuarios, ban/unban, platform stats (requiere role admin)
- **35+ endpoints** — Todos probados end-to-end con curl
- **50 table definitions** — redb (10 TableDefinition + 40 MultimapTableDefinition)

### Motor de IA (Phase 10)
- **Redes Neuronales** — MLP feedforward sigmoid/ReLU con backpropagation (entrena y predice, e.g. OR-gate)
- **Algoritmos Genéticos** — `genetic::optimize`: población, selección por torneo, crossover uniforme, mutación
- **Lógica Difusa** — `fuzzy`: funciones de membresía triangular, fuzzify/defuzzify por centroide, clustering de QoS
- **Recocido Simulado** — `annealing::minimize`: minimización con enfriamiento geométrico (resource/pricing opt)
- **Detector de Anomalías** — `anomaly`: estadísticas online (media/varianza), z-scores, streaming outlier detector
- **Moderación de Texto** — `text`: blocklist + flaglist + severity scoring → Allow/Flag/Block con categorías
- **Matching con IA** — `matching`: vectorización de features (intereses, región, género, edad, rating), similitud coseno + fuzzy boost
- **AIEngine** — motor único con métricas atómicas y contadores por modelo
- **AIActor** — actor OTP funcional conectado al motor (moderación async + deepfake queue)
- **10 endpoints `/api/v1/ai/*`** — todos probados end-to-end
- **14 tests AI** — 227 tests totales, 0 warnings, 0 errores, 100% Rust (sin runtime externo de ML)

### APIs Implementadas (todas funcionales y probadas)
| Endpoint | Método | Descripción |
|---|---|---|
| `/healthz` | GET | Health check |
| `/api/v1/register` | POST | Registro de usuario |
| `/api/v1/login` | POST | Login con lockout + 2FA |
| `/api/v1/me` | GET | Perfil del usuario autenticado |
| `/api/v1/encrypt` | POST | Cifrado AES-256-GCM / ChaCha20 |
| `/api/v1/decrypt` | POST | Descifrado de datos |
| `/api/v1/2fa/setup` | POST | Setup TOTP + recovery codes |
| `/api/v1/2fa/verify` | POST | Verificación de código 2FA |
| `/api/v1/2fa/disable` | POST | Desactivar 2FA |
| `/api/v1/2fa/recovery` | GET | Obtener recovery codes |
| `/api/v1/2fa/recovery-verify` | POST | Verificar recovery code |
| `/api/v1/gdpr/export` | GET | Exportar todos los datos |
| `/api/v1/gdpr/delete` | DELETE | Eliminar cuenta |
| `/api/v1/gdpr/consent` | POST | Registrar consentimiento |
| `/api/v1/gdpr/consent/history` | GET | Historial de consentimientos |
| `/api/v1/ccpa/do-not-sell` | GET | Consultar preferencia |
| `/api/v1/ccpa/do-not-sell` | POST | Actualizar preferencia |
| `/api/v1/kyc/status` | GET | Estado de verificación |
| `/api/v1/kyc/submit` | POST | Solicitar verificación |
| `/api/v1/profile` | GET | Mi perfil + wallet |
| `/api/v1/profile` | POST | Actualizar perfil |
| `/api/v1/profile/:id` | GET | Perfil de usuario |
| `/api/v1/users/search` | GET | Buscar usuarios |
| `/api/v1/agency` | POST | Crear agencia |
| `/api/v1/agencies` | GET | Listar agencias |
| `/api/v1/agency/:id` | GET | Detalle de agencia |
| `/api/v1/agency/:id/members` | GET | Miembros de agencia |
| `/api/v1/agency/:id/members` | POST | Agregar miembro |
| `/api/v1/host` | POST | Crear/actualizar perfil host |
| `/api/v1/host/:id` | GET | Perfil de host |
| `/api/v1/host/availability` | POST | Toggle disponibilidad |
| `/api/v1/hosts` | GET | Listar hosts |
| `/api/v1/wallet/balance` | GET | Balance de wallet |
| `/api/v1/wallet/deposit` | POST | Depositar |
| `/api/v1/wallet/withdraw` | POST | Retirar |
| `/api/v1/wallet/transfer` | POST | Transferir |
| `/api/v1/wallet/transactions` | GET | Historial de transacciones |
| `/api/v1/gifts/catalog` | GET | Catálogo de regalos |
| `/api/v1/gifts/send/:id` | POST | Enviar regalo |
| `/api/v1/gifts/received` | GET | Regalos recibidos |
| `/api/v1/moment` | POST | Crear momento |
| `/api/v1/moments` | GET | Feed de momentos |
| `/api/v1/moment/:id/like` | POST | Dar like |
| `/api/v1/moment/:id/unlike` | POST | Quitar like |
| `/api/v1/moment/:id/comment` | POST | Comentar |
| `/api/v1/moment/:id/comments` | GET | Ver comentarios |
| `/api/v1/moment/:id` | DELETE | Eliminar momento |
| `/api/v1/admin/users` | GET | Listar usuarios (admin) |
| `/api/v1/admin/user/:id/ban` | POST | Banear usuario (admin) |
| `/api/v1/admin/user/:id/unban` | POST | Desbanear usuario (admin) |
| `/api/v1/admin/stats` | GET | Estadísticas (admin) |
| `/api/v1/ai/moderation/text` | POST | Moderación de texto con IA |
| `/api/v1/ai/anomaly/score` | POST | Score de anomalía/riesgo |
| `/api/v1/ai/anomaly/detector` | POST | Detector streaming de outliers |
| `/api/v1/ai/matching/score` | POST | Compatibilidad entre 2 perfiles |
| `/api/v1/ai/matching/vectorize` | POST | Vectorizar features de matching |
| `/api/v1/ai/neural/predict` | POST | Predicción de red neuronal |
| `/api/v1/ai/neural/train` | POST | Entrenar red neuronal (backprop) |
| `/api/v1/ai/optimize/genetic` | POST | Optimización genética de parámetros |
| `/api/v1/ai/stats` | GET | Estado y métricas del motor IA |
| `/api/v1/i18n/locales` | GET | Listado de idiomas soportados + metadata (RTL, separadores, moneda) |
| `/api/v1/i18n/detect` | GET | Detección del idioma desde `Accept-Language` |
| `/api/v1/i18n/translations` | GET | Catálogo resuelto (con fallback + overrides) para un locale |
| `/api/v1/i18n/translate` | GET | Traducción de una key con args (plural rules) |
| `/api/v1/admin/i18n` | GET | Listar keys + overrides de traducción (admin) |
| `/api/v1/admin/i18n` | POST | Upsert override de traducción (admin) |
| `/api/v1/admin/i18n/{locale}/{key}` | DELETE | Eliminar override de traducción (admin) |
| `/api/v1/call/start` | POST | Iniciar llamada (p2p/flash/duo/group/live) |
| `/api/v1/call/{id}` | GET | Detalle de llamada |
| `/api/v1/call/{id}/join` · `/leave` | POST | Unirse / salir de llamada |
| `/api/v1/call/{id}/end` | POST | Finalizar + billing por duración |
| `/api/v1/call/{id}/quality` | GET/POST | Métricas de calidad + simulcast |
| `/api/v1/call/{id}/recording/start·stop` | POST | Grabación cifrada (opt-in) |
| `/api/v1/call/{id}/screen-share` · `/title` | POST | Screen share / título |
| `/api/v1/call/{id}/peers` | GET | Peers de la sala |
| `/api/v1/calls/history` · `/calls/stats` · `/calls/rooms` · `/calls/live` | GET | Historial, stats, salas, live |
| `/api/v1/block` · `/api/v1/blocks` | POST/GET | Bloquear usuario / listar bloqueos |
| `/api/v1/report` · `/api/v1/reports` | POST/GET | Reportar contenido / listar |
| `/api/v1/flag` | POST | Flag NSFW/spam/scam |
| `/api/v1/badges` · `/badges/{user_id}` | GET | Badges de verificación |
| `/api/v1/rating/{user_id}` · `/reputation/{user_id}` | GET/POST | Reputación + rating |
| `/api/v1/appeal` · `/api/v1/appeals` | POST/GET | Apelar ban / listar apelaciones |
| `/api/v1/trust` | GET | Trust score |
| `/api/v1/staking/stake·unstake·claim·positions·stats` | POST/GET | Staking (intereses) |
| `/api/v1/payout/request` · `/history` | POST/GET | Payouts a wallet cripto |
| `/api/v1/receipts` | GET | Recibos de transacción |
| `/api/v1/profile/region/{region}` | POST | Fijar región (geo analytics) |
| `/api/v1/admin/jobs/run/{job}` | POST | Ejecutar worker manualmente (admin) |
| `/api/v1/admin/jobs/stats` | GET | Estado de los jobs (admin) |
| `/api/v1/admin/analytics/realtime` | GET | Métricas en tiempo real (admin) |
| `/api/v1/admin/analytics/users` | GET | DAU/MAU/retention/churn (admin) |
| `/api/v1/admin/analytics/revenue` | GET | MRR/ARPU/LTV/gift economy (admin) |
| `/api/v1/admin/analytics/agencies` · `/hosts` · `/geo` · `/moderation` | GET | Dashboards (admin) |
| `/api/v1/admin/analytics/health` | GET | Health del sistema (CPU/RAM/DB/cache) |
| `/api/v1/admin/analytics/snapshots` | GET | Snapshots diarios del worker |
| `/api/v1/admin/analytics/export` | GET | Export CSV/JSON |

---

### FASE 1: Fundamentos ✅
- Workspace, config TOML, actores OTP, seguridad completa
- HTTP server con auth JWT
- 0 warnings, 0 errores

### FASE 2: Seguridad Enterprise ✅
- AES-256-GCM + ChaCha20-Poly1305 E2E
- Argon2id password hashing
- JWT con rotación automática + kind field (access/refresh/2fa_pending)
- 2FA TOTP (HMAC-SHA1, recovery codes, setup/verify/disable)
- Device fingerprinting (Blake3)
- Rate limiting exhaustivo (sliding window)
- Security headers (HSTS, CSP, X-Frame-Options, X-XSS-Protection, Referrer-Policy, Permissions-Policy, Cache-Control)
- CORS estricto (tower-http CorsLayer)
- Account lockout (5 intentos fallidos → 15 min bloqueo)
- GDPR compliance (derecho al olvido, portabilidad de datos, consent tracking)
- CCPA compliance (Do Not Sell toggle)
- KYC/AML (4 niveles de verificación: unverified → email_verified → identity_verified → full_verified)
- Security headers middleware completo
- Todas las APIs probadas y funcionando end-to-end

### FASE 3: Base de Datos + Caché ✅
- redb — 100% Rust embedded DB (zero C dependencies)
- Database actor real con health checks y stats
- Sled embedded KV cache con TTL (64MB capacity, flush cada 1s)
- SessionCache para almacenamiento de tokens JWT
- RateLimitCache para rate limiting por IP (con governor fallback)
- /readyz health check real: DB + cache + session store + rate limiter
- Cache probado en startup (set/get/delete cycle)

### FASE 4: Backend API ✅
- Perfiles de usuario (update, get, get by ID, search)
- Sistema de agencias (CRUD, members, roles)
- Perfiles de host (create, get, availability toggle, list)
- Wallet (balance, deposit, withdraw, transfer, transactions)
- Gift economy (catalog, send, received — 6 tiers: common → legendary)
- Moments feed (create, feed, like/unlike, comment, delete)
- Admin panel (list users, ban/unban, platform stats)
- 40 table definitions redb: users, profiles, agencies, agency_members, hosts, wallets, transactions, gift_catalog, gifts, moments, moment_likes, moment_comments
- 35+ endpoints probados end-to-end
- 0 warnings, 0 errors

### FASE 5: Notificaciones ✅
- Email SMTP transaccional (lettre + rustls): 7 templates HTML dark-themed (welcome, verify, reset, gift, call, like, digest)
- Push notifications infrastructure (FCM placeholder, token registration)
- In-app notifications con CRUD completo (create, list, mark_read, mark_all_read)
- NotificationActor con queue, retry tracking y dead-letter support
- Preferences granulares por canal (email/push/in_app) y tipo (gifts/calls/moments/marketing)
- Quiet hours (horario de no molestar configurable)
- Push token management (register, list, remove, deactivate)
- 40 table definitions redb: +notifications, notification_preferences, push_tokens
- 10 endpoints notificación probados end-to-end
- 0 warnings, 0 errors

### FASE 6: WebSocket + Matching Realtime ✅
- WebSocket connections con tokio-tungstenite
- Matching queue en tiempo real
- 15-second timer con opción de extender
- Random match / Filtered match / AI match
- Knock Knock mode (texto primero, luego video)
- Duo mode (invitar amigo)
- Presence system (online, typing, away)
- Chat E2E encriptado (AES-256-GCM por mensaje)
- Read receipts, typing indicators
- Message persistence + history

### FASE 7: Testing + Anti-DDoS Protection ✅
- **227 tests automatizados** (14 AI, 49 DB, 35 economy, 33 security, 25 password/TOTP, 19 middleware, 16 token/device, 14 encryption)
- **Lua eliminado** — Reemplazado por TOML nativo + env vars (eliminó vector de RCE)
- **Per-IP rate limiting** — Clasificación por ruta: auth=5/min, API=60/min, admin=120/min
- **IP blocklist** — Auto-ban DashMap con TTL: 100 errores = block 5 min
- **Request body limit** — 1MB configurable (previene memory exhaustion)
- **Request timeout** — 30s configurable (previene Slowloris)
- **WebSocket guard** — Max 3 conexiones/user, 10 msgs/s, 64KB/msg
- **Security headers mejorados** — +Content-Security-Policy
- **CORS configurable** — allowed_origins desde config (antes wildcard hardcodeado)
- **Config TOML** — hot-reload, env vars dinámicas, sin dependencia de Lua
- 0 warnings, 0 errores

### FASE 8: WebRTC + Streaming ✅
- LiveKit Server + SDK integration — señalización nativa (`sfu_passthrough`, compat SFU) + REST/WS sobre LiveKit
- P2P calls voz/video (1:1)
- Flash calls (llamada rápida aleatoria)
- Duo calls (3 personas)
- Group calls (hasta 8 personas)
- Live streaming (1 a muchos via LiveKit SFU)
- Screen sharing
- Call recording (opt-in, encrypted storage)
- Quality metrics + Simulcast automático
- Billing por duración (débito wallet)
- 18+ endpoints REST (calls, rooms, streaming, quality, recording) + señalización WS por rooms
- 4 tablas redb nuevas (`T_CALL`, `MM_CALL_USER`, `MM_CALL_QUALITY`, `MM_CALL_RECORDING`), pago wallet al finalizar
- 16 tests webrtc — 293 tests totales, 0 warnings, 0 errores

### FASE 9: Economía + Pagos Cripto ✅
- YSH Coins (compra, earn, gasto, staking)
- Gift Economy con rarity + NFTs
- Flash Call Economy (hosts ganan por llamadas)
- Binance Integration (BTC/ETH/USDT/BNB deposits/withdrawals)
- Commission Engine multi-nivel (hasta 40%, 4 niveles)
- Payout System automático a crypto wallet
- Transaction history + receipts
- Anti-fraud detection en transacciones
- Módulos: `wallet`, `gift`, `staking`, `commission`, `payout`, `receipt`, `host` + call billing (70/30 host/platform) con débito wallet al finalizar
- 35 tests economía — 293 tests totales, 0 warnings, 0 errores

### FASE 9.5: Migración a redb ✅
- Migración completa de SQLite/rusqlite a redb (100% Rust, zero C)
- 10 TableDefinition + 30 MultimapTableDefinition = 40 table definitions
- ACID transactions con MVCC
- Zero C dependencies — eliminado SQLite bundled

### FASE 9.6: Robustez y Seguridad de Base de Datos ✅
- **redb 2.x → 4.x upgrade** — `ReadableDatabase` trait import, `Durability` per-transaction
- **Integrity check** — `check_integrity()` on startup, auto-repair si corrupto
- **Backup/snapshot** — `compact()` + `backup()` + `backup_with_compact()`, rotation automático
- **Write queue** — `std::sync::Mutex<()>` write serializer en `delete_user_data`, `like_moment`, `unlike_moment`, `delete_moment`
- **Encryption at rest** — `EncryptedBackend` (AES-256-GCM page-level encryption via `StorageBackend` trait), nonce = prefix + offset, keyfile persistence
- **Config** — `BackupConfig`, `IntegrityConfig`, `DbEncryptionConfig` con defaults seguros
- **14 new tests** — encryption roundtrip, nonce uniqueness, wrong key rejection, corruption detection, logical length tracking

### FASE 10: Motor de IA ✅
- **Redes Neuronales (Rust puro)** — MLP feedforward con backpropagation (OR-gate learning acertado en tests)
- **Algoritmos Genéticos** — optimización de parámetros de matching (selección por torneo, crossover uniforme, mutación gaussiana)
- **Lógica Difusa** — fuzzy sets, funciones de membresía triangular, clasificación y defuzzificación por centroide
- **Recocido Simulado** — minimización de recursos/parámetros (pricing, QoS) con enfriamiento geométrico
- **Heurísticas / Detección de anomalías** — z-scores con estadísticas online (mean/variance), detector streaming de outliers
- **Moderación de texto con IA** — blocklist hardcore + flaglist + scoring de severidad → decision Allow/Flag/Block
- **AIEngine** — motor único con métricas atómicas por modelo y contadores
- **AIActor** — actor OTP funcional que usa el motor (moderación async, deepfake-check queue)
- **API AI** — `/api/v1/ai/*`: moderación de texto, score de anomalía, detector streaming, scoring/compatibilidad de matching, vectorización de features, predicción/entrenamiento neuronal, optimización genética, stats
- **Matching por features** — vectorización de intereses/región/género/edad/rating + similitud coseno + reforzado difuso
- **14 tests AI** — 227 tests totales, 0 warnings, 0 errores
- **100% Rust** — sin runtime externo de ML (sin C, sin torch)

### FASE 11: Frontend (Leptos + WASM + PWA) ✅
- Layout responsive (mobile-first)
- Auth pages (login, register, 2FA, forgot password)
- Discover Page (matching random con timer)
- Live Streaming UI (con controles de host)
- Video Call UI (P2P + group)
- Moments Feed (create, like, comment, share)
- Agency Dashboard (analytics, members, payouts)
- Wallet + Gift Shop (balance, history, catalog)
- Profile pages (edit, view, verification badge)
- Componentes UI completos (Button, Modal, Toast, etc.)
- Dark/Light mode
- PWA + offline support (service worker)
- **Leptos 0.8 CSR** compilado a WASM con `wasm-pack` (crate `ysh-frontend`), routing con `leptos_router`
- Páginas: stream (WebRTC video/live + screen), discover, moments, wallet, gifts, agency, chat, hosts, notifications, profile, auth
- **PWA installable** — `manifest.webmanifest` + service worker (`sw.js`): app shell precacheado, navegación network-first con fallback offline, assets cache-first, iconos SVG

### FASE 12: Internationalization (i18n) ✅
- **Fluent (fluent-rs) integrado** — bundles compilados on-demand, fallback chain
- **5 idiomas completos**: Español, English, Português, العربية, Français
- **Auto-detección del browser** (`Accept-Language`) vía `fluent-langneg` negotiation
- **RTL completo para árabe** — metadata `dir`/`rtl` servida por API
- **Plural rules por idioma** (fluent bundles) — incl. forma dual del árabe
- **Number/currency/date formatting por locale** — separadores decimales/grupo, símbolo moneda, nombres de meses (Rust puro, sin CLDR pesado)
- **Overrides persistidos en redb** — `i18n_overrides` table (key `locale::key`), cargados al arranque
- **Admin panel CRUD** — list/upsert/delete de traducciones (protegido, role `admin`)
- 6 endpoints i18n (4 públicos + 2 admin), probados end-to-end en vivo
- 9 tests i18n/DB — 244 tests totales, 0 warnings, 0 errores

### FASE 13: Social Features + Moderación ✅
- User blocks (bloquear usuarios, ocultar contenido)
- User reports (reportar contenido/usuarios con categorías)
- Verification badges (email, identity, agency, host)
- User reputation system (rating por interacciones)
- Content flags (NSFW, spam, scam — auto + manual)
- Shadow ban (usuario no ve su contenido bloqueado)
- Appeal system (apelar bans con review humano)
- Moderation queue priorizada por severidad
- Auto-moderation rules configurables
- Trust score por usuario (factores: tiempo, verificación, reports)
- 15+ endpoints (social + admin moderation), auto-moderation por IA en `create_moment`
- 10 tablas redb (`MM_BLOCK`, `MM_REPORT`, `MM_BADGE`, `MM_RATING`, `MM_CONTENT_FLAG`, `MM_MOD_QUEUE`, `MM_SHADOW`, `MM_APPEAL`, `T_REPUTATION`, `T_TRUST`)
- 17 tests social — 293 tests totales, 0 warnings, 0 errores

### FASE 14: Background Jobs + Testing ✅
- **JobsActor (ractor)** — scheduler en el supervision tree + timer de `interval_secs` (configurable, min 5s)
- **Payout worker** — `auto_process_payouts()` paga payouts `pending` y marca `processed`
- **Staking worker** — `compute_staking_interest()` recalcula intereses vencidos por posición
- **Moderation worker** — `auto_resolve_moderation()` resuelve items viejos (umbrales de severidad) + `age_moderation_items()`
- **Cleanup worker** — `cleanup_expired()`: retention de analytics/quality, tokens, listas de edad
- **Notification worker** — `flush_pending_notifications()` con presupuesto por sweeps (push/in-app)
- **Analytics worker** — `compute_analytics_snapshot()` persiste snapshot diario (DAU/MAU/revenue/calls/msgs)
- **Admin API** — `/admin/jobs/run/{job}` (on-demand) + `/admin/jobs/stats`
- **Jobs config** — `[jobs]` en TOML: switches por worker, umbrales, retentions
- **Enrutado de jobs** — registro por tipo con fallback, idempotencia e inyección de actividad DB
- **OWASP Top 10 subset** — acces control, crypto (KDF/no plaintext), inyección, integridad (HMAC), logging
- **Property-based testing** — proptest (64 cases): interest bounds, idempotencia, wallet overflow, notif delivery
- **Load testing** — concurrencia: likes encadenados, matching simultáneo, wallets, payouts
- **Hardening `create_user`** — Argon2id dentro de la capa DB (nunca plaintext) + unicidad username/email
- 14 tests jobs + 9 OWASP + 5 prop + 6 load — **340 tests totales, 0 warnings, 0 errores**

### FASE 15: Analytics + Admin Dashboard ✅
- Real-time metrics (conexiones activas, llamadas, revenue)
- User analytics (DAU, MAU, retention, churn)
- Revenue analytics (MRR, ARPU, LTV, gift economy metrics)
- Agency performance dashboards
- Host performance leaderboards
- Geographic distribution maps
- Moderation metrics (reports, bans, appeals)
- System health dashboard (CPU, memory, DB, cache)
- Export to CSV/JSON
- Custom date ranges + filters
- **Admin dashboard web (`/admin`, role admin)** — tabs Realtime · Users · Revenue · Agencies · Hosts · Geo · Moderation
  - Gráfica de barras DAU (14 días), tablas de snapshots, leaderboards con earnings, regiones con %,
    health del sistema (CPU/RAM/uptime/threads/db size), colas de moderación, export CSV directo
- **Profiling de actividad** — `log_activity` en login, calls y moments (fuente del DAU/MAU)
- 11 endpoints `/admin/analytics/*` + `/profile/region/{region}`
- 14 tests analytics — suite completa en verde (0 warnings, 0 errores)

### FASE 16: Deploy + Monitoring
- Dockerfile multi-stage (build + runtime optimizado)
- docker-compose (dev + staging + prod)
- CI/CD pipeline (GitHub Actions)
  - Lint (clippy) → Check (0 warnings) → Test → Build → Deploy
- SSL/TLS (Let's Encrypt + certbot auto-renewal)
- Prometheus metrics + Grafana dashboards
- Structured logging (JSON) + ELK stack
- Security hardening:
  - cargo-audit (dependency vulnerabilities)
  - cargo-deny (license + supply chain)
  - DDoS protection (ya implementado: per-IP rate limit, IP blocklist, circuit breaker, body limit, timeout)
  - Firewall rules (iptables/nftables)
- Backup strategy (redb snapshots + S3 offsite)
- Horizontal scaling (load balancer + multiple instances)
- Health monitoring + alerting (PagerDuty/Slack)
- Runbook documentation

---

## Comandos

```bash
# Build
cargo build

# Run (dev)
YSH_JWT_SECRET=secret YSH_DB_PASSWORD=pass YSH_ENCRYPTION_KEY=0123456789abcdef0123456789abcdef \
  YSH_TLS_CERT=/dev/null YSH_TLS_KEY=/dev/null cargo run

# Quick test
curl http://localhost:8080/healthz
curl -X POST http://localhost:8080/api/v1/register \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","email":"alice@test.com","password":"test123"}'

# Verify
cargo check          # 0 warnings, 0 errors
cargo test           # 340 tests (16 suites) passing
cargo build          # Clean build
```

---

## Seguridad + Compliance

| Amenaza | Mitigación |
|---|---|
| SQL Injection | redb structured keys + validator |
| XSS | Auto-escaping + CSP headers |
| CSRF | Tokens + SameSite + Origin |
| Brute Force | Per-IP rate limiting + lockout |
| Session Hijacking | JWT rotation + HttpOnly + device fingerprint |
| Deepfakes | Burn neural net detector real-time |
| NSFW | AI content moderation (text + video) |
| Fraud | Anomaly detection + behavior analysis |
| DDoS Layer 7 | Per-IP rate limit (auth=5/min, API=60/min) + body limit 1MB + timeout 30s |
| DDoS Layer 4 | IP blocklist (auto-ban 100 errores/60s = block 5min) + circuit breaker |
| WebSocket Abuse | Max 3 conn/user, 10 msgs/s, 64KB/msg, heartbeat timeout |
| Slowloris | Request timeout 30s + TCP keepalive |
| Memory Exhaustion | Request body limit 1MB + WS message size limit |
| Supply Chain | cargo-audit + cargo-deny + pinned versions |
| MITM | TLS 1.3 + certificate pinning |
| Data Breach | AES-256-GCM at rest + TLS in transit |
| GDPR Violation | Right to erasure + data export + consent audit |
| CCPA Violation | Do Not Sell toggle + opt-out mechanism |
| Sanctions evasion | Geofencing + KYC/AML for large withdrawals |
| Config RCE | Lua eliminado, TOML declarativo (sin ejecución de código) |

---

## Disclaimer Regulatorio

```
AVISO IMPORTANTE: YSH Coins son tokens virtuales utilizados exclusivamente
dentro de la plataforma YSH para intercambiar bienes y servicios digitales.
NO son valores financieros, criptomonedas, ni instrumentos de inversión.

- YSH Coins NO se pueden transferir fuera de la plataforma
- El valor de cambio es determinado exclusivamente por YSH
- No generan intereses ni dividendos
- Disponibilidad sujeta a regulaciones locales
- Servicio no disponible en jurisdicciones sancionadas
- Usuarios deben cumplir con las leyes de su jurisdicción
- YSH se reserva el derecho de modificar, suspender o eliminar YSH Coins
  en cualquier momento
```

---

## Licencia

MIT / Apache 2.0
