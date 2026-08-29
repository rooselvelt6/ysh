use axum::{Json, extract::State, http::StatusCode};

use crate::auth::jwt::AuthUser;
use crate::server::AppState;

pub async fn get_balance(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;

    state
        .db
        .ensure_wallet(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let balance = state
        .db
        .get_balance(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let frozen = state.db.is_wallet_frozen(user_id).unwrap_or(false);

    Ok(Json(serde_json::json!({
        "balance": balance,
        "currency": "YSH",
        "frozen": frozen,
    })))
}

pub async fn deposit(
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

    let frozen = state.db.is_wallet_frozen(user_id).unwrap_or(false);
    if frozen {
        return Err((StatusCode::FORBIDDEN, "Wallet is frozen".into()));
    }

    let description = req["description"].as_str().unwrap_or("Deposit");

    state
        .db
        .ensure_wallet(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let balance = state
        .db
        .deposit(user_id, amount, description)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let _ = state
        .db
        .create_receipt(user_id, "deposit", 0, amount, "YSH", description, "{}");

    Ok(Json(serde_json::json!({
        "balance": balance,
        "deposited": amount,
    })))
}

pub async fn withdraw(
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

    let frozen = state.db.is_wallet_frozen(user_id).unwrap_or(false);
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

    let description = req["description"].as_str().unwrap_or("Withdraw");

    let balance = state
        .db
        .withdraw(user_id, amount, description)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let _ = state
        .db
        .create_receipt(user_id, "withdraw", 0, amount, "YSH", description, "{}");

    Ok(Json(serde_json::json!({
        "balance": balance,
        "withdrawn": amount,
    })))
}

pub async fn transfer(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let from_user: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;

    let to_user = req["to_user_id"]
        .as_i64()
        .ok_or((StatusCode::BAD_REQUEST, "to_user_id required".into()))?;

    let amount = req["amount"]
        .as_i64()
        .ok_or((StatusCode::BAD_REQUEST, "amount required".into()))?;

    if amount <= 0 {
        return Err((StatusCode::BAD_REQUEST, "amount must be positive".into()));
    }

    if from_user == to_user {
        return Err((StatusCode::BAD_REQUEST, "Cannot transfer to self".into()));
    }

    let frozen = state.db.is_wallet_frozen(from_user).unwrap_or(false);
    if frozen {
        return Err((StatusCode::FORBIDDEN, "Wallet is frozen".into()));
    }

    let (ok, msg) = state
        .db
        .check_spending_limit(from_user, amount)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if !ok {
        return Err((StatusCode::FORBIDDEN, msg));
    }

    let description = req["description"].as_str().unwrap_or("Transfer");

    state
        .db
        .transfer(from_user, to_user, amount, description)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let _ = state.db.create_receipt(
        from_user,
        "transfer_out",
        0,
        amount,
        "YSH",
        description,
        "{}",
    );

    Ok(Json(serde_json::json!({
        "message": "Transfer completed",
        "amount": amount,
        "to_user_id": to_user,
    })))
}

pub async fn get_transactions(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;

    let transactions = state
        .db
        .get_transactions(user_id, 50)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "transactions": transactions,
        "count": transactions.len(),
    })))
}

pub async fn get_spending_limits(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;

    let limits = state
        .db
        .get_spending_limits(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(limits))
}

pub async fn set_spending_limit(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;

    let daily = req["daily_limit"].as_i64().unwrap_or(100000);
    let monthly = req["monthly_limit"].as_i64().unwrap_or(1000000);

    if daily <= 0 || monthly <= 0 {
        return Err((StatusCode::BAD_REQUEST, "Limits must be positive".into()));
    }

    state
        .db
        .set_spending_limit(user_id, daily, monthly)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "daily_limit": daily,
        "monthly_limit": monthly,
    })))
}

pub async fn freeze_wallet(
    auth: AuthUser,
    State(state): State<AppState>,
    axum::extract::Path(target_user_id): axum::extract::Path<i64>,
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

    state
        .db
        .freeze_wallet(target_user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let _ = state.db.create_fraud_alert(
        Some(target_user_id),
        "wallet_frozen",
        "high",
        &format!("Wallet frozen by admin #{}", user_id),
        "{}",
        None,
    );

    Ok(Json(serde_json::json!({
        "user_id": target_user_id,
        "frozen": true,
    })))
}

pub async fn unfreeze_wallet(
    auth: AuthUser,
    State(state): State<AppState>,
    axum::extract::Path(target_user_id): axum::extract::Path<i64>,
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

    state
        .db
        .unfreeze_wallet(target_user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "user_id": target_user_id,
        "frozen": false,
    })))
}
