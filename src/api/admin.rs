use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::auth::jwt::AuthUser;
use crate::server::AppState;

fn require_admin(auth: &AuthUser) -> Result<(), (StatusCode, String)> {
    if auth.role != "admin" {
        return Err((StatusCode::FORBIDDEN, "Admin access required".into()));
    }
    Ok(())
}

fn limit_from(
    params: &std::collections::HashMap<String, String>,
    default: i64,
) -> i64 {
    params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .map(|l: i64| l.min(500))
        .unwrap_or(default)
}

pub async fn list_users(
    auth: AuthUser,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let offset: i64 = params
        .get("offset")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let limit: i64 = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);

    let users = state
        .db
        .list_users(offset, limit)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "users": users,
        "count": users.len(),
    })))
}

pub async fn ban_user(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    state
        .db
        .ban_user(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "User banned",
        "user_id": user_id,
    })))
}

pub async fn unban_user(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    state
        .db
        .unban_user(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "User unbanned",
        "user_id": user_id,
    })))
}

pub async fn platform_stats(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let stats = state
        .db
        .platform_stats()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(stats))
}

pub async fn set_role(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    Json(req): Json<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let role = req
        .get("role")
        .cloned()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing 'role'".into()))?;
    if role != "user" && role != "admin" && role != "moderator" && role != "host" {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid role (user|admin|moderator|host)".into(),
        ));
    }

    state
        .db
        .set_user_role(user_id, &role)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "Role updated",
        "user_id": user_id,
        "role": role,
    })))
}

pub async fn admin_wallets(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let wallets = state
        .db
        .list_wallets()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "wallets": wallets,
        "count": wallets.len(),
    })))
}

pub async fn admin_transactions(
    auth: AuthUser,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let limit = limit_from(&params, 100);
    let transactions = state
        .db
        .list_all_transactions(limit)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "transactions": transactions,
        "count": transactions.len(),
    })))
}

pub async fn admin_moments(
    auth: AuthUser,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let limit = limit_from(&params, 100);
    let moments = state
        .db
        .list_all_moments(limit)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "moments": moments,
        "count": moments.len(),
    })))
}

pub async fn admin_delete_moment(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(moment_id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    state
        .db
        .delete_moment(auth.user_id.parse::<i64>().unwrap_or(0), moment_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "Moment deleted",
        "moment_id": moment_id,
    })))
}

pub async fn admin_receipts(
    auth: AuthUser,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let limit = limit_from(&params, 100);
    let receipts = state
        .db
        .list_receipts_all(limit)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "receipts": receipts,
        "count": receipts.len(),
    })))
}

pub async fn admin_calls(
    auth: AuthUser,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let limit = limit_from(&params, 100);
    let calls = state
        .db
        .list_calls_all(limit)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let stats = state
        .db
        .get_call_stats()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "calls": calls,
        "count": calls.len(),
        "stats": stats,
    })))
}

pub async fn admin_payouts(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let payouts = state
        .db
        .list_payouts_all()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "payouts": payouts,
        "count": payouts.len(),
    })))
}

pub async fn admin_fraud(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let alerts = state
        .db
        .list_fraud_alerts_all()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "alerts": alerts,
        "count": alerts.len(),
    })))
}

pub async fn admin_resolve_fraud(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(alert_id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let resolver_id = auth
        .user_id
        .parse::<i64>()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid admin id".into()))?;

    state
        .db
        .resolve_fraud_alert(alert_id, resolver_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "Fraud alert resolved",
        "alert_id": alert_id,
    })))
}

pub async fn admin_adjust_balance(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    Json(req): Json<std::collections::HashMap<String, serde_json::Value>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let amount: i64 = req
        .get("amount")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing integer 'amount'".into()))?;
    let description = req
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("admin adjustment")
        .to_string();

    let balance = if amount >= 0 {
        state
            .db
            .deposit(user_id, amount, &description)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else {
        state
            .db
            .withdraw(user_id, -amount, &description)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };

    Ok(Json(serde_json::json!({
        "message": "Balance adjusted",
        "user_id": user_id,
        "amount": amount,
        "balance": balance,
    })))
}

pub async fn admin_remove_agency_member(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((agency_id, user_id)): Path<(i64, i64)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    require_admin(&auth)?;

    let removed = state
        .db
        .remove_agency_member(agency_id, user_id)
        .map_err(|e| {
            if e.to_string().contains("last member") {
                (StatusCode::CONFLICT, e.to_string())
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        })?;

    Ok(Json(serde_json::json!({
        "message": "Member removed",
        "agency_id": agency_id,
        "user_id": user_id,
        "removed": removed,
    })))
}
