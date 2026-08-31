# Moderación — Sistema de Moderación

## Overview

Sistema de moderación con auto-moderación AI, reports de usuarios, content flags, shadow bans, badges, appeals, y trust score.

**Base path:** `/api/v1`
**Auth:** Requerida (user endpoints) / Admin (admin endpoints)

---

## Data Models

### Report
```json
{
  "report_id": "i64",
  "target_type": "string (user|moment|message|host|agency)",
  "target_id": "i64",
  "category": "string|null",
  "description": "string|null",
  "status": "string (pending|reviewed|resolved|dismissed)",
  "created_at": "string (ISO 8601)"
}
```

### ContentFlag
```json
{
  "flag_id": "i64",
  "target_type": "string (moment|message|user|host)",
  "target_id": "i64",
  "flag_type": "string|null",
  "description": "string|null",
  "status": "string (pending|reviewed|resolved|dismissed)",
  "created_at": "string (ISO 8601)"
}
```

### Appeal
```json
{
  "appeal_id": "i64",
  "target_type": "string (ban|shadow_ban|content_flag)",
  "target_id": "i64",
  "reason": "string",
  "status": "string (open|approved|denied)",
  "created_at": "string (ISO 8601)"
}
```

### TrustScore
```json
{
  "user_id": "i64",
  "score": "f64",
  "level": "string"
}
```

---

## User Endpoints

### POST /report — Crear reporte

- **Auth:** Requerida
- **Request:**
```json
{
  "target_type": "string (user|moment|message|host|agency)",
  "target_id": "i64",
  "category?": "string",
  "description?": "string"
}
```
- **Response 200:**
```json
{
  "report_id": "i64",
  "message": "Report created"
}
```
- **Reglas:**
  - Auto-shadow-ban si usuarios distintos reportan >= threshold (5 reports)

### GET /reports — Mis reportes

- **Auth:** Requerida
- **Response:** `{ "reports": [...], "count": "i64" }`

### POST /flag — Flag content

- **Auth:** Requerida
- **Request:**
```json
{
  "target_type": "string (moment|message|user|host)",
  "target_id": "i64",
  "flag_type?": "string",
  "description?": "string"
}
```
- **Response 200:**
```json
{
  "flag_id": "i64",
  "message": "Content flagged"
}
```

### POST /appeal — Crear apelación

- **Auth:** Requerida
- **Request:**
```json
{
  "target_type": "string (ban|shadow_ban|content_flag)",
  "target_id": "i64",
  "reason": "string"
}
```
- **Response 200:**
```json
{
  "appeal_id": "i64",
  "message": "Appeal created"
}
```

### GET /appeals — Mis apelaciones

- **Auth:** Requerida
- **Response:** `{ "appeals": [...], "count": "i64" }`

---

## Social Endpoints

### POST /block — Bloquear usuario

- **Auth:** Requerida
- **Request:** `{ "target_user_id": "i64" }`
- **Response 200:** `{ "message": "User blocked", "blocked_user_id": "i64" }`

### DELETE /block/{user_id} — Desbloquear

- **Auth:** Requerida
- **Response 200:** `{ "message": "User unblocked" }`

### GET /blocks — Lista de bloqueados

- **Auth:** Requerida
- **Response:** `{ "blocked": [...], "count": "i64" }`

### GET /badges — Mis badges

- **Auth:** Requerida
- **Response:** `{ "badges": [...], "count": "i64" }`

### GET /badges/{user_id} — Badges de un usuario

- **Auth:** No requerida

### POST /rating/{user_id} — Calificar usuario

- **Auth:** Requerida
- **Request:** `{ "score": "f64" }`
- **Response 200:**
```json
{
  "message": "Rating submitted",
  "rating_avg": "f64",
  "rating_count": "i64"
}
```

### GET /rating/{user_id} — Ver reputación

- **Auth:** No requerida

### GET /trust — Mi trust score

- **Auth:** Requerida
- **Response:** TrustScore

---

## Admin Endpoints

### GET /admin/moderation/queue — Cola de moderación

- **Auth:** Admin
- **Query:** `?status=pending|reviewed|resolved`
- **Response:**
```json
{
  "queue": [...],
  "count": "i64",
  "pending_total": "i64"
}
```

