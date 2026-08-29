use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use serde::{Deserialize, Serialize};

use crate::security::token::validate_token;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub user_id: String,
    pub role: String,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let secret =
            std::env::var("YSH_JWT_SECRET").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let claims =
            validate_token(token, secret.as_bytes()).map_err(|_| StatusCode::UNAUTHORIZED)?;

        if claims.kind == "2fa_pending" {
            return Err(StatusCode::UNAUTHORIZED);
        }

        Ok(AuthUser {
            user_id: claims.sub,
            role: claims.role,
        })
    }
}
