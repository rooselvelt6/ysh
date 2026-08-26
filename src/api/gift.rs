use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::auth::jwt::AuthUser;
use crate::server::AppState;

pub async fn get_catalog(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let gifts = state
        .db
        .get_gift_catalog()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "gifts": gifts,
        "count": gifts.len(),
    })))
}

pub async fn send_gift(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(to_user_id): Path<i64>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let from_user: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let gift_id = req["gift_id"]
        .as_i64()
        .ok_or((StatusCode::BAD_REQUEST, "gift_id required".into()))?;

    if from_user == to_user_id {
        return Err((StatusCode::BAD_REQUEST, "Cannot gift to self".into()));
    }

    state
        .db
        .ensure_wallet(from_user)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let gift_record_id = state
        .db
        .send_gift(from_user, to_user_id, gift_id)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "Gift sent",
        "gift_record_id": gift_record_id,
        "to_user_id": to_user_id,
        "gift_id": gift_id,
    })))
}

pub async fn get_received_gifts(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let gifts = state
        .db
        .get_received_gifts(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "gifts": gifts,
        "count": gifts.len(),
    })))
}
