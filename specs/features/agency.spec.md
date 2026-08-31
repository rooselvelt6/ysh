# Agency — Agencias

## Overview

Sistema de agencias con creación, membresos, y roles. Los usuarios pueden crear agencias y gestionar miembros.

**Base path:** `/api/v1`
**Auth:** Requerida

---

## Data Models

### Agency
```json
{
  "id": "i64",
  "name": "string",
  "description": "string",
  "owner_id": "i64",
  "member_count": "i64",
  "members": ["object[]"]
}
```

---

## Endpoints

### POST /agency — Crear agencia

- **Auth:** Requerida
- **Request:**
```json
{
  "name": "string",
  "description": "string"
}
```
- **Response 200:**
```json
{
  "id": "i64",
  "name": "string",
  "description": "string"
}
```
- **Reglas:** creador se auto-agrega como `owner`

---

### GET /agencies — Listar agencias

- **Auth:** No requerida (público)
- **Response 200:**
```json
{
  "agencies": ["Agency[]"],
  "count": "i64"
}
```

---

### GET /agency/{agency_id} — Ver agencia

- **Auth:** No requerida
- **Response:** Agency JSON
- **Errores:** `404`

---

### GET /agency/{agency_id}/members — Ver miembros

- **Auth:** No requerida
- **Response 200:**
```json
{
  "members": ["object[]"],
  "count": "i64"
}
```

---

### POST /agency/{agency_id}/members — Agregar miembro

- **Auth:** Requerida (owner de la agencia)
- **Request:**
```json
{
  "user_id": "i64",
  "role?": "string (default: host)"
}
```
- **Response 200:**
```json
{
  "message": "Member added",
  "user_id": "i64",
  "role": "string"
}
```
- **Errores:**
  - `409` — usuario ya es miembro
  - `404` — agencia no existe

---

## Admin Endpoints

### DELETE /admin/agency/{agency_id}/members/{user_id} — Remover miembro

- **Auth:** Admin
- **Response 200:**
```json
{
  "message": "Member removed",
  ...
}
```
- **Errores:**
  - `409` — último miembro no se puede remover

---

## Dependencies

- **Admin:** remoción de miembros
