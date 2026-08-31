# Auth — Autenticación y Autorización

## Overview

Sistema de autenticación con JWT, 2FA (TOTP), recuperación por código de recovery, roles, GDPR/CCPA compliance, y KYC.

**Base path:** `/api/v1`
**Auth header:** `Authorization: Bearer {access_token}`

---

## Endpoints

### POST /register — Registrar usuario nuevo

- **Auth:** No requerida
- **Request:**
```json
{
  "username": "string (3-32 chars)",
  "email": "string (must contain @)",
  "password": "string (>= 8 chars)"
}
```
- **Response 200:**
```json
{
  "id": "i64",
  "username": "string",
  "email": "string",
  "role": "user"
}
```
- **Errores:**
  - `400` — validación de campos
  - `409` — username o email ya existe
  - `500` — error interno

---

### POST /login — Iniciar sesión

- **Auth:** No requerida
- **Request:**
```json
{
  "username": "string",
  "password": "string"
}
```
- **Response 200 (sin 2FA):**
```json
{
  "access_token": "string (JWT)",
  "refresh_token": "string (JWT)",
  "token_type": "Bearer",
  "expires_in": "i64 (seconds)"
}
```
- **Response 200 (con 2FA habilitado):**
```json
{
  "requires_2fa": true,
  "temp_token": "string (JWT temporal, 5min TTL)"
}
```
- **Errores:**
  - `400` — credenciales inválidas
  - `429` — 5 intentos fallidos → lock 15 minutos
  - `401` — account locked

**Reglas de negocio:**
- 5 intentos fallidos consecutivos → lockout 15 minutos
- Si `totp_enabled` en la cuenta → retorna `temp_token` en vez de tokens completos
- `temp_token` tiene TTL de 5 minutos, solo sirve para `/login/2fa`

---

### POST /login/2fa — Verificar código 2FA

- **Auth:** Usa `temp_token` (en body, no header)
- **Request:**
```json
{
  "temp_token": "string (JWT temporal)",
  "code": "string (6 dígitos TOTP)"
}
```
- **Response 200:**
```json
{
  "access_token": "string",
  "refresh_token": "string",
  "token_type": "Bearer",
  "expires_in": "i64"
}
```
- **Errores:**
  - `400` — temp_token inválido o expirado
  - `401` — código TOTP incorrecto

---

### POST /refresh — Renovar access token

- **Auth:** No requerida (usa refresh_token en body)
- **Request:**
```json
{
  "refresh_token": "string (JWT refresh)"
}
```
- **Response 200:**
```json
{
  "access_token": "string (nuevo JWT)",
  "token_type": "Bearer",
  "expires_in": "i64"
}
```
- **Errores:**
  - `401` — refresh_token inválido o expirado

---

### GET /me — Información del usuario actual

- **Auth:** Requerida
- **Response 200:**
```json
{
  "user_id": "i64",
  "role": "string (user|admin|moderator|host)",
  "username": "string"
}
```
- **Errores:**
  - `401` — token inválido

---

## 2FA (Two-Factor Authentication)

### POST /2fa/setup — Configurar 2FA

- **Auth:** Requerida
- **Request:** body vacío
- **Response 200:**
```json
{
  "secret": "string (TOTP secret, 32 chars)",
  "uri": "string (otpauth:// URI para QR)",
  "recovery_codes": ["string × 10"]
}
```
- **Reglas:** genera 10 códigos de recovery, almacena hasheados

### POST /2fa/verify — Activar 2FA

- **Auth:** Requerida
- **Request:**
```json
{
  "code": "string (6 dígitos)"
}
```
- **Response 200:** `{ "message": "2FA enabled" }`

### POST /2fa/disable — Desactivar 2FA

- **Auth:** Requerida
- **Request:**
```json
{
  "code": "string (6 dígitos para confirmar)"
}
```
- **Response 200:** `{ "message": "2FA disabled" }`
- **Reglas:** requiere código válido para confirmar, elimina recovery codes

### GET /2fa/recovery-codes — Ver estado de recovery codes

