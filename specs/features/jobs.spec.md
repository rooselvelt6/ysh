# Jobs — Tareas en Background

## Overview

Sistema de jobs programados para procesamiento batch: payouts, staking rewards, moderación, cleanup, notificaciones, y analytics.

**Base path:** `/api/v1`
**Auth:** Admin

---

## Jobs Disponibles

| Job | Descripción | Intervalo |
|-----|------------|-----------|
| `payouts` | Procesar payouts pendientes | 60s |
| `staking` | Calcular rewards de staking | 60s |
| `moderation` | Auto-resolver items antiguos | 60s |
| `cleanup` | Limpiar datos expirados | 60s |
| `notifications` | Enviar notificaciones batch | 60s |
| `analytics` | Calcular métricas y snapshots | 60s |

---

## Endpoints

### POST /admin/jobs/run/{job} — Ejecutar job manualmente

- **Auth:** Admin
- **Path params:** job ∈ {payouts, staking, moderation, cleanup, notifications, analytics}
- **Response 200:**
```json
{
  "job": "string",
  "triggered": true
}
```

---

### GET /admin/jobs/stats — Estadísticas de jobs

- **Auth:** Admin
- **Response 200:**
```json
{
  "jobs": "object (contadores + últimos resultados)"
}
```

---

## Configuración

```toml
[jobs]
enabled = true
interval_secs = 60
payouts = true
staking = true
moderation = true
cleanup = true
notifications = true
analytics = true
moderation_auto_resolve_secs = 604800   # 7 días
moderation_dismiss_below = 0.4
moderation_action_above = 0.8
analytics_retention_days = 30
quality_retention_days = 7
```

---

## Moderation Auto-Resolve

El job de moderación automáticamente:
- Resuelve items con severity < 0.4 después de 7 días
- Toma acción en items con severity > 0.8
- Retiene flags y reports por `analytics_retention_days`

---

## Dependencies

- **Wallet:** procesamiento de payouts
- **Staking:** cálculo de rewards
- **Moderation:** auto-resolución de items
- **Analytics:** cálculo de métricas
- **Notifications:** envío batch
