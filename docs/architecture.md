# YSH - Architecture Plan

## Lua Config + Ractor Actors + Security Layer

---

## 1. Project Structure

```
ysh/
├── Cargo.toml
├── config/
│   └── default.lua               # Default system configuration
├── docs/
│   └── architecture.md           # This file
└── src/
    ├── main.rs
    ├── config/
    │   ├── mod.rs
    │   ├── lua_engine.rs
    │   └── settings.rs
    ├── actors/
    │   ├── mod.rs
    │   ├── supervisor_tree.rs
    │   ├── config_actor.rs
    │   ├── server_actor.rs
    │   ├── database_actor.rs
    │   ├── webrtc_actor.rs
    │   ├── ai_actor.rs
    │   ├── crypto_actor.rs
    │   └── session_supervisor.rs
    ├── security/
    │   ├── mod.rs
    │   ├── crypto.rs
    │   ├── keys.rs
    │   ├── password.rs
    │   ├── token.rs
    │   ├── zeroize.rs
    │   ├── tls.rs
    │   └── nonce.rs
    ├── health/
    │   └── mod.rs
    ├── middleware/
    │   ├── rate_limit.rs
    │   └── circuit_breaker.rs
    └── observability/
        └── mod.rs
```

---

## 2. Cargo.toml

```toml
[package]
name = "ysh"
version = "0.1.0"
edition = "2024"

[dependencies]
# ═══════════════════════════════════════════
# CORE
# ═══════════════════════════════════════════
tokio = { version = "1", features = ["full"] }
anyhow = "1"
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }

# ═══════════════════════════════════════════
# CONFIG (Lua 5.4 embebido, vendored)
# ═══════════════════════════════════════════
mlua = { version = "0.10", features = ["lua54", "vendored", "serialize"] }

# ═══════════════════════════════════════════
# ACTORS (Erlang/OTP - ractor + ractor-supervisor)
# ═══════════════════════════════════════════
ractor = "0.10"
ractor-supervisor = "0.3"

# ═══════════════════════════════════════════
# SEGURIDAD - Memoria (zeroize + secrecy)
# ═══════════════════════════════════════════
zeroize = { version = "1", features = ["derive"] }
secrecy = { version = "0.10", features = ["serde"] }

# ═══════════════════════════════════════════
# SEGURIDAD - Cifrado simetrico (AEAD)
# ═══════════════════════════════════════════
aes-gcm = { version = "0.10.3", features = ["zeroize"] }
chacha20poly1305 = { version = "0.10", features = ["zeroize"] }

# ═══════════════════════════════════════════
# SEGURIDAD - Hashing de contrasenas
# ═══════════════════════════════════════════
argon2 = { version = "0.5", features = ["std"] }
blake3 = "1"

# ═══════════════════════════════════════════
# SEGURIDAD - Criptografia asimetrica
# ═══════════════════════════════════════════
x25519-dalek = { version = "2", features = ["static_secrets"] }
ed25519-dalek = { version = "2", features = ["rand_core"] }
rand = "0.9"

# ═══════════════════════════════════════════
# SEGURIDAD - JWT / Tokens
# ═══════════════════════════════════════════
jsonwebtoken = "9"

# ═══════════════════════════════════════════
# SEGURIDAD - TLS
# ═══════════════════════════════════════════
rustls = { version = "0.23", features = ["ring"] }
tokio-rustls = "0.26"

# ═══════════════════════════════════════════
# OBSERVABILIDAD (tracing + metrics)
# ═══════════════════════════════════════════
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
metrics = "0.22"
metrics-exporter-prometheus = "0.16"

# ═══════════════════════════════════════════
# RESILIENCIA (tower + governor)
# ═══════════════════════════════════════════
tower = { version = "0.5", features = ["full"] }
tower-http = { version = "0.6", features = ["cors", "trace"] }
governor = "0.8"
```

---

## 3. OTP Supervision Tree

```
Root Supervisor (OneForOne, max_restarts: 5, max_seconds: 10)
│
├── InfrastructureSupervisor (RestForOne, max_restarts: 3, max_seconds: 30)
│   ├── ConfigActor          <- Loads and watches config
│   ├── DatabaseActor        <- If config fails, DB restarts too
│   └── CacheActor
│
├── ServiceSupervisor (OneForAll, max_restarts: 2, max_seconds: 60)
│   ├── ServerActor          <- HTTP + WebSocket (axum)
│   ├── WebRTCActor          <- Video calls (LiveKit)
│   └── AIActor              <- Content moderation
│
├── SecuritySupervisor (OneForOne, max_restarts: 3, max_seconds: 10)
│   ├── CryptoActor          <- AES-256-GCM / ChaCha20
│   ├── AuthActor            <- JWT + Argon2id
│   └── ModerationActor      <- NSFW / deepfake
│
└── TaskSupervisor (Dynamic, max_children: 10000)
    └── SessionWorkers       <- 1 actor per active user
```