- **Auth:** Requerida
- **Response 200:**
```json
{
  "total": "i64",
  "used": "i64",
  "remaining": "i64"
}
```

### POST /2fa/recovery-codes/regenerate — Regenerar recovery codes

- **Auth:** Requerida
- **Response 200:**
```json
{
  "recovery_codes": ["string × 10"]
}
```

### POST /2fa/recovery/verify — Login con recovery code

- **Auth:** No requerida
- **Request:**
```json
{
  "username": "string",
  "code": "string (recovery code)"
}
```
- **Response 200:**
```json
{
  "access_token": "string",
  "refresh_token": "string",
  "token_type": "Bearer",
  "expires_in": "i64"
}
```

---

## GDPR

### GET /gdpr/export — Exportar datos del usuario

- **Auth:** Requerida
- **Response 200:** dump completo de datos del usuario

### POST /gdpr/delete — Eliminar cuenta

- **Auth:** Requerida
- **Request:**
```json
{
  "password": "string (confirmación)"
}
```
- **Response 200:** `{ "message": "Account deleted" }`

### POST /gdpr/consent — Registrar consentimiento

- **Auth:** Requerida
- **Request:**
```json
{
  "consent_type": "string",
  "granted": "bool"
}
```
- **Response 200:**
```json
{
  "message": "Consent recorded",
  "consent_type": "string",
  "granted": "bool"
}
```

### GET /gdpr/consent/history — Historial de consentimientos

- **Auth:** Requerida
- **Response 200:**
```json
{
  "consent_records": "array"
}
```

---

## CCPA

### GET /ccpa/do-not-sell

- **Auth:** Requerida
- **Response 200:** `{ "do_not_sell": "bool" }`

### POST /ccpa/do-not-sell

- **Auth:** Requerida
- **Request:** `{ "do_not_sell": "bool" }`
- **Response 200:** `{ "do_not_sell": "bool" }`

---

## KYC (Know Your Customer)

### GET /kyc/status

- **Auth:** Requerida
- **Response 200:**
```json
{
  "kyc_level": "i32 (0-3)",
  "status": "string"
}
```
- **Niveles:** 0=no verificado, 1=email verificado, 2=ID enviado, 3=completo

### POST /kyc/submit

- **Auth:** Requerida
- **Request:**
```json
{
  "level": "i32 (1-3)"
}
```
- **Reglas:** debe incrementar secuencialmente (+1), máximo nivel 3

---

## Crypto (demo)

### POST /encrypt — Cifrar datos

- **Auth:** Requerida
- **Request:** `{ "data": "string" }`
- **Response 200:**
```json
{
  "ciphertext": "string (base64)",
  "nonce": "string (base64)",
  "algorithm": "string"
}
```

### POST /decrypt — Descifrar datos

- **Auth:** Requerida
- **Request:**
```json
{
  "ciphertext": "string (base64)",
  "nonce": "string (base64)"
}
```
- **Response 200:** `{ "plaintext": "string" }`

---

## JWT Token Structure

### Access Token Claims
```json
{
  "sub": "user_id (i64)",
  "exp": "timestamp",
  "iat": "timestamp",
  "role": "string (user|admin|moderator|host)",
  "kind": "access"
}
```

### Refresh Token Claims
```json
{
  "sub": "user_id (i64)",
  "exp": "timestamp",
  "iat": "timestamp",
  "role": "string",
  "kind": "refresh"
}
```

### Temp Token Claims (2FA flow)
```json
{
  "sub": "user_id (i64)",
  "exp": "timestamp (+5min)",
  "iat": "timestamp",
  "kind": "temp_2fa"
}
```

**Configuración:**
- `expiry_hours = 24` para access tokens
- `refresh_expiry_days = 30` para refresh tokens
- Secret desde env var `YSH_JWT_SECRET`

---

## Dependencies

- **security:** JWT signing/verification, Argon2 password hashing
- **wallet:** auto-crear wallet al registrar usuario
- **config:** parámetros de JWT expiry, rate limits
