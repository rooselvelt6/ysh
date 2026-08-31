# YSH — Spec-Driven Development (SDD)

## Filosofía

> "El código es un detalle de implementación de la spec. Cuando la spec cambia, el código le sigue. Cuando el código se desvía de la spec, el código está mal."

YSH采用 **Spec-Anchored SDD**: la especificación vive junto al código, ambos evolucionan juntos, y ambos son revisados por el equipo.

## Workflow

```
1. Actualizar spec (o crear una nueva)
2. Revisar la spec con el equipo / AI agent
3. Implementar contra la spec
4. Validar que el código cumple la spec
5. Mantener spec sincronizada con cambios
```

## Convenciones de Naming

| Elemento | Convención |
|----------|-----------|
| Archivo de spec | `{domain}.spec.md` |
| Endpoint ID | `METHOD /path` |
| Schema | `NombreSchema` (PascalCase) |
| Campo | `snake_case` |
| Enum variant | `"value"` (string literal) |
| Codigo de error | `HTTP status` |

## Estructura de una Spec

Cada archivo `.spec.md` sigue esta estructura:

```markdown
# {Nombre del Dominio}

## Overview
Descripción del dominio y su propósito.

## Endpoints
### METHOD /path — Descripción
- **Auth**: requerida / no requerida / admin
- **Request**: schema JSON
- **Response**: schema JSON
- **Errores**: códigos posibles
- **Reglas de negocio**: lógica específica

## Data Models
Schemas de entidades persistentes.

## State Machine
Transiciones de estado (si aplica).

## Dependencies
Dependencias con otros módulos.
```

## Formato de Request/Response

```json
{
  "campo_tipo": "description",
  "campo_opcional?": "description"
}
```

- Campos con `?` son opcionales
- Strings entre comillas indican valores fijos
- `|` indica unión de tipos
- Arrays: `Tipo[]`

## Índice de Specs

### Core
| Spec | Archivo | Estado |
|------|---------|--------|
| Autenticación | [core/auth.spec.md](core/auth.spec.md) | ✅ Extracción completa |
| API General | [core/api.spec.md](core/api.spec.md) | ✅ Extracción completa |
| WebSocket | [core/websocket.spec.md](core/websocket.spec.md) | ✅ Extracción completa |

### Features
| Spec | Archivo | Estado |
|------|---------|--------|
| Moments (Feed) | [features/moments.spec.md](features/moments.spec.md) | ✅ Extracción completa |
| Wallet | [features/wallet.spec.md](features/wallet.spec.md) | ✅ Extracción completa |
| Staking | [features/staking.spec.md](features/staking.spec.md) | ✅ Extracción completa |
| WebRTC/Calls | [features/webrtc.spec.md](features/webrtc.spec.md) | ✅ Extracción completa |
| Chat | [features/chat.spec.md](features/chat.spec.md) | ✅ Extracción completa |
| Moderación | [features/moderation.spec.md](features/moderation.spec.md) | ✅ Extracción completa |
| Notificaciones | [features/notifications.spec.md](features/notifications.spec.md) | ✅ Extracción completa |
| Gifts | [features/gifts.spec.md](features/gifts.spec.md) | ✅ Extracción completa |
| Social | [features/social.spec.md](features/social.spec.md) | ✅ Extracción completa |
| Profile | [features/profile.spec.md](features/profile.spec.md) | ✅ Extracción completa |
| Agency | [features/agency.spec.md](features/agency.spec.md) | ✅ Extracción completa |
| Host | [features/host.spec.md](features/host.spec.md) | ✅ Extracción completa |
| Admin | [features/admin.spec.md](features/admin.spec.md) | ✅ Extracción completa |
| AI Engine | [features/ai.spec.md](features/ai.spec.md) | ✅ Extracción completa |
| i18n | [features/i18n.spec.md](features/i18n.spec.md) | ✅ Extracción completa |
| Payouts | [features/payouts.spec.md](features/payouts.spec.md) | ✅ Extracción completa |
| Receipts | [features/receipts.spec.md](features/receipts.spec.md) | ✅ Extracción completa |
| Analytics | [features/analytics.spec.md](features/analytics.spec.md) | ✅ Extracción completa |
| Jobs | [features/jobs.spec.md](features/jobs.spec.md) | ✅ Extracción completa |
| Gifts | [features/gifts.spec.md](features/gifts.spec.md) | ✅ Extracción completa |
| Social | [features/social.spec.md](features/social.spec.md) | ✅ Extracción completa |
| Profile | [features/profile.spec.md](features/profile.spec.md) | ✅ Extracción completa |
| Agency | [features/agency.spec.md](features/agency.spec.md) | ✅ Extracción completa |
| Host | [features/host.spec.md](features/host.spec.md) | ✅ Extracción completa |
| Admin | [features/admin.spec.md](features/admin.spec.md) | ✅ Extracción completa |
| AI Engine | [features/ai.spec.md](features/ai.spec.md) | ✅ Extracción completa |
| i18n | [features/i18n.spec.md](features/i18n.spec.md) | ✅ Extracción completa |
| Payouts | [features/payouts.spec.md](features/payouts.spec.md) | ✅ Extracción completa |
| Receipts | [features/receipts.spec.md](features/receipts.spec.md) | ✅ Extracción completa |
| Analytics | [features/analytics.spec.md](features/analytics.spec.md) | ✅ Extracción completa |
| Jobs | [features/jobs.spec.md](features/jobs.spec.md) | ✅ Extracción completa |

### Infraestructura
| Spec | Archivo | Estado |
|------|---------|--------|
| Seguridad | [infra/security.spec.md](infra/security.spec.md) | ✅ Extracción completa |
| Observabilidad | [infra/observability.spec.md](infra/observability.spec.md) | ✅ Extracción completa |
| Deployment | [infra/deployment.spec.md](infra/deployment.spec.md) | ✅ Extracción completa |

### Frontend
| Spec | Archivo | Estado |
|------|---------|--------|
| Routing | [frontend/routing.spec.md](frontend/routing.spec.md) | ✅ Extracción completa |
| Componentes | [frontend/components.spec.md](frontend/components.spec.md) | ✅ Extracción completa |

## Reglas de Mantenimiento

1. **Spec primero**: cualquier feature nueva se especifica antes de implementar
2. **Sincronización**: al cambiar un endpoint, actualizar la spec en el mismo commit
3. **Revisión**: los PRs deben incluir cambios en spec si afectan contratos
4. **Versionado**: usar git history para trackear evolución de specs
5. **AI agents**: las specs son la fuente de verdad para generación de código asistida
