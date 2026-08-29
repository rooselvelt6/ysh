use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};

use crate::auth::jwt::AuthUser;
use crate::server::AppState;

fn parse_uid(auth: &AuthUser) -> Result<i64, (StatusCode, String)> {
    auth.user_id
        .parse()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))
}

// ═══════════════════════════════════════════
// USER BLOCKS
// ═══════════════════════════════════════════

pub async fn block_user(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = parse_uid(&auth)?;
    let target: i64 = req["target_user_id"]
        .as_i64()
        .ok_or((StatusCode::BAD_REQUEST, "target_user_id required".into()))?;

    state
        .db
        .block_user(user_id, target)
        .map_err(|e| http_err(&e))?;

    Ok(Json(
        serde_json::json!({ "message": "User blocked", "blocked_user_id": target }),
    ))
}

pub async fn unblock_user(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(target_id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = parse_uid(&auth)?;

    let removed = state
        .db
        .unblock_user(user_id, target_id)
        .map_err(|e| http_err(&e))?;

    if removed {
        Ok(Json(serde_json::json!({ "message": "User unblocked" })))
    } else {
        Err((StatusCode::NOT_FOUND, "Block not found".into()))
    }
}

pub async fn get_blocked_users(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = parse_uid(&auth)?;

    let blocks = state
        .db
        .get_blocked_users(user_id)
        .map_err(|e| http_err(&e))?;

    let usernames: Vec<serde_json::Value> = blocks
        .into_iter()
        .map(|b| {
            let username = state
                .db
                .find_user_by_id(b.blocked_user_id)
                .ok()
                .flatten()
                .map(|u| u.username)
                .unwrap_or_default();
            serde_json::json!({
                "user_id": b.blocked_user_id,
                "username": username,
                "created_at": b.created_at,
            })
        })
        .collect();

    Ok(Json(
        serde_json::json!({ "blocked": usernames, "count": usernames.len() }),
    ))
}

// ═══════════════════════════════════════════
// USER REPORTS
// ═══════════════════════════════════════════

pub async fn create_report(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = parse_uid(&auth)?;

    let target_type = req["target_type"]
        .as_str()
        .ok_or((StatusCode::BAD_REQUEST, "target_type required".into()))?;
    let target_id: i64 = req["target_id"]
        .as_i64()
        .ok_or((StatusCode::BAD_REQUEST, "target_id required".into()))?;
    let category = req["category"].as_str().unwrap_or("other").to_string();
    let description = req["description"].as_str().unwrap_or("").to_string();

    let valid_types = ["user", "moment", "message", "host", "agency"];
    if !valid_types.contains(&target_type) {
        return Err((StatusCode::BAD_REQUEST, "invalid target_type".into()));
    }

    let report_id = state
        .db
        .create_report(user_id, target_type, target_id, &category, &description)
        .map_err(|e| http_err(&e))?;

    let mut message = format!("Report #{report_id} submitted");
    if target_type == "user" {
        let threshold = state.config.moderation.auto_shadow_ban_after_reports;
        let distinct = state
            .db
            .distinct_reporters_for("user", target_id)
            .map_err(|e| http_err(&e))?;
        if distinct >= threshold {
            let duration = state.config.moderation.shadow_ban_duration_secs;
            state
                .db
                .shadow_ban_user(
                    target_id,
                    "Auto shadow ban: too many distinct reports",
                    Some(duration),
                )
                .map_err(|e| http_err(&e))?;
            message = format!("Report #{report_id} submitted; target auto-shadow-banned");
        }
    }

    Ok(Json(serde_json::json!({
        "report_id": report_id,
        "message": message,
    })))
}

pub async fn get_my_reports(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = parse_uid(&auth)?;

    let reports = state
        .db
        .get_user_reports(user_id)
        .map_err(|e| http_err(&e))?;

    Ok(Json(serde_json::json!({
        "reports": reports,
        "count": reports.len(),
    })))
}

// ═══════════════════════════════════════════
// VERIFICATION BADGES
// ═══════════════════════════════════════════

pub async fn get_my_badges(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = parse_uid(&auth)?;

    let badges = state
        .db
        .get_user_badges(user_id)
        .map_err(|e| http_err(&e))?;
    Ok(Json(
        serde_json::json!({ "badges": badges, "count": badges.len() }),
    ))
}

pub async fn get_user_badges(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let badges = state
        .db
        .get_user_badges(user_id)
        .map_err(|e| http_err(&e))?;

    Ok(Json(
        serde_json::json!({ "user_id": user_id, "badges": badges, "count": badges.len() }),
    ))
}

// ═══════════════════════════════════════════
// RATING + REPUTATION
// ═══════════════════════════════════════════

pub async fn rate_user(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let rater_id = parse_uid(&auth)?;

    let score = req["score"]
        .as_f64()
        .ok_or((StatusCode::BAD_REQUEST, "score required".into()))?;

    state
        .db
        .rate_user(rater_id, user_id, score)
        .map_err(|e| http_err(&e))?;

    let reputation = state.db.get_reputation(user_id).map_err(|e| http_err(&e))?;
    Ok(Json(serde_json::json!({
        "message": "Rating submitted",
        "rating_avg": reputation.rating_avg,
        "rating_count": reputation.rating_count,
    })))
}

pub async fn get_user_reputation(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let reputation = state.db.get_reputation(user_id).map_err(|e| http_err(&e))?;

    Ok(Json(serde_json::json!({
        "user_id": reputation.user_id,
        "rating_avg": reputation.rating_avg,
        "rating_count": reputation.rating_count,
    })))
}

// ═══════════════════════════════════════════
// MANUAL CONTENT FLAGS
// ═══════════════════════════════════════════

pub async fn flag_content(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let _user_id = parse_uid(&auth)?;

    let target_type = req["target_type"]
        .as_str()
        .ok_or((StatusCode::BAD_REQUEST, "target_type required".into()))?;
    let target_id: i64 = req["target_id"]
        .as_i64()
        .ok_or((StatusCode::BAD_REQUEST, "target_id required".into()))?;
    let flag_type = req["flag_type"].as_str().unwrap_or("other").to_string();
    let description = req["description"].as_str().unwrap_or("").to_string();

    let valid_types = ["moment", "message", "user", "host"];
    if !valid_types.contains(&target_type) {
        return Err((StatusCode::BAD_REQUEST, "invalid target_type".into()));
    }

    let flag_id = state
        .db
        .flag_content(
            &flag_type,
            "manual",
            target_type,
            target_id,
            0.5,
            &description,
        )
        .map_err(|e| http_err(&e))?;

    Ok(Json(serde_json::json!({
        "flag_id": flag_id,
        "message": "Content flagged for review",
    })))
}

// ═══════════════════════════════════════════
// TRUST SCORE
// ═══════════════════════════════════════════

pub async fn get_my_trust(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = parse_uid(&auth)?;

    let trust = state
        .db
        .get_trust_score(user_id)
        .map_err(|e| http_err(&e))?;

    Ok(Json(trust))
}

// ═══════════════════════════════════════════
// APPEALS
// ═══════════════════════════════════════════

pub async fn create_appeal(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = parse_uid(&auth)?;

    let target_type = req["target_type"]
        .as_str()
        .ok_or((StatusCode::BAD_REQUEST, "target_type required".into()))?;
    let target_id: i64 = req["target_id"]
        .as_i64()
        .ok_or((StatusCode::BAD_REQUEST, "target_id required".into()))?;
    let reason = req["reason"]
        .as_str()
        .ok_or((StatusCode::BAD_REQUEST, "reason required".into()))?;

    let valid_types = ["ban", "shadow_ban", "content_flag"];
    if !valid_types.contains(&target_type) {
        return Err((StatusCode::BAD_REQUEST, "invalid target_type".into()));
    }

    let appeal_id = state
        .db
        .create_appeal(user_id, target_type, target_id, reason)
        .map_err(|e| http_err(&e))?;

    Ok(Json(serde_json::json!({
        "appeal_id": appeal_id,
        "message": "Appeal submitted for review",
    })))
}

pub async fn get_my_appeals(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = parse_uid(&auth)?;

    let appeals = state
        .db
        .get_user_appeals(user_id)
        .map_err(|e| http_err(&e))?;

    Ok(Json(
        serde_json::json!({ "appeals": appeals, "count": appeals.len() }),
    ))
}

fn http_err(e: &anyhow::Error) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, e.to_string())
}

/// Shared query-param helper used by several handlers.
#[allow(dead_code)]
fn query_status(params: &Query<std::collections::HashMap<String, String>>) -> Option<String> {
    params.get("status").cloned()
}
