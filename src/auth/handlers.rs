use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;

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
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
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
        return Err((StatusCode::BAD_REQUEST, "Username must be 3-32 chars".into()));
    }
    if req.password.len() < 8 {
        return Err((StatusCode::BAD_REQUEST, "Password must be at least 8 chars".into()));
    }
    if !req.email.contains('@') {
        return Err((StatusCode::BAD_REQUEST, "Invalid email".into()));
    }

    if state.db
        .user_exists(&req.username, &req.email)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    {
        return Err((StatusCode::CONFLICT, "Username or email already exists".into()));
    }

    let password_hash =
        hash_password(&req.password).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let user = state.db
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
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let user = state.db
        .find_user_by_username(&req.username)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Invalid credentials".into()))?;

    let valid = verify_password(&req.password, &user.password_hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !valid {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".into()));
    }

    let secret = std::env::var("YSH_JWT_SECRET")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "JWT secret not configured".into()))?;

    let access_token = create_token(&user.id.to_string(), &user.role, secret.as_bytes())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let refresh_token =
        create_refresh_token(&user.id.to_string(), &user.role, secret.as_bytes(), 30)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tracing::info!("User logged in: {}", req.username);

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".into(),
        expires_in: 86400,
    }))
}

pub async fn me(
    auth: crate::auth::jwt::AuthUser,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "user_id": auth.user_id,
        "role": auth.role,
    }))
}
