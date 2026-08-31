# Profile — Perfil de Usuario

## Overview

Gestión de perfiles de usuario con información personal, búsqueda, y región.

**Base path:** `/api/v1`
**Auth:** Requerida (excepto perfil público)

---

## Data Models

### Profile
```json
{
  "user_id": "i64",
  "username": "string",
  "display_name": "string|null",
  "bio": "string|null",
  "avatar_url": "string|null",
  "country": "string|null"
}
```

---

## Endpoints

### GET /profile — Mi perfil

- **Auth:** Requerida
- **Response 200:**
```json
{
  "user_id": "i64",
  "profile": "Profile",
  "wallet_balance": "f64"
}
```

---

### POST /profile — Actualizar mi perfil

- **Auth:** Requerida
- **Request:**
```json
{
  "display_name?": "string",
  "bio?": "string",
  "avatar_url?": "string",
  "country?": "string"
}
```
- **Response 200:**
```json
{
  "message": "Profile updated",
  "display_name": "string",
  "bio": "string",
  "country": "string"
}
```

---

### GET /profile/{user_id} — Perfil público

- **Auth:** No requerida
- **Response 200:** Profile JSON
- **Errores:** `404` si no existe

---

### GET /users/search — Buscar usuarios

- **Auth:** No requerida
- **Query:** `?q=string&limit=i64`
- **Response 200:**
```json
{
  "users": ["Profile[]"],
  "count": "i64"
}
```
- **Reglas:** `q` es requerido

---

### POST /profile/region/{region} — Establecer región

- **Auth:** Requerida
- **Response 200:**
```json
{
  "region": "string"
}
```
- **Reglas:** longitud 1-64 chars

---

## Dependencies

- **Wallet:** balance en perfil propio
