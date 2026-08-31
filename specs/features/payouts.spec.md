# Payouts — Retiros y Pagos

## Overview

Sistema de solicitudes de retiro con procesamiento admin, múltiples redes/cryptomonedas, y verificación.

**Base path:** `/api/v1`
**Auth:** Requerida

---

## Data Models

### Payout
```json
{
  "payout_id": "i64",
  "user_id": "i64",
  "amount": "f64",
  "currency": "string (USDT)",
  "network": "string (TRC20)",
  "wallet_address": "string",
  "status": "string (pending|approved|rejected|completed)",
  "tx_hash": "string|null",
  "created_at": "string (ISO 8601)"
}
```

---

## User Endpoints

### POST /payout/request — Solicitar retiro

- **Auth:** Requerida
- **Request:**
```json
{
  "amount": "f64 (> 0)",
  "wallet_address": "string (>= 10 chars)",
  "currency?": "string (default: USDT)",
  "network?": "string (default: TRC20)"
}
```
- **Response 200:**
```json
{
  "payout_id": "i64",
  "amount": "f64",
  "currency": "string",
  "network": "string",
  "wallet_address": "string",
  "status": "pending"
}
```
- **Errores:**
  - `400` — amount <= 0, dirección < 10 chars
  - `403` — wallet congelada o spending limit
- **Reglas:**
  - Débito inmediato del wallet
  - Status inicial: pending
  - Crea receipt verificable

### GET /payout/history — Mi historial de payouts

- **Auth:** Requerida
- **Response:**
```json
{
  "payouts": ["Payout[]"],
  "count": "i64"
}
```

---

## Admin Endpoints

### GET /admin/payouts — Payouts pendientes

- **Auth:** Admin
- **Response:**
```json
{
  "payouts": ["Payout[]"],
  "count": "i64"
}
```

### POST /admin/payouts/process — Procesar payout

- **Auth:** Admin
- **Request:**
```json
{
  "payout_id": "i64",
  "approved": "bool",
  "tx_hash?": "string"
}
```
- **Response 200:**
```json
{
  "payout_id": "i64",
  "status": "approved|rejected",
  "tx_hash": "string|null"
}
```
- **Reglas:** si rejected → reembolsa al wallet del usuario

---

## State Machine

```
[pending] → [approved] → completed (con tx_hash)
         → [rejected] → reembolsado al wallet
```

---

## Dependencies

- **Wallet:** débito al solicitar, reembolso al rechazar
- **Receipts:** creación de receipt verificable
