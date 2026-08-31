# WebSocket — Conexión en Tiempo Real

## Overview

Conexión WebSocket unificada para signaling de WebRTC, chat en tiempo real, y notificaciones push in-app.

**Endpoint:** `GET /ws?token={jwt}`
**Protocol:** WebSocket (ws/wss)
**Auth:** JWT en query parameter `token`

---

## Conexión

### Handshake

```
GET /ws?token={access_token_jwt} HTTP/1.1
Upgrade: websocket
Connection: Upgrade
```

El servidor valida el JWT del query param. Si es inválido → cierra con `1008 Policy Violation`.

---

## Arquitectura

```
Client ←→ ConnectionManager ←→ WsGuard
                                    ├── RoomManager (WebRTC rooms)
                                    ├── ChatManager (chat sessions)
                                    └── NotificationBroadcast
```

- **ConnectionManager:** registry de conexiones activas por user_id
- **WsGuard:** valida estado de conexión y rate limiting
- **RoomManager:** gestiona salas de WebRTC
- **ChatManager:** distribuye mensajes de chat
- **NotificationBroadcast:** envía notificaciones in-app

---

## Mensajes WebSocket

### Formato

Todos los mensajes son JSON con campo `type` para discriminación:

```json
{
  "type": "string",
  ...payload
}
```

### Mensajes Enviados por el Cliente

#### call_invite — Invitar a llamada

```json
{
  "type": "call_invite",
  "target_user_id": "i64",
  "call_type": "string (flash|live|p2p|duo|group)"
}
```

#### ice_candidate — Candidato ICE para WebRTC

```json
{
  "type": "ice_candidate",
  "peer_id": "string",
  "candidate": "string (SDP candidate)",
  "sdp_mid": "string",
  "sdp_m_line_index": "u16"
}
```

#### call_hangup — Colgar llamada

```json
{
  "type": "call_hangup",
  "peer_id": "string"
}
```

#### chat_message — Mensaje de chat

```json
{
  "type": "chat_message",
  "session_id": "string",
  "content": "string"
}
```

### Mensajes Enviados por el Servidor

#### call_invite — Notificación de llamada entrante

```json
{
  "type": "call_invite",
  "from_user_id": "i64",
  "call_type": "string"
}
```

#### ice_candidate — Candidato ICE del peer

```json
{
  "type": "ice_candidate",
  "peer_id": "string",
  "candidate": "string",
  "sdp_mid": "string",
  "sdp_m_line_index": "u16"
}
```

#### call_hangup — Peer colgó

```json
{
  "type": "call_hangup",
  "peer_id": "string"
}
```

#### notification — Notificación in-app

```json
{
  "type": "notification",
  "id": "i64",
  "title": "string",
  "body": "string",
  "ntype": "string"
}
```

#### chat_message — Mensaje de chat entrante

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

## Límites WebSocket

| Parámetro | Valor |
|-----------|-------|
| Max conexiones por usuario | 3 |
| Max tamaño de mensaje | 64 KB |
| Heartbeat timeout | 60 segundos |
| Rate messages | 10 msg/segundo |

---

## Online Users

El `ConnectionManager` mantiene un registro de usuarios conectados. El endpoint `GET /api/v1/chat/online` lee de este registro.

---

## Dependencies

- **tokio-tungstenite:** WebSocket protocol
- **axum:** upgrade handler
- **actors/ConnectionManager:** registry de conexiones
- **auth:** JWT validation para handshake
