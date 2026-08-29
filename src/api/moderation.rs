use axum::{
    Json,
    extract::{Path, Query, State},
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

fn err(e: &anyhow::Error) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, e.to_string())
}

// ═══════════════════════════════════════════
// MODERATION QUEUE
// ═══════════════════════════════════════════

pub async fn get_moderation_queue(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let status = params.get("status").map(|s| s.as_str());
    let items = state.db.get_moderation_queue(status).map_err(|e| err(&e))?;

    Ok(Json(serde_json::json!({
        "queue": items,
        "count": items.len(),
        "pending_total": state.db.pending_moderation_count().unwrap_or(0),
    })))
}

pub async fn resolve_moderation_item(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(item_id): Path<i64>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let status = req["status"].as_str().unwrap_or("reviewed").to_string();

    state
        .db
        .resolve_moderation_item(item_id, &status)
        .map_err(|e| err(&e))?;

    Ok(Json(serde_json::json!({
        "message": "Moderation item resolved",
        "item_id": item_id,
        "status": status,
    })))
}

// ═══════════════════════════════════════════
// REPORTS
// ═══════════════════════════════════════════

pub async fn list_reports(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let status = params.get("status").map(|s| s.as_str());
    let reports = state.db.get_reports(status).map_err(|e| err(&e))?;

    Ok(Json(
        serde_json::json!({ "reports": reports, "count": reports.len() }),
    ))
}

pub async fn resolve_report(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(report_id): Path<i64>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let admin_id: i64 = auth.user_id.parse().unwrap_or(0);
    require_admin(&auth)?;

    let status = req["status"].as_str().unwrap_or("reviewed").to_string();
    let action_user: Option<i64> = req["action_user_id"].as_i64();

    state
        .db
        .resolve_report(report_id, admin_id, &status)
        .map_err(|e| err(&e))?;

    if let Some(user_id) = action_user {
        state.db.ban_user(user_id).map_err(|e| err(&e))?;
    }

    Ok(Json(serde_json::json!({
        "message": "Report resolved",
        "report_id": report_id,
        "status": status,
    })))
}

// ═══════════════════════════════════════════
// CONTENT FLAGS
// ═══════════════════════════════════════════

pub async fn list_content_flags(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let status = params.get("status").map(|s| s.as_str());
    let flags = state.db.get_content_flags(status).map_err(|e| err(&e))?;

    Ok(Json(
        serde_json::json!({ "flags": flags, "count": flags.len() }),
    ))
}

pub async fn resolve_content_flag(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(flag_id): Path<i64>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let admin_id: i64 = auth.user_id.parse().unwrap_or(0);
    require_admin(&auth)?;

    let status = req["status"].as_str().unwrap_or("reviewed").to_string();

    state
        .db
        .resolve_content_flag(flag_id, admin_id, &status)
        .map_err(|e| err(&e))?;

    Ok(Json(serde_json::json!({
        "message": "Content flag resolved",
        "flag_id": flag_id,
        "status": status,
    })))
}

// ═══════════════════════════════════════════
// APPEALS
// ═══════════════════════════════════════════

pub async fn list_appeals(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let status = params.get("status").map(|s| s.as_str());
    let appeals = state.db.get_appeals(status).map_err(|e| err(&e))?;

    Ok(Json(
        serde_json::json!({ "appeals": appeals, "count": appeals.len() }),
    ))
}

pub async fn resolve_appeal(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(appeal_id): Path<i64>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let admin_id: i64 = auth.user_id.parse().unwrap_or(0);
    require_admin(&auth)?;

    let approved = req["approved"].as_bool().unwrap_or(false);
    let notes = req["notes"].as_str().unwrap_or("").to_string();

    state
        .db
        .resolve_appeal(appeal_id, admin_id, approved, &notes)
        .map_err(|e| err(&e))?;

    Ok(Json(serde_json::json!({
        "message": if approved { "Appeal approved" } else { "Appeal rejected" },
        "appeal_id": appeal_id,
    })))
}

// ═══════════════════════════════════════════
// SHADOW BANS
// ═══════════════════════════════════════════

pub async fn shadow_ban_user(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let reason = req["reason"].as_str().unwrap_or("Moderator action");
    let duration = req["duration_secs"].as_i64();

    state
        .db
        .shadow_ban_user(user_id, reason, duration)
        .map_err(|e| err(&e))?;

    Ok(Json(serde_json::json!({
        "message": "User shadow-banned",
        "user_id": user_id,
        "duration_secs": duration,
    })))
}

pub async fn unshadow_ban_user(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let removed = state.db.unshadow_ban_user(user_id).map_err(|e| err(&e))?;

    Ok(Json(serde_json::json!({
        "message": "Shadow ban lifted",
        "user_id": user_id,
        "was_banned": removed,
    })))
}

pub async fn list_shadow_bans(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let bans = state.db.get_shadow_bans().map_err(|e| err(&e))?;
    Ok(Json(
        serde_json::json!({ "shadow_bans": bans, "count": bans.len() }),
    ))
}

// ═══════════════════════════════════════════
// VERIFICATION BADGES (admin grant/revoke)
// ═══════════════════════════════════════════

pub async fn grant_badge(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let badge_type = req["badge_type"]
        .as_str()
        .ok_or((StatusCode::BAD_REQUEST, "badge_type required".into()))?;

    let badge_id = state
        .db
        .grant_badge(user_id, badge_type)
        .map_err(|e| err(&e))?;

    Ok(Json(serde_json::json!({
        "message": "Badge granted",
        "badge_id": badge_id,
        "user_id": user_id,
        "badge_type": badge_type,
    })))
}

pub async fn revoke_badge(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((user_id, badge_type)): Path<(i64, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let removed = state
        .db
        .revoke_badge(user_id, &badge_type)
        .map_err(|e| err(&e))?;

    Ok(Json(serde_json::json!({
        "message": "Badge revoked",
        "user_id": user_id,
        "badge_type": badge_type,
        "was_revoked": removed,
    })))
}

// ═══════════════════════════════════════════
// MODERATION STATS
// ═══════════════════════════════════════════

pub async fn moderation_stats(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let pending_queue = state.db.pending_moderation_count().unwrap_or(0);
    let open_appeals = state
        .db
        .get_appeals(Some("open"))
        .map_err(|e| err(&e))?
        .len();
    let pending_reports = state
        .db
        .get_reports(Some("pending"))
        .map_err(|e| err(&e))?
        .len();
    let pending_flags = state
        .db
        .get_content_flags(Some("pending"))
        .map_err(|e| err(&e))?
        .len();
    let active_shadow_bans = state.db.active_shadow_ban_ids().map_err(|e| err(&e))?.len();

    Ok(Json(serde_json::json!({
        "pending_queue": pending_queue,
        "open_appeals": open_appeals,
        "pending_reports": pending_reports,
        "pending_flags": pending_flags,
        "active_shadow_bans": active_shadow_bans,
    })))
}
