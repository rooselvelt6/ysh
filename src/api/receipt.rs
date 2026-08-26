use axum::{extract::State, http::StatusCode, Json};

use crate::auth::jwt::AuthUser;
use crate::server::AppState;

pub async fn get_receipt(
    auth: AuthUser,
    State(state): State<AppState>,
    axum::extract::Path(receipt_id): axum::extract::Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let _user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let receipt = state.db.get_receipt(receipt_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Receipt not found".into()))?;

    Ok(Json(receipt))
}

pub async fn get_my_receipts(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let receipts = state.db.get_user_receipts(user_id, 50)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "receipts": receipts,
        "count": receipts.len(),
    })))
}

pub async fn verify_receipt(
    auth: AuthUser,
    State(state): State<AppState>,
    axum::extract::Path(receipt_id): axum::extract::Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let _user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let valid = state.db.verify_receipt(receipt_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "receipt_id": receipt_id,
        "valid": valid,
    })))
}
