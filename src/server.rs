use axum::{
    Router,
    body::Body,
    extract::State,
    http::{Method, Request, StatusCode, Uri},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use std::sync::Arc;
use tower::util::ServiceExt;
use tower_http::cors::{AllowHeaders, AllowOrigin, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::services::ServeDir;
use tower_http::timeout::TimeoutLayer;

use crate::actors::ai_actor::AIActorMsg;
use crate::actors::notification_actor::NotificationMsg;
use crate::actors::session_supervisor::SessionSupervisorMsg;
use crate::ai::AIEngine;
use crate::cache::{Cache, RateLimitCache, SessionCache};
use crate::config::YshConfig;
use crate::db::Database;
use crate::middleware::circuit_breaker::CircuitBreaker;
use crate::middleware::ddos_protection::{DdosProtection, extract_client_ip};
use crate::middleware::ip_blocklist::IpBlocklist;
use crate::middleware::rate_limit::PerIpRateLimiter;
use crate::middleware::ws_guard::WsGuard;
use crate::security::zeroize::{EncryptedKey, SecureBuffer, SecureString};
use crate::webrtc::RoomManager;
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
    pub ai_actor: ractor::ActorRef<AIActorMsg>,
    pub ai_engine: std::sync::Arc<AIEngine>,
    pub i18n_engine: std::sync::Arc<crate::i18n::I18nEngine>,
    pub circuit_breaker: CircuitBreaker,
    pub ws_connections: Arc<tokio::sync::Mutex<ConnectionManager>>,
    pub read_receipts: Arc<tokio::sync::Mutex<std::collections::HashMap<(i64, i64), i64>>>,
    pub webrtc_actor: ractor::ActorRef<crate::actors::webrtc_actor::WebRTCActorMsg>,
    pub webrtc_rooms: Arc<tokio::sync::Mutex<RoomManager>>,
    pub jobs_actor: ractor::ActorRef<crate::actors::jobs_actor::JobsActorMsg>,
    pub match_tx: tokio::sync::mpsc::UnboundedSender<MatchEvent>,
    pub ip_blocklist: Arc<IpBlocklist>,
    pub per_ip_limiter: Arc<PerIpRateLimiter>,
    #[allow(dead_code)]
    pub ws_guard: Arc<WsGuard>,
    pub ddos_protection: DdosProtection,
}

pub fn build_router(state: AppState) -> Router {
    let health_routes = Router::new()
        .route("/healthz", get(health_check))
        .route("/readyz", get(readiness_check))
        .route("/metrics", get(metrics_handler));

    let auth_routes = Router::new()
        .route("/register", post(crate::auth::handlers::register))
        .route("/login", post(crate::auth::handlers::login))
        .route("/login/2fa", post(crate::auth::handlers::verify_2fa_login))
        .route("/me", get(crate::auth::handlers::me));

    let crypto_routes = Router::new()
        .route("/encrypt", post(crate::auth::handlers::encrypt_message))
        .route("/decrypt", post(crate::auth::handlers::decrypt_message));

    let two_factor_routes = Router::new()
        .route("/2fa/setup", post(crate::auth::two_factor::setup_2fa))
        .route("/2fa/verify", post(crate::auth::two_factor::verify_2fa))
        .route("/2fa/disable", post(crate::auth::two_factor::disable_2fa))
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
        .route("/gdpr/export", get(crate::auth::gdpr::export_user_data))
        .route("/gdpr/delete", post(crate::auth::gdpr::delete_user_data))
        .route("/gdpr/consent", post(crate::auth::gdpr::record_consent))
        .route(
            "/gdpr/consent/history",
            get(crate::auth::gdpr::get_consent_history),
        );

    let ccpa_routes = Router::new()
        .route("/ccpa/do-not-sell", get(crate::auth::ccpa::get_do_not_sell))
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
        .route("/gifts/send/{user_id}", post(crate::api::gift::send_gift))
        .route("/gifts/received", get(crate::api::gift::get_received_gifts))
        .route("/gifts/sent", get(crate::api::gift::get_sent_gifts))
        .route("/gifts/stats", get(crate::api::gift::get_gift_stats))
        .route("/gifts/nft", get(crate::api::gift::get_nft_gifts));

    let staking_routes = Router::new()
        .route("/staking/stake", post(crate::api::staking::stake))
        .route("/staking/unstake", post(crate::api::staking::unstake))
        .route("/staking/claim", post(crate::api::staking::claim_rewards))
        .route(
            "/staking/positions",
            get(crate::api::staking::get_positions),
        )
        .route("/staking/stats", get(crate::api::staking::get_stats));

    let payout_routes = Router::new()
        .route("/payout/request", post(crate::api::payout::request_payout))
        .route("/payout/history", get(crate::api::payout::get_my_payouts))
        .route(
            "/admin/payouts/pending",
            get(crate::api::payout::get_pending_payouts),
        )
        .route(
            "/admin/payouts/process",
            post(crate::api::payout::process_payout),
        );

    let receipt_routes = Router::new()
        .route("/receipts", get(crate::api::receipt::get_my_receipts))
        .route(
            "/receipt/{receipt_id}",
            get(crate::api::receipt::get_receipt),
        )
        .route(
            "/receipt/{receipt_id}/verify",
            get(crate::api::receipt::verify_receipt),
        );

    let commission_routes = Router::new()
        .route(
            "/commissions",
            get(crate::api::commission::get_my_commissions),
        )
        .route(
            "/referral/stats",
            get(crate::api::commission::get_referral_stats),
        )
        .route(
            "/referral/register",
            post(crate::api::commission::register_referral),
        );

    let wallet_extra_routes = Router::new()
        .route(
            "/wallet/limits",
            get(crate::api::wallet::get_spending_limits),
        )
        .route(
            "/wallet/limits",
            post(crate::api::wallet::set_spending_limit),
        )
        .route(
            "/admin/wallet/{user_id}/freeze",
            post(crate::api::wallet::freeze_wallet),
        )
        .route(
            "/admin/wallet/{user_id}/unfreeze",
            post(crate::api::wallet::unfreeze_wallet),
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
        .route("/admin/stats", get(crate::api::admin::platform_stats))
        .route(
            "/admin/user/{user_id}/role",
            post(crate::api::admin::set_role),
        )
        .route(
            "/admin/user/{user_id}/shadow-ban",
            post(crate::api::moderation::shadow_ban_user),
        )
        .route(
            "/admin/user/{user_id}/unshadow-ban",
            post(crate::api::moderation::unshadow_ban_user),
        )
        .route(
            "/admin/shadow-bans",
            get(crate::api::moderation::list_shadow_bans),
        )
        .route(
            "/admin/user/{user_id}/badge",
            post(crate::api::moderation::grant_badge),
        )
        .route(
            "/admin/user/{user_id}/badge/{badge_type}",
            axum::routing::delete(crate::api::moderation::revoke_badge),
        );

    let social_routes = Router::new()
        .route("/block", post(crate::api::social::block_user))
        .route(
            "/block/{user_id}",
            axum::routing::delete(crate::api::social::unblock_user),
        )
        .route("/blocks", get(crate::api::social::get_blocked_users))
        .route("/report", post(crate::api::social::create_report))
        .route("/reports", get(crate::api::social::get_my_reports))
        .route("/badges", get(crate::api::social::get_my_badges))
        .route(
            "/badges/{user_id}",
            get(crate::api::social::get_user_badges),
        )
        .route("/rating/{user_id}", post(crate::api::social::rate_user))
        .route(
            "/rating/{user_id}",
            get(crate::api::social::get_user_reputation),
        )
        .route(
            "/reputation/{user_id}",
            get(crate::api::social::get_user_reputation),
        )
        .route("/trust", get(crate::api::social::get_my_trust))
        .route("/flag", post(crate::api::social::flag_content))
        .route("/appeal", post(crate::api::social::create_appeal))
        .route("/appeals", get(crate::api::social::get_my_appeals));

    let moderation_routes = Router::new()
        .route(
            "/admin/moderation/queue",
            get(crate::api::moderation::get_moderation_queue),
        )
        .route(
            "/admin/moderation/queue/{item_id}",
            post(crate::api::moderation::resolve_moderation_item),
        )
        .route(
            "/admin/moderation/reports",
            get(crate::api::moderation::list_reports),
        )
        .route(
            "/admin/moderation/report/{report_id}",
            post(crate::api::moderation::resolve_report),
        )
        .route(
            "/admin/moderation/flags",
            get(crate::api::moderation::list_content_flags),
        )
        .route(
            "/admin/moderation/flag/{flag_id}",
            post(crate::api::moderation::resolve_content_flag),
        )
        .route(
            "/admin/moderation/appeals",
            get(crate::api::moderation::list_appeals),
        )
        .route(
            "/admin/moderation/appeal/{appeal_id}",
            post(crate::api::moderation::resolve_appeal),
        )
        .route(
            "/admin/moderation/stats",
            get(crate::api::moderation::moderation_stats),
        );

    let webrtc_routes = Router::new()
        .route("/call/start", post(crate::api::webrtc::start_call))
        .route("/call/{call_id}/join", post(crate::api::webrtc::join_call))
        .route(
            "/call/{call_id}/leave",
            post(crate::api::webrtc::leave_call),
        )
        .route("/call/{call_id}/end", post(crate::api::webrtc::end_call))
        .route(
            "/call/{call_id}/screen-share",
            post(crate::api::webrtc::toggle_screen_share),
        )
        .route(
            "/call/{call_id}/recording/start",
            post(crate::api::webrtc::start_recording),
        )
        .route(
            "/call/{call_id}/recording/stop",
            post(crate::api::webrtc::stop_recording),
        )
        .route(
            "/call/{call_id}/quality",
            post(crate::api::webrtc::report_quality),
        )
        .route(
            "/call/{call_id}/quality",
            get(crate::api::webrtc::call_quality),
        )
        .route("/call/{call_id}", get(crate::api::webrtc::get_call))
        .route("/call/{call_id}/peers", get(crate::api::webrtc::room_peers))
        .route(
            "/call/{call_id}/title",
            post(crate::api::webrtc::update_live_title),
        )
        .route("/calls/history", get(crate::api::webrtc::call_history))
        .route("/calls/live", get(crate::api::webrtc::live_streams))
        .route("/calls/rooms", get(crate::api::webrtc::active_rooms))
        .route("/calls/stats", get(crate::api::webrtc::call_stats))
        .route("/webrtc/stats", get(crate::api::webrtc::webrtc_stats));

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
        .route("/chat/sessions", get(crate::api::chat::list_sessions))
        .route("/chat/session", post(crate::api::chat::create_session))
        .route(
            "/chat/session/{session_id}/messages",
            get(crate::api::chat::get_messages),
        )
        .route(
            "/chat/session/{session_id}/read",
            post(crate::api::chat::mark_read),
        )
        .route("/chat/unread", get(crate::api::chat::unread_count))
        .route("/chat/online", get(crate::api::chat::online_users));

    let config_routes = Router::new().route("/config", get(get_config));

    let ai_routes = Router::new()
        .route("/ai/moderation/text", post(crate::api::ai::moderate_text))
        .route("/ai/anomaly/score", post(crate::api::ai::anomaly_score))
        .route(
            "/ai/anomaly/detector",
            post(crate::api::ai::anomaly_detector_demo),
        )
        .route("/ai/matching/score", post(crate::api::ai::match_score))
        .route(
            "/ai/matching/vectorize",
            post(crate::api::ai::match_vectorize),
        )
        .route("/ai/neural/predict", post(crate::api::ai::neural_predict))
        .route("/ai/neural/train", post(crate::api::ai::neural_train))
        .route(
            "/ai/optimize/genetic",
            post(crate::api::ai::optimize_genetic),
        )
        .route("/ai/stats", get(crate::api::ai::ai_stats));

    let i18n_routes = Router::new()
        .route("/i18n/locales", get(crate::api::i18n::list_locales))
        .route("/i18n/detect", get(crate::api::i18n::detect))
        .route("/i18n/translations", get(crate::api::i18n::translations))
        .route("/i18n/translate", get(crate::api::i18n::translate));
    let i18n_admin_routes = Router::new()
        .route("/admin/i18n", get(crate::api::i18n::admin_list))
        .route("/admin/i18n", post(crate::api::i18n::admin_upsert))
        .route(
            "/admin/i18n/{locale}/{key}",
            axum::routing::delete(crate::api::i18n::admin_delete),
        );

    let jobs_routes = Router::new()
        .route("/admin/jobs/run/{job}", post(crate::api::jobs::run_job))
        .route("/admin/jobs/stats", get(crate::api::jobs::jobs_stats));

    let analytics_routes = Router::new()
        .route(
            "/admin/analytics/realtime",
            get(crate::api::analytics::realtime_analytics),
        )
        .route(
            "/admin/analytics/users",
            get(crate::api::analytics::user_analytics),
        )
        .route(
            "/admin/analytics/revenue",
            get(crate::api::analytics::revenue_analytics),
        )
        .route(
            "/admin/analytics/agencies",
            get(crate::api::analytics::agency_analytics),
        )
        .route(
            "/admin/analytics/hosts",
            get(crate::api::analytics::hosts_leaderboard),
        )
        .route(
            "/admin/analytics/geo",
            get(crate::api::analytics::geo_analytics),
        )
        .route(
            "/admin/analytics/moderation",
            get(crate::api::analytics::moderation_analytics),
        )
        .route(
            "/admin/analytics/health",
            get(crate::api::analytics::system_health),
        )
        .route(
            "/admin/analytics/snapshots",
            get(crate::api::analytics::analytics_snapshots),
        )
        .route(
            "/admin/analytics/export",
            get(crate::api::analytics::export_analytics),
        )
        .route(
            "/profile/region/{region}",
            post(crate::api::analytics::set_my_region),
        );

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
        .merge(wallet_extra_routes)
        .merge(gift_routes)
        .merge(staking_routes)
        .merge(payout_routes)
        .merge(receipt_routes)
        .merge(commission_routes)
        .merge(moment_routes)
        .merge(admin_routes)
        .merge(social_routes)
        .merge(moderation_routes)
        .merge(webrtc_routes)
        .merge(notification_routes)
        .merge(chat_routes)
        .merge(ai_routes)
        .merge(i18n_routes)
        .merge(i18n_admin_routes)
        .merge(jobs_routes)
        .merge(analytics_routes);

    let cors_has_wildcard = state.config.cors.allowed_origins.iter().any(|o| o == "*");

    let mut cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(AllowHeaders::any())
        .max_age(std::time::Duration::from_secs(
            state.config.cors.max_age_secs,
        ));

    if cors_has_wildcard {
        cors = cors.allow_origin(AllowOrigin::any());
    } else {
        let origins: Vec<axum::http::HeaderValue> = state
            .config
            .cors
            .allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        if !origins.is_empty() {
            cors = cors.allow_origin(origins);
        }
    }

    let ws_routes = Router::new().route("/ws", get(ws_upgrade_handler));

    let body_limit = state.config.ddos.max_body_bytes;
    let timeout_secs = state.config.ddos.request_timeout_secs;

    let app = Router::new()
        .merge(health_routes)
        .merge(ws_routes)
        .nest("/api/v1", api_routes)
        .layer(middleware::from_fn(
            crate::middleware::security_headers::security_headers_middleware,
        ))
        .layer(cors)
        .layer(RequestBodyLimitLayer::new(body_limit))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            std::time::Duration::from_secs(timeout_secs),
        ))
        .layer(middleware::from_fn_with_state(
            state.ddos_protection.clone(),
            crate::middleware::ddos_protection::ddos_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            per_ip_rate_limit_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            circuit_breaker_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            metrics_middleware,
        ));

    let static_dir = state.config.server.static_dir.clone();
    let index_html = std::path::Path::new(&static_dir).join("index.html");
    if std::path::Path::new(&static_dir).is_dir() && index_html.exists() {
        tracing::info!("Serving frontend static files from {}", static_dir);
        app.fallback(spa_fallback).with_state(state)
    } else {
        tracing::warn!(
            "Static dir {} not found — API only (frontend WASM no disponible en /)",
            static_dir
        );
        app.fallback(static_not_found).with_state(state)
    }
}

