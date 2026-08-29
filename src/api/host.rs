use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::auth::jwt::AuthUser;
use crate::server::AppState;

pub async fn create_or_update_host(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;

    let languages = req["languages"].as_str().unwrap_or("en");
    let hourly_rate = req["hourly_rate"].as_i64().unwrap_or(0);

    state
        .db
        .create_host_profile(user_id, languages, hourly_rate)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "Host profile created",
        "languages": languages,
        "hourly_rate": hourly_rate,
    })))
}

pub async fn get_host(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let host = state
        .db
        .get_host_profile(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match host {
        Some(h) => Ok(Json(h)),
        None => Err((StatusCode::NOT_FOUND, "Host not found".into())),
    }
}

pub async fn set_availability(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;

    let available = req["available"].as_bool().unwrap_or(false);

    state
        .db
        .set_host_availability(user_id, available)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "available": available,
    })))
}

pub async fn list_hosts(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let available_only = params
        .get("available")
        .map(|s| s == "true")
        .unwrap_or(false);

    let hosts = state
        .db
        .list_hosts(available_only)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "hosts": hosts,
        "count": hosts.len(),
    })))
}
