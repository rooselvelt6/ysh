use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::auth::jwt::AuthUser;
use crate::server::AppState;

fn require_admin(auth: &AuthUser) -> Result<(), (StatusCode, String)> {
    if auth.role != "admin" {
        return Err((StatusCode::FORBIDDEN, "Admin access required".into()));
    }
    Ok(())
}

/// Manually trigger a single background job, synchronously.
pub async fn run_job(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;
    let msg = match name.as_str() {
        "payouts" => crate::actors::jobs_actor::JobsActorMsg::RunPayouts,
        "staking" => crate::actors::jobs_actor::JobsActorMsg::RunStaking,
        "moderation" => crate::actors::jobs_actor::JobsActorMsg::RunModeration,
        "cleanup" => crate::actors::jobs_actor::JobsActorMsg::RunCleanup,
        "notifications" => crate::actors::jobs_actor::JobsActorMsg::RunNotifications,
        "analytics" => crate::actors::jobs_actor::JobsActorMsg::RunAnalyticsSnapshot,
        _ => return Err((StatusCode::BAD_REQUEST, "Unknown job".into())),
    };
    state
        .jobs_actor
        .cast(msg)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{:#}", e)))?;
    Ok(Json(serde_json::json!({ "job": name, "triggered": true })))
}

/// Job runner state (counters + last results).
pub async fn jobs_stats(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;
    let (tx, rx) = tokio::sync::oneshot::channel();
    state
        .jobs_actor
        .cast(crate::actors::jobs_actor::JobsActorMsg::GetStats { reply_to: tx })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{:#}", e)))?;
    let stats = rx
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Actor stopped".into()))?;
    Ok(Json(stats))
}
