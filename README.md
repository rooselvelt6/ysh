# YSH Platform — Superando LivChat & Monkey.app

Plataforma web 100% Rust que combina lo mejor de **LivChat** (agencias, hosts, regalos, llamadas privadas) y **Monkey.app** (matching aleatorio, 15s timer, filtros IA, Moments) en una experiencia **superior**: web-first, sin app stores, pagos cripto, IA avanzada y seguridad de nivel enterprise.

---

## Comparativa: YSH vs Competencia

| Funcionalidad | LivChat | Monkey.app | **YSH Platform** |
|---|---|---|---|
| **Plataforma** | Mobile only (Android/iOS) | Mobile + Web | **Web-first (PWA, responsive)** |
| **Matching** | Filtros básicos | 15s timer random | **IA genética + difusa + interés** |
| **Llamadas** | Voz + Video 1:1 | Video 1:1 random | **P2P WebRTC, Duo, Grupal, Streaming (LiveKit)** |
| **Agencias** | Sistema de agencias + hosts | No tiene | **Multi-nivel jerárquico + comisiones** |
| **Regalos** | Coins virtuales | Coins + Super Likes | **Economía cripto real (NFTs + tokens)** |
| **Pagos** | Moneda fiat | Moneda fiat | **Binance + BTC/ETH/USDT/BNB** |
| **Moderación** | Básica | IA anti-bots | **Burn neural nets + deepfake detection** |
| **Seguridad** | Perfiles verificados | Google sign-in | **Verificación biométrica + 2FA + AES-256** |
| **Streaming** | Solo uno a uno | Solo uno a uno | **LiveKit SFU: multistream + salas + eventos en vivo** |
| **Comunidades** | Básico | No tiene | **Grupos, eventos, rankings, challenges** |
| **Economía** | Solo gasto | Solo gasto | **Earn + spend + stake + referral** |
| **i18n** | Parcial | Inglés | **Multi-idioma completo (ES/EN/PT/AR/FR)** |
| **Datos** | Propietario cerrado | Propietario cerrado | **Open source** |
| **Compliance** | Básico | Básico | **GDPR + CCPA + KYC/AML** |

---

## Stack Tecnológico

| Componente | Tecnología | Versión | Notas |
|---|---|---|---|
| **Lenguaje** | Rust | 1.85+ (Edition 2024) | |
| **Runtime Async** | Tokio | 1.40+ | |
| **Backend** | Axum | 0.8.9 | REST + WebSocket |
| **Frontend** | Leptos | 0.8.19 | WASM |
| **Base de Datos** | SQLx + SQLite/PostgreSQL | 0.8 | SQLite dev, Postgres prod |
| **Caché/PubSub** | Valkey (fred.rs) | 10.x | Cache, sessions, realtime |
| **Almacenamiento KV** | Sled | 0.34.7 | Offline, queues, config |
| **Encriptación** | AES-256-GCM (aes-gcm) | 0.11.1 | E2E messages |
| **WebRTC/SFU** | **LiveKit** (livekit-server SDK) | 1.x | **SFU managed, no from scratch** |
| **IA/ML** | Burn | 0.21.0 | Neural nets, fuzzy, genetic |
| **Pagos Cripto** | binance-sdk | 1.0.0 | Binance connector |
| **Email** | lettre | 0.11 | SMTP (verification, reset) |
| **Push Notifications** | fcm (firebase-messaging) | 0.5 | FCM para Android/Web |
| **Object Storage** | s3s + rusoto_s3 | 0.14 / 0.48 | S3-compatible (MinIO local) |
| **i18n** | fluent | 0.16 | Localización multi-idioma |
| **Background Jobs** | loom | 0.7 | Tareas asíncronas (payouts, IA) |
| **Validation** | validator | 0.18 | Validación de structs |
| **OpenAPI** | utoipa | 5.x | Documentación API |
| **CSS** | Tailwind CSS | 4.x | |
| **Iconos** | Lucide Icons | latest | |

---

## Arquitectura del Sistema

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         YSH PLATFORM v2.0                              │
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
│  BACKEND (Axum + Tokio)                                                │
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
│  │ 2FA TOTP │ │ Device   │ │ Anomaly  │ │ Deepfake │ │ CSRF +     │  │
│  │          │ │ Fingerprint│ │ Detection│ │ Filter   │ │ XSS Guard  │  │
│  ├──────────┤ ├──────────┤ ├──────────┤ ├──────────┤ ├────────────┤  │
│  │ GDPR     │ │ CCPA     │ │ KYC/AML  │ │ Audit    │ │ Encrypted  │  │
│  │ Compliance│ │Compliance│ │ Crypto   │ │ Log      │ │ Backups    │  │
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
│  SERVICIOS AUXILIARES                                                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────────┐  │
│  │ Email    │ │ Push     │ │ Object   │ │ i18n     │ │ Background │  │
│  │ (lettre) │ │ (FCM)    │ │ Storage  │ │ (fluent) │ │ Jobs       │  │
│  │          │ │          │ │ (S3/MinIO)│ │          │ │ (loom)     │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └────────────┘  │
├─────────────────────────────────────────────────────────────────────────┤
│  DATOS                                                                 │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐                 │
│  │ SQLx     │ │ Valkey   │ │ Sled     │ │ CDN      │                 │
│  │ (SQLite/ │ │ (Cache/  │ │ (KV      │ │ (Media)  │                 │
│  │ Postgres)│ │ Pub/Sub) │ │ Offline) │ │          │                 │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘                 │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Estructura del Workspace (14 crates)

