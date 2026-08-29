use axum::{Json, extract::State, http::StatusCode};

use crate::auth::jwt::AuthUser;
use crate::server::AppState;

pub async fn export_user_data(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".into()))?;

    let data = state
        .db
        .get_user_data(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(data))
}

pub async fn delete_user_data(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".into()))?;

    let password = req["password"]
        .as_str()
        .ok_or((StatusCode::BAD_REQUEST, "Password required".into()))?;

    let user = state
        .db
        .find_user_by_id(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found".into()))?;

    let valid = crate::security::password::verify_password(password, &user.password_hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !valid {
        return Err((StatusCode::UNAUTHORIZED, "Invalid password".into()));
    }

    state
        .db
        .delete_user_data(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tracing::info!("User {} exercised right to erasure (GDPR)", user_id);

    Ok(Json(serde_json::json!({
        "message": "All user data has been deleted",
    })))
}

pub async fn record_consent(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".into()))?;

    let consent_type = req["consent_type"]
        .as_str()
        .ok_or((StatusCode::BAD_REQUEST, "consent_type required".into()))?;
    let granted = req["granted"]
        .as_bool()
        .ok_or((StatusCode::BAD_REQUEST, "granted boolean required".into()))?;

    state
        .db
        .record_consent(user_id, consent_type, granted)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "Consent recorded",
        "consent_type": consent_type,
        "granted": granted,
    })))
}

pub async fn get_consent_history(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".into()))?;

    let records = state
        .db
        .get_consent_history(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "consent_records": records,
    })))
}
