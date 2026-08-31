# Gifts — Sistema de Regalos

## Overview

Catálogo de regalos virtuales con envío, NFT minting para rarezas legendarias/épicas, y estadísticas.

**Base path:** `/api/v1`
**Auth:** Requerida

---

## Data Models

### Gift
```json
{
  "id": "i64",
  "name": "string",
  "price": "f64",
  "rarity": "string (common|rare|epic|legendary)",
  "emoji": "string"
}
```

### GiftRecord
```json
{
  "gift_record_id": "i64",
  "gift_id": "i64",
  "gift_name": "string",
  "from_user_id": "i64",
  "from_username": "string",
  "to_user_id": "i64",
  "rarity": "string",
  "created_at": "string (ISO 8601)"
}
```

---

## Endpoints

### GET /gifts/catalog — Ver catálogo

- **Auth:** No requerida (público)
- **Response 200:**
```json
{
  "gifts": ["Gift[]"],
  "count": "i64"
}
```

---

### POST /gifts/send/{user_id} — Enviar regalo

- **Auth:** Requerida
- **Request:**
```json
{
  "gift_id": "i64"
}
```
- **Response 200:**
```json
{
  "message": "Gift sent",
  "gift_record_id": "i64",
  "to_user_id": "i64",
  "gift_id": "i64"
}
```
- **Errores:**
  - `400` — self-gift
  - `403` — wallet congelada
  - `404` — gift o destinatario no existe
- **Reglas de negocio:**
  - No auto-envío
  - Débito del precio del gift del wallet del remitente
  - Si `rarity == "legendary"` o `"epic"` → mintea NFT automáticamente
  - Crea receipt verificable

---

### GET /gifts/received — Regalos recibidos

- **Auth:** Requerida
- **Response 200:**
```json
{
  "gifts": ["GiftRecord[]"],
  "count": "i64"
}
```

---

### GET /gifts/sent — Regalos enviados

- **Auth:** Requerida
- **Response 200:**
```json
{
  "gifts": ["GiftRecord[]"],
  "count": "i64"
}
```

---

### GET /gifts/stats — Estadísticas de regalos

- **Auth:** Requerida
- **Response:** estadísticas agregadas

---

### GET /gifts/nft — Regalos NFT

- **Auth:** Requerida
- **Response 200:**
```json
{
  "nft_gifts": ["GiftRecord[]"],
  "count": "i64"
}
```

---

## NFT Minting

Regalos con `rarity == "legendary"` o `"epic"` se mintean como NFT automáticamente al enviarlos. El NFT se almacena como metadata en la gift record.

---

## Dependencies

- **Wallet:** débito de precio del regalo
- **Receipts:** creación de receipt verificable
