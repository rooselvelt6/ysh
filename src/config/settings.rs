use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YshConfig {
    pub secrets: SecretsConfig,
    pub encryption: EncryptionConfig,
    pub password: PasswordConfig,
    pub tls: TlsConfig,
    pub jwt: JwtConfig,
    pub supervision: SupervisionConfig,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub backpressure: BackpressureConfig,
    pub rate_limit: RateLimitConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsConfig {
    pub jwt_secret: String,
    pub db_password: String,
    pub encryption_key: String,
}

impl Zeroize for SecretsConfig {
    fn zeroize(&mut self) {
        self.jwt_secret.zeroize();
        self.db_password.zeroize();
        self.encryption_key.zeroize();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub algorithm: String,
    pub nonce_strategy: String,
    pub key_rotation_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordConfig {
    pub algorithm: String,
    pub memory_cost: u32,
    pub time_cost: u32,
    pub parallelism: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub min_version: String,
    pub cert_path: String,
    pub key_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    pub expiry_hours: u32,
    pub refresh_expiry_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisionConfig {
    pub root: SupervisorStrategy,
    pub infrastructure: SupervisorStrategy,
    pub services: SupervisorStrategy,
    pub security: SupervisorStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisorStrategy {
    pub strategy: String,
    pub max_restarts: u32,
    pub max_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub shutdown_timeout_secs: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub connect_timeout_secs: u32,
    pub query_timeout_secs: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpressureConfig {
    pub server_channel_size: usize,
    pub database_channel_size: usize,
    pub webrtc_channel_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_size: u32,
}

impl YshConfig {
    pub fn from_lua_table(table: mlua::Table) -> Result<Self> {
        let json: serde_json::Value = serde_json::to_value(&table)
            .context("Failed to convert Lua table to JSON")?;

        let config: YshConfig =
            serde_json::from_value(json).context("Failed to deserialize Lua config into YshConfig")?;

        Ok(config)
    }
}
