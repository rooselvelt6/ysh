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
    pub economy: EconomyConfig,
    #[serde(default)]
    pub backup: BackupConfig,
    #[serde(default)]
    pub integrity: IntegrityConfig,
    #[serde(default)]
    pub db_encryption: DbEncryptionConfig,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyConfig {
    pub min_payout: i64,
    pub max_daily_spending: i64,
    pub max_monthly_spending: i64,
    pub commission_tiers: i32,
    pub platform_fee_pct: f64,
    #[serde(default = "default_staking")]
    pub staking: StakingConfig,
    #[serde(default = "default_commission")]
    pub commission: CommissionConfig,
    #[serde(default = "default_fraud")]
    pub fraud: FraudConfig,
    #[serde(default = "default_call_billing")]
    pub call_billing: CallBillingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingConfig {
    pub min_stake: i64,
    pub max_stake: i64,
    pub default_apy: f64,
    pub min_lock_days: i64,
    pub max_lock_days: i64,
    pub reward_calc_interval_hours: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionConfig {
    pub tier1_pct: f64,
    pub tier2_pct: f64,
    pub tier3_pct: f64,
    pub tier4_pct: f64,
    pub min_purchase_for_commission: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudConfig {
    pub velocity_window_secs: i64,
    pub max_tx_per_window: i64,
    pub max_amount_per_window: i64,
    pub large_tx_threshold: i64,
    pub auto_freeze_on_fraud: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallBillingConfig {
    pub min_cost_per_min: i64,
    pub default_cost_per_min: i64,
    pub host_earnings_pct: f64,
}

fn default_staking() -> StakingConfig {
    StakingConfig { min_stake: 100, max_stake: 10000000, default_apy: 0.05, min_lock_days: 1, max_lock_days: 365, reward_calc_interval_hours: 24 }
}
fn default_commission() -> CommissionConfig {
    CommissionConfig { tier1_pct: 0.40, tier2_pct: 0.20, tier3_pct: 0.10, tier4_pct: 0.05, min_purchase_for_commission: 100 }
}
fn default_fraud() -> FraudConfig {
    FraudConfig { velocity_window_secs: 300, max_tx_per_window: 20, max_amount_per_window: 500000, large_tx_threshold: 10000, auto_freeze_on_fraud: true }
}
fn default_call_billing() -> CallBillingConfig {
    CallBillingConfig { min_cost_per_min: 1, default_cost_per_min: 5, host_earnings_pct: 0.70 }
}

// ═══════════════════════════════════════════
// BACKUP CONFIG
// ═══════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub enabled: bool,
    pub interval_secs: u64,
    pub backup_dir: String,
    pub max_backups: usize,
    pub compact_before_backup: bool,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_secs: 3600,
            backup_dir: "./backups".into(),
            max_backups: 7,
            compact_before_backup: true,
        }
    }
}

// ═══════════════════════════════════════════
// INTEGRITY CONFIG
// ═══════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityConfig {
    pub check_on_startup: bool,
    pub auto_repair: bool,
}

impl Default for IntegrityConfig {
    fn default() -> Self {
        Self {
            check_on_startup: true,
            auto_repair: true,
        }
    }
}

// ═══════════════════════════════════════════
// DB ENCRYPTION CONFIG
// ═══════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbEncryptionConfig {
    pub enabled: bool,
    #[serde(default)]
    pub key_env: String,
    #[serde(default)]
    pub key_file: String,
    #[serde(default = "default_db_enc_algorithm")]
    pub algorithm: String,
}

fn default_db_enc_algorithm() -> String {
    "AES-256-GCM".into()
}

impl Default for DbEncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            key_env: "YSH_DB_KEY".into(),
            key_file: "./db_keyfile".into(),
            algorithm: default_db_enc_algorithm(),
        }
    }
}
