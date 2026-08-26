#![allow(dead_code)]

use anyhow::Result;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub from_name: String,
    pub from_email: String,
}

impl EmailConfig {
    pub fn from_env() -> Self {
        Self {
            smtp_host: std::env::var("YSH_SMTP_HOST").unwrap_or_else(|_| "localhost".into()),
            smtp_port: std::env::var("YSH_SMTP_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(587),
            smtp_user: std::env::var("YSH_SMTP_USER").unwrap_or_default(),
            smtp_pass: std::env::var("YSH_SMTP_PASS").unwrap_or_default(),
            from_name: std::env::var("YSH_FROM_NAME").unwrap_or_else(|_| "YSH".into()),
            from_email: std::env::var("YSH_FROM_EMAIL")
                .unwrap_or_else(|_| "noreply@ysh.app".into()),
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.smtp_host.is_empty() && !self.smtp_user.is_empty()
    }
}

pub async fn send_email(
    config: &EmailConfig,
    to_email: &str,
    to_name: &str,
    subject: &str,
    html_body: &str,
) -> Result<()> {
    if !config.is_configured() {
        tracing::debug!(
            "SMTP not configured, skipping email to {} (subject: {})",
            to_email,
            subject
        );
        return Ok(());
    }

    let from = format!("{} <{}>", config.from_name, config.from_email);
    let to = format!("{} <{}>", to_name, to_email);

    let email = Message::builder()
        .from(from.parse()?)
        .to(to.parse()?)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(html_body.to_string())?;

    let creds = Credentials::new(config.smtp_user.clone(), config.smtp_pass.clone());

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)?
        .port(config.smtp_port)
        .credentials(creds)
        .build();

    match mailer.send(email).await {
        Ok(_) => {
            tracing::info!("Email sent to {}: {}", to_email, subject);
            Ok(())
        }
        Err(e) => {
            tracing::error!("Email send failed to {}: {}", to_email, e);
            Err(anyhow::anyhow!("SMTP error: {}", e))
        }
    }
}

pub fn render_welcome(to: &str, verify_url: &str) -> (String, String, String) {
    let (subject, html) = super::templates::welcome_email(to, verify_url);
    (to.to_string(), subject, html)
}

pub fn render_verify(to: &str, verify_url: &str) -> (String, String, String) {
    let (subject, html) = super::templates::verify_email(to, verify_url);
    (to.to_string(), subject, html)
}

pub fn render_reset(to: &str, reset_url: &str) -> (String, String, String) {
    let (subject, html) = super::templates::reset_password(to, reset_url);
    (to.to_string(), subject, html)
}

pub fn render_gift_received(to: &str, from: &str, gift: &str) -> (String, String, String) {
    let (subject, html) = super::templates::gift_received(to, from, gift);
    (to.to_string(), subject, html)
}

pub fn render_call_missed(to: &str, caller: &str) -> (String, String, String) {
    let (subject, html) = super::templates::call_missed(to, caller);
    (to.to_string(), subject, html)
}

pub fn render_moment_liked(to: &str, liker: &str) -> (String, String, String) {
    let (subject, html) = super::templates::moment_liked(to, liker);
    (to.to_string(), subject, html)
}

pub fn render_digest(to: &str, stats: &str) -> (String, String, String) {
    let (subject, html) = super::templates::weekly_digest(to, stats);
    (to.to_string(), subject, html)
}
