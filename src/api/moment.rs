use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::auth::jwt::AuthUser;
use crate::server::AppState;

pub async fn create_moment(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let _ = state.db.log_activity(user_id, "moment");

    let content = req["content"].as_str().unwrap_or("");
    let media_url = req["media_url"].as_str().unwrap_or("");
    let media_type = req["media_type"].as_str().unwrap_or("text");

    let cfg = &state.config.moderation;
    let mut auto_mod = None;
    if cfg.auto_moderation_enabled && cfg.auto_moderate_moments {
        let moderation = state
            .ai_engine
            .moderate_text(crate::ai::ModerateRequest {
                content: content.to_string(),
            });
        match moderation.decision {
            crate::ai::ModerationDecision::Block => {
                let _ = state.db.flag_content(
                    "spam",
                    "auto",
                    "moment",
                    0,
                    moderation.severity,
                    &format!("Auto block: {}", moderation.matches.join(", ")),
                );
                return Err((
                    StatusCode::FORBIDDEN,
                    format!(
                        "Content blocked by moderation: {}",
                        moderation.matches.join(", ")
                    ),
                ));
            }
            crate::ai::ModerationDecision::Flag => {
                auto_mod = Some(moderation);
            }
            crate::ai::ModerationDecision::Allow => {}
        }
    }

    let moment_id = state
        .db
        .create_moment(user_id, content, media_url, media_type)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(moderation) = auto_mod {
        let _ = state.db.flag_content(
            "other",
            "auto",
            "moment",
            moment_id,
            moderation.severity,
            "Auto-flagged by AI moderation for review",
        );
    }

    Ok(Json(serde_json::json!({
        "id": moment_id,
        "message": "Moment created",
    })))
}

pub async fn get_feed(
    auth: AuthUser,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let offset: i64 = params
        .get("offset")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let limit: i64 = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);

    let moments = state
        .db
        .get_moment_feed(user_id, offset, limit)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "moments": moments,
        "count": moments.len(),
    })))
}

pub async fn like_moment(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(moment_id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    state
        .db
        .like_moment(user_id, moment_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "Moment liked",
    })))
}

pub async fn unlike_moment(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(moment_id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    state
        .db
        .unlike_moment(user_id, moment_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "Moment unliked",
    })))
}

pub async fn comment(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(moment_id): Path<i64>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let content = req["content"]
        .as_str()
        .ok_or((StatusCode::BAD_REQUEST, "content required".into()))?;

    let comment_id = state
        .db
        .comment_on_moment(user_id, moment_id, content)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "id": comment_id,
        "message": "Comment added",
    })))
}

pub async fn get_comments(
    State(state): State<AppState>,
    Path(moment_id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let comments = state
        .db
        .get_moment_comments(moment_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "comments": comments,
        "count": comments.len(),
    })))
}

pub async fn delete_moment(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(moment_id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let deleted = state
        .db
        .delete_moment(user_id, moment_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if deleted {
        Ok(Json(serde_json::json!({"message": "Moment deleted"})))
    } else {
        Err((StatusCode::NOT_FOUND, "Moment not found or not yours".into()))
    }
}
