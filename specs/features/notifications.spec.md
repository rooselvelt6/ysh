# Notificaciones — Sistema de Notificaciones

## Overview

Sistema de notificaciones in-app, email, y push con preferencias de usuario, quiet hours, y tokens de device.

**Base path:** `/api/v1`
**Auth:** Requerida

---

## Data Models

### Notification
```json
{
  "id": "i64",
  "user_id": "i64",
  "title": "string",
  "body": "string",
  "ntype": "string (gift|call|message|moderation|system|staking|payout)",
  "read": "bool",
  "created_at": "string (ISO 8601)"
}
```

### NotificationPreferences
```json
{
  "email_enabled": "bool",
  "push_enabled": "bool",
  "in_app_enabled": "bool",
  "email_gifts": "bool",
  "email_calls": "bool",
  "email_messages": "bool",
  "push_gifts": "bool",
  "push_calls": "bool",
  "push_messages": "bool"
}
```

---

## Endpoints

### GET /notifications — Listar notificaciones

- **Auth:** Requerida
- **Query:** `?limit=50` (default)
- **Response 200:**
```json
{
  "notifications": ["Notification[]"],
  "count": "i64",
  "unread_count": "i64"
}
```

---

### POST /notification/{notification_id}/read — Marcar como leída

- **Auth:** Requerida
- **Response 200:** `{ "message": "Notification marked as read" }`
- **Errores:** `404` si no existe

---

### POST /notifications/read-all — Marcar todas como leídas

- **Auth:** Requerida
- **Response 200:**
```json
{
  "message": "All notifications marked as read",
  "count": "i64"
}
```

---

### GET /notifications/preferences — Ver preferencias

- **Auth:** Requerida
- **Response:** NotificationPreferences

---

### POST /notifications/preferences — Actualizar preferencia

- **Auth:** Requerida
- **Request:**
```json
{
  "email_enabled?": "bool",
  "push_enabled?": "bool",
  "in_app_enabled?": "bool",
  "email_gifts?": "bool",
  "email_calls?": "bool",
  "email_messages?": "bool",
  "push_gifts?": "bool",
  "push_calls?": "bool",
  "push_messages?": "bool"
}
```
- **Reglas:** solo campos whitelistados booleanos
- **Response 200:** `{ "message": "Preferences updated" }`

---

### POST /notifications/quiet-hours — Configurar quiet hours

- **Auth:** Requerida
- **Request:**
```json
{
  "start": "string (HH:MM)",
  "end": "string (HH:MM)"
}
```
- **Response 200:**
```json
{
  "message": "Quiet hours updated",
  "quiet_hours_start": "string",
  "quiet_hours_end": "string"
}
```
- **Defaults:** 22:00 - 08:00

---

### POST /notifications/push/register — Registrar token push

- **Auth:** Requerida
- **Request:**
```json
{
  "token": "string",
  "platform?": "string (ios|android|web)"
}
```
- **Response 200:**
```json
{
  "message": "Push token registered",
  "platform": "string"
}
```

---

### POST /notifications/push/remove — Eliminar token push

- **Auth:** Requerida
- **Request:** `{ "token": "string" }`
- **Response 200:** `{ "message": "Push token removed" }`
- **Errores:** `404`

---

### GET /notifications/push/tokens — Ver tokens registrados

- **Auth:** Requerida
- **Response:**
```json
{
  "tokens": ["string[]"],
  "count": "i64"
}
```

---

### POST /notifications/test — Enviar notificación de prueba

- **Auth:** Requerida
- **Response 200:**
```json
{
  "message": "Test notification sent",
  "notification_id": "i64"
}
```
- **Reglas:** crea notificación in-app de prueba

---

## Delivery

Las notificaciones se entregan por 3 canales:
1. **In-app:** via WebSocket (`type: "notification"`)
2. **Email:** via `lettre` SMTP (con pool de conexiones)
3. **Push:** via tokens FCM/APNs (si configurado)

Las preferencias del usuario controlan qué canales activos.
Quiet hours silencian email y push (in-app siempre visible).

---

## Dependencies

- **WebSocket:** delivery in-app
- **Jobs:** background job para notificaciones batch
- **Config:** intervalos y configuración de entrega
