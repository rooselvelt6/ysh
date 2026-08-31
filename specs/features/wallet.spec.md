# Wallet — Economía Interna

## Overview

Sistema de billetera virtual con balance, transacciones, depósitos, retiros, transferencias, límites de gasto, y congelamiento de cuentas.

**Base path:** `/api/v1`
**Auth:** Requerida

---

## Data Models

### Wallet
```json
{
  "user_id": "i64",
  "balance": "f64",
  "currency": "YSH",
  "frozen": "bool"
}
```

### Transaction
```json
{
  "id": "i64",
  "tx_type": "string (deposit|withdraw|transfer_in|transfer_out|gift_send|gift_receive|staking_stake|staking_unstake|staking_reward|payout)",
  "amount": "f64",
  "description": "string|null",
  "created_at": "string (ISO 8601)"
}
```

### Spending Limits
```json
{
  "daily_limit": "f64",
  "monthly_limit": "f64"
}
```

---

## Endpoints

### GET /wallet/balance — Ver balance

- **Auth:** Requerida
- **Response 200:**
```json
{
  "balance": "f64",
  "currency": "YSH",
  "frozen": "bool"
}
```
- **Reglas:** auto-crear wallet si no existe

---

### POST /wallet/deposit — Depositar

- **Auth:** Requerida
- **Request:**
```json
{
  "amount": "f64 (> 0)",
  "description?": "string"
}
```
- **Response 200:**
```json
{
  "balance": "f64",
  "deposited": "f64"
}
```
- **Errores:**
  - `400` — amount <= 0
  - `403` — wallet congelada

---

### POST /wallet/withdraw — Retirar

- **Auth:** Requerida
- **Request:**
```json
{
  "amount": "f64 (> 0)",
  "description?": "string"
}
```
- **Response 200:**
```json
{
  "balance": "f64",
  "withdrawn": "f64"
}
```
- **Errores:**
  - `400` — amount <= 0 o insuficiente
  - `403` — wallet congelada o spending limit excedido

---

### POST /wallet/transfer — Transferir a otro usuario

- **Auth:** Requerida
- **Request:**
```json
{
  "to_user_id": "i64",
  "amount": "f64 (> 0)",
  "description?": "string"
}
```
- **Response 200:**
```json
{
  "message": "Transfer completed",
  "amount": "f64",
  "to_user_id": "i64"
}
```
- **Errores:**
  - `400` — self-transfer, amount inválido
  - `403` — wallet congelada o spending limit
  - `404` — destinatario no existe

---

### GET /wallet/transactions — Historial de transacciones

- **Auth:** Requerida
- **Response 200:**
```json
{
  "transactions": ["Transaction[] (últimas 50)"],
  "count": "i64"
}
```

---

### GET /wallet/limits — Ver límites de gasto

- **Auth:** Requerida
- **Response 200:**
```json
{
  "daily_limit": "f64",
  "monthly_limit": "f64"
}
```

---

### POST /wallet/limits — Establecer límites de gasto

- **Auth:** Requerida
- **Request:**
```json
{
  "daily_limit": "f64 (> 0)",
  "monthly_limit": "f64 (> 0)"
}
```
- **Response 200:**
```json
{
  "daily_limit": "f64",
  "monthly_limit": "f64"
}
```
- **Defaults:** daily=100,000 / monthly=1,000,000

---

## Admin Endpoints

### POST /admin/wallet/{user_id}/freeze — Congelar wallet

- **Auth:** Admin
- **Response 200:**
```json
{
  "user_id": "i64",
  "frozen": true
}
```
- **Reglas:** crea fraud alert automáticamente

### POST /admin/wallet/{user_id}/unfreeze — Descongelar wallet

- **Auth:** Admin
- **Response 200:**
```json
{
  "user_id": "i64",
  "frozen": false
}
```

### POST /admin/wallet/{user_id}/adjust — Ajustar balance

- **Auth:** Admin
- **Request:**
```json
{
  "amount": "f64 (negativo = retiro)",
  "description?": "string"
}
```
- **Response 200:**
```json
{
  "message": "Balance adjusted",
  "user_id": "i64",
  "amount": "f64",
  "balance": "f64"
}
```

---

## Spending Limits Logic

- Cada transacción (withdraw, transfer, gift, staking, payout) verifica:
  1. `frozen == false`
  2. Gasto acumulado en ventana de 24h < `daily_limit`
  3. Gasto acumulado en ventana de 30 días < `monthly_limit`
  4. Balance suficiente

---

## Fraud Detection

| Parámetro | Valor |
|-----------|-------|
| velocity_window | 300 segundos |
| max_tx_per_window | 20 |
| max_amount_per_window | 500,000 |
| large_tx_threshold | 10,000 |
| auto_freeze_on_fraud | true |

---

## Receipts

Cada transacción genera un receipt verificable con hash SHA-256.

---

## Dependencies

- **Receipts:** cada tx crea receipt
- **Moderation:** fraud alerts van al sistema de moderación
- **Admin:** freeze/unfreeze/adjust son admin-only