### Supervision Strategies

| Supervisor | Strategy | When |
|---|---|---|
| Root | OneForOne | Independent subsystems |
| Infrastructure | RestForOne | Config depends on nothing; DB depends on config |
| Services | OneForAll | Server, WebRTC, AI are tightly coupled |
| Security | OneForOne | Crypto, Auth, Moderation are independent |
| Task | Dynamic | Spawn sessions on demand |

---

## 4. System Configuration (config/default.lua)

```lua
-- ═══════════════════════════════════════════
-- UTILITIES
-- ═══════════════════════════════════════════
local function require_env(name)
    local val = os.getenv(name)
    if not val or val == "" then
        error("FATAL: Environment variable " .. name .. " is required")
    end
    return val
end

local function optional_env(name, default)
    return os.getenv(name) or default
end

-- ═══════════════════════════════════════════
-- MAIN CONFIGURATION
-- ═══════════════════════════════════════════
return {

    -- SECRETS: NEVER hardcoded, ALWAYS from env vars
    secrets = {
        jwt_secret     = require_env("YSH_JWT_SECRET"),
        db_password    = require_env("YSH_DB_PASSWORD"),
        encryption_key = require_env("YSH_ENCRYPTION_KEY"),
    },

    -- ENCRYPTION
    encryption = {
        algorithm = "aes-256-gcm",
        nonce_strategy = "counter",
        key_rotation_days = 30,
    },

    -- PASSWORD HASHING
    password = {
        algorithm = "argon2id",
        memory_cost = 19456,
        time_cost = 2,
        parallelism = 1,
    },

    -- TLS
    tls = {
        min_version = "1.3",
        cert_path = require_env("YSH_TLS_CERT"),
        key_path  = require_env("YSH_TLS_KEY"),
    },

    -- JWT
    jwt = {
        expiry_hours = 24,
        refresh_expiry_days = 30,
    },

    -- SUPERVISION (OTP strategies)
    supervision = {
        root           = { strategy = "one_for_one",  max_restarts = 5,  max_seconds = 10 },
        infrastructure = { strategy = "rest_for_one",  max_restarts = 3,  max_seconds = 30 },
        services       = { strategy = "one_for_all",   max_restarts = 2,  max_seconds = 60 },
        security       = { strategy = "one_for_one",   max_restarts = 3,  max_seconds = 10 },
    },

    -- SERVER
    server = {
        host = "0.0.0.0",
        port = optional_env("YSH_PORT", "8080"),
        workers = 4,
        shutdown_timeout_secs = 30,
    },

    -- DATABASE
    database = {
        url = optional_env("YSH_DATABASE_URL", "sqlite://ysh.db"),
        max_connections = 10,
        connect_timeout_secs = 5,
        query_timeout_secs = 30,
    },

    -- BACKPRESSURE (bounded channels)
    backpressure = {
        server_channel_size = 1024,
        database_channel_size = 256,
        webrtc_channel_size = 512,
    },

    -- RATE LIMITING
    rate_limit = {
        requests_per_second = 100,
        burst_size = 200,
    },
}
```

---

## 5. Security Modules

### 5.1 `security/crypto.rs` — AEAD Encryption

- **AES-256-GCM**: Authenticated encryption (NCC Group audited)
- **ChaCha20-Poly1305**: Faster on CPUs without AES-NI
- Feature `zeroize` enabled to clean expanded keys from memory
- Counter-based nonce (never reuse nonces — critical for AEAD)

### 5.2 `security/zeroize.rs` — Secure Memory

