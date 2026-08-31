# Deployment — Despliegue

## Overview

Configuración de despliegue con Docker, docker-compose, monitoreo, y flujos de CI/CD.

---

## Docker

### Dockerfile

Multi-stage build:
1. **Builder stage:** compila backend Rust + frontend WASM
2. **Runtime stage:** imagen mínima con binario + frontend dist

### Variables de Entorno Requeridas

| Variable | Descripción |
|----------|------------|
| `YSH_JWT_SECRET` | Secret para firmar JWT |
| `YSH_DB_PASSWORD` | Password de la DB |
| `YSH_ENCRYPTION_KEY` | Key para cifrado AEAD |
| `YSH_TLS_CERT` | Path al certificado TLS |
| `YSH_TLS_KEY` | Path a la key TLS |
| `YSH_PORT` | Puerto del servidor (default: 8080) |
| `YSH_DATABASE_URL` | URL de la DB (default: sqlite://ysh.db) |
| `YSH_LOG_JSON` | 1 para formato JSON |

---

## Docker Compose

```yaml
services:
  ysh:
    build: .
    ports:
      - "8080:8080"
      - "9091:9091"  # metrics
    env_file: .env
    volumes:
      - ./data:/app/data
      - ./config:/app/config

  # Profile: monitoring
  prometheus:
    image: prom/prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./deploy/prometheus.yml:/etc/prometheus/prometheus.yml

  grafana:
    image: grafana/grafana
    ports:
      - "3000:3000"
```

### Profiles

| Profile | Servicios |
|---------|----------|
| default | ysh |
| monitoring | ysh + prometheus + grafana |

---

## SPA Frontend

### Build

```bash
cd frontend
wasm-pack build --target web --out-dir pkg
cp -r pkg/* dist/pkg/
```

### Serving

- Rutas `/api/v1/*` → backend Axum
- Rutas `/ws` → WebSocket upgrade
- Rutas `/healthz`, `/readyz`, `/metrics` → backend
- Todo lo demás → `index.html` (SPA fallback, status 200)

### Service Worker

- **Archivo:** `public/sw.js` → `dist/sw.js`
- **Estrategia:** stale-while-revalidate para `/pkg/*`
- **Cache version:** `ysh-v{N}` (incrementar en cada deploy)

---

## Certificados TLS

- **Opción 1:** Certificados manuales (Let's Encrypt)
- **Opción 2:** ACME auto-renewal (futuro)
- **Mínimo:** TLS 1.3

---

## Monitoring Stack

```
YSH Server
  ├── /metrics → Prometheus (scrape cada 15s)
  │                  └── Grafana dashboards
  ├── /healthz → Kubernetes liveness probe
  └── /readyz  → Kubernetes readiness probe
```

---

## Backup

```toml
[backup]
enabled = false
interval_secs = 3600
backup_dir = "./backups"
max_backups = 7
compact_before_backup = true
```

---

## Integrity Check

```toml
[integrity]
check_on_startup = true
auto_repair = true
```

---

## Deployment Commands

```bash
# Build
docker compose build

# Run
docker compose --profile monitoring up -d

# Logs
docker compose logs -f ysh

# Stop
docker compose down
```

---

## Dependencies

- **Docker:** containerización
- **Prometheus:** métricas
- **Grafana:** dashboards
- **Let's Encrypt:** certificados TLS
