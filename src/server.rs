use axum::{routing::get, Router};
use std::sync::Arc;

use crate::config::YshConfig;

pub fn build_router(config: Arc<YshConfig>) -> Router {
    let health_routes = Router::new()
        .route("/healthz", get(health_check))
        .route("/readyz", get(readiness_check));

    let api_routes = Router::new()
        .route("/config", get(get_config));

    Router::new()
        .merge(health_routes)
        .nest("/api/v1", api_routes)
        .with_state(config)
}

async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn readiness_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ready",
        "database": true,
        "cache": true,
        "actors": true,
    }))
}

async fn get_config(
    axum::extract::State(config): axum::extract::State<Arc<YshConfig>>,
) -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "server": {
            "host": config.server.host,
            "port": config.server.port,
            "workers": config.server.workers,
        },
        "database": {
            "url": config.database.url,
            "max_connections": config.database.max_connections,
        },
        "encryption": {
            "algorithm": config.encryption.algorithm,
        },
    }))
}
