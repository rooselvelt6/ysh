use axum::{Json, extract::State, http::StatusCode};

use crate::auth::jwt::AuthUser;
use crate::server::AppState;

pub async fn request_payout(
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

    let wallet_address = req["wallet_address"]
        .as_str()
        .ok_or((StatusCode::BAD_REQUEST, "wallet_address required".into()))?;

    if wallet_address.len() < 10 {
        return Err((StatusCode::BAD_REQUEST, "Invalid wallet address".into()));
    }

    let currency = req["currency"].as_str().unwrap_or("USDT");
    let network = req["network"].as_str().unwrap_or("TRC20");

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

    let payout_id = state
        .db
        .request_payout(user_id, amount, currency, wallet_address, network)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let _ = state.db.create_receipt(
        user_id,
        "payout",
        payout_id,
        amount,
        currency,
        &format!("Payout to {}", wallet_address),
        "{}",
    );

    Ok(Json(serde_json::json!({
        "payout_id": payout_id,
        "amount": amount,
        "currency": currency,
        "network": network,
        "wallet_address": wallet_address,
        "status": "pending",
    })))
}

pub async fn get_my_payouts(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;

    let payouts = state
        .db
        .get_user_payouts(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "payouts": payouts,
        "count": payouts.len(),
    })))
}

pub async fn get_pending_payouts(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;

    let user = state
        .db
        .find_user_by_id(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "User not found".into()))?;

    if user.role != "admin" {
        return Err((StatusCode::FORBIDDEN, "Admin only".into()));
    }

    let payouts = state
        .db
        .get_pending_payouts()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "payouts": payouts,
        "count": payouts.len(),
    })))
}

pub async fn process_payout(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;

    let user = state
        .db
        .find_user_by_id(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "User not found".into()))?;

    if user.role != "admin" {
        return Err((StatusCode::FORBIDDEN, "Admin only".into()));
    }

    let payout_id = req["payout_id"]
        .as_i64()
        .ok_or((StatusCode::BAD_REQUEST, "payout_id required".into()))?;

    let approved = req["approved"].as_bool().unwrap_or(false);
    let tx_hash = req["tx_hash"].as_str().unwrap_or("");

    state
        .db
        .process_payout(payout_id, user_id, tx_hash, approved)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "payout_id": payout_id,
        "status": if approved { "completed" } else { "rejected" },
        "tx_hash": tx_hash,
    })))
}
