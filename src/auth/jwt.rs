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
        let _path = parts.uri.path().to_string();
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok());

        let Some(auth_header) = auth_header else {
            tracing::warn!(path = %_path, "AUTH-DEBUG: no authorization header");
            return Err(StatusCode::UNAUTHORIZED);
        };

        let token = match auth_header.strip_prefix("Bearer ") {
            Some(t) => t,
            None => {
                tracing::warn!(path = %_path, header = %auth_header, "AUTH-DEBUG: missing Bearer prefix");
                return Err(StatusCode::UNAUTHORIZED);
            }
        };

        let secret =
            std::env::var("YSH_JWT_SECRET").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let claims = match validate_token(token, secret.as_bytes()) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(
                    path = %_path,
                    token_preview = &token[..token.len().min(80)],
                    err = %e,
                    "AUTH-DEBUG: token rejected"
                );
                return Err(StatusCode::UNAUTHORIZED);
            }
        };

        if claims.kind == "2fa_pending" {
            return Err(StatusCode::UNAUTHORIZED);
        }

        Ok(AuthUser {
            user_id: claims.sub,
            role: claims.role,
        })
    }
}