```
ysh/
├── Cargo.toml                  # Workspace root
├── README.md
├── .env
├── .env.example
├── docker-compose.yml          # Dev: SQLite + Valkey + MinIO
├── docker-compose.prod.yml     # Prod: Postgres + Valkey + S3
│
├── crates/
│   ├── ysh-core/               # Modelos de dominio y traits
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── models/
│   │       │   ├── mod.rs
│   │       │   ├── user.rs
│   │       │   ├── agency.rs
│   │       │   ├── host.rs
│   │       │   ├── call.rs
│   │       │   ├── gift.rs
│   │       │   ├── prize.rs
│   │       │   ├── transaction.rs
│   │       │   ├── match_session.rs
│   │       │   ├── moment.rs
│   │       │   ├── community.rs
│   │       │   └── notification.rs
│   │       ├── traits/
│   │       │   ├── mod.rs
│   │       │   ├── repository.rs
│   │       │   ├── cacheable.rs
│   │       │   ├── encryptable.rs
│   │       │   └── payable.rs
│   │       ├── errors.rs
│   │       └── config.rs
│   │
│   ├── ysh-crypto/             # Seguridad enterprise
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── aes.rs
│   │       ├── hash.rs
│   │       ├── jwt.rs
│   │       ├── totp.rs
│   │       ├── biometric.rs
│   │       ├── fingerprint.rs
│   │       └── sanitize.rs
│   │
│   ├── ysh-db/                 # Capa de persistencia
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── migrations/
│   │       │   ├── 001_initial_schema.sql
│   │       │   ├── 002_agency_hierarchy.sql
│   │       │   ├── 003_economy_tables.sql
│   │       │   ├── 004_moments_feed.sql
│   │       │   └── 005_analytics.sql
│   │       ├── repositories/
│   │       │   ├── mod.rs
│   │       │   ├── user.rs
│   │       │   ├── agency.rs
│   │       │   ├── host.rs
│   │       │   ├── call.rs
│   │       │   ├── gift.rs
│   │       │   ├── transaction.rs
│   │       │   ├── moment.rs
│   │       │   └── match_session.rs
│   │       └── pool.rs
│   │
│   ├── ysh-cache/              # Caché + Pub/Sub
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── valkey.rs
│   │       ├── sled_store.rs
│   │       ├── session.rs
│   │       ├── presence.rs
│   │       └── pubsub.rs
│   │
│   ├── ysh-api/                # Backend API (Axum)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── routes/
│   │       │   ├── mod.rs
│   │       │   ├── v1/              # API v1 namespace
│   │       │   │   ├── mod.rs
│   │       │   │   ├── auth.rs
│   │       │   │   ├── users.rs
│   │       │   │   ├── agencies.rs
│   │       │   │   ├── hosts.rs
│   │       │   │   ├── matching.rs
│   │       │   │   ├── calls.rs
│   │       │   │   ├── gifts.rs
│   │       │   │   ├── prizes.rs
│   │       │   │   ├── wallet.rs
│   │       │   │   ├── moments.rs
│   │       │   │   ├── communities.rs
│   │       │   │   ├── notifications.rs
│   │       │   │   └── admin.rs
│   │       ├── middleware/
│   │       │   ├── auth.rs
│   │       │   ├── rate_limit.rs
│   │       │   ├── cors.rs
│   │       │   ├── security_headers.rs
│   │       │   ├── device_fingerprint.rs
│   │       │   └── versioning.rs
│   │       ├── websocket/
│   │       │   ├── mod.rs
│   │       │   ├── realtime.rs
│   │       │   ├── matchmaker.rs
│   │       │   └── signaling.rs
│   │       └── state.rs
│   │
│   ├── ysh-webrtc/             # WebRTC + Streaming (LiveKit)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── signaling.rs
│   │       ├── room.rs
│   │       ├── media.rs
│   │       ├── livekit_client.rs   # LiveKit SDK wrapper
│   │       ├── recording.rs
│   │       └── quality.rs
│   │
│   ├── ysh-matching/           # Motor de Matching
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── random.rs
│   │       ├── filtered.rs
│   │       ├── ai_matcher.rs
│   │       ├── queue.rs
│   │       └── timer.rs
│   │
│   ├── ysh-agency/             # Sistema de Agencias
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── hierarchy.rs
│   │       ├── commission.rs
│   │       ├── host_management.rs
│   │       ├── sub_agents.rs
│   │       ├── payout.rs
│   │       └── analytics.rs
│   │
│   ├── ysh-economy/            # Economía Virtual + Cripto
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── coins.rs
│   │       ├── gifts_economy.rs
│   │       ├── staking.rs
│   │       ├── referral.rs
│   │       ├── leaderboard.rs
│   │       └── flash_calls.rs
│   │
│   ├── ysh-ai/                 # Motor de IA
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── neural/
│   │       │   ├── mod.rs
│   │       │   ├── recommender.rs
│   │       │   ├── content_detector.rs
│   │       │   ├── deepfake_detector.rs
│   │       │   ├── churn_predictor.rs
│   │       │   └── pricing_model.rs
│   │       ├── genetic/
│   │       │   ├── mod.rs
│   │       │   ├── matching_optimizer.rs
│   │       │   └── commission_optimizer.rs
│   │       ├── swarm/
│   │       │   ├── mod.rs
│   │       │   ├── abc_algorithm.rs
│   │       │   └── aco_algorithm.rs
│   │       ├── fuzzy/
│   │       │   ├── mod.rs
│   │       │   ├── user_classifier.rs
│   │       │   ├── qos_evaluator.rs
│   │       │   └── rating_engine.rs
│   │       ├── annealing/
│   │       │   ├── mod.rs
│   │       │   └── resource_optimizer.rs
│   │       ├── heuristic/
│   │       │   ├── mod.rs
│   │       │   ├── anomaly_detector.rs
│   │       │   ├── fraud_detector.rs
│   │       │   └── pattern_analyzer.rs
│   │       └── moderation/
│   │           ├── mod.rs
│   │           ├── text_moderator.rs
│   │           └── video_moderator.rs
│   │
│   ├── ysh-payments/           # Pagos Cripto
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── binance.rs
│   │       ├── wallet.rs
│   │       ├── transactions.rs
│   │       ├── webhooks.rs
│   │       └── compliance.rs       # KYC/AML
│   │
│   ├── ysh-storage/            # Object Storage (NUEVO)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── s3_client.rs        # S3-compatible (MinIO/AWS)
│   │       ├── media.rs            # Image/video processing
│   │       ├── avatar.rs           # Avatar upload + resize
│   │       ├── moments_media.rs    # Moments video/image
│   │       └── recordings.rs       # Call recordings
│   │
│   ├── ysh-notifications/      # Push + Email (NUEVO)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── push.rs             # FCM push notifications
│   │       ├── email.rs            # Email sending (lettre)
│   │       ├── templates/          # Email templates (MJML/HTML)
│   │       │   ├── verify.rs
│   │       │   ├── reset_password.rs
│   │       │   └── welcome.rs
│   │       └── preferences.rs      # User notification prefs
│   │
│   ├── ysh-i18n/               # Internacionalización (NUEVO)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── locales/
│   │       │   ├── es/             # Español
│   │       │   ├── en/             # English
│   │       │   ├── pt/             # Português
│   │       │   ├── ar/             # العربية
│   │       │   └── fr/             # Français
│   │       └── detector.rs         # Auto-detect user language
│   │
│   ├── ysh-jobs/               # Background Jobs (NUEVO)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── scheduler.rs        # Cron-like scheduling
│   │       ├── workers/
│   │       │   ├── mod.rs
│   │       │   ├── payout_worker.rs     # Weekly agency payouts
│   │       │   ├── analytics_worker.rs  # Usage analytics
│   │       │   ├── moderation_worker.rs # Content review
│   │       │   ├── staking_worker.rs    # Staking rewards calc
│   │       │   └── cleanup_worker.rs    # Expired sessions cleanup
│   │       └── queue.rs            # Job queue (Valkey-backed)
│   │
│   └── ysh-ui/                 # Frontend (Leptos + Tailwind)
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── app.rs
│           ├── components/
│           │   ├── mod.rs
│           │   ├── ui/
│           │   │   ├── button.rs
│           │   │   ├── input.rs
│           │   │   ├── modal.rs
│           │   │   ├── card.rs
│           │   │   ├── table.rs
│           │   │   ├── avatar.rs
│           │   │   ├── badge.rs
│           │   │   ├── toast.rs
│           │   │   ├── skeleton.rs
│           │   │   └── pagination.rs
│           │   ├── layout/
│           │   │   ├── navbar.rs
│           │   │   ├── sidebar.rs
│           │   │   └── mobile_nav.rs
│           │   ├── chat/
│           │   │   ├── chat_window.rs
│           │   │   ├── message_bubble.rs
│           │   │   └── emoji_picker.rs
│           │   ├── video/
│           │   │   ├── video_call.rs
│           │   │   ├── random_match.rs
│           │   │   ├── live_stream.rs
│           │   │   └── screen_share.rs
│           │   ├── agency/
│           │   │   ├── agency_card.rs
│           │   │   ├── host_list.rs
│           │   │   └── commission_chart.rs
│           │   ├── economy/
│           │   │   ├── coin_balance.rs
│           │   │   ├── gift_shop.rs
│           │   │   └── nft_gallery.rs
│           │   └── moments/
│           │       ├── moment_card.rs
│           │       ├── moment_feed.rs
│           │       └── moment_creator.rs
│           ├── pages/
│           │   ├── mod.rs
│           │   ├── home.rs
│           │   ├── auth/
│           │   │   ├── mod.rs
│           │   │   ├── login.rs
│           │   │   ├── register.rs
│           │   │   └── two_factor.rs
│           │   ├── discover.rs
│           │   ├── dashboard.rs
│           │   ├── agency/
│           │   │   ├── mod.rs
│           │   │   ├── create.rs
│           │   │   ├── manage.rs
│           │   │   └── analytics.rs
│           │   ├── host/
│           │   │   ├── mod.rs
│           │   │   ├── profile.rs
│           │   │   └── earnings.rs
│           │   ├── live/
│           │   │   ├── mod.rs
│           │   │   ├── stream.rs
│           │   │   └── watch.rs
│           │   ├── chat.rs
│           │   ├── moments.rs
│           │   ├── prizes.rs
│           │   ├── gifts.rs
│           │   ├── wallet.rs
│           │   ├── communities.rs
│           │   └── admin/
│           │       ├── mod.rs
│           │       ├── users.rs
│           │       ├── agencies.rs
│           │       └── analytics.rs
│           └── api/
│
├── migrations/
├── assets/
├── locales/                    # Fluent .ftl translation files
├── tests/
└── benches/
```

