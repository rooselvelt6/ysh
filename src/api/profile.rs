use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::auth::jwt::AuthUser;
use crate::server::AppState;

pub async fn update_profile(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;

    let display_name = req["display_name"].as_str().unwrap_or("");
    let bio = req["bio"].as_str().unwrap_or("");
    let avatar_url = req["avatar_url"].as_str().unwrap_or("");
    let country = req["country"].as_str().unwrap_or("");

    state
        .db
        .update_user_profile(user_id, display_name, bio, avatar_url, country)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "Profile updated",
        "display_name": display_name,
        "bio": bio,
        "country": country,
    })))
}

pub async fn get_profile(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let profile = state
        .db
        .get_profile(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match profile {
        Some(p) => Ok(Json(p)),
        None => Err((StatusCode::NOT_FOUND, "Profile not found".into())),
    }
}

pub async fn get_my_profile(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;

    let profile = state
        .db
        .get_profile(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let balance = state
        .db
        .get_balance(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "profile": profile.unwrap_or(serde_json::json!({})),
        "wallet_balance": balance,
    })))
}

pub async fn search_users(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let q = params.get("q").map(|s| s.as_str()).unwrap_or("");
    let limit: i64 = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);

    if q.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Query parameter 'q' required".into(),
        ));
    }

    let users = state
        .db
        .search_users(q, limit)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "users": users,
        "count": users.len(),
    })))
}
