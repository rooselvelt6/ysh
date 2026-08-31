# Chat — Mensajería en Tiempo Real

## Overview

Sistema de chat directo 1:1 con sesiones, mensajes, estado de lectura, y presencia de usuarios online.

**Base path:** `/api/v1`
**Auth:** Requerida

---

## Data Models

### ChatSession
```json
{
  "session_id": "string (UUID)",
  "participants": "i64[]",
  "created_at": "string (ISO 8601)",
  "updated_at": "string (ISO 8601)"
}
```

### ChatMessage
```json
{
  "id": "i64",
  "session_id": "string",
  "from_user_id": "i64",
  "content": "string",
  "read": "bool",
  "created_at": "string (ISO 8601)"
}
```

---

## Endpoints

### GET /chat/sessions — Listar sesiones

- **Auth:** Requerida
- **Response 200:**
```json
{
  "count": "i64",
  "sessions": ["ChatSession[]"]
}
```

---

### POST /chat/session — Crear o obtener sesión

- **Auth:** Requerida
- **Request:**
```json
{
  "user_id": "i64"
}
```
- **Response 200:**
```json
{
  "session_id": "string (UUID)",
  "type": "direct",
  "existing": "bool"
}
```
- **Reglas:** no crear sesión con uno mismo; retorna existente si ya existe

---

### GET /chat/session/{session_id}/messages — Obtener mensajes

- **Auth:** Requerida (solo participantes)
- **Response 200:**
```json
{
  "count": "i64",
  "messages": ["ChatMessage[] (últimos 50)"]
}
```
- **Errores:**
  - `403` — no eres participante
  - `404` — sesión no existe

---

### POST /chat/session/{session_id}/read — Marcar como leído

- **Auth:** Requerida
- **Response 200:**
```json
{
  "count": "i64 (mensajes marcados)"
}
```

---

### GET /chat/unread — Conteo de no leídos

- **Auth:** Requerida
- **Response 200:**
```json
{
  "unread_count": "i64"
}
```

---

### GET /chat/online — Usuarios en línea

- **Auth:** No requerida (público)
- **Response 200:**
```json
{
  "count": "i64",
  "users": ["i64[]"]
}
```
- **Fuente:** lee del ConnectionManager (WS activos)

---

## Real-time (WebSocket)

Los mensajes de chat se envían/receiben via WebSocket:

### Enviar mensaje
```json
{
  "type": "chat_message",
  "session_id": "string",
  "content": "string"
}
```

### Recibir mensaje
```json
{
  "type": "chat_message",
  "session_id": "string",
  "from_user_id": "i64",
  "content": "string",
  "created_at": "string (ISO 8601)"
}
```

---

## Dependencies

- **WebSocket:** Channel de delivery en tiempo real
- **ConnectionManager:** presencia de usuarios online