---

## Funcionalidades Clave (Superando LivChat + Monkey)

### 1. Matching Inteligente (vs Monkey 15s Timer)
- **Monkey:** Timer de 15s, matching aleatorio sin control
- **YSH:** Sistema híbrido:
  - Modo rápido (15s como Monkey, pero con IA)
  - Matching por intereses (filtros de personalidad, area, hobbies)
  - Matching geográfico (país, región, ciudad)
  - Matching por preferencias (género, edad)
  - AI-powered matching (Burn neural nets aprenden tus preferencias)
  - **Knock Knock mode** (texto primero, luego video)
  - **Duo mode** (invitar amigo a chatear juntos)

### 2. Sistema de Agencias (vs LivChat)
- **LivChat:** Agencias que reclutan hosts, comisiones hasta 30%
- **YSH:** Sistema mejorado:
  - Jerarquía multi-nivel (Owner > Agency > Sub-Agent > Host)
  - Comisiones configurables por nivel (hasta 40%)
  - Dashboard de analytics en tiempo real
  - KPIs automáticos (horas streaming, engagement, earnings)
  - Reclutamiento automatizado con referral tracking
  - Payouts automáticos via crypto
  - Leaderboards de agencias

### 3. Llamadas y Streaming (vs ambos) — powered by LiveKit
- **LivChat:** Video 1:1, flash calls
- **Monkey:** Video random 15s
- **YSH:** Todo combinado con **LiveKit SFU** (no desde cero):
  - Llamadas 1:1 voz/video (P2P WebRTC)
  - **Flash calls** (llamada rápida aleatoria)
  - **Duo calls** (3 personas)
  - **Group calls** (hasta 8 personas)
  - **Live streaming** (1 a muchos via LiveKit SFU)
  - **Screen sharing**
  - **Call recording** (opt-in con consentimiento, encrypted storage)
  - Calidad adaptativa (simulcast, simulcast automático de LiveKit)
  - **Billing por duración** (débito de wallet)

