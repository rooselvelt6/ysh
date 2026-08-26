use axum::{extract::State, http::StatusCode, Json};
use base64::Engine;
use serde::Deserialize;

use crate::actors::session_supervisor::SessionSupervisorMsg;
use crate::security::password::{hash_password, verify_password};
use crate::security::token::{create_refresh_token, create_token};
use crate::server::AppState;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(serde::Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub role: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<UserResponse>, (StatusCode, String)> {
    if req.username.len() < 3 || req.username.len() > 32 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Username must be 3-32 chars".into(),
        ));
    }
    if req.password.len() < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Password must be at least 8 chars".into(),
        ));
    }
    if !req.email.contains('@') {
        return Err((StatusCode::BAD_REQUEST, "Invalid email".into()));
    }

    if state
        .db
        .user_exists(&req.username, &req.email)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    {
        return Err((
            StatusCode::CONFLICT,
            "Username or email already exists".into(),
        ));
    }

    let password_hash = hash_password(&req.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let user = state
        .db
        .create_user(&req.username, &req.email, &password_hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tracing::info!("User registered: {}", req.username);

    Ok(Json(UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        role: user.role,
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user = state
        .db
        .find_user_by_username(&req.username)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Invalid credentials".into()))?;

    if let Some(ref locked_until) = user.locked_until {
        if let Ok(lock_time) =
            chrono::DateTime::parse_from_rfc3339(locked_until)
        {
            if chrono::Utc::now() < lock_time.with_timezone(&chrono::Utc) {
                return Err((
                    StatusCode::TOO_MANY_REQUESTS,
                    "Account locked. Try again later.".into(),
                ));
            }
        }
    }

    let valid = verify_password(&req.password, &user.password_hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !valid {
        let new_attempts = user.failed_login_attempts + 1;
        if new_attempts >= 5 {
            let lock_until =
                (chrono::Utc::now() + chrono::Duration::minutes(15)).to_rfc3339();
            state
                .db
                .lock_account(user.id, &lock_until)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
        state
            .db
            .set_failed_attempts(user.id, new_attempts)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".into()));
    }

    state
        .db
        .reset_failed_attempts(user.id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let _ = state
        .session_actor
        .send_message(SessionSupervisorMsg::SessionStarted {
            user_id: user.id.to_string(),
        });

    let user_agent = "unknown".to_string();
    let fingerprint = crate::security::device::compute_fingerprint(&user_agent, "", "");
    let _ = state.db.store_device(user.id, &fingerprint, &user_agent);

    if user.totp_enabled {
        let temp_token = crate::security::token::create_token_with_kind(
            &user.id.to_string(),
            &user.role,
            state.secure_jwt_secret.as_str().as_bytes(),
            "2fa_pending",
            300,
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        return Ok(Json(serde_json::json!({
            "requires_2fa": true,
            "temp_token": temp_token,
        })));
    }

    tracing::info!(
        "User logged in: {} (created_at: {})",
        req.username,
        user.created_at
    );

    let access_token = create_token(
        &user.id.to_string(),
        &user.role,
        state.secure_jwt_secret.as_str().as_bytes(),
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let refresh_token = create_refresh_token(
        &user.id.to_string(),
        &user.role,
        state.secure_jwt_secret.as_str().as_bytes(),
        30,
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "access_token": access_token,
        "refresh_token": refresh_token,
        "token_type": "Bearer",
        "expires_in": 86400,
    })))
}

pub async fn me(auth: crate::auth::jwt::AuthUser) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "user_id": auth.user_id,
        "role": auth.role,
    }))
}

#[derive(Deserialize)]
pub struct CryptoRequest {
    pub data: String,
}

pub async fn encrypt_message(
    State(state): State<AppState>,
    Json(req): Json<CryptoRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    use crate::security::crypto::Cipher;

    let key_bytes: [u8; 32] = {
        let key_str = state.secure_encryption_key.as_bytes();
        let mut key = [0u8; 32];
        let len = key_str.len().min(32);
        key[..len].copy_from_slice(&key_str[..len]);
        key
    };

    let cipher = match state.encrypted_key.algorithm() {
        "aes-256-gcm" => Cipher::Aes(
            crate::security::crypto::AesCipher::new(&key_bytes)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
        ),
        "chacha20-poly1305" => Cipher::ChaCha(
            crate::security::crypto::ChaChaCipher::new(&key_bytes)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
        ),
        _ => return Err((StatusCode::BAD_REQUEST, "Unknown algorithm".into())),
    };

    let nonce = {
        let mut n = [0u8; 12];
        use rand_core::OsRng;
        use rand_core::RngCore;
        OsRng.fill_bytes(&mut n);
        n
    };

    let ciphertext = match &cipher {
        Cipher::Aes(c) => c
            .encrypt(&nonce, req.data.as_bytes(), b"")
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
        Cipher::ChaCha(c) => c
            .encrypt(&nonce, req.data.as_bytes(), b"")
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
    };

    Ok(Json(serde_json::json!({
        "ciphertext": base64::engine::general_purpose::STANDARD.encode(&ciphertext),
        "nonce": base64::engine::general_purpose::STANDARD.encode(&nonce),
        "algorithm": state.config.encryption.algorithm,
    })))
}

pub async fn decrypt_message(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    use crate::security::crypto::Cipher;

    let ciphertext_b64 = req["ciphertext"]
        .as_str()
        .ok_or((StatusCode::BAD_REQUEST, "Missing ciphertext".into()))?;
    let nonce_b64 = req["nonce"]
        .as_str()
        .ok_or((StatusCode::BAD_REQUEST, "Missing nonce".into()))?;

    let ciphertext = base64::engine::general_purpose::STANDARD
        .decode(ciphertext_b64)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let nonce_bytes = base64::engine::general_purpose::STANDARD
        .decode(nonce_b64)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let nonce: [u8; 12] = nonce_bytes
        .try_into()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid nonce length".into()))?;

    let key_bytes: [u8; 32] = {
        let key_str = state.secure_encryption_key.as_bytes();
        let mut key = [0u8; 32];
        let len = key_str.len().min(32);
        key[..len].copy_from_slice(&key_str[..len]);
        key
    };

    let cipher = match state.encrypted_key.algorithm() {
        "aes-256-gcm" => Cipher::Aes(
            crate::security::crypto::AesCipher::new(&key_bytes)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
        ),
        "chacha20-poly1305" => Cipher::ChaCha(
            crate::security::crypto::ChaChaCipher::new(&key_bytes)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
        ),
        _ => return Err((StatusCode::BAD_REQUEST, "Unknown algorithm".into())),
    };

    let plaintext = match &cipher {
        Cipher::Aes(c) => c
            .decrypt(&nonce, &ciphertext, b"")
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
        Cipher::ChaCha(c) => c
            .decrypt(&nonce, &ciphertext, b"")
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
    };

    let plaintext_str = String::from_utf8(plaintext)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "plaintext": plaintext_str,
    })))
}

