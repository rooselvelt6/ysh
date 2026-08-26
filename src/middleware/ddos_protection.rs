use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    middleware::Next,
};
use std::sync::Arc;

use crate::config::settings::DdosConfig;
use crate::middleware::ip_blocklist::IpBlocklist;

#[derive(Clone)]
pub struct DdosProtection {
    pub config: Arc<DdosConfig>,
    pub blocklist: Arc<IpBlocklist>,
}

impl DdosProtection {
    pub fn new(config: Arc<DdosConfig>, blocklist: Arc<IpBlocklist>) -> Self {
        Self { config, blocklist }
    }
}

pub async fn ddos_middleware(
    axum::extract::State(state): axum::extract::State<DdosProtection>,
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    if !state.config.enabled {
        return Ok(next.run(request).await);
    }

    let ip = extract_client_ip(&request);
    if state.blocklist.is_blocked(&ip) {
        tracing::warn!("Blocked IP: {}", ip);
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}

pub fn extract_client_ip(request: &Request<Body>) -> String {
    if let Some(forwarded) = request.headers().get("x-forwarded-for") {
        if let Ok(s) = forwarded.to_str() {
            if let Some(first) = s.split(',').next() {
                let trimmed = first.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }

    if let Some(real_ip) = request.headers().get("x-real-ip") {
        if let Ok(s) = real_ip.to_str() {
            return s.trim().to_string();
        }
    }

    "unknown".to_string()
}