### 4. Economía (vs ambos)
- **LivChat:** Coins virtuales, regalos, comisiones
- **Monkey:** Coins + Monkey Plus subscription
- **YSH:** Economía completa:
  - **YSH Coins** (token interno, convertible a crypto)
  - **Gift economy** (regalos virtuales con rareza y valores)
  - **NFT collectibles** (regalos especiales como NFTs)
  - **Staking** (gana intereses por mantener coins)
  - **Referral system** (gana por invitar usuarios)
  - **Flash call earnings** (hosts ganan por llamadas)
  - **Agency commissions** (multi-nivel automático)
  - **Withdrawal** directo a Binance/crypto wallet
  - ⚠️ **Disclaimer regulatorio:** YSH Coins son tokens virtuales, no securities. Disponibilidad sujeta a regulaciones locales.

### 5. Moments Feed (superando Monkey)
- **Monkey:** Moments básicos (stories)
- **YSH:** Feed social completo:
  - Posts con video, imagen, texto
  - Likes, comentarios, shares
  - Trending topics
  - Filtros de contenido
  - Monetización de contenido (tips en coins)
  - Storage en S3-compatible con CDN

### 6. Moderación con IA (superando ambos)
- **LivChat:** Moderación manual
- **Monkey:** AI anti-bots básico
- **YSH:** Moderación enterprise:
  - **Deepfake detection** (Burn neural nets)
  - **NSFW content detection** (clasificador de video/texto)
  - **Text moderation** (chat content analysis)
  - **Behavior analysis** (detección de scams/spam)
  - **Anomaly detection** (transacciones sospechosas)
  - **Auto-report** + human review pipeline

### 7. Soporte Multi-idioma (NUEVO)
- **LivChat:** Español/Inglés básico
- **Monkey:** Inglés
- **YSH:** i18n completo con Fluent:
  - Español, English, Português, العربية, Français
  - Auto-detección de idioma del browser
  - Users pueden cambiar idioma en settings
  - RTL support para árabe

### 8. Notificaciones Inteligentes (NUEVO)
- **Push notifications** (FCM) para Android/Web
- **Email transaccional** (verificación, reset, welcome)
- **In-app notifications** con preferencias granulares
- **Digest emails** (resumen semanal de actividad)

---

## Las 12 Fases del Proyecto (Timeline Realista: 29-41 semanas)

---

### FASE 1: Fundamentos y Modelos de Dominio
**Duración: 2-3 semanas**

**Objetivo:** Workspace, configuración, y todos los modelos de dominio necesarios.

**Tareas:**
- [ ] Inicializar Cargo workspace con todos los 14 crates
- [ ] Configurar `ysh-core` con modelos completos:
  - `User` (id, username, email, password_hash, role, avatar_url, bio, interests, location, language, is_verified, is_host, agency_id, wallet_balance, 2fa_enabled, device_fingerprint, created_at)
  - `Agency` (id, name, owner_id, description, logo_url, is_active, commission_rate, max_hosts, min_hours_per_week, country, created_at)
  - `Host` (user_id, agency_id, rating, total_earnings, hours_streamed, calls_completed, gifts_received, level, badge, schedule_json)
  - `Call` (id, caller_id, receiver_id, type: voice/video/flash/duo/group, status, duration_seconds, cost_coins, quality_score, recording_url)
  - `Gift` (id, sender_id, receiver_id, gift_type, value_coins, rarity: common/rare/epic/legendary, nft_id, animation_url)
  - `Prize` (id, agency_id, title, description, cost_coins, stock, image_url, category)
  - `Transaction` (id, user_id, amount, currency, tx_type, status, tx_hash, created_at)
  - `MatchSession` (id, user_a_id, user_b_id, mode: random/filtered/ai, duration_seconds, extended, rating)
  - `Moment` (id, user_id, media_url, caption, likes_count, comments_count, created_at)
  - `Community` (id, name, description, owner_id, member_count, is_public)
  - `Notification` (id, user_id, n_type, title, body, data_json, is_read, created_at)
- [ ] Traits: `Repository<T>`, `Cacheable`, `Encryptable`, `Payable`, `Moderatable`
- [ ] Sistema de errores con `thiserror` + `anyhow`
- [ ] Configuración con `.env` y `dotenvy`
- [ ] Validación de modelos con `validator`

---

### FASE 2: Seguridad Enterprise + Compliance
**Duración: 2-3 semanas**

**Objetivo:** Seguridad que supere a LivChat y Monkey en todos los aspectos + compliance legal.

