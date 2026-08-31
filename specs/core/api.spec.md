# API — Infraestructura HTTP General

## Overview

Capa HTTP construida con Axum. Define middleware stack, manejo de errores, formato de respuesta, y endpoints de infraestructura.

**Base path:** `/api/v1`
**Content-Type:** `application/json` (excepto `/metrics`)

---

## Middleware Stack (orden de aplicación)

1. **Metrics** — colecta de métricas por endpoint
2. **Circuit Breaker** — protección contra cascada de fallos
3. **Per-IP Rate Limit** — rate limiting por IP (governor)
4. **DDoS Protection** — protección contra abuso
5. **Body Limit** — máximo 1 MB por request
6. **Timeout** — 30 segundos máximo por request
7. **CORS** — configuração desde config
8. **Security Headers** — headers de seguridad

---

## Endpoints de Infraestructura

### GET /healthz — Liveness probe

- **Auth:** No requerida
- **Response 200:**
```json
{
  "status": "ok",
  "version": "string"
}
```

### GET /readyz — Readiness probe

- **Auth:** No requerida
- **Response 200:**
```json
{
  "status": "ok",
  "database": "ok|error",
  "cache": "ok|error",
  "session_store": "ok|error",
  "rate_limiter": "ok|error",
  "security": "ok|error",
  "version": "string"
}
```

### GET /metrics — Prometheus metrics

- **Auth:** No requerida
- **Response:** `text/plain` formato Prometheus
- **Puerto:** expuesto también en puerto dedicado (9091)

### GET /config — Configuración pública del servidor

- **Auth:** No requerida
- **Response 200:**
```json
{
  "server": { "host": "string", "port": "u16" },
  "database": { "url": "string" },
  "encryption": { "algorithm": "string" }
}
```

---

## Formato de Error Estándar

Todos los errores retornan JSON:

```json
{
  "error": "string (mensaje descriptivo)"
}
```

### Códigos de Error

| HTTP Status | Significado | Uso típico |
|-------------|------------|------------|
| `400` | Bad Request | Validación de campos, request malformado |
| `401` | Unauthorized | Token inválido/expirado, credenciales incorrectas |
| `403` | Forbidden | Wallet congelada, cuenta baneada, sin permisos admin |
| `404` | Not Found | Recurso no existe |
| `409` | Conflict | Username/email duplicado, miembro ya existe en agency |
| `429` | Too Many Requests | Rate limit excedido, account locked (5 intentos fallidos) |
| `500` | Internal Server Error | Error inesperado del servidor |
| `503` | Service Unavailable | Circuit breaker abierto |

---

## SPA Fallback

Rutas que no son `/api/v1/*` ni `/ws` ni `/healthz` ni `/readyz` ni `/metrics` sirven `index.html` con status 200 para soporte de SPA routing.

**Excepciones que retornan 404 JSON:**
- Rutas `/api/v1/*` inexistentes
- Rutas `/ws` inexistentes
- Archivos estáticos no encontrados

---

## CORS

```toml
allowed_origins = ["*"]  # restringir en producción
allowed_methods = ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
max_age_secs = 3600
```

---

## Rate Limiting (DDoS Protection)

| Categoría | Límite |
|-----------|--------|
| Auth (login, register, 2FA) | 5 req/min |
| API general | 60 req/min |
| WebSocket messages | 30 msg/min |
| Admin endpoints | 120 req/min |

### IP Auto-Ban

| Parámetro | Valor |
|-----------|-------|
| Threshold | 100 errores en ventana |
| Ventana | 60 segundos |
| Duración ban | 300 segundos (5 min) |
| Max blocklist | 10,000 IPs |

---

## Configuración del Server

```toml
[server]
host = "0.0.0.0"
port = 8080          # configurable via YSH_PORT
workers = 4
shutdown_timeout_secs = 30
static_dir = "./frontend/dist"
```

---

## Dependencies

- **tower + tower-http:** middleware framework
- **governor:** rate limiting
- **dashmap:** concurrent hash map para caches
- **axum:** HTTP framework
