# YSH Platform

Plataforma web 100% Rust: web-first, sin app stores, pagos cripto, IA avanzada y seguridad de nivel enterprise.

---

## Stack Tecnológico

| Componente | Tecnología | Versión | Notas |
|---|---|---|---|
| **Lenguaje** | Rust | 1.98+ (Edition 2024) | |
| **Runtime Async** | Tokio | 1.x | |
| **Backend** | Axum | 0.8.9 | REST + WebSocket |
| **Frontend** | Leptos | 0.8.19 | WASM |
| **Base de Datos** | rusqlite | 0.40 | SQLite dev, Postgres prod |
| **Config** | mlua | 0.10 | Lua 5.4 embebido |
| **Actores** | ractor | 0.16 | OTP supervision tree |
| **Encriptación** | AES-256-GCM / ChaCha20-Poly1305 | 0.10 / 0.11 | E2E messages |
| **Passwords** | Argon2id + Blake3 | 0.5 / 1.x | Hashing seguro |
| **JWT** | jsonwebtoken | 9.x | Tokens |
| **TLS** | rustls | 0.23 | TLS 1.3 |
| **Rate Limiting** | governor | 0.8 | Sliding window |
| **WebRTC/SFU** | LiveKit | 1.x | SFU managed |
| **IA/ML** | Burn | 0.21.0 | Neural nets, fuzzy, genetic |
| **Pagos Cripto** | Binance API | - | BTC/ETH/USDT/BNB |
| **Email** | lettre | 0.11 | SMTP |
| **Push** | fcm | 0.5 | FCM para Android/Web |
| **Object Storage** | s3s | 0.14 | S3-compatible (MinIO) |
| **i18n** | fluent | 0.16 | Multi-idioma |
| **CSS** | Tailwind CSS | 4.x | |
| **Observabilidad** | tracing | 0.1 | Structured logs |
| **Zeroize** | zeroize + secrecy | 1.x / 0.10 | Secretos en memoria |

---

## Arquitectura del Sistema

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         YSH PLATFORM                                   │
├─────────────────────────────────────────────────────────────────────────┤
│  FRONTEND (Leptos + Tailwind CSS + WASM)                              │
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
│  ACTORES OTP (ractor)                                                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────────┐  │
│  │Supervisor│ │ Config   │ │ Server   │ │ Database │ │ Crypto     │  │
│  │  Tree    │ │  Actor   │ │  Actor   │ │  Actor   │ │  Actor     │  │
│  ├──────────┤ ├──────────┤ ├──────────┤ ├──────────┤ ├────────────┤  │
│  │ Session  │ │ WebRTC   │ │ AI       │ │          │ │            │  │
│  │Supervisor│ │  Actor   │ │  Actor   │ │          │ │            │  │
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

## Fases del Proyecto

### FASE 1: Fundamentos ✅
- Workspace, config Lua, actores OTP, seguridad completa
- HTTP server con auth JWT
- 0 warnings, 0 errores

### FASE 2: Seguridad Enterprise
- AES-256-GCM + ChaCha20-Poly1305 E2E
- Argon2id password hashing
- JWT con rotación automática
- 2FA TOTP
- Device fingerprinting
- Rate limiting exhaustivo
- Security headers (CSP, HSTS, X-Frame-Options)
- CORS estricto
- Account lockout after failed attempts
- GDPR compliance (derecho al olvido, portabilidad de datos)
- CCPA compliance (Do Not Sell toggle)
- KYC/AML para crypto

### FASE 3: Base de Datos + Caché
- SQLx con SQLite/PostgreSQL
- Valkey (fred.rs) para sessions, rate limiting, Pub/Sub
- Sled para offline data, queues
- Object Storage (S3-compatible)
- Connection pool con health checks

### FASE 4: Backend API
- API REST completa versionada (v1/v2)
- Auth completo, Users, Agencies, Hosts
- Matching, Calls, Gifts, Prizes, Wallet
- Moments, Communities, Notifications
- Admin endpoints
- OpenAPI docs (utoipa)