/// Fallback SPA: sirve assets estáticos reales de `static_dir` tal cual; para
/// cualquier otra ruta (rutas del router frontend como `/login`, `/wallet`,
/// `/moments`...) devuelve `index.html` con status 200 para que el SPA decida.
/// Las rutas de API/WS/health no coincidentes devuelven 404 JSON (no deben
/// enmascararse con el HTML del SPA).
async fn spa_fallback(
    State(state): State<AppState>,
    uri: Uri,
    request: Request<Body>,
) -> Response {
    let path = uri.path().trim_start_matches('/');
    if path.starts_with("api/") || path == "ws" || path == "metrics" || path == "healthz" || path == "readyz" {
        return static_not_found().await.into_response();
    }

    let static_dir = state.config.server.static_dir.clone();
    let root = std::path::Path::new(&static_dir);
    let safe = path
        .split('/')
        .filter(|seg| !seg.is_empty() && *seg != "." && *seg != "..")
        .collect::<Vec<_>>()
        .join("/");
    let file_path = if safe.is_empty() {
        root.join("index.html")
    } else {
        root.join(&safe)
    };

    if file_path.is_file() {
        if let Ok(resp) = ServeDir::new(&static_dir)
            .oneshot(request)
            .await
        {
            return resp.into_response();
        }
    }

    match tokio::fs::read(root.join("index.html")).await {
        Ok(bytes) => {
            let nf = static_not_found().await;
            Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/html; charset=utf-8")
                .body(Body::from(bytes))
                .unwrap_or_else(|_| nf.into_response())
        }
        Err(_) => static_not_found().await.into_response(),
    }
}

