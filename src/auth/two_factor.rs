use axum::{Json, extract::State, http::StatusCode};
use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::auth::jwt::AuthUser;
use crate::security::totp;
use crate::server::AppState;

#[derive(Serialize)]
pub struct SetupResponse {
    pub secret: String,
    pub uri: String,
    pub recovery_codes: Vec<String>,
}

pub async fn setup_2fa(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<SetupResponse>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".into()))?;

    let user = state
        .db
        .find_user_by_id(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found".into()))?;

    let (secret_bytes, secret_base32) = totp::generate_secret();
    let uri = totp::generate_uri(&secret_base32, &user.email, "YSH");

    let encoded_secret = base64::engine::general_purpose::STANDARD.encode(secret_bytes);
    state
        .db
        .set_totp_secret(user_id, &encoded_secret)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    state
        .db
        .delete_recovery_codes(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let plaintext_codes = totp::generate_recovery_codes(10);
    let hashed_codes: Vec<(String, bool)> = plaintext_codes
        .iter()
        .map(|c| (totp::hash_recovery_code(c), false))
        .collect();
    state
        .db
        .store_recovery_codes(user_id, &hashed_codes)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(SetupResponse {
        secret: secret_base32,
        uri,
        recovery_codes: plaintext_codes,
    }))
}

#[derive(Deserialize)]
pub struct VerifyRequest {
    pub code: String,
}

pub async fn verify_2fa(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<VerifyRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".into()))?;

    let totp_secret = state
        .db
        .get_totp_secret(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "2FA not set up".into()))?;

    let secret_bytes = base64::engine::general_purpose::STANDARD
        .decode(&totp_secret)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !totp::verify_code(&secret_bytes, &req.code) {
        return Err((StatusCode::UNAUTHORIZED, "Invalid code".into()));
    }

    state
        .db
        .enable_totp(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tracing::info!("2FA enabled for user: {}", auth.user_id);

    Ok(Json(serde_json::json!({
        "message": "2FA enabled successfully",
    })))
}

#[derive(Deserialize)]
pub struct DisableRequest {
    pub code: String,
}

pub async fn disable_2fa(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<DisableRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".into()))?;

    let totp_secret = state
        .db
        .get_totp_secret(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "2FA not set up".into()))?;

    let secret_bytes = base64::engine::general_purpose::STANDARD
        .decode(&totp_secret)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !totp::verify_code(&secret_bytes, &req.code) {
        return Err((StatusCode::UNAUTHORIZED, "Invalid code".into()));
    }

    state
        .db
        .disable_totp(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    state
        .db
        .delete_recovery_codes(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tracing::info!("2FA disabled for user: {}", auth.user_id);

    Ok(Json(serde_json::json!({
        "message": "2FA disabled successfully",
    })))
}

pub async fn get_recovery_codes(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".into()))?;

    let codes = state
        .db
        .get_recovery_codes(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let total = codes.len();
    let used = codes.iter().filter(|c| c.used).count();

    Ok(Json(serde_json::json!({
        "total": total,
        "used": used,
        "remaining": total - used,
    })))
}

pub async fn regenerate_recovery_codes(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id: i64 = auth
        .user_id
        .parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".into()))?;

    state
        .db
        .delete_recovery_codes(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let plaintext_codes = totp::generate_recovery_codes(10);
    let hashed_codes: Vec<(String, bool)> = plaintext_codes
        .iter()
        .map(|c| (totp::hash_recovery_code(c), false))
        .collect();
    state
        .db
        .store_recovery_codes(user_id, &hashed_codes)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "recovery_codes": plaintext_codes,
    })))
}

pub async fn verify_recovery(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let username = req["username"]
        .as_str()
        .ok_or((StatusCode::BAD_REQUEST, "username required".into()))?;
    let code = req["code"]
        .as_str()
        .ok_or((StatusCode::BAD_REQUEST, "code required".into()))?;

    let user = state
        .db
        .find_user_by_username(username)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found".into()))?;

    let recovery_codes = state
        .db
        .get_recovery_codes(user.id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    for rc in &recovery_codes {
        if !rc.used && totp::verify_recovery_code(code, &rc.code_hash) {
            state
                .db
                .mark_recovery_code_used(user.id, rc.id)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            let access_token = crate::security::token::create_token(
                &user.id.to_string(),
                &user.role,
                state.secure_jwt_secret.as_str().as_bytes(),
            )
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            let refresh_token = crate::security::token::create_refresh_token(
                &user.id.to_string(),
                &user.role,
                state.secure_jwt_secret.as_str().as_bytes(),
                30,
            )
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            state
                .db
                .reset_failed_attempts(user.id)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            return Ok(Json(serde_json::json!({
                "access_token": access_token,
                "refresh_token": refresh_token,
                "token_type": "Bearer",
                "expires_in": 86400,
            })));
        }
    }

    Err((StatusCode::UNAUTHORIZED, "Invalid recovery code".into()))
}
