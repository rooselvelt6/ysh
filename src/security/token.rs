use anyhow::Result;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub role: String,
}

pub fn create_token(user_id: &str, role: &str, secret: &[u8]) -> Result<String> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        exp: (now + Duration::hours(24)).timestamp() as usize,
        iat: now.timestamp() as usize,
        role: role.to_string(),
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )?;
    Ok(token)
}

pub fn create_refresh_token(user_id: &str, role: &str, secret: &[u8], days: u32) -> Result<String> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        exp: (now + Duration::days(days as i64)).timestamp() as usize,
        iat: now.timestamp() as usize,
        role: role.to_string(),
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )?;
    Ok(token)
}

pub fn validate_token(token: &str, secret: &[u8]) -> Result<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}
