use axum::{routing::get, routing::post, Router};
use std::sync::Arc;

use crate::auth::handlers::{login, me, register};
use crate::config::YshConfig;
use crate::db::Database;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<YshConfig>,
    pub db: Arc<Database>,
}

pub fn build_router(state: AppState) -> Router {
    let health_routes = Router::new()
        .route("/healthz", get(health_check))
        .route("/readyz", get(readiness_check));

    let auth_routes = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/me", get(me));

    let config_routes = Router::new()
        .route("/config", get(get_config));

    let api_routes = Router::new()
        .merge(auth_routes)
        .merge(config_routes);

    Router::new()
        .merge(health_routes)
        .nest("/api/v1", api_routes)
        .with_state(state)
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
    axum::extract::State(state): axum::extract::State<AppState>,
) -> axum::Json<serde_json::Value> {
    let config = &state.config;
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