**Tareas:**
- [ ] AES-256-GCM para E2E encryption de mensajes y datos PII
- [ ] Argon2id para hashing de contraseñas
- [ ] JWT con rotación automática (access 15min + refresh 7d)
- [ ] 2FA TOTP (Google Authenticator, Authy)
- [ ] Device fingerprinting (fingerprint único por dispositivo)
- [ ] Biometric verification helpers (WebAuthn)
- [ ] Rate limiting exhaustivo:
  - **Por endpoint:** login: 5/min, register: 3/min, matching: 30/min, gifts: 10/min, calls: 20/min
  - **Por tier:** free (1x), premium (3x), host (5x), agency (10x)
  - **Implementación:** sliding window con Valkey (no token bucket)
  - **Burst handling:** allow short bursts, then throttle
- [ ] Security headers (CSP, HSTS, X-Frame-Options, X-Content-Type)
- [ ] CORS estricto (solo dominios permitidos)
- [ ] Input sanitization (prevención SQL injection, XSS)
- [ ] CSRF protection para formularios
- [ ] Account lockout after failed attempts (5 fallos → lockout 15min)
- [ ] Session management con invalidación remota
- [ ] **GDPR compliance:**
  - Derecho al olvido (endpoint DELETE /api/v1/users/me con borrado cascade)
  - Consentimiento explícito para grabaciones (checkbox + audit log)
  - Portabilidad de datos (export JSON de todos los datos del usuario)
  - Data Processing Agreement template
  - Privacy policy page
- [ ] **CCPA compliance:**
  - "Do Not Sell My Data" toggle
  - Opt-out of data sharing
- [ ] **KYC/AML para crypto:**
  - Verificación de identidad para withdrawals > $1000
  - Geofencing (bloquear países sancionados: US, CN, KP, IR)
  - Transaction limits por tier
  - Suspicious activity reporting

---

### FASE 3: Base de Datos + Caché + Storage
**Duración: 2-3 semanas**

**Objetivo:** Persistencia robusta, caché de alta performance, y almacenamiento de media.

**Tareas:**
- [ ] SQLx con SQLite (dev) / PostgreSQL (prod)
- [ ] 5 migraciones SQL completas
- [ ] Repositorios: User, Agency, Host, Call, Gift, Transaction, Moment, MatchSession
- [ ] Valkey (fred.rs) para:
  - Sessions con TTL
  - Rate limiting counters (sliding window)
  - Pub/Sub para matching en tiempo real
  - Cache de feeds y queries
  - Presence system (online/offline/busy)
  - Matching queue (usuarios esperando match)
- [ ] Sled para:
  - Cola de mensajes offline
  - Cache de modelos de IA
  - Datos de configuración local
  - Queue de transacciones pendientes
- [ ] **Object Storage (ysh-storage):**
  - S3-compatible client (MinIO para dev, AWS S3 / Cloudflare R2 para prod)
  - Avatar upload + auto-resize (256x256, 512x512)
  - Moments video/image upload (max 60s video, 10MB images)
  - Call recordings storage (encrypted at rest)
  - Gift animation storage
  - CDN integration (Cloudflare / CloudFront)
- [ ] Connection pool con health checks
- [ ] Prepared statements everywhere

---

### FASE 4: Backend API (Axum) + API Versioning
**Duración: 3-4 semanas**

**Objetivo:** API REST completa con todas las funcionalidades, versionada desde el inicio.

**Tareas:**
- [ ] Configurar Axum con middleware stack
- [ ] **API Versioning:** todas las rutas bajo `/api/v1/` (preparado para `/api/v2/`)
- [ ] **Auth:** register, login, refresh, logout, 2FA setup/verify, password reset, email verification
- [ ] **Users:** profile CRUD, preferences, verification status, block/report, account deletion (GDPR)
- [ ] **Agencies:** CRUD, hierarchy, host management, sub-agents, analytics
- [ ] **Hosts:** earnings, schedule, KPIs, go-live, go-offline
- [ ] **Matching:** start queue, cancel, accept/extend/end match, filters
- [ ] **Calls:** initiate, accept, reject, end, report, quality feedback
- [ ] **Gifts:** catalog, send, receive, NFT mint, gift history
- [ ] **Prizes:** catalog, redeem, stock management
- [ ] **Wallet:** balance, deposit address, withdraw, transfer, history
- [ ] **Moments:** create, feed, like, comment, delete, trending
- [ ] **Communities:** create, join, leave, events, challenges
- [ ] **Notifications:** list, mark read, preferences, push token registration
- [ ] **Admin:** user management, agency oversight, analytics, moderation queue
- [ ] All middleware: auth, rate_limit, cors, security_headers, device_fingerprint
- [ ] Health check, OpenAPI docs (utoipa)
- [ ] Request validation with `validator`

---

### FASE 5: Email + Push Notifications
**Duración: 1 semana**

**Objetivo:** Sistema de notificaciones completo (email + push + in-app).

**Tareas:**
- [ ] **Email (ysh-notifications):**
  - SMTP integration con `lettre`
  - Email de verificación de cuenta
  - Email de reset de contraseña
  - Email de bienvenida
  - Email de payout recibido
  - Templates HTML responsivos
  - Rate limiting de emails (no spam)
- [ ] **Push Notifications (FCM):**
  - Firebase Cloud Messaging integration
  - Push para: match found, gift received, call incoming, new moment
  - Device token management (register/unregister)
  - Batch push for broadcast (events, challenges)
- [ ] **In-app notifications:**
  - Realtime via WebSocket
  - Mark as read/unread
  - Notification preferences (categorías on/off)
  - Notification history
- [ ] **Digest emails:**
  - Weekly summary (earnings, new followers, moments)
  - Configurable frequency (daily/weekly/monthly)

---

### FASE 6: WebSocket + Matching en Tiempo Real
**Duración: 2-3 semanas**

