# Analytics — Analítica del Sistema

## Overview

Dashboard de analytics con métricas en tiempo real, usuarios, revenue, agencias, hosts, geolocalización, moderación, y exportación de datos.

**Base path:** `/api/v1`
**Auth:** Admin (excepto `set_my_region`)

---

## Data Models

### DailySnapshot
```json
{
  "date": "string (YYYY-MM-DD)",
  "users": "i64",
  "moments": "i64",
  "transactions": "i64",
  "revenue": "f64"
}
```

---

## Endpoints

### GET /admin/analytics/realtime — Métricas en tiempo real

- **Auth:** Admin
- **Response 200:**
```json
{
  "online_users": "i64",
  "active_rooms": "i64",
  "db": "object",
  "cache_*": "object",
  "today": "object",
  "captured_at": "string (ISO 8601)"
}
```

---

### GET /admin/analytics/users — Analytics de usuarios

- **Auth:** Admin
- **Query:** `?days=30`
- **Response:** DAU, MAU, retención, churn

---

### GET /admin/analytics/revenue — Analytics de revenue

- **Auth:** Admin
- **Query:** `?days=30`
- **Response:** MRR, ARPU, LTV, gift economy

---

### GET /admin/analytics/agencies — Analytics de agencias

- **Auth:** Admin
- **Response:**
```json
{
  "agencies": ["object[]"]
}
```

---

### GET /admin/analytics/hosts — Leaderboard de hosts

- **Auth:** Admin
- **Query:** `?limit=10`
- **Response:** leaderboard JSON

---

### GET /admin/analytics/geo — Distribución geográfica

- **Auth:** Admin
- **Response:** geo distribution JSON

---

### GET /admin/analytics/moderation — Métricas de moderación

- **Auth:** Admin
- **Response:** moderation metrics

---

### GET /admin/analytics/health — Salud del sistema

- **Auth:** Admin
- **Response 200:**
```json
{
  "uptime_secs": "i64",
  "memory": "object",
  "cpu_usage_pct": "f64",
  "cache": "object",
  "db_size_bytes": "i64",
  "threads": "i64"
}
```
- **Reglas:** lee de `/proc/*`

---

### GET /admin/analytics/snapshots — Snapshots diarios

- **Auth:** Admin
- **Query:** `?limit=30`
- **Response:** `{ "snapshots": ["DailySnapshot[]"] }`

---

### GET /admin/analytics/export — Exportar datos

- **Auth:** Admin
- **Query:**
  - `dataset` (users|revenue|hosts|geo|snapshots)
  - `format` (csv|json, default: csv)
  - `days` (default: 30)
- **Response:** CSV o JSON según `format`
- **Content-Type:** `text/csv` o `application/json`

---

## User Endpoint

### POST /profile/region/{region} — Establecer región

- **Auth:** Requerida
- **Response:** `{ "region": "string" }`
- **Reglas:** 1-64 chars

---

## Configuración

```toml
[analytics]
enabled = true
default_range_days = 30
snapshot_retention_days = 90
```

---

## Background Job

El job `analytics` corre periódicamente y:
1. Calcula métricas del día
2. Almacena snapshot
3. Limpia snapshots antiguos (> retention_days)

---

## Dependencies

- **Jobs:** cálculo periódico de métricas
- **DB:** queries de agregación
- **System:** lectura de /proc para health
