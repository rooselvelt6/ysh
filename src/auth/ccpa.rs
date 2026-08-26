use axum::{extract::State, http::StatusCode, Json};

use crate::auth::jwt::AuthUser;
use crate::server::AppState;

pub async fn get_do_not_sell(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".into()))?;

    let user = state
        .db
        .find_user_by_id(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found".into()))?;

    Ok(Json(serde_json::json!({
        "do_not_sell": user.do_not_sell,
    })))
}

pub async fn set_do_not_sell(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".into()))?;

    let value = req["do_not_sell"]
        .as_bool()
        .ok_or((StatusCode::BAD_REQUEST, "do_not_sell boolean required".into()))?;

    state
        .db
        .set_do_not_sell(user_id, value)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tracing::info!("CCPA do_not_sell={} for user: {}", value, user_id);

    Ok(Json(serde_json::json!({
        "do_not_sell": value,
    })))
}
