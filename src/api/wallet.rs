use axum::{extract::State, http::StatusCode, Json};

use crate::auth::jwt::AuthUser;
use crate::server::AppState;

pub async fn get_balance(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    state
        .db
        .ensure_wallet(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let balance = state
        .db
        .get_balance(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "balance": balance,
        "currency": "YSH",
    })))
}

pub async fn deposit(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let amount = req["amount"]
        .as_i64()
        .ok_or((StatusCode::BAD_REQUEST, "amount required".into()))?;

    if amount <= 0 {
        return Err((StatusCode::BAD_REQUEST, "amount must be positive".into()));
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
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let amount = req["amount"]
        .as_i64()
        .ok_or((StatusCode::BAD_REQUEST, "amount required".into()))?;

    if amount <= 0 {
        return Err((StatusCode::BAD_REQUEST, "amount must be positive".into()));
    }

    let description = req["description"].as_str().unwrap_or("Withdraw");

    let balance = state
        .db
        .withdraw(user_id, amount, description)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

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
    let from_user: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

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

    let description = req["description"].as_str().unwrap_or("Transfer");

    state
        .db
        .transfer(from_user, to_user, amount, description)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

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
    let user_id: i64 = auth.user_id.parse().map_err(|_| {
        (StatusCode::UNAUTHORIZED, "Invalid token".into())
    })?;

    let transactions = state
        .db
        .get_transactions(user_id, 50)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "transactions": transactions,
        "count": transactions.len(),
    })))
}
