# YSH — Deploy & Runbook (FASE 16)

Guía de despliegue, monitorización y operación en producción.

## Índice
- [Requisitos](#requisitos)
- [Despliegue con Docker Compose](#despliegue-con-docker-compose)
- [Variables de entorno](#variables-de-entorno)
- [Monitorización (Prometheus + Grafana)](#monitorización-prometheus--grafana)
- [Métricas expuestas](#métricas-expuestas)
- [TLS / Let's Encrypt](#tls--lets-encrypt)
- [Backups](#backups)
- [Escalado horizontal](#escalado-horizontal)
- [CI/CD](#cicd)
- [Runbook de operación](#runbook-de-operación)

---

## Requisitos

- Docker ≥ 24 + Docker Compose v2
- 1 GB RAM mínimo, 1 vCPU
- (Opcional) dominio + DNS apuntando al host

## Despliegue con Docker Compose

```bash
# 1. Configurar secretos
cp .env.example .env
# abre .env y pon valores reales (openssl rand -hex 32)

# 2. Levantar API + monitorización
docker compose --profile monitoring up -d --build

# 3. Verificar
curl http://localhost:8080/healthz        # → {"status":"ok",...}
curl http://localhost:9091/metrics        # métricas Prometheus
# Grafana: http://localhost:3000  (admin / GRAFANA_ADMIN_PASSWORD)
```

### Perfiles
| Perfil        | Comando                              | Qué incluye            |
|---------------|--------------------------------------|------------------------|
| `dev`         | `docker compose up -d --build`       | YSH API                |
| `monitoring`  | `docker compose --profile monitoring up -d` | YSH + Prometheus + Grafana |
| `prod`        | `docker compose --profile prod up -d` | YSH (single)          |

## Variables de entorno

Todas se gestionan por `env` en `config/default.toml`.

| Variable                | Obligatoria | Descripción                              |
|-------------------------|-------------|------------------------------------------|
| `YSH_JWT_SECRET`        | ✅           | Firma de JWT (≥32 bytes aleatorios)      |
| `YSH_DB_PASSWORD`       | ✅           | Protege la BD en reposo                  |
| `YSH_ENCRYPTION_KEY`    | ✅           | Clave AEAD (32 bytes hex, AES-256-GCM)   |
| `YSH_PORT`              | —           | Puerto HTTP (default 8080)               |
| `YSH_DATABASE_URL`      | —           | `sqlite:///data/ysh.db` en contenedor    |
| `YSH_LOG_JSON`          | —           | `1` = logs JSON estructurados            |
| `RUST_LOG`              | —           | Verbosidad de logging (default `info`)   |

> ⚠️ **Nunca** commitees `.env` ni imprimas estos valores. El CI falla con warns
> y el `.gitignore` ya excluye `.env`.

## Monitorización (Prometheus + Grafana)

El endpoint `/metrics` está en el puerto `9091` (configurable en
`[observability]` de `config/default.toml`).

- **Prometheus** scrapea `ysh:9091` cada 15s → `http://localhost:9090`
- **Grafana** usa el datasource Prometheus y carga automáticamente el dashboard
  `YSH — Overview` (provisioning en `deploy/grafana/`)

### Métricas expuestas

| Métrica                       | Tipo    | Significado                          |
|-------------------------------|---------|--------------------------------------|
| `http_requests_total{code}`   | counter | Peticiones HTTP por familia (1xx-5xx)|
| `http_rate_limited_total`     | counter | Rechazadas por rate-limit por IP     |
| `circuit_breaker_open`        | gauge   | 1 si el breaker está abierto         |
| `ysh_ws_connections_active`   | gauge   | Websockets activos                   |
| `ysh_ws_connections_total`    | counter | Websockets aceptados desde arranque  |
| `ysh_uptime_seconds`          | gauge   | Uptime del proceso                   |
| `ysh_db_size_bytes`           | gauge   | Tamaño de la BD                      |
| `ysh_cache_entries`           | gauge   | Entradas en caché KV                 |
| `ysh_blocked_ips`             | gauge   | IPs bloqueadas por DDoS              |
| `ysh_users_total`             | gauge   | Usuarios registrados                 |

## TLS / Let's Encrypt

El binario soporta TLS nativo (rustls, TLS ≥1.3) vía `YSH_TLS_CERT` / `YSH_TLS_KEY`.
Para producción con dominio:

```bash
# Crea los certificados con certbot (webroot apuntando al static_dir frontend/dist)
certbot certonly --webroot -w frontend/dist -d tu-dominio.com --email admin@tu-dominio.com
# Y luego apunta las variables al fichero .pem

# Renovación automática (cron)
@daily docker run --rm -v /etc/letsencrypt:/etc/letsencrypt certbot/certbot renew --quiet && docker compose restart ysh
```

Para un despliegue simple detrás de un proxy, configura Caddy/Traefik para
terminar TLS y reenviar al contenedor en `:8080`.

## Backups

`config/default.toml` soporta snapshots automáticos en `[backup]`
(`enabled`, `interval_secs`, `backup_dir`, `max_backups`, `compact_before_backup`).

La BD vive en el volumen `ysh-data` (`/data/ysh.db`).

```bash
# Snapshot manual
docker compose exec ysh /app/ysh   # (los snapshots automáticos ya compactan)

# Copia del volumen a S3 (offsite) — ejemplo con rclone
rclone sync /var/lib/docker/volumes/ysh_ysh-data/_data s3:mi-bucket/ysh/backups
# Programa con un cron diario
```

> ⚠️ Con AES-256-GCM en reposo, perder la `YSH_ENCRYPTION_KEY` equivale a perder
> los datos. Almacénala en un gestor de secretos (Vault, SSM, etc.).

## Escalado horizontal

El binario es stateless respecto al frontend; la BD es un fichero SQLite así que
el escalado multi-instancia requiere mover CDB a algo compartido o usar
replicación. Para 2+ instancias:

1. Replica la BD (p. ej. Litestream → S3) o migra a PostgreSQL.
2. Pon un **load balancer** delante (traefik/caddy/nginx):
   - balanceo por IP de origen (sticky) para WebSocket/WebRTC.
   - compresión + cache de estáticos del frontend.
3. Cada instancia expone `/metrics` en `9091`; Prometheus scrapea todas.

## CI/CD

`.github/workflows/ci.yml` ejecuta en cada push/PR:

1. **clippy** — `-D warnings` (0 warnings obligatorio)
2. **rustfmt** — `--check`
3. **test** — dev y release (341 tests)
4. **cargo-audit** — vulnerabilidades de dependencias
5. **build+push** (solo push a master) — imagen a GHCR

## Runbook de operación

### 1. Health checks
| Endpoint  | Código | Uso              |
|-----------|--------|------------------|
| `/healthz` | 200    | Liveness (el proceso está vivo) |
| `/readyz`  | 200    | Readiness (listo para tráfico)  |

### 2. Incidentes comunes

| Síntoma | Diagnóstico | Acción |
|---------|-------------|--------|
| 429 masivos | Rate-limit por IP (`http_rate_limited_total`) | identifica IP abusiva; si es legítimo, sube `[ddos.rate_limit]` |
| 503 "circuit open" | Degradación de un backend | `circuit_breaker_open=1`; revisa log, espera cooldown |
| `/metrics` vacío/503 | `metrics_enabled=false` o versión 0.22 de `metrics` | comprueba `[observability]` y `cargo tree -i metrics` |
| Logs ilegibles | formato | `YSH_LOG_JSON=1` para JSON a la pila ELK/Loki |
| 0 usuarios | BD no arrancó | verifica `YSH_DATABASE_URL` y permisos del volumen |

### 3. Procedimientos
- **Ver liveness en prod:** `curl -f http://localhost:8080/healthz`
- **Ver métricas:** `curl http://localhost:9091/metrics`
- **Reinicio limpio:** `docker compose --profile monitoring restart ysh`
- **Logs:** `docker compose logs -f ysh`
- **Rollback imagen:** `docker compose up -d ysh:<tag-anterior>`
