use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;

use super::settings::YshConfig;

#[derive(Clone)]
pub struct ConfigLoader;

impl ConfigLoader {
    pub fn new() -> Self {
        Self
    }

    pub fn load_file(&self, path: &str) -> Result<YshConfig> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;

        let raw: RawConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML config: {}", path))?;

        let config = raw.resolve_env_vars()
            .with_context(|| format!("Failed to resolve env vars in config: {}", path))?;

        tracing::info!("Config loaded from {}", path);
        Ok(config)
    }

    pub fn reload(&self, path: &str) -> Result<YshConfig> {
        tracing::info!("Reloading config from: {}", path);
        self.load_file(path)
    }
}

#[derive(Deserialize)]
struct RawConfig {
    secrets: RawSecretsConfig,
    encryption: super::settings::EncryptionConfig,
    password: super::settings::PasswordConfig,
    tls: RawTlsConfig,
    jwt: super::settings::JwtConfig,
    supervision: super::settings::SupervisionConfig,
    server: RawServerConfig,
    database: RawDatabaseConfig,
    backpressure: super::settings::BackpressureConfig,
    rate_limit: super::settings::RateLimitConfig,
    ddos: super::settings::DdosConfig,
    cors: super::settings::CorsConfig,
    economy: super::settings::EconomyConfig,
    #[serde(default)]
    backup: super::settings::BackupConfig,
    #[serde(default)]
    integrity: super::settings::IntegrityConfig,
    #[serde(default)]
    db_encryption: super::settings::DbEncryptionConfig,
    #[serde(default = "default_ai")]
    ai: super::settings::AiConfig,
    #[serde(default = "default_moderation")]
    moderation: super::settings::ModerationConfig,
    #[serde(default = "default_trust")]
    trust: super::settings::TrustConfig,
    #[serde(default = "default_webrtc")]
    webrtc: super::settings::WebRtcConfig,
    #[serde(default = "default_jobs")]
    jobs: super::settings::JobsConfig,
    #[serde(default = "default_analytics")]
    analytics: super::settings::AnalyticsConfig,
}

fn default_jobs() -> super::settings::JobsConfig {
    super::settings::default_jobs()
}

fn default_analytics() -> super::settings::AnalyticsConfig {
    super::settings::default_analytics()
}

fn default_webrtc() -> super::settings::WebRtcConfig {
    super::settings::default_webrtc()
}

fn default_moderation() -> super::settings::ModerationConfig {
    super::settings::default_moderation()
}

fn default_trust() -> super::settings::TrustConfig {
    super::settings::default_trust()
}

fn default_ai() -> super::settings::AiConfig {
    super::settings::default_ai()
}

#[derive(Deserialize)]
struct RawSecretsConfig {
    jwt_secret: EnvValue,
    db_password: EnvValue,
    encryption_key: EnvValue,
}

#[derive(Deserialize)]
struct RawTlsConfig {
    min_version: String,
    cert_path: EnvValue,
    key_path: EnvValue,
}

#[derive(Deserialize)]
struct RawServerConfig {
    host: String,
    port: EnvValueU16,
    workers: usize,
    shutdown_timeout_secs: u32,
}

#[derive(Deserialize)]
struct RawDatabaseConfig {
    url: EnvValueOpt,
    max_connections: u32,
    connect_timeout_secs: u32,
    query_timeout_secs: u32,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum EnvValue {
    Literal(String),
    Reference { env: String, #[serde(default)] default: Option<String> },
}

impl EnvValue {
    fn resolve(self) -> Result<String> {
        match self {
            EnvValue::Literal(s) => Ok(s),
            EnvValue::Reference { env: name, default } => {
                match env::var(&name) {
                    Ok(val) if !val.is_empty() => Ok(val),
                    _ => default.ok_or_else(|| anyhow::anyhow!("Required env var {} not set", name)),
                }
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum EnvValueU16 {
    Literal(u16),
    Reference { env: String, #[serde(default)] default: Option<u16>, #[serde(rename = "type")] _type: Option<String> },
}

impl EnvValueU16 {
    fn resolve(self) -> Result<u16> {
        match self {
            EnvValueU16::Literal(v) => Ok(v),
            EnvValueU16::Reference { env: name, default, .. } => {
                match env::var(&name) {
                    Ok(val) if !val.is_empty() => val.parse().context(format!("Invalid u16 for {}", name)),
                    _ => default.ok_or_else(|| anyhow::anyhow!("Required env var {} not set", name)),
                }
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum EnvValueOpt {
    Literal(String),
    Reference { env: String, #[serde(default)] default: Option<String> },
}

impl EnvValueOpt {
    fn resolve(self) -> String {
        match self {
            EnvValueOpt::Literal(s) => s,
            EnvValueOpt::Reference { env: name, default } => {
                env::var(&name).ok().filter(|v| !v.is_empty()).or(default).unwrap_or_default()
            }
        }
    }
}

impl RawConfig {
    fn resolve_env_vars(self) -> Result<YshConfig> {
        Ok(YshConfig {
            secrets: super::settings::SecretsConfig {
                jwt_secret: self.secrets.jwt_secret.resolve()?,
                db_password: self.secrets.db_password.resolve()?,
                encryption_key: self.secrets.encryption_key.resolve()?,
            },
            encryption: self.encryption,
            password: self.password,
            tls: super::settings::TlsConfig {
                min_version: self.tls.min_version,
                cert_path: self.tls.cert_path.resolve()?,
                key_path: self.tls.key_path.resolve()?,
            },
            jwt: self.jwt,
            supervision: self.supervision,
            server: super::settings::ServerConfig {
                host: self.server.host,
                port: self.server.port.resolve()?,
                workers: self.server.workers,
                shutdown_timeout_secs: self.server.shutdown_timeout_secs,
            },
            database: super::settings::DatabaseConfig {
                url: self.database.url.resolve(),
                max_connections: self.database.max_connections,
                connect_timeout_secs: self.database.connect_timeout_secs,
                query_timeout_secs: self.database.query_timeout_secs,
            },
            backpressure: self.backpressure,
            rate_limit: self.rate_limit,
            ddos: self.ddos,
            cors: self.cors,
            economy: self.economy,
            backup: self.backup,
            integrity: self.integrity,
            db_encryption: self.db_encryption,
            ai: self.ai,
            moderation: self.moderation,
            trust: self.trust,
            webrtc: self.webrtc,
            jobs: self.jobs,
            analytics: self.analytics,
        })
    }
}
