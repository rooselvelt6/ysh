use axum::{Json, extract::State, http::StatusCode};

use crate::auth::jwt::AuthUser;
use crate::server::AppState;

pub async fn get_kyc_status(
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
        "kyc_level": user.kyc_level,
        "status": match user.kyc_level {
            0 => "unverified",
            1 => "email_verified",
            2 => "id_submitted",
            3 => "fully_verified",
            _ => "unknown",
        },
    })))
}

pub async fn submit_kyc(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<serde_json::Value>,
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

    let target_level = req["level"]
        .as_i64()
        .ok_or((StatusCode::BAD_REQUEST, "level integer required".into()))?
        as i32;

    if target_level != user.kyc_level + 1 {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Invalid KYC transition: current={}, requested={}",
                user.kyc_level, target_level
            ),
        ));
    }

    if target_level > 3 {
        return Err((StatusCode::BAD_REQUEST, "Maximum KYC level is 3".into()));
    }

    state
        .db
        .set_kyc_level(user_id, target_level)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tracing::info!("KYC level -> {} for user: {}", target_level, user_id);

    Ok(Json(serde_json::json!({
        "kyc_level": target_level,
        "status": match target_level {
            1 => "email_verified",
            2 => "id_submitted",
            3 => "fully_verified",
            _ => "unknown",
        },
    })))
}
