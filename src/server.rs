use axum::{
    body::Body,
    extract::State,
    http::{Method, Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

use crate::actors::notification_actor::NotificationMsg;
use crate::actors::session_supervisor::SessionSupervisorMsg;
use crate::cache::{Cache, RateLimitCache, SessionCache};
use crate::config::YshConfig;
use crate::db::Database;
use crate::middleware::circuit_breaker::CircuitBreaker;
use crate::middleware::rate_limit::GlobalRateLimiter;
use crate::security::zeroize::{EncryptedKey, SecureBuffer, SecureString};
use crate::ws::{ConnectionManager, MatchEvent, WsAuthQuery};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<YshConfig>,
    pub db: Arc<Database>,
    pub cache: Arc<Cache>,
    pub session_cache: Arc<SessionCache>,
    pub rate_limit_cache: Arc<RateLimitCache>,
    pub secure_jwt_secret: SecureString,
    pub secure_encryption_key: SecureBuffer,
    pub encrypted_key: EncryptedKey,
    pub session_actor: ractor::ActorRef<SessionSupervisorMsg>,
    pub notification_actor: ractor::ActorRef<NotificationMsg>,
    pub rate_limiter: GlobalRateLimiter,
    pub circuit_breaker: CircuitBreaker,
    pub ws_connections: Arc<tokio::sync::Mutex<ConnectionManager>>,
    pub read_receipts: Arc<tokio::sync::Mutex<std::collections::HashMap<(i64, i64), i64>>>,
    pub match_tx: tokio::sync::mpsc::UnboundedSender<MatchEvent>,
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

    let profile_routes = Router::new()
        .route("/profile", get(crate::api::profile::get_my_profile))
        .route("/profile", post(crate::api::profile::update_profile))
        .route("/profile/{user_id}", get(crate::api::profile::get_profile))
        .route("/users/search", get(crate::api::profile::search_users));

    let agency_routes = Router::new()
        .route("/agency", post(crate::api::agency::create_agency))
        .route("/agencies", get(crate::api::agency::list_agencies))
        .route("/agency/{agency_id}", get(crate::api::agency::get_agency))
        .route(
            "/agency/{agency_id}/members",
            get(crate::api::agency::get_members),
        )
        .route(
            "/agency/{agency_id}/members",
            post(crate::api::agency::add_member),
        );

    let host_routes = Router::new()
        .route("/host", post(crate::api::host::create_or_update_host))
        .route("/host/{user_id}", get(crate::api::host::get_host))
        .route(
            "/host/availability",
            post(crate::api::host::set_availability),
        )
        .route("/hosts", get(crate::api::host::list_hosts));

    let wallet_routes = Router::new()
        .route("/wallet/balance", get(crate::api::wallet::get_balance))
        .route("/wallet/deposit", post(crate::api::wallet::deposit))
        .route("/wallet/withdraw", post(crate::api::wallet::withdraw))
        .route("/wallet/transfer", post(crate::api::wallet::transfer))
        .route(
            "/wallet/transactions",
            get(crate::api::wallet::get_transactions),
        );

    let gift_routes = Router::new()
        .route("/gifts/catalog", get(crate::api::gift::get_catalog))
        .route(
            "/gifts/send/{user_id}",
            post(crate::api::gift::send_gift),
        )
        .route(
            "/gifts/received",
            get(crate::api::gift::get_received_gifts),
        );

    let moment_routes = Router::new()
        .route("/moment", post(crate::api::moment::create_moment))
        .route("/moments", get(crate::api::moment::get_feed))
        .route(
            "/moment/{moment_id}/like",
            post(crate::api::moment::like_moment),
        )
        .route(
            "/moment/{moment_id}/unlike",
            post(crate::api::moment::unlike_moment),
        )
        .route(
            "/moment/{moment_id}/comment",
            post(crate::api::moment::comment),
        )
        .route(
            "/moment/{moment_id}/comments",
            get(crate::api::moment::get_comments),
        )
        .route(
            "/moment/{moment_id}",
            delete(crate::api::moment::delete_moment),
        );

    let admin_routes = Router::new()
        .route("/admin/users", get(crate::api::admin::list_users))
        .route(
            "/admin/user/{user_id}/ban",
            post(crate::api::admin::ban_user),
        )
        .route(
            "/admin/user/{user_id}/unban",
            post(crate::api::admin::unban_user),
        )
        .route("/admin/stats", get(crate::api::admin::platform_stats));

    let notification_routes = Router::new()
        .route(
            "/notifications",
            get(crate::api::notification::list_notifications),
        )
        .route(
            "/notification/{notification_id}/read",
            post(crate::api::notification::mark_read),
        )
        .route(
            "/notifications/read-all",
            post(crate::api::notification::mark_all_read),
        )
        .route(
            "/notifications/preferences",
            get(crate::api::notification::get_preferences),
        )
        .route(
            "/notifications/preferences",
            post(crate::api::notification::update_preference),
        )
        .route(
            "/notifications/quiet-hours",
            post(crate::api::notification::update_quiet_hours),
        )
        .route(
            "/notifications/push/register",
            post(crate::api::notification::register_push_token),
        )
        .route(
            "/notifications/push/remove",
            post(crate::api::notification::remove_push_token),
        )
        .route(
            "/notifications/push/tokens",
            get(crate::api::notification::get_push_tokens),
        )
        .route(
            "/notifications/test",
            post(crate::api::notification::send_test_notification),
        );

    let chat_routes = Router::new()
        .route(
            "/chat/sessions",
            get(crate::api::chat::list_sessions),
        )
        .route(
            "/chat/session",
            post(crate::api::chat::create_session),
        )
        .route(
            "/chat/session/{session_id}/messages",
            get(crate::api::chat::get_messages),
        )
        .route(
            "/chat/session/{session_id}/read",
            post(crate::api::chat::mark_read),
        )
        .route(
            "/chat/unread",
            get(crate::api::chat::unread_count),
        )
        .route(
            "/chat/online",
            get(crate::api::chat::online_users),
        );

    let config_routes = Router::new().route("/config", get(get_config));

    let api_routes = Router::new()
        .merge(auth_routes)
        .merge(crypto_routes)
        .merge(config_routes)
        .merge(two_factor_routes)
        .merge(gdpr_routes)
        .merge(ccpa_routes)
        .merge(kyc_routes)
        .merge(profile_routes)
        .merge(agency_routes)
        .merge(host_routes)
        .merge(wallet_routes)
        .merge(gift_routes)
        .merge(moment_routes)
        .merge(admin_routes)
        .merge(notification_routes)
        .merge(chat_routes);

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

    let ws_routes = Router::new()
        .route("/ws", get(ws_upgrade_handler));

    Router::new()
        .merge(health_routes)
        .merge(ws_routes)
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
    let ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("direct");
    let key = format!("{}:{}", ip, request.uri().path());

    match state.rate_limit_cache.check_rate_limit(&key, 100, std::time::Duration::from_secs(60)) {
        Ok(result) => {
            if !result.allowed {
                tracing::warn!("Rate limit exceeded for {}", key);
                return Err(StatusCode::TOO_MANY_REQUESTS);
            }
        }
        Err(e) => {
            tracing::error!("Rate limit check failed: {}, falling back", e);
            state.rate_limiter.until_ready().await;
        }
    }
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

async fn readiness_check(State(state): State<AppState>) -> axum::Json<serde_json::Value> {
    let db_ok = state.db.health_check().is_ok();
    let cache_ok = state.cache.health_check().is_ok();
    let user_count = state.db.user_count().unwrap_or(0);
    let cache_stats = state.cache.stats();
    let session_ok = state.session_cache.health_check().is_ok();
    let rate_limit_ok = state.rate_limit_cache.health_check().is_ok();
    let ready = db_ok && cache_ok && session_ok && rate_limit_ok;
    axum::Json(serde_json::json!({
        "status": if ready { "ready" } else { "not_ready" },
        "database": {
            "healthy": db_ok,
            "users": user_count,
        },
        "cache": {
            "healthy": cache_ok,
            "entries": cache_stats.total_entries,
            "bytes": cache_stats.total_bytes,
        },
        "session_store": {
            "healthy": session_ok,
        },
        "rate_limiter": {
            "healthy": rate_limit_ok,
        },
        "version": env!("CARGO_PKG_VERSION"),
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

async fn ws_upgrade_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    axum::extract::Query(query): axum::extract::Query<WsAuthQuery>,
    State(state): State<AppState>,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| crate::ws::handle_ws(socket, query, state))
}
