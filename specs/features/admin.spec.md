# Admin — Panel de Administración

## Overview

Endpoints de administración para gestión de usuarios, contenido, pagos, fraude, y configuración del sistema. Todos requieren `role == "admin"`.

**Base path:** `/api/v1`
**Auth:** Admin requerida

---

## Users

### GET /admin/users — Listar usuarios

- **Query:** `?offset=0&limit=50`
- **Response:**
```json
{
  "users": ["object[]"],
  "count": "i64"
}
```

### POST /admin/user/{user_id}/ban — Banear usuario

- **Response 200:** `{ "message": "...", "user_id": "i64" }`

### POST /admin/user/{user_id}/unban — Desbanear

- **Response 200:** `{ "message": "...", "user_id": "i64" }`

### POST /admin/user/{user_id}/role — Cambiar rol

- **Request:** `{ "role": "string (user|admin|moderator|host)" }`
- **Response 200:** `{ "message": "...", "user_id": "i64", "role": "string" }`

---

## Platform Stats

### GET /admin/stats — Estadísticas generales

- **Response:** users, agencies, hosts, moments, gifts counts

---

## Wallets & Transactions

### GET /admin/wallets — Todas las wallets

- **Response:** `{ "wallets": [...], "count": "i64" }`

### GET /admin/transactions — Todas las transacciones

- **Query:** `?limit=50`
- **Response:** `{ "transactions": [...], "count": "i64" }`

### GET /admin/receipts — Todos los receipts

- **Query:** `?limit=50`
- **Response:** `{ "receipts": [...], "count": "i64" }`

### POST /admin/wallet/{user_id}/freeze — Congelar wallet
### POST /admin/wallet/{user_id}/unfreeze — Descongelar wallet
### POST /admin/wallet/{user_id}/adjust — Ajustar balance

_(Detalles en wallet.spec.md)_

---

## Fraud

### GET /admin/fraud — Alertas de fraude

- **Response:** `{ "alerts": [...], "count": "i64" }`

### POST /admin/fraud/{alert_id}/resolve — Resolver alerta

- **Response 200:** `{ "message": "...", "alert_id": "i64" }`

---

## Moments

### GET /admin/moments — Todos los moments

- **Query:** `?limit=50`

### POST /admin/moment/{moment_id}/delete — Eliminar moment

- **Response 200:** `{ "message": "...", "moment_id": "i64" }`

---

## Calls

### GET /admin/calls — Historial de llamadas

- **Query:** `?limit=50`
- **Response:**
```json
{
  "calls": [...],
  "count": "i64",
  "stats": "object"
}
```

---

## Payouts

### GET /admin/payouts — Pagos pendientes
### POST /admin/payouts/process — Aprobar/rechazar pago

_(Detalles en payouts.spec.md)_

---

## Moderación

_(Detalles completos en moderation.spec.md)_

- `GET /admin/moderation/queue`
- `POST /admin/moderation/queue/{item_id}`
- `GET /admin/moderation/reports`
- `POST /admin/moderation/report/{report_id}`
- `GET /admin/moderation/flags`
- `POST /admin/moderation/flag/{flag_id}`
- `GET /admin/moderation/appeals`
- `POST /admin/moderation/appeal/{appeal_id}`
- `POST /admin/user/{user_id}/shadow-ban`
- `POST /admin/user/{user_id}/unshadow-ban`
- `GET /admin/shadow-bans`
- `POST /admin/user/{user_id}/badge`
- `DELETE /admin/user/{user_id}/badge/{badge_type}`
- `GET /admin/moderation/stats`

---

## i18n

- `GET /admin/i18n`
- `POST /admin/i18n`
- `DELETE /admin/i18n/{locale}/{key}_

_(Detalles en i18n.spec.md)_

---

## Jobs

- `POST /admin/jobs/run/{job}`
- `GET /admin/jobs/stats`

_(Detalles en jobs.spec.md)_

---

## Analytics

_(Detalles completos en analytics.spec.md)_

- `GET /admin/analytics/realtime`
- `GET /admin/analytics/users`
- `GET /admin/analytics/revenue`
- `GET /admin/analytics/agencies`
- `GET /admin/analytics/hosts`
- `GET /admin/analytics/geo`
- `GET /admin/analytics/moderation`
- `GET /admin/analytics/health`
- `GET /admin/analytics/snapshots`
- `GET /admin/analytics/export`

---

## Dependencies

- **Wallet:** freeze/adjust
- **Moderation:** queue management
- **Analytics:** metrics endpoints
- **Jobs:** background job triggers
