use axum::{
    body::Body,
    extract::State,
    http::{Method, Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

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
        .route(
            "/login/2fa",
            post(crate::auth::handlers::verify_2fa_login),
        )
        .route("/me", get(crate::auth::handlers::me));

    let crypto_routes = Router::new()
        .route("/encrypt", post(crate::auth::handlers::encrypt_message))
        .route("/decrypt", post(crate::auth::handlers::decrypt_message));

    let two_factor_routes = Router::new()
        .route("/2fa/setup", post(crate::auth::two_factor::setup_2fa))
        .route(
            "/2fa/verify",
            post(crate::auth::two_factor::verify_2fa),
        )
        .route(
            "/2fa/disable",
            post(crate::auth::two_factor::disable_2fa),
        )
        .route(
            "/2fa/recovery-codes",
            get(crate::auth::two_factor::get_recovery_codes),
        )
        .route(
            "/2fa/recovery-codes/regenerate",
            post(crate::auth::two_factor::regenerate_recovery_codes),
        )
        .route(
            "/2fa/recovery/verify",
            post(crate::auth::two_factor::verify_recovery),
        );

    let gdpr_routes = Router::new()
        .route(
            "/gdpr/export",
            get(crate::auth::gdpr::export_user_data),
        )
        .route(
            "/gdpr/delete",
            post(crate::auth::gdpr::delete_user_data),
        )
        .route(
            "/gdpr/consent",
            post(crate::auth::gdpr::record_consent),
        )
        .route(
            "/gdpr/consent/history",
            get(crate::auth::gdpr::get_consent_history),
        );

    let ccpa_routes = Router::new()
        .route(
            "/ccpa/do-not-sell",
            get(crate::auth::ccpa::get_do_not_sell),
        )
        .route(
            "/ccpa/do-not-sell",
            post(crate::auth::ccpa::set_do_not_sell),
        );

    let kyc_routes = Router::new()
        .route("/kyc/status", get(crate::auth::kyc::get_kyc_status))
        .route("/kyc/submit", post(crate::auth::kyc::submit_kyc));

    let config_routes = Router::new().route("/config", get(get_config));

    let api_routes = Router::new()
        .merge(auth_routes)
        .merge(crypto_routes)
        .merge(config_routes)
        .merge(two_factor_routes)
        .merge(gdpr_routes)
        .merge(ccpa_routes)
        .merge(kyc_routes);

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::any())
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any)
        .max_age(std::time::Duration::from_secs(3600));

    Router::new()
        .merge(health_routes)
        .nest("/api/v1", api_routes)
        .layer(middleware::from_fn(
            crate::middleware::security_headers::security_headers_middleware,
        ))
        .layer(cors)
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

async fn get_config(State(state): State<AppState>) -> axum::Json<serde_json::Value> {
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