/// Router independiente para el endpoint `/metrics` en el puerto de
/// observabilidad (`metrics_host:metrics_port`, default 0.0.0.0:9091).
/// Se sirve aparte del API para que Prometheus pueda scrapear sin pasar
/// por los middleware de rate-limit/anti-DDoS del servidor principal.
pub fn build_metrics_router(state: AppState) -> Router {
    Router::new()
        .route("/metrics", get(metrics_handler))
        .with_state(state)
}

async fn static_not_found() -> (StatusCode, axum::Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        axum::Json(serde_json::json!({ "error": "not_found" })),
    )
}

async fn metrics_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let response = next.run(request).await;
    if state.config.observability.metrics_enabled {
        let code = match response.status().as_u16() {
            100..=199 => "1xx",
            200..=299 => "2xx",
            300..=399 => "3xx",
            400..=499 => "4xx",
            _ => "5xx",
        };
        metrics::counter!("http_requests_total", "code" => code).increment(1);
    }
    Ok(response)
}

async fn metrics_handler(State(state): State<AppState>) -> (StatusCode, String) {
    if !state.config.observability.metrics_enabled
        || !crate::observability::metrics::is_initialized()
    {
        return (StatusCode::SERVICE_UNAVAILABLE, "metrics_disabled".into());
    }

    metrics::gauge!("ysh_uptime_seconds").set(crate::observability::metrics::uptime_secs() as f64);
    if let Ok(meta) = std::fs::metadata("ysh.db") {
        metrics::gauge!("ysh_db_size_bytes").set(meta.len() as f64);
    }
    let cache_stats = state.cache.stats();
    metrics::gauge!("ysh_cache_entries").set(cache_stats.total_entries as f64);
    metrics::gauge!("ysh_blocked_ips").set(state.ip_blocklist.blocked_count() as f64);
    if let Ok(ws_guard) = state.ws_connections.try_lock() {
        metrics::gauge!("ysh_ws_connections_active").set(ws_guard.online_count() as f64);
    }
    metrics::gauge!("ysh_users_total").set(state.db.user_count().unwrap_or(0) as f64);

    match crate::observability::metrics::render() {
        Some(body) => (
            StatusCode::OK,
            format!("{body}\n# ysh_version {}\n", env!("CARGO_PKG_VERSION")),
        ),
        None => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "metrics_unavailable".into(),
        ),
    }
}

async fn per_ip_rate_limit_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let ip = extract_client_ip(&request);
    let path = request.uri().path().to_string();

    let result = state.per_ip_limiter.check(&ip, &path);
    if !result.allowed {
        tracing::warn!("Per-IP rate limit exceeded: {} on {}", ip, path);
        metrics::counter!("http_rate_limited_total").increment(1);
        return Err(StatusCode::TOO_MANY_REQUESTS);
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
        metrics::gauge!("circuit_breaker_open").set(1.0);
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    let response = next.run(request).await;
    state.circuit_breaker.record_success();
    metrics::gauge!("circuit_breaker_open").set(0.0);
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
        "security": {
            "blocked_ips": state.ip_blocklist.blocked_count(),
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
    metrics::counter!("ysh_ws_connections_total").increment(1);
    ws.on_upgrade(move |socket| crate::ws::handle_ws(socket, query, state))
}
