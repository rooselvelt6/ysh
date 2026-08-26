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
    pub ddos: DdosConfig,
    pub cors: CorsConfig,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdosConfig {
    pub enabled: bool,
    pub max_body_bytes: usize,
    pub request_timeout_secs: u64,
    #[serde(default)]
    pub trusted_proxies: Vec<String>,
    #[serde(default = "default_ddos_rate_limit")]
    pub rate_limit: DdosRateLimit,
    #[serde(default = "default_ddos_ip_block")]
    pub ip_block: DdosIpBlock,
    #[serde(default = "default_ddos_ws")]
    pub ws: DdosWs,
}

fn default_ddos_rate_limit() -> DdosRateLimit {
    DdosRateLimit { auth_max_per_minute: 5, api_max_per_minute: 60, ws_max_per_minute: 30, admin_max_per_minute: 120 }
}
fn default_ddos_ip_block() -> DdosIpBlock {
    DdosIpBlock { auto_block_threshold: 100, auto_block_window_secs: 60, auto_block_duration_secs: 300, max_blocklist_size: 10000 }
}
fn default_ddos_ws() -> DdosWs {
    DdosWs { max_connections_per_user: 3, max_message_size_bytes: 65536, heartbeat_timeout_secs: 60, message_rate_per_second: 10 }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdosRateLimit {
    pub auth_max_per_minute: u64,
    pub api_max_per_minute: u64,
    pub ws_max_per_minute: u64,
    pub admin_max_per_minute: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdosIpBlock {
    pub auto_block_threshold: u64,
    pub auto_block_window_secs: u64,
    pub auto_block_duration_secs: u64,
    pub max_blocklist_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdosWs {
    pub max_connections_per_user: u32,
    pub max_message_size_bytes: usize,
    pub heartbeat_timeout_secs: u64,
    pub message_rate_per_second: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub max_age_secs: u64,
}