- `SecureBuffer`: Vec of bytes with `ZeroizeOnDrop`
- `EncryptedKey`: Encrypted key that auto-cleans on drop
- `SecureSecret`: Wrapper of `secrecy::Secret` with serde support
- **Note**: `aes-gcm >= 0.10.3` propagates `zeroize` to expanded round keys (issue #825 closed Jun 2026)

### 5.3 `security/keys.rs` — Asymmetric Crypto

- **X25519 (x25519-dalek)**: ECDH key exchange for sessions
- **Ed25519 (ed25519-dalek)**: Digital signatures for transactions/messages
- CSPRNG via `rand::rngs::OsRng` (os-level entropy)

### 5.4 `security/password.rs` — Hashing

- **Argon2id**: Password Hashing Competition winner
- Params: 19456 KB memory, 2 iterations, 1 parallelism (OWASP recommended)
- Never use SHA/bcrypt for passwords

### 5.5 `security/token.rs` — JWT

- `jsonwebtoken` for creation and validation
- Claims: `sub`, `exp`, `iat`, `role`
- Secret from env, never hardcoded

### 5.6 `security/nonce.rs` — Anti-reuse

- Counter-based nonce: `[4 random bytes | 8 counter bytes]`
- `AtomicU64` for concurrent-safe operation between actors
- Prevents nonce reuse (fatal attack on AEAD)

### 5.7 `security/tls.rs` — TLS

- **rustls** (memory-safe, no OpenSSL)
- TLS 1.3 minimum in production
- Let's Encrypt / ACME support

---

## 6. Execution Flow

```
main.rs
  |
  +-- setup_tracing()                         <- Observability first
  |
  +-- LuaEngine::load("config/default.lua")   <- Load config
  |     +-- Validate secrets from env vars
  |
  +-- build_supervision_tree(config)           <- Build OTP tree
  |     |
  |     +-- InfrastructureSupervisor (RestForOne)
  |     |     +-- ConfigActor     <- #1: Load and watch config
  |     |     +-- DatabaseActor   <- #2: Connection pool
  |     |     +-- CacheActor      <- #3: Local cache
  |     |
  |     +-- ServiceSupervisor (OneForAll)
  |     |     +-- ServerActor     <- HTTP + WebSocket
  |     |     +-- WebRTCActor     <- Video calls
  |     |     +-- AIActor         <- Content moderation
  |     |
  |     +-- SecuritySupervisor (OneForOne)
  |     |     +-- CryptoActor
  |     |     +-- AuthActor
  |     |     +-- ModerationActor
  |     |
  |     +-- TaskSupervisor (Dynamic)
  |           +-- SessionWorkers  <- 1 actor per active user
  |
  +-- tokio::select! {                        <- Graceful shutdown
  |       shutdown_signal => graceful_shutdown(),
  |   }
  |
  +-- shutdown: drain_mailboxes -> zeroize_keys -> close_pools -> exit
```

---

## 7. Hot Reload

```
ConfigActor
  |
  +-- file_watcher.watch("config/")
  |
  +-- on file changed:
  |     +-- LuaEngine::reload("config/default.lua")
  |     +-- Validate new config (no crash on bad config)
  |     +-- broadcast::send(ConfigChanged { new_config })
  |     +-- Other actors react to the message
  |
  +-- Each actor decides if it needs to restart
```

---

## 8. Benefits Summary

| Aspect | Description |
|--------|-------------|
| Hot reload | Change config without recompiling |
| Let-it-crash | Actor crashes -> auto restart without affecting others |
| Supervision Tree | Hierarchical with OTP strategies |
| Meltdown detection | Too many restarts -> escalate or stop |
| Graceful shutdown | Ctrl+C -> drain -> zeroize -> clean exit |
| Backpressure | Bounded channels -> prevents OOM |
| Zeroize | Secrets cleaned from memory on drop |
| AEAD encryption | AES-256-GCM / ChaCha20-Poly1305 |
| Argon2id | GPU/ASIC-resistant password hashing |
| TLS 1.3 | Encrypted traffic in transit |
| JWT + Ed25519 | Stateless auth + digital signatures |
| Rate limiting | Abuse protection |
| Observability | Tracing + metrics for monitoring |
| Dynamic | DynamicSupervisor for user sessions |

---

## 9. Pre-production Checklist

| # | Check | Tool |
|---|-------|------|
| 1 | `cargo audit` for CVEs | `cargo-audit` |
| 2 | `cargo clippy --all-targets` | `cargo-clippy` |
| 3 | Nonce reuse resistance | `NonceGenerator` (counter-based) |
| 4 | Auto key rotation | `encryption.key_rotation_days` |
| 5 | Zeroize on all secrets | `ZeroizeOnDrop` structs |
| 6 | TLS 1.3 minimum | `rustls` config |
| 7 | Rate limiting enabled | `governor` crate |
| 8 | No secrets in logs | `tracing` with field redaction |
| 9 | Argon2id for passwords | `argon2` crate |
| 10 | AEAD (AES-GCM/ChaCha20-Poly1305) | Never ECB/CBC |

---

## 10. Note: aes-gcm + zeroize

Issue #825 (closed Jun 2026) propagated `zeroize` to expanded round keys inside `Aes256Gcm`. Use **`aes-gcm >= 0.10.3`** with feature `zeroize` so round keys are also cleaned from memory on drop.

```toml
# Correct:
aes-gcm = { version = "0.10.3", features = ["zeroize"] }

# Also propagate zeroize to internal deps (if needed):
aes = { version = "0.8", features = ["zeroize"] }
polyval = { version = "0.6", features = ["zeroize"] }
```

---

## 11. Crate Audit Status (2026)

| Crate | Audited | FIPS | Post-Quantum | Downloads |
|-------|---------|------|--------------|-----------|
| aes-gcm | Yes (NCC) | No | No | 148M+ |
| chacha20poly1305 | Yes (NCC) | No | No | 268M+ |
| argon2 | Partial | No | No | 12M+ |
| x25519-dalek | No (trusted by Signal) | No | No | 250M+ |
| ed25519-dalek | No (trusted by Signal) | No | No | 250M+ |
| rustls | Yes | Via ring | Via aws-lc-rs | 377M+ |
