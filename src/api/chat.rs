use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::Deserialize;

use crate::auth::jwt::AuthUser;
use crate::server::AppState;

pub async fn list_sessions(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let sessions = state.db.get_user_sessions(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "count": sessions.len(),
        "sessions": sessions,
    })))
}

#[derive(Deserialize)]
pub struct CreateSessionRequest {
    pub user_id: i64,
}

pub async fn create_session(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    if user_id == req.user_id {
        return Err((StatusCode::BAD_REQUEST, "Cannot create session with yourself".into()));
    }

    if let Some(session_id) = state.db.find_direct_session(user_id, req.user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))? {
        return Ok(Json(serde_json::json!({
            "session_id": session_id,
            "type": "direct",
            "existing": true,
        })));
    }

    let session_id = state.db.create_chat_session("direct", &[user_id, req.user_id])
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "session_id": session_id,
        "type": "direct",
        "existing": false,
    })))
}

pub async fn get_messages(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(session_id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let participants = state.db.get_session_participants(session_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let is_participant = participants.iter().any(|p| {
        p["user_id"].as_i64() == Some(user_id)
    });

    if !is_participant {
        return Err((StatusCode::FORBIDDEN, "Not a participant".into()));
    }

    let messages = state.db.get_messages(session_id, 50, None)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let count = messages.len();
    Ok(Json(serde_json::json!({
        "count": count,
        "messages": messages,
    })))
}

pub async fn mark_read(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(session_id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let count = state.db.mark_messages_read(session_id, user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "count": count,
    })))
}

pub async fn unread_count(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let count = state.db.get_unread_message_count(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "unread_count": count,
    })))
}

pub async fn online_users(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mgr = state.ws_connections.lock().await;

    let users: Vec<serde_json::Value> = mgr.online_users.iter().map(|(&uid, status)| {
        serde_json::json!({
            "user_id": uid,
            "status": status,
        })
    }).collect();

    Ok(Json(serde_json::json!({
        "count": users.len(),
        "users": users,
    })))
}
