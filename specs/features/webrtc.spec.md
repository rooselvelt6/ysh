# WebRTC — Videollamadas y Streaming

## Overview

Sistema de videollamadas con múltiples modos (flash, P2P, duo, group, live), signaling via WebSocket, SFU passthrough, simulcast, grabación, y billing por uso.

**Base path:** `/api/v1`
**Auth:** Requerida

---

## Data Models

### Call
```json
{
  "call_id": "string (UUID)",
  "call_type": "string (flash|live|p2p|duo|group)",
  "host_id": "i64",
  "title": "string|null",
  "participants": "i64[]",
  "viewer_count": "i64",
  "simulcast_tiers": ["q", "h", "f"],
  "created_at": "string (ISO 8601)"
}
```

### QualityReport
```json
{
  "simulcast_tier": "string (q|h|f)",
  "bitrate_kbps": "f64",
  "packet_loss_pct": "f64",
  "rtt_ms": "f64",
  "resolution": "string"
}
```

---

## Call Types

| Tipo | Capacidad | Descripción |
|------|-----------|------------|
| `flash` | random peer | Llamada aleatoria con usuario disponible |
| `p2p` | 2 | 1:1, requiere `target_user_id` |
| `duo` | 3 | Llamada grupal pequeña |
| `group` | 8 | Llamada grupal |
| `live` | 1000 viewers | Streaming 1 a muchos |

---

## Endpoints

### POST /call/start — Iniciar llamada

- **Auth:** Requerida
- **Request:**
```json
{
  "call_type": "string (flash|live|p2p|duo|group)",
  "title?": "string",
  "target_user_id?": "i64 (requerido para p2p)"
}
```
- **Response 200:**
```json
{
  "call_id": "string (UUID)",
  "call_type": "string",
  "host_id": "i64",
  "participants": ["i64"],
  "simulcast_tiers": ["q", "h", "f"]
}
```
- **Errores:**
  - `400` — p2p sin target_user_id
  - `403` — WebRTC deshabilitado en config

---

### POST /call/{call_id}/join — Unirse a llamada

- **Auth:** Requerida
- **Response 200:**
```json
{
  "call_id": "string",
  "mode": "sfu_passthrough",
  "participants": ["i64"],
  "viewer_count": "i64"
}
```
- **Errores:**
  - `404` — room no existe
  - `409` — room lleno o rechazado

---

### POST /call/{call_id}/leave — Salir de llamada

- **Auth:** Requerida
- **Response 200:**
```json
{
  "left": true,
  "room_empty": "bool",
  "participants": ["i64"],
  "viewer_count": "i64"
}
```

---

### POST /call/{call_id}/end — Terminar llamada

- **Auth:** Requerida (solo host)
- **Response 200:**
```json
{
  "ended": true,
  "call_id": "string",
  "participants": ["i64"]
}
```

---

### POST /call/{call_id}/screen-share — Toggle screen share

- **Auth:** Requerida
- **Request:** `{ "active": "bool" }`
- **Response 200:** `{ "screen_share": "bool" }`

---

### POST /call/{call_id}/recording/start — Iniciar grabación

- **Auth:** Requerida
- **Response 200:**
```json
{
  "recording": true,
  "encrypted": true
}
```
- **Reglas:** verifica `recording_enabled` en config

### POST /call/{call_id}/recording/stop — Detener grabación

- **Auth:** Requerida (solo host)
- **Response 200:** `{ "recording": false }`

---

### POST /call/{call_id}/quality — Reportar calidad

- **Auth:** Requerida
- **Request:** QualityReport schema
- **Response 200:** `{ "recorded": true }`

### GET /call/{call_id}/quality — Ver calidad agregada

- **Auth:** No requerida
- **Response:**
```json
{
  "aggregate": "object",
  "last_n": "QualityReport[]"
}
```

---

### GET /call/{call_id} — Info de llamada

- **Auth:** No requerida
- **Response:** Call + quality + recordings

### GET /call/{call_id}/peers — Peer info

- **Auth:** No requerida
- **Response:**
```json
{
  "participants": "i64[]",
  "viewers": "i64",
  "screen_share": "bool",
  "call_type": "string"
}
```

---

### POST /call/{call_id}/title — Actualizar título (live)

- **Auth:** Requerida (solo host)
- **Request:** `{ "title": "string" }`
- **Response:** `{ "title": "string" }`

---

### GET /calls/history — Historial de llamadas

- **Auth:** Requerida
- **Response:** últimas 100 llamadas

### GET /calls/live — Streams en vivo activos

- **Auth:** No requerida
- **Response:** `{ "live": [...] }`

### GET /calls/rooms — Salas activas

- **Auth:** No requerida
- **Response:** `{ "rooms": [...] }`

### GET /calls/stats — Estadísticas de llamadas

- **Auth:** No requerida

### GET /webrtc/stats — Stats del actor WebRTC

- **Auth:** No requerida

---

## Configuración

```toml
[webrtc]
enabled = true
signal_mode = "sfu_passthrough"
p2p_capacity = 2
duo_capacity = 3
group_capacity = 8
max_live_viewers = 1000
cost_per_minute = 30
billing_per_second = true
recording_enabled = true
recording_encryption = true
simulcast_tiers = ["q", "h", "f"]
call_timeout_secs = 30
```

---

## Billing

- Flash/P2P: cobro por minuto de uso
- Débito automático de wallet cada segundo si `billing_per_second = true`
- Host gana 70% del costo de la llamada

---

## Signaling (WebSocket)

El signaling corre por el WebSocket unificado (`/ws`):
- `call_invite` → notificar peer
- `ice_candidate` → intercambio de candidatos ICE
- `call_hangup` → colgar

---

## Dependencies

- **WebSocket:** signaling channel
- **Wallet:** billing por uso
- **Actors:** WebRTC actor para manejo de salas
