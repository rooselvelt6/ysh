use axum::{Json, extract::State, http::StatusCode};

use crate::auth::jwt::AuthUser;
use crate::server::AppState;

pub async fn stake(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;

    let amount = req["amount"]
        .as_i64()
        .ok_or((StatusCode::BAD_REQUEST, "amount required".into()))?;

    if amount <= 0 {
        return Err((StatusCode::BAD_REQUEST, "amount must be positive".into()));
    }

    let apy_rate = req["apy_rate"].as_f64().unwrap_or(0.05);
    let unlock_days = req["unlock_days"].as_i64().unwrap_or(30);

    if !(1..=365).contains(&unlock_days) {
        return Err((StatusCode::BAD_REQUEST, "unlock_days must be 1-365".into()));
    }

    let frozen = state
        .db
        .is_wallet_frozen(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if frozen {
        return Err((StatusCode::FORBIDDEN, "Wallet is frozen".into()));
    }

    let (ok, msg) = state
        .db
        .check_spending_limit(user_id, amount)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if !ok {
        return Err((StatusCode::FORBIDDEN, msg));
    }

    let stake_id = state
        .db
        .stake(user_id, amount, apy_rate, unlock_days)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let _ = state.db.create_receipt(
        user_id,
        "stake",
        stake_id,
        amount,
        "YSH",
        &format!("Staked {} YSH", amount),
        "{}",
    );

    Ok(Json(serde_json::json!({
        "stake_id": stake_id,
        "amount": amount,
        "apy_rate": apy_rate,
        "unlock_days": unlock_days,
    })))
}

pub async fn unstake(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;

    let stake_id = req["stake_id"]
        .as_i64()
        .ok_or((StatusCode::BAD_REQUEST, "stake_id required".into()))?;

    let total = state
        .db
        .unstake(user_id, stake_id)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "Unstaked successfully",
        "returned": total,
        "stake_id": stake_id,
    })))
}

pub async fn claim_rewards(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;

    let stake_id = req["stake_id"]
        .as_i64()
        .ok_or((StatusCode::BAD_REQUEST, "stake_id required".into()))?;

    let rewards = state
        .db
        .claim_staking_rewards(user_id, stake_id)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "claimed": rewards,
        "stake_id": stake_id,
    })))
}

pub async fn get_positions(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;

    let positions = state
        .db
        .get_staking_positions(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "positions": positions,
        "count": positions.len(),
    })))
}

pub async fn get_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let stats = state
        .db
        .get_staking_stats()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(stats))
}
