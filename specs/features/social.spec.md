# Social — Interacciones Sociales

## Overview

Endpoints para interacciones sociales: bloqueo, reportes, ratings, reputación, trust score, badges, content flags, y appeals. Ver también `moderation.spec.md` para la parte admin de estas funcionalidades.

**Base path:** `/api/v1`
**Auth:** Requerida (excepto endpoints públicos de reputación)

---

## User Endpoints

### POST /block — Bloquear usuario

- **Auth:** Requerida
- **Request:** `{ "target_user_id": "i64" }`
- **Response 200:**
```json
{
  "message": "User blocked",
  "blocked_user_id": "i64"
}
```

### DELETE /block/{user_id} — Desbloquear

- **Auth:** Requerida
- **Response 200:** `{ "message": "User unblocked" }`
- **Errores:** `404`

### GET /blocks — Bloqueados

- **Auth:** Requerida
- **Response 200:** `{ "blocked": [...], "count": "i64" }`

---

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
- **Response:**
```json
{
  "user_id": "i64",
  "rating_avg": "f64",
  "rating_count": "i64"
}
```

### GET /reputation/{user_id} — Alias de /rating

- **Auth:** No requerida
- **Response:** igual que /rating/{user_id}

---

### GET /badges — Mis badges

- **Auth:** Requerida
- **Response 200:**
```json
{
  "badges": ["string[]"],
  "count": "i64"
}
```

### GET /badges/{user_id} — Badges de usuario

- **Auth:** No requerida
- **Response 200:**
```json
{
  "user_id": "i64",
  "badges": ["string[]"],
  "count": "i64"
}
```

---

### GET /trust — Mi trust score

- **Auth:** Requerida
- **Response 200:**
```json
{
  "user_id": "i64",
  "score": "f64",
  "level": "string"
}
```

---

## Admin Endpoints

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

---

## Trust Score Config

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

---

## Dependencies

- **Moderation:** shadow bans, reports
- **Admin:** badges management
