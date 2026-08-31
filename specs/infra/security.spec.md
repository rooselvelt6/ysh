# Security — Seguridad

## Overview

Capa de seguridad completa: cifrado simétrico/asimétrico, hashing de contraseñas, JWT, TLS, zeroize de memoria, y protección de secrets.

---

## Encryption (AEAD)

### AES-256-GCM

- **Crate:** `aes-gcm` v0.10.3+ con feature `zeroize`
- **Uso:** cifrado de datos en reposo y en tránsito
- **Nonce:** counter-based (`AtomicU64`) — nunca reutilizar nonces
- **Audit:** NCC Group auditado
- **Zeroize:** expanded keys se limpian de memoria en drop

### ChaCha20-Poly1305

- **Crate:** `chacha20poly1305` v0.11+ con feature `zeroize`
- **Uso:** alternativa más rápida en CPUs sin AES-NI
- **Audit:** NCC Group auditado

### Nonce Generation

```rust
struct NonceGenerator {
    random_prefix: [u8; 4],    // 4 bytes aleatorios por instancia
    counter: AtomicU64,         // counter monótonamente creciente
}
// Nonce = [4 random bytes | 8 counter bytes] = 12 bytes
```

---

## Password Hashing

### Argon2id

- **Crate:** `argon2` v0.5+
- **Params (OWASP recommended):**
  - memory_cost: 19456 KB (19 MB)
  - time_cost: 2 iterations
  - parallelism: 1
- **Reglas:** nunca usar SHA/bcrypt para contraseñas

---

## Asymmetric Crypto

### X25519 (Key Exchange)

- **Crate:** `x25519-dalek` v2
- **Uso:** ECDH key exchange para sesiones
- **Feature:** `static_secrets`

### Ed25519 (Digital Signatures)

- **Crate:** `ed25519-dalek` v2
- **Uso:** firmas digitales para transacciones/mensajes
- **Feature:** `rand_core`
- **CSPRNG:** `rand::rngs::OsRng` (os-level entropy)

---

## JWT

- **Crate:** `jsonwebtoken` v9
- **Secret:** desde env var `YSH_JWT_SECRET`, nunca hardcodeado
- **Claims:** sub, exp, iat, role, kind
- **Kinds:** access (24h), refresh (30 días), temp_2fa (5min)

---

## TLS

- **Crate:** `rustls` v0.23+ con `ring`
- **Mínimo:** TLS 1.3 en producción
- **Certificados:** Let's Encrypt / ACME
- **Binder:** `tokio-rustls` v0.26

---

## Secure Memory

### Zeroize

- **Crate:** `zeroize` v1 con feature `derive`
- **Uso:** `ZeroizeOnDrop` en todas las estructuras que contienen secrets
- **Incluye:** `aes-gcm >= 0.10.3` propaga zeroize a expanded round keys

### Secrecy

- **Crate:** `secrecy` v0.10 con feature `serde`
- **Uso:** wrapping de secrets con serde support

---

## Password Hashing

- **Crate:** `blake3` v1
- **Uso:** hashing rápido para verificación de integridad
- **Crate:** `hmac` v0.12 + `sha1` v0.10
- **Uso:** HMAC para verificación de tokens

---

## Rate Limiting

- **Crate:** `governor` v0.8
- **Uso:** rate limiting per-IP
- **Config:** 100 req/s, burst 200

---

## Configuración

```toml
[secrets]
jwt_secret     = { env = "YSH_JWT_SECRET", required = true }
db_password    = { env = "YSH_DB_PASSWORD", required = true }
encryption_key = { env = "YSH_ENCRYPTION_KEY", required = true }

[encryption]
algorithm = "aes-256-gcm"
nonce_strategy = "counter"
key_rotation_days = 30

[password]
algorithm = "argon2id"
memory_cost = 19456
time_cost = 2
parallelism = 1

[tls]
min_version = "1.3"
cert_path = { env = "YSH_TLS_CERT", required = true }
key_path  = { env = "YSH_TLS_KEY", required = true }
```

---

## Crate Audit (2026)

| Crate | Audited | FIPS | Post-Quantum |
|-------|---------|------|--------------|
| aes-gcm | Yes (NCC) | No | No |
| chacha20poly1305 | Yes (NCC) | No | No |
| argon2 | Partial | No | No |
| x25519-dalek | No (trusted by Signal) | No | No |
| ed25519-dalek | No (trusted by Signal) | No | No |
| rustls | Yes | Via ring | Via aws-lc-rs |

---

## Pre-production Security Checklist

| # | Check |
|---|-------|
| 1 | `cargo audit` para CVEs |
| 2 | `cargo clippy --all-targets` |
| 3 | Nonce reuse resistance (counter-based) |
| 4 | Auto key rotation (30 días) |
| 5 | Zeroize en todos los secrets |
| 6 | TLS 1.3 mínimo |
| 7 | Rate limiting habilitado |
| 8 | No secrets en logs |
| 9 | Argon2id para contraseñas |
| 10 | AEAD (AES-GCM/ChaCha20) — nunca ECB/CBC |

---

## Dependencies

- **aes-gcm, chacha20poly1305:** cifrado AEAD
- **argon2, blake3:** hashing
- **x25519-dalek, ed25519-dalek:** crypto asimétrica
- **jsonwebtoken:** JWT
- **rustls, tokio-rustls:** TLS
- **zeroize, secrecy:** memoria segura
- **governor:** rate limiting