**Objetivo:** Matching instantáneo estilo Monkey con la infraestructura de LivChat.

**Tareas:**
- [ ] WebSocket server con Axum
- [ ] **Matching queue** (usuarios entran, sistema los empareja)
- [ ] **15-second timer** (estilo Monkey, con opción de extend)
- [ ] **Random match** (matching aleatorio rápido)
- [ ] **Filtered match** (por género, edad, ubicación, intereses)
- [ ] **AI match** (Burn neural nets aprenden preferencias)
- [ ] **Knock Knock** (texto primero, luego video)
- [ ] **Duo mode** (invitar amigo al chat)
- [ ] Presence system (online/offline/busy/in-call)
- [ ] Typing indicators
- [ ] Chat E2E encriptado (AES-256-GCM)
- [ ] Notificaciones push (match found, gift received, call incoming)
- [ ] Reconnection automática con exponential backoff
- [ ] Mensajes offline queue (Sled)
- [ ] Valkey Pub/Sub para broadcasting

---

### FASE 7: WebRTC + Streaming (LiveKit)
**Duración: 3-4 semanas** (reducido de 6+ gracias a LiveKit)

**Objetivo:** Llamadas de voz/video y streaming en vivo con LiveKit SFU.

**Tareas:**
- [ ] **LiveKit Server** setup (self-hosted o cloud)
- [ ] **LiveKit SDK integration** en backend (Rust SDK)
- [ ] **LiveKit JS/Leptos** integration en frontend
- [ ] WebRTC signaling (SDP, ICE) via LiveKit
- [ ] P2P connections (1:1 calls) via LiveKit
- [ ] SFU para group calls y streaming
- [ ] ICE/STUN/TURN configuration (LiveKit handles this)
- [ ] Llamadas de voz (audio only)
- [ ] Videollamadas (HD video + audio)
- [ ] **Flash calls** (llamada random de 15s estilo Monkey)
- [ ] **Duo calls** (3 personas)
- [ ] **Group calls** (hasta 8 personas)
- [ ] **Live streaming** (1 a muchos via LiveKit)
- [ ] Screen sharing
- [ ] Mute/unmute audio/video
- [ ] Call recording (LiveKit recording API, opt-in, con consentimiento, encrypted)
- [ ] Quality metrics (jitter, packet loss, bitrate, resolution)
- [ ] Simulcast (LiveKit handles this automatically)
- [ ] Rate limiting de llamadas
- [ ] Billing por duración (débito automático de wallet)
- [ ] Call rating (1-5 estrellas post-call)

---

### FASE 8: Economía + Pagos Cripto
**Duración: 3-4 semanas**

**Objetivo:** Economía virtual completa con pagos cripto reales + compliance.

**Tareas:**
- [ ] **YSH Coins:**
  - Compra con crypto (Binance, BTC, ETH, USDT, BNB)
  - Earn por actividad (streaming, referrals, challenges)
  - Gasto en regalos, premium features, flash calls
  - Staking con recompensas
  - **Transaction limits:** free: $100/day, verified: $1000/day, KYC: unlimited
  - **Regulatory:** "YSH Coins are virtual tokens, not securities. Not available in sanctioned jurisdictions."
- [ ] **Gift Economy:**
  - Catálogo de regalos (basic → legendary)
  - Rarity system con probabilidades
  - NFT collectibles (regalos únicos)
  - Gift animations (stored in S3)
- [ ] **Flash Call Economy:**
  - Costo por segundo configurable
  - Host earnings por flash call
  - Bonus por engagement
- [ ] **Binance Integration:**
  - Depósitos (BTC, ETH, USDT, BNB, BUSD)
  - Withdrawals con 2FA
  - Price conversion (multi-currency)
  - Transaction verification on-chain
  - **Geofencing:** block US, CN, KP, IR
- [ ] **Commission Engine:**
  - Multi-level agency commissions (automáticas)
  - Referral commissions
  - Platform fee configurable
- [ ] **Payout System:**
  - Weekly automatic payouts a agencies (via background job)
  - Crypto payouts directos
  - Audit trail inmutable

---

### FASE 9: Motor de IA (Burn)
**Duración: 6-8 semanas** (realista para IA completa)

**Objetivo:** IA que supere la moderación y matching de ambas plataformas.

**Tareas:**

#### 9.1 Redes Neuronales (Burn 0.21) — 3-4 semanas
- [ ] **Matching Engine** (collaborative filtering + content-based)
- [ ] **Deepfake Detector** (clasificador de video en tiempo real)
- [ ] **NSFW Detector** (texto + video + imagen)
- [ ] **Churn Predictor** (predicción de abandono de usuarios)
- [ ] **Dynamic Pricing** (precios adaptativos de llamadas/regalos)
- [ ] **Sentiment Analyzer** (análisis de sentimiento en chat)

#### 9.2 Algoritmos Genéticos — 1 semana
- [ ] Evolución de parámetros de matching
- [ ] Optimización de comisiones por agencia
- [ ] Optimización de layouts (A/B testing automatizado)
- [ ] Evolución de precios de regalos

#### 9.3 Enjambre — 1 semana
- [ ] **Abejas (ABC):** Balanceo de carga WebRTC, optimización de servidores
- [ ] **Hormigas (ACO):** Routing de matching, optimización de rutas de recomendación

#### 9.4 Lógica Difusa — 1 semana
- [ ] Clasificación de usuarios (VIP, Regular, Nuevo, Host)
- [ ] QoS evaluation de llamadas
- [ ] Priorización de mensajes
- [ ] Rating multi-factorial de agencias

#### 9.5 Recocido Simulado — 0.5 semanas
- [ ] Asignación óptima de recursos (servidores, bandwidth)
- [ ] Resolución de conflictos de horarios
- [ ] Optimización de precios dinámicos

