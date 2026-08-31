# Moments — Feed Social

## Overview

Sistema de publicaciones (posts/moments) con feed, likes, comentarios, y moderación automática por AI.

**Base path:** `/api/v1`
**Auth:** Requerida (excepto comentarios que son públicos)

---

## Data Models

### Moment
```json
{
  "id": "i64",
  "user_id": "i64",
  "username": "string",
  "content": "string",
  "media_url": "string|null",
  "media_type": "string|null (image|video)",
  "likes": "i64",
  "comments": "i64",
  "liked": "bool",
  "created_at": "string (ISO 8601)"
}
```

### Comment
```json
{
  "id": "i64",
  "user_id": "i64",
  "username": "string",
  "content": "string",
  "created_at": "string (ISO 8601)"
}
```

---

## Endpoints

### POST /moment — Crear moment

- **Auth:** Requerida
- **Request:**
```json
{
  "content": "string",
  "media_url?": "string",
  "media_type?": "string (image|video)"
}
```
- **Response 200:**
```json
{
  "id": "i64",
  "message": "Moment created"
}
```
- **Errores:**
  - `400` — content vacío
  - `403` — contenido bloqueado por moderación AI
- **Reglas de negocio:**
  - Moderación automática al crear:
    - AI `moderate_text` → si `decision == "block"` → 403 + content flagged
    - AI `moderate_text` → si `decision == "flag"` → auto-flag for review (no bloquea)

---

### GET /moments — Obtener feed

- **Auth:** Requerida
- **Query params:**
  - `offset` (default: 0)
  - `limit` (default: 20)
- **Response 200:**
```json
{
  "moments": ["Moment[]"],
  "count": "i64"
}
```

---

### POST /moment/{moment_id}/like — Dar like

- **Auth:** Requerida
- **Response 200:**
```json
{
  "message": "Moment liked"
}
```

---

### POST /moment/{moment_id}/unlike — Quitar like

- **Auth:** Requerida
- **Response 200:**
```json
{
  "message": "Like removed"
}
```

---

### POST /moment/{moment_id}/comment — Comentar

- **Auth:** Requerida
- **Request:**
```json
{
  "content": "string (requerido)"
}
```
- **Response 200:**
```json
{
  "id": "i64",
  "message": "Comment added"
}
```

---

### GET /moment/{moment_id}/comments — Obtener comentarios

- **Auth:** No requerida (público)
- **Response 200:**
```json
{
  "comments": ["Comment[]"],
  "count": "i64"
}
```

---

### DELETE /moment/{moment_id} — Eliminar moment

- **Auth:** Requerida (solo owner)
- **Response 200:**
```json
{
  "message": "Moment deleted"
}
```
- **Errores:**
  - `404` — moment no existe
  - `403` — no eres el owner

---

## State Machine

```
[created] → AI moderation
  ├── approved → visible in feed
  ├── flagged → visible but queued for review
  └── blocked → 403, not created
```

---

## Dependencies

- **AI Engine:** `POST /ai/moderation/text` para auto-moderación
- **Moderation:** flagged moments van al moderation queue
- **Social:** likes y reports