### FASE 5: Notificaciones
- Email SMTP (lettre)
- Push notifications (FCM)
- In-app notifications realtime
- Digest emails

### FASE 6: WebSocket + Matching Realtime
- Matching queue en tiempo real
- 15-second timer con opción de extender
- Random match / Filtered match / AI match
- Knock Knock / Duo mode
- Presence system
- Chat E2E encriptado

### FASE 7: WebRTC + Streaming
- LiveKit Server + SDK
- P2P calls, Flash calls, Duo, Group
- Live streaming (LiveKit SFU)
- Screen sharing
- Call recording
- Quality metrics + Simulcast
- Billing por duración

### FASE 8: Economía + Pagos Cripto
- YSH Coins (compra, earn, gasto, staking)
- Gift Economy con rarity + NFTs
- Flash Call Economy
- Binance Integration (deposits, withdrawals)
- Commission Engine multi-nivel
- Payout System automático

### FASE 9: Motor de IA
- Redes Neuronales (Burn): Matching, Deepfake, NSFW, Churn, Pricing
- Algoritmos Genéticos: optimización de parámetros
- Enjambre (ABC/ACO): balanceo de carga
- Lógica Difusa: clasificación de usuarios, QoS
- Recocido Simulado: optimización de recursos
- Heurísticas: anomalías, fraud, patrones
- Moderación IA: text + video + auto-report
- A/B Testing Framework

### FASE 10: Frontend (Leptos + Tailwind)
- Layout responsive
- Discover Page (matching random)
- Live Streaming UI
- Video Call UI
- Moments Feed
- Agency Dashboard
- Wallet + Gift Shop
- Componentes UI completos
- Dark/Light mode
- PWA + offline support

### FASE 11: Background Jobs + Testing
- Payout, Analytics, Moderation, Staking, Cleanup workers
- Unit tests (80%+), Integration, E2E
- Load testing, Security testing (OWASP)
- Mutation testing, Property-based testing

### FASE 12: Deploy + Monitoring
- Dockerfile multi-stage
- docker-compose (dev + prod)
- CI/CD (GitHub Actions)
- SSL/TLS (Let's Encrypt)
- Prometheus + Grafana
- Structured logging (JSON)
- Security hardening (cargo-audit, DDoS protection)

---

## Comandos

```bash
# Dev setup
docker-compose up -d

# Development
cargo build
YSH_JWT_SECRET=secret YSH_DB_PASSWORD=pass YSH_ENCRYPTION_KEY=key \
  YSH_TLS_CERT=/dev/null YSH_TLS_KEY=/dev/null cargo run

# Testing
cargo check
cargo build
cargo nextest run
cargo audit
cargo clippy --workspace
cargo fmt --check

# Production
docker build -t ysh-platform .
docker-compose -f docker-compose.prod.yml up -d
```

---

## Seguridad + Compliance

| Amenaza | Mitigación |
|---|---|
| SQL Injection | Prepared statements + validator |
| XSS | Auto-escaping + CSP headers |
| CSRF | Tokens + SameSite + Origin |
| Brute Force | Sliding window rate limit + lockout |
| Session Hijacking | JWT rotation + HttpOnly + device fingerprint |
| Deepfakes | Burn neural net detector real-time |
| NSFW | AI content moderation (text + video) |
| Fraud | Anomaly detection + behavior analysis |
| DDoS | Rate limiting + connection limits + CDN |
| Supply Chain | cargo-audit + cargo-deny + pinned versions |
| MITM | TLS 1.3 + certificate pinning |
| Data Breach | AES-256-GCM at rest + TLS in transit |
| GDPR Violation | Right to erasure + data export + consent audit |
| CCPA Violation | Do Not Sell toggle + opt-out mechanism |
| Sanctions evasion | Geofencing + KYC/AML for large withdrawals |

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