#### 9.6 Heurísticas — 0.5 semanas
- [ ] Detección de anomalías en transacciones
- [ ] Fraud detection en pagos
- [ ] Análisis de patrones de uso
- [ ] Alertas inteligentes para admins

#### 9.7 Moderación IA — 1 semana
- [ ] Text moderation (chat en tiempo real)
- [ ] Video moderation (stream analysis)
- [ ] Auto-report pipeline → ysh-jobs
- [ ] Human review queue

#### 9.8 A/B Testing Framework — 0.5 semanas
- [ ] Feature flags (percent-based rollout)
- [ ] Experiment tracking (variant → metric)
- [ ] Statistical significance calculator
- [ ] Integration con AI engine para optimización automática

---

### FASE 10: Frontend (Leptos + Tailwind)
**Duración: 4-6 semanas**

**Objetivo:** UI/UX que supere la experiencia de LivChat y Monkey.

**Tareas:**
- [ ] **Layout:** sidebar responsive + navbar + content area
- [ ] **i18n integration:** change language, RTL support
- [ ] **Discover Page** (Monkey-style random matching):
  - Botón "Start" grande (estilo Monkey)
  - 15s timer animation
  - Skip/Next button
  - Extend button
  - Filters panel (premium)
- [ ] **Live Streaming:**
  - Grid de streams en vivo
  - Chat overlay
  - Gift animation overlay
  - Viewer count
- [ ] **Video Call UI:**
  - Full-screen video (LiveKit VideoTrack)
  - Controls (mute, camera, end, screen share)
  - Quality indicator
  - Duration timer
  - Gift button overlay
- [ ] **Moments Feed:**
  - Stories-style top bar
  - Video/image posts (from S3/CDN)
  - Like/comment/share
  - Create moment button
- [ ] **Agency Dashboard:**
  - Host management table
  - Commission charts
  - Earnings graphs
  - KPI cards
- [ ] **Wallet:**
  - Balance display
  - Deposit/Withdraw buttons
  - Transaction history
  - Coin purchase (crypto)
- [ ] **Gift Shop:**
  - Grid of gifts with rarity colors
  - Price in coins
  - Preview animation
  - NFT section
- [ ] **Componentes UI:**
  - Button, Input, Modal, Card, Table
  - Avatar, Badge, Tooltip, Toast
  - Skeleton, Pagination, Tabs
  - Toggle, Select, DatePicker
- [ ] **Responsive:** mobile-first, PWA, offline support
- [ ] **Dark/Light mode**
- [ ] **Animations:** CSS transitions + Leptos animations

---

### FASE 11: Background Jobs + Testing
**Duración: 3-4 semanas**

**Objetivo:** Tareas automatizadas + calidad production-ready.

**Tareas:**

#### 11.1 Background Jobs — 1-2 semanas
- [ ] **Payout Worker:** weekly commission calculations + crypto payouts
- [ ] **Analytics Worker:** daily usage stats, agency reports
- [ ] **Moderation Worker:** queued content review (AI + human)
- [ ] **Staking Worker:** daily staking rewards calculation
- [ ] **Cleanup Worker:** expired sessions, old data purge
- [ ] **Email Digest Worker:** weekly/monthly email summaries
- [ ] Job queue backed by Valkey (reliable, retriable)

#### 11.2 Testing — 2 semanas
- [ ] Unit tests (80%+ coverage)
- [ ] Integration tests (API endpoints)
- [ ] E2E tests (matching flow, call flow, payment flow)
- [ ] Load testing (1000+ concurrent users)
- [ ] Security testing (OWASP Top 10)
- [ ] Mutation testing (cargo-mutants)
- [ ] Property-based testing (proptest)

---

### FASE 12: Deploy + Optimización + Monitoring
**Duración: 2-3 semanas**

**Objetivo:** Production-ready con observabilidad completa.

