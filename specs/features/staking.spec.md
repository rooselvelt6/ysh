# Staking — Staking de Tokens

## Overview

Sistema de staking con posiciones, APY configurable, lock periods, y rewards automáticos.

**Base path:** `/api/v1`
**Auth:** Requerida

---

## Data Models

### StakePosition
```json
{
  "stake_id": "i64",
  "user_id": "i64",
  "amount": "f64",
  "apy_rate": "f64",
  "unlock_days": "i32",
  "created_at": "string (ISO 8601)",
  "unlock_at": "string (ISO 8601)",
  "status": "string (locked|unlocked|unstaked)"
}
```

---

## Endpoints

### POST /staking/stake — Crear posición de staking

- **Auth:** Requerida
- **Request:**
```json
{
  "amount": "f64 (> 0)",
  "apy_rate?": "f64 (default: 0.05 = 5%)",
  "unlock_days?": "i32 (1-365, default: 30)"
}
```
- **Response 200:**
```json
{
  "stake_id": "i64",
  "amount": "f64",
  "apy_rate": "f64",
  "unlock_days": "i32"
}
```
- **Errores:**
  - `400` — amount <= 0, unlock_days fuera de rango
  - `403` — wallet congelada o spending limit
- **Reglas:**
  - amount debe ser > 0
  - unlock_days: 1-365
  - Default APY: 5%
  - Débito inmediato de wallet

---

### POST /staking/unstake — Retirar posición

- **Auth:** Requerida
- **Request:**
```json
{
  "stake_id": "i64"
}
```
- **Response 200:**
```json
{
  "message": "Stake unstaked",
  "returned": "f64",
  "stake_id": "i64"
}
```
- **Errores:**
  - `400` — lock period no cumplido (DB constraint)
  - `404` — stake no existe

---

### POST /staking/claim — Reclamar rewards

- **Auth:** Requerida
- **Request:**
```json
{
  "stake_id": "i64"
}
```
- **Response 200:**
```json
{
  "claimed": "f64",
  "stake_id": "i64"
}
```

---

### GET /staking/positions — Ver posiciones activas

- **Auth:** Requerida
- **Response 200:**
```json
{
  "positions": ["StakePosition[]"],
  "count": "i64"
}
```

---

### GET /staking/stats — Estadísticas globales de staking

- **Auth:** No requerida (público)
- **Response:** estadísticas agregadas

---

## Configuración

```toml
[economy.staking]
min_stake = 100
max_stake = 10000000
default_apy = 0.05          # 5%
min_lock_days = 1
max_lock_days = 365
reward_calc_interval_hours = 24
```

---

## Background Job

El job `staking` corre cada `interval_secs` (60s) y:
1. Calcula rewards para posiciones locked
2. Acumula rewards en la posición
3. Permite claim de rewards acumulados

---

## Dependencies

- **Wallet:** débito al stakear, crédito al unstakear/claim
- **Jobs:** reward calculation automática
