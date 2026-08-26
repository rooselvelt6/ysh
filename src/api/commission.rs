use axum::{extract::State, http::StatusCode, Json};

use crate::auth::jwt::AuthUser;
use crate::server::AppState;

pub async fn get_my_commissions(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let commissions = state.db.get_user_commissions(user_id, None)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let summary = state.db.get_commission_summary(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "commissions": commissions,
        "count": commissions.len(),
        "summary": summary,
    })))
}

pub async fn get_referral_stats(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let stats = state.db.get_referral_stats(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(stats))
}

pub async fn register_referral(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let code = req["code"]
        .as_str()
        .ok_or((StatusCode::BAD_REQUEST, "referral code required".into()))?;

    let (referrer_id, _referred_id) = state.db.find_referral_by_code(code)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Invalid referral code".into()))?;

    if referrer_id == user_id {
        return Err((StatusCode::BAD_REQUEST, "Cannot refer yourself".into()));
    }

    state.db.create_referral(referrer_id, user_id, code)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "Referral registered",
        "referrer_id": referrer_id,
    })))
}
