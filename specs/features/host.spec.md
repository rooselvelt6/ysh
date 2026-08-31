# Host — Perfil de Host

## Overview

Sistema de hosts (usuarios que ofrecen servicios de videollamada/conversación) con perfil, disponibilidad, y leaderboard.

**Base path:** `/api/v1`
**Auth:** Requerida

---

## Data Models

### HostProfile
```json
{
  "user_id": "i64",
  "languages": "string[]",
  "hourly_rate": "f64",
  "available": "bool",
  "rating": "f64"
}
```

---

## Endpoints

### POST /host — Crear o actualizar perfil de host

- **Auth:** Requerida
- **Request:**
```json
{
  "languages": ["string[]"],
  "hourly_rate": "f64"
}
```
- **Response 200:**
```json
{
  "message": "Host profile updated",
  "languages": ["string[]"],
  "hourly_rate": "f64"
}
```

---

### GET /host/{user_id} — Ver perfil de host

- **Auth:** No requerida
- **Response:** HostProfile JSON
- **Errores:** `404`

---

### POST /host/availability — Toggle disponibilidad

- **Auth:** Requerida
- **Request:** `{ "available": "bool" }`
- **Response 200:** `{ "available": "bool" }`

---

### GET /hosts — Listar hosts

- **Auth:** No requerida
- **Query:** `?available=true`
- **Response 200:**
```json
{
  "hosts": ["HostProfile[]"],
  "count": "i64"
}
```

---

## Admin Endpoints

### GET /admin/analytics/hosts — Leaderboard de hosts

- **Auth:** Admin
- **Query:** `?limit=10`
- **Response:** leaderboard JSON

---

## Dependencies

- **WebRTC:** hosts participan en llamadas
- **Analytics:** leaderboard y métricas de hosts
