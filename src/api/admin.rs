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

pub async fn list_users(
    auth: AuthUser,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let offset: i64 = params
        .get("offset")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let limit: i64 = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);

    let users = state
        .db
        .list_users(offset, limit)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "users": users,
        "count": users.len(),
    })))
}

pub async fn ban_user(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    state
        .db
        .ban_user(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "User banned",
        "user_id": user_id,
    })))
}

pub async fn unban_user(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    state
        .db
        .unban_user(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "User unbanned",
        "user_id": user_id,
    })))
}

pub async fn platform_stats(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let stats = state
        .db
        .platform_stats()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(stats))
}

pub async fn set_role(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    Json(req): Json<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let role = req
        .get("role")
        .cloned()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing 'role'".into()))?;
    if role != "user" && role != "admin" && role != "moderator" && role != "host" {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid role (user|admin|moderator|host)".into(),
        ));
    }

    state
        .db
        .set_user_role(user_id, &role)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "Role updated",
        "user_id": user_id,
        "role": role,
    })))
}
