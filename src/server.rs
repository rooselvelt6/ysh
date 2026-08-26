use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::actors::session_supervisor::SessionSupervisorMsg;
use crate::config::YshConfig;
use crate::db::Database;
use crate::middleware::circuit_breaker::CircuitBreaker;
use crate::middleware::rate_limit::GlobalRateLimiter;
use crate::security::zeroize::{EncryptedKey, SecureBuffer, SecureString};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<YshConfig>,
    pub db: Arc<Database>,
    pub secure_jwt_secret: SecureString,
    pub secure_encryption_key: SecureBuffer,
    pub encrypted_key: EncryptedKey,
    pub session_actor: ractor::ActorRef<SessionSupervisorMsg>,
    pub rate_limiter: GlobalRateLimiter,
    pub circuit_breaker: CircuitBreaker,
}

pub fn build_router(state: AppState) -> Router {
    let health_routes = Router::new()
        .route("/healthz", get(health_check))
        .route("/readyz", get(readiness_check));

    let auth_routes = Router::new()
        .route("/register", post(crate::auth::handlers::register))
        .route("/login", post(crate::auth::handlers::login))
        .route("/me", get(crate::auth::handlers::me));

    let crypto_routes = Router::new()
        .route("/encrypt", post(crate::auth::handlers::encrypt_message))
        .route("/decrypt", post(crate::auth::handlers::decrypt_message));

    let config_routes = Router::new().route("/config", get(get_config));

    let api_routes = Router::new()
        .merge(auth_routes)
        .merge(crypto_routes)
        .merge(config_routes);

    Router::new()
        .merge(health_routes)
        .nest("/api/v1", api_routes)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            circuit_breaker_middleware,
        ))
        .with_state(state)
}

async fn rate_limit_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    state.rate_limiter.until_ready().await;
    Ok(next.run(request).await)
}

async fn circuit_breaker_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    if !state.circuit_breaker.is_available() {
        tracing::warn!("Circuit breaker open, rejecting request");
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    let response = next.run(request).await;
    state.circuit_breaker.record_success();
    Ok(response)
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
    State(state): State<AppState>,
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
