# Receipts — Recibos Verificables

## Overview

Sistema de recibos con hash SHA-256 para verificación de integridad de transacciones.

**Base path:** `/api/v1`
**Auth:** Requerida

---

## Data Models

### Receipt
```json
{
  "receipt_id": "i64",
  "user_id": "i64",
  "tx_type": "string",
  "amount": "f64",
  "description": "string|null",
  "hash": "string (SHA-256 hex)",
  "created_at": "string (ISO 8601)"
}
```

---

## Endpoints

### GET /receipts — Mis recibos

- **Auth:** Requerida
- **Response 200:**
```json
{
  "receipts": ["Receipt[] (últimos 50)"],
  "count": "i64"
}
```

---

### GET /receipt/{receipt_id} — Ver recibo

- **Auth:** Requerida
- **Response:** Receipt JSON
- **Errores:** `404`

---

### GET /receipt/{receipt_id}/verify — Verificar integridad

- **Auth:** Requerida
- **Response 200:**
```json
{
  "receipt_id": "i64",
  "valid": "bool"
}
```
- **Reglas:** recalcula hash y compara con almacenado

---

## Hashing

Cada receipt se firma con SHA-256 sobre los campos:
`{user_id}:{tx_type}:{amount}:{description}:{timestamp}`

---

## Dependencies

- **Wallet:** genera receipts para cada transacción
- **Staking:** genera receipts para stake/unstake/claim
- **Payouts:** genera receipts para solicitudes de retiro
- **Gifts:** genera receipts para envío de regalos
