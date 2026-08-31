# Observability — Observabilidad

## Overview

Sistema de observabilidad con métricas Prometheus, structured logging (tracing), health checks, y monitoreo de recursos del sistema.

---

## Metrics

### Exporter

- **Crate:** `metrics-exporter-prometheus` v0.16
- **Endpoint:** `GET /metrics` (formato Prometheus text)
- **Puerto dedicado:** 9091 (configurable)
- **Bind:** `0.0.0.0` (restringir a `127.0.0.1` si scraper local)

### Métricas Colectadas

| Métrica | Tipo | Descripción |
|---------|------|------------|
| `http_requests_total` | Counter | Requests HTTP por endpoint/method/status |
| `http_request_duration_seconds` | Histogram | Duración de requests |
| `ws_connections_active` | Gauge | Conexiones WebSocket activas |
| `db_query_duration_seconds` | Histogram | Duración de queries DB |
| `cache_hits_total` | Counter | Cache hits |
| `cache_misses_total` | Counter | Cache misses |
| `active_users` | Gauge | Usuarios activos |
| `wallet_balance_total` | Gauge | Balance total del sistema |
| `staking_locked_total` | Gauge | Tokens en staking |
| `moderation_queue_size` | Gauge | Items en cola de moderación |
| `job_runs_total` | Counter | Ejecuciones de jobs |
| `job_duration_seconds` | Histogram | Duración de jobs |
| `circuit_breaker_state` | Gauge | Estado del circuit breaker (0=closed, 1=open) |

### Snapshot Interval

```toml
snapshot_interval_secs = 15
```

---

## Logging

### Framework

- **Crate:** `tracing` v0.1 + `tracing-subscriber` v0.3
- **Features:** `env-filter`, `json`

### Configuration

| Variable | Efecto |
|----------|--------|
| `RUST_LOG` | Filtro de verbosidad (ej: `info,ysh=debug`) |
| `YSH_LOG_JSON=1` | Formato JSON (producción) |
| Default | Formato texto con colores (desarrollo) |

### Reglas

- Secrets se redactan en logs (field redaction)
- Nunca logear tokens, passwords, o keys
- Structured logging para ingestion por Datadog/Grafana

---

## Health Checks

### GET /healthz — Liveness

Verifica que el proceso está vivo. No valida dependencias.

### GET /readyz — Readiness

Verifica todas las dependencias:
- Database connection
- Cache (sled)
- Session store
- Rate limiter caches
- Security subsystem

---

## System Health (Admin)

### GET /admin/analytics/health

Lee de `/proc/*` para métricas del sistema:
- Uptime
- Memory usage
- CPU usage %
- DB size
- Thread count
- Cache stats

---

## Configuración

```toml
[observability]
metrics_enabled = true
metrics_host = "0.0.0.0"
metrics_port = 9091
snapshot_interval_secs = 15
```

---

## Dependencies

- **tracing + tracing-subscriber:** structured logging
- **metrics + metrics-exporter-prometheus:** métricas
- **axum:** health check endpoints
