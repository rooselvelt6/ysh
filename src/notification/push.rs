#![allow(dead_code)]

use anyhow::Result;

pub struct PushConfig {
    pub fcm_server_key: String,
    pub fcm_sender_id: String,
}

impl PushConfig {
    pub fn from_env() -> Self {
        Self {
            fcm_server_key: std::env::var("YSH_FCM_SERVER_KEY").unwrap_or_default(),
            fcm_sender_id: std::env::var("YSH_FCM_SENDER_ID").unwrap_or_default(),
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.fcm_server_key.is_empty()
    }
}

pub async fn send_push(
    _config: &PushConfig,
    tokens: &[String],
    title: &str,
    body: &str,
) -> Result<()> {
    if tokens.is_empty() {
        tracing::debug!("No push tokens, skipping push notification");
        return Ok(());
    }

    tracing::info!(
        "Push notification: title='{}' body='{}' tokens={}",
        title,
        body,
        tokens.len()
    );

    for token in tokens {
        tracing::debug!("Push -> {}: {} - {}", token, title, body);
    }

    Ok(())
}
