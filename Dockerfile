# ═══════════════════════════════════════════
# YSH — Dockerfile multi-stage
# FASE 16: Deploy + Monitoring
# ═══════════════════════════════════════════
# Build:    docker build -t ysh:latest .
# Run:      docker run --rm -p 8080:8080 -v ysh-data:/data -e YSH_JWT_SECRET=... -e ...
# Scrape:   http://host:9091/metrics  (Prometheus)
# ═══════════════════════════════════════════

# ── STAGE 1: builder ──────────────────────────
FROM rust:1.98-slim AS builder

# system deps para compilar (la app es 100% Rust pero algunas crates necesitan build tools)
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev build-essential ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Cache de dependencias: copiar solo manifests primero
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs \
    && cargo build --release && rm -rf src

# Compilar la app real
COPY src ./src
COPY config ./config
RUN cargo build --release

# ── STAGE 2: runtime ──────────────────────────
# trixie = misma base (glibc) que rust:1.98-slim usado en el builder
FROM debian:trixie-slim AS runtime

# CA certificates (TLS/Let's Encrypt) + timezone + curl para healthcheck
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tzdata curl \
    && rm -rf /var/lib/apt/lists/*

# Usuario sin privilegios
RUN groupadd -r ysh && useradd -r -g ysh -m -d /app ysh

WORKDIR /app

# Binario compilado
COPY --from=builder /build/target/release/ysh /app/ysh

# Config (default.toml gestiona secrets por env)
COPY config/default.toml /app/config/default.toml

# Directorio de datos (base de datos redb + backups) — volumen externo
RUN mkdir -p /data && chown -R ysh:ysh /app /data
VOLUME ["/data"]

USER ysh

EXPOSE 8080
EXPOSE 9091

# ENTRYPOINT con variables razonables para contenedores
ENV YSH_LOG_JSON=1 \
    YSH_TLS_CERT=/dev/null \
    YSH_TLS_KEY=/dev/null

# permitir override de args (p.ej. --config /custom.toml)
ENTRYPOINT ["/app/ysh"]
CMD []