**Tareas:**
- [ ] Dockerfile multi-stage (rust builder → slim runtime)
- [ ] docker-compose (dev + prod)
- [ ] **Dev environment:** SQLite + Valkey + MinIO (1 command setup)
- [ ] **Prod environment:** PostgreSQL + Valkey + S3 + LiveKit
- [ ] CI/CD (GitHub Actions)
- [ ] SSL/TLS (Let's Encrypt)
- [ ] **Monitoring:**
  - Prometheus metrics (request latency, error rates, WS connections)
  - Grafana dashboards (API, DB, Valkey, LiveKit, AI)
  - Custom metrics (matching success rate, call quality, economy flow)
- [ ] **Logging:** structured tracing (JSON)
- [ ] **Alertas:** PagerDuty / Telegram / Discord alerts
- [ ] **Performance:**
  - Query benchmarking
  - Profiling (perf/flamegraph)
  - WASM bundle optimization
  - Connection pool tuning
  - Cache hit ratio optimization
  - CDN para assets (Cloudflare)
- [ ] **Security hardening:**
  - cargo-audit + cargo-deny
  - DDoS protection (nginx/HAProxy)
  - Secret management (Vault / env injection)
  - Encrypted backups (daily, 30-day retention)
- [ ] Runbooks for common operations

---

## Tabla de Dependencias (Actualizada)

| Crate | Versión | Función | Nuevo |
|---|---|---|---|
| `tokio` | 1.40+ | Runtime async | |
| `axum` | 0.8.9 | Web framework | |
| `leptos` | 0.8.19 | Frontend WASM | |
| `sqlx` | 0.8 | Database | |
| `fred` | 10.x | Valkey/Redis | |
| `sled` | 0.34.7 | Embedded KV | |
| `aes-gcm` | 0.11.1 | AES-256-GCM | |
| `argon2` | 0.5 | Password hashing | |
| `jsonwebtoken` | 9.x | JWT | |
| `totp-rs` | 5.x | 2FA TOTP | |
| `livekit-server-sdk` | 1.x | SFU/WebRTC | **NEW** |
| `burn` | 0.21.0 | AI/ML | |
| `binance-sdk` | 1.0.0 | Binance API | |
| `lettre` | 0.11 | Email SMTP | **NEW** |
| `firebase-messaging` | 0.5 | FCM push | **NEW** |
| `s3s` | 0.14 | S3-compatible | **NEW** |
| `fluent` | 0.16 | i18n | **NEW** |
| `loom` | 0.7 | Background jobs | **NEW** |
| `validator` | 0.18 | Input validation | **NEW** |
| `serde` | 1.x | Serialization | |
| `thiserror` | 2.x | Errors | |
| `anyhow` | 1.x | Error context | |
| `tracing` | 0.1 | Logging | |
| `reqwest` | 0.12 | HTTP client | |
| `uuid` | 1.x | UUIDs | |
| `chrono` | 0.4 | DateTime | |
| `rand` | 0.9 | Random | |
| `utoipa` | 5.x | OpenAPI docs | |

---

## Seguridad + Compliance

| Amenaza | Mitigación |
|---|---|
| SQL Injection | Prepared statements + validator |
| XSS | Auto-escaping Leptos + CSP headers |
| CSRF | Tokens + SameSite + Origin |
| Brute Force | Sliding window rate limit + account lockout |
| Session Hijacking | JWT rotation + HttpOnly + device fingerprint |
| Deepfakes | Burn neural net detector real-time |
| NSFW | AI content moderation (text + video) |
| Fraud | Anomaly detection + behavior analysis |
| DDoS | Rate limiting + connection limits + CDN |
| Supply Chain | cargo-audit + cargo-deny + pinned versions |
| MITM | TLS 1.3 + certificate pinning |
| Data Breach | AES-256-GCM at rest + TLS in transit |
| **GDPR Violation** | **Right to erasure + data export + consent audit** |
| **CCPA Violation** | **Do Not Sell toggle + opt-out mechanism** |
| **Sanctions evasion** | **Geofencing + KYC/AML for large withdrawals** |
| **Securities classification** | **Disclaimer + virtual token classification** |

---

## Rate Limiting Detallado

| Endpoint | Free | Premium | Host | Agency |
|---|---|---|---|---|
| POST /auth/login | 5/min | 5/min | 5/min | 5/min |
| POST /auth/register | 3/min | - | - | - |
| POST /matching/start | 10/min | 30/min | 50/min | 100/min |
| POST /calls/initiate | 10/min | 30/min | 50/min | 100/min |
| POST /gifts/send | 5/min | 20/min | 30/min | 50/min |
| GET /moments/feed | 30/min | 60/min | 60/min | 100/min |
| POST /moments/create | 5/min | 20/min | 30/min | 50/min |
| POST /wallet/withdraw | 2/day | 5/day | 10/day | 20/day |
| WebSocket connections | 1 | 3 | 5 | 10 |
| API-wide | 100/min | 300/min | 500/min | 1000/min |

**Implementación:** Sliding window counter en Valkey por (user_id, endpoint, window).

---

## Disclaimer Regulatorio (YSH Coins)

```
AVISO IMPORTANTE: YSH Coins son tokens virtuales utilizados exclusivamente 
dentro de la plataforma YSH para intercambiar bienes y servicios digitales. 
NO son valores financieros, criptomonedas, ni instrumentos de inversión.

- YSH Coins NO se pueden transferir fuera de la plataforma
- El valor de cambio es determinado exclusivamente por YSH
- No generan intereses ni dividendos
- Disponibilidad sujeta a regulaciones locales
- Servicio no disponible en: Estados Unidos, China, Corea del Norte, Irán, 
  Cuba, Crimea, y otras jurisdicciones sancionadas
- Usuarios deben cumplir con las leyes de su jurisdicción
- YSH se reserva el derecho de modificar, suspender o eliminar YSH Coins 
  en cualquier momento
```

---

## Requisitos

- **Rust:** 1.85+ (Edition 2024)
- **Valkey:** 8.x
- **SQLite:** 3.x (dev) / **PostgreSQL:** 16+ (prod)
- **MinIO:** latest (dev storage) / **AWS S3 / Cloudflare R2** (prod)
- **LiveKit:** 1.x (self-hosted or cloud)
- **Docker:** Para dev environment
- **Node.js:** Solo para build de Tailwind CSS (opcional, puede usar CDN)

---

## Comandos

```bash
# Dev setup (1 command)
docker-compose up -d              # SQLite + Valkey + MinIO + LiveKit

# Development
cargo build                       # Build workspace
cargo run --bin ysh-api           # Run backend (port 3000)
cargo leptos serve                # Run frontend (port 8080)

# Testing
cargo nextest run                 # Unit + integration tests
cargo audit                       # Security audit
cargo clippy --workspace          # Linting
cargo fmt --check                 # Format check

# Production
docker build -t ysh-platform .
docker-compose -f docker-compose.prod.yml up -d

# Database
sqlx migrate run                  # Run migrations
sqlx migrate add <name>           # Create new migration
```

---

## Licencia

MIT / Apache 2.0

---

> **Filosofía:** YSH no es solo una copia de LivChat o Monkey.app. Es una plataforma **superior** que combina lo mejor de ambos mundos (agencias + matching aleatorio) con tecnología de vanguardia (Rust + IA + crypto + LiveKit) para crear una experiencia que ninguna de las dos ofrece actualmente. Cada decisión técnica está validada: LiveKit para SFU (no reinventar la rueda), GDPR/CCPA desde el día 1, rate limiting granular, y un timeline realista de 29-41 semanas.