pub async fn verify_2fa_login(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let temp_token = req["temp_token"]
        .as_str()
        .ok_or((StatusCode::BAD_REQUEST, "temp_token required".into()))?;
    let code = req["code"]
        .as_str()
        .ok_or((StatusCode::BAD_REQUEST, "code required".into()))?;

    let claims = crate::security::token::validate_token(
        temp_token,
        state.secure_jwt_secret.as_str().as_bytes(),
    )
    .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid or expired temp token".into()))?;

    if claims.kind != "2fa_pending" {
        return Err((StatusCode::UNAUTHORIZED, "Invalid token type".into()));
    }

    let user_id: i64 = claims
        .sub
        .parse()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;

    let totp_secret = state
        .db
        .get_totp_secret(user_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "2FA not configured".into()))?;

    let secret_bytes = base64::engine::general_purpose::STANDARD
        .decode(&totp_secret)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !crate::security::totp::verify_code(&secret_bytes, code) {
        return Err((StatusCode::UNAUTHORIZED, "Invalid 2FA code".into()));
    }

    tracing::info!("2FA verified for user: {}", claims.sub);

    let access_token = create_token(
        &claims.sub,
        &claims.role,
        state.secure_jwt_secret.as_str().as_bytes(),
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let refresh_token = create_refresh_token(
        &claims.sub,
        &claims.role,
        state.secure_jwt_secret.as_str().as_bytes(),
        30,
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "access_token": access_token,
        "refresh_token": refresh_token,
        "token_type": "Bearer",
        "expires_in": 86400,
    })))
}