### POST /admin/moderation/queue/{item_id} — Resolver item

- **Auth:** Admin
- **Request:** `{ "status?": "string" }` (default: "reviewed")
- **Response 200:** `{ "message": "...", "item_id": "i64", "status": "string" }`

### GET /admin/moderation/reports — Listar reportes

- **Auth:** Admin
- **Query:** `?status=`
- **Response:** `{ "reports": [...], "count": "i64" }`

### POST /admin/moderation/report/{report_id} — Resolver reporte

- **Auth:** Admin
- **Request:**
```json
{
  "status?": "string",
  "action_user_id?": "i64 (opcional, banea al usuario)"
}
```
- **Response 200:** `{ "message": "...", "report_id": "i64", "status": "string" }`

### GET /admin/moderation/flags — Listar flags

- **Auth:** Admin
- **Response:** `{ "flags": [...], "count": "i64" }`

### POST /admin/moderation/flag/{flag_id} — Resolver flag

- **Auth:** Admin
- **Request:** `{ "status?": "string" }`
- **Response 200:** `{ "message": "...", "flag_id": "i64", "status": "string" }`

### GET /admin/moderation/appeals — Listar apelaciones

- **Auth:** Admin
- **Response:** `{ "appeals": [...], "count": "i64" }`

### POST /admin/moderation/appeal/{appeal_id} — Resolver apelación

- **Auth:** Admin
- **Request:**
```json
{
  "approved": "bool",
  "notes?": "string"
}
```
- **Response 200:** `{ "message": "...", "appeal_id": "i64" }`

### POST /admin/user/{user_id}/shadow-ban — Shadow ban

- **Auth:** Admin
- **Request:**
```json
{
  "reason?": "string",
  "duration_secs?": "i64 (default: 86400 = 24h)"
}
```
- **Response 200:**
```json
{
  "message": "...",
  "user_id": "i64",
  "duration_secs": "i64"
}
```

### POST /admin/user/{user_id}/unshadow-ban — Quitar shadow ban

- **Auth:** Admin
- **Response 200:** `{ "message": "...", "user_id": "i64", "was_banned": "bool" }`

### GET /admin/shadow-bans — Listar shadow bans

- **Auth:** Admin
- **Response:** `{ "shadow_bans": [...], "count": "i64" }`

### POST /admin/user/{user_id}/badge — Asignar badge

- **Auth:** Admin
- **Request:** `{ "badge_type": "string" }`
- **Response 200:**
```json
{
  "message": "...",
  "badge_id": "i64",
  "user_id": "i64",
  "badge_type": "string"
}
```

### DELETE /admin/user/{user_id}/badge/{badge_type} — Revocar badge

- **Auth:** Admin
- **Response 200:**
```json
{
  "message": "...",
  "user_id": "i64",
  "badge_type": "string",
  "was_revoked": "bool"
}
```

### GET /admin/moderation/stats — Estadísticas

- **Auth:** Admin
- **Response:**
```json
{
  "pending_queue": "i64",
  "open_appeals": "i64",
  "pending_reports": "i64",
  "pending_flags": "i64",
  "active_shadow_bans": "i64"
}
```

---

## Trust Score

```toml
[trust]
starting_score = 60.0
report_penalty = 8.0
flag_penalty = 5.0
shadow_ban_penalty = 25.0
ban_penalty = 40.0
badge_bonus = 10.0
account_age_bonus_max = 15.0
```

**Cálculo:** score base ± penalties/bonuses, clampado en [0, 100].

---

## Auto-Moderation Config

```toml
[moderation]
auto_moderation_enabled = true
auto_moderate_moments = true
auto_moderate_chat = true
auto_flag_threshold = 0.30
auto_shadow_ban_after_reports = 5
shadow_ban_duration_secs = 86400
reports_to_action_threshold = 3
```

---

## State Machine

### Report
```
[pending] → [reviewed] → [resolved|dismissed]
```

### Content Flag
```
[pending] → [reviewed] → [resolved|dismissed]
```

### Appeal
```
[open] → [approved|denied]
```

### Shadow Ban
```
[active] → [expired|manually_removed]
```

---

## Dependencies

- **AI Engine:** auto-moderation de texto
- **Wallet:** freeze en fraude
- **WS:** notificaciones de acciones de moderación
