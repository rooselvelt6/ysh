use anyhow::Result;
use rand::Rng;
use redb::{
    Database as RedbDatabase, MultimapTableDefinition, ReadableDatabase, ReadableMultimapTable,
    ReadableTable, ReadableTableMetadata, TableDefinition,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::sync::Mutex;

// ═══════════════════════════════════════════
// redb TABLE DEFINITIONS
// ═══════════════════════════════════════════

// Key-value tables (primary lookups)
const T_USER: TableDefinition<&str, &str> = TableDefinition::new("users");
const T_WALLET: TableDefinition<&str, &str> = TableDefinition::new("wallets");
const T_PROFILE: TableDefinition<&str, &str> = TableDefinition::new("profiles");
const T_HOST: TableDefinition<&str, &str> = TableDefinition::new("hosts");
const T_AGENCY: TableDefinition<&str, &str> = TableDefinition::new("agencies");
const T_CHAT_SESSION: TableDefinition<&str, &str> = TableDefinition::new("chat_sessions");
const T_NOTIF_PREF: TableDefinition<&str, &str> = TableDefinition::new("notif_preferences");
const T_STAKE: TableDefinition<&str, &str> = TableDefinition::new("staking");
const T_SPENDING: TableDefinition<&str, &str> = TableDefinition::new("spending_limits");
const T_COUNTER: TableDefinition<&str, &str> = TableDefinition::new("counters");
const T_I18N: TableDefinition<&str, &str> = TableDefinition::new("i18n_overrides");

// Multimap tables (one-to-many)
const MM_RECOVERY: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("recovery_codes");
const MM_CONSENT: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("consent_records");
const MM_DEVICE: MultimapTableDefinition<&str, &str> = MultimapTableDefinition::new("devices");
const MM_AGENCY_MEMBER: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("agency_members");
const MM_TRANSACTION: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("transactions");
const MM_GIFT: MultimapTableDefinition<&str, &str> = MultimapTableDefinition::new("gifts");
const MM_MOMENT: MultimapTableDefinition<&str, &str> = MultimapTableDefinition::new("moments");
const MM_MOMENT_LIKE: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("moment_likes");
const MM_MOMENT_COMMENT: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("moment_comments");
const MM_NOTIFICATION: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("notifications");
const MM_PUSH_TOKEN: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("push_tokens");
const MM_MSG: MultimapTableDefinition<&str, &str> = MultimapTableDefinition::new("messages");
const MM_CHAT_PARTICIPANT: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("chat_participants");
const MM_MATCH_QUEUE: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("matching_queue");
const MM_STAKING: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("staking_list");
const MM_REFERRAL: MultimapTableDefinition<&str, &str> = MultimapTableDefinition::new("referrals");
const MM_CALL_BILLING: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("call_billing");
const MM_COMMISSION: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("commissions");
const MM_PAYOUT: MultimapTableDefinition<&str, &str> = MultimapTableDefinition::new("payouts");
const MM_FRAUD: MultimapTableDefinition<&str, &str> = MultimapTableDefinition::new("fraud_alerts");
const MM_RECEIPT: MultimapTableDefinition<&str, &str> = MultimapTableDefinition::new("receipts");
const MM_NFT: MultimapTableDefinition<&str, &str> = MultimapTableDefinition::new("nft_gifts");
const MM_GIFT_CATALOG: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("gift_catalog");

// Index tables
const IX_USER_BY_USERNAME: TableDefinition<&str, &str> = TableDefinition::new("ix_user_username");
const IX_USER_BY_EMAIL: TableDefinition<&str, &str> = TableDefinition::new("ix_user_email");
const IX_TX_USER: MultimapTableDefinition<&str, &str> = MultimapTableDefinition::new("ix_tx_user");
const IX_GIFT_FROM: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("ix_gift_from");
const IX_GIFT_TO: MultimapTableDefinition<&str, &str> = MultimapTableDefinition::new("ix_gift_to");
const IX_REFERRAL_CODE: TableDefinition<&str, &str> = TableDefinition::new("ix_referral_code");
const IX_NOTIF_USER: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("ix_notif_user");
const IX_MSG_SESSION: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("ix_msg_session");
const IX_CHAT_USER: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("ix_chat_user");
const IX_NFT_USER: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("ix_nft_user");

// Phase 13: Social + Moderation tables
const T_TRUST: TableDefinition<&str, &str> = TableDefinition::new("trust_scores");
const T_REPUTATION: TableDefinition<&str, &str> = TableDefinition::new("reputation");
const MM_BLOCK: MultimapTableDefinition<&str, &str> = MultimapTableDefinition::new("user_blocks");
const MM_REPORT: MultimapTableDefinition<&str, &str> = MultimapTableDefinition::new("reports");
const MM_BADGE: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("verification_badges");
const MM_RATING: MultimapTableDefinition<&str, &str> = MultimapTableDefinition::new("user_ratings");
const MM_CONTENT_FLAG: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("content_flags");
const MM_MOD_QUEUE: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("moderation_queue");
const MM_APPEAL: MultimapTableDefinition<&str, &str> = MultimapTableDefinition::new("appeals");
const MM_SHADOW: MultimapTableDefinition<&str, &str> = MultimapTableDefinition::new("shadow_bans");

// Phase 8: WebRTC + Streaming tables
const T_CALL: TableDefinition<&str, &str> = TableDefinition::new("calls");
const MM_CALL_USER: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("mm_call_user");
const MM_CALL_QUALITY: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("call_quality");
const MM_CALL_RECORDING: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("call_recordings");
const MM_USER_ACTIVITY: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("user_activity");
const MM_ANALYTICS_DAY: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("analytics_days");

// ═══════════════════════════════════════════
// DATA STRUCTS (serde)
// ═══════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: String,
    pub failed_login_attempts: i32,
    pub locked_until: Option<String>,
    pub totp_secret: Option<String>,
    pub totp_enabled: bool,
    pub kyc_level: i32,
    pub do_not_sell: bool,
    #[serde(default)]
    pub region: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryCode {
    pub id: i64,
    pub code_hash: String,
    pub used: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    pub id: i64,
    pub consent_type: String,
    pub granted: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub user_id: i64,
    pub balance: i64,
    pub frozen: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: i64,
    pub user_id: i64,
    pub tx_type: String,
    pub amount: i64,
    pub description: String,
    pub target_user_id: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct GiftCatalog {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub price: i64,
    pub rarity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiftRecord {
    pub id: i64,
    pub from_user_id: i64,
    pub to_user_id: i64,
    pub gift_id: i64,
    pub price: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Moment {
    pub id: i64,
    pub user_id: i64,
    pub content: String,
    pub media_url: String,
    pub media_type: String,
    pub likes_count: i64,
    pub comments_count: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: i64,
    pub user_id: i64,
    pub ntype: String,
    pub title: String,
    pub body: String,
    pub data: String,
    pub read: bool,
    pub channel: String,
    pub status: String,
    pub retries: i32,
    pub created_at: String,
    pub sent_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: i64,
    pub session_type: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub session_id: i64,
    pub sender_id: i64,
    pub content: String,
    pub msg_type: String,
    pub encrypted: bool,
    pub read: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Staking {
    pub id: i64,
    pub user_id: i64,
    pub amount: i64,
    pub apy_rate: f64,
    pub status: String,
    pub staked_at: String,
    pub unlocks_at: String,
    pub rewards_earned: i64,
    pub last_reward_calc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Referral {
    pub id: i64,
    pub referrer_id: i64,
    pub referred_id: i64,
    pub code: String,
    pub status: String,
    pub referred_at: String,
    pub first_purchase_at: Option<String>,
    pub total_earned: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct CallBilling {
    pub id: i64,
    pub caller_id: i64,
    pub host_id: i64,
    pub call_type: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_secs: i64,
    pub cost_per_min: i64,
    pub total_cost: i64,
    pub host_earnings: i64,
    pub platform_fee: i64,
    pub status: String,
    #[serde(default)]
    pub paid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commission {
    pub id: i64,
    pub user_id: i64,
    pub source_user_id: i64,
    pub source_tx_id: Option<i64>,
    pub tier: i32,
    pub percentage: f64,
    pub amount: i64,
    pub status: String,
    pub created_at: String,
    pub paid_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payout {
    pub id: i64,
    pub user_id: i64,
    pub amount: i64,
    pub currency: String,
    pub wallet_address: String,
    pub network: String,
    pub status: String,
    pub tx_hash: Option<String>,
    pub requested_at: String,
    pub processed_at: Option<String>,
    pub admin_id: Option<i64>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudAlert {
    pub id: i64,
    pub user_id: Option<i64>,
    pub alert_type: String,
    pub severity: String,
    pub description: String,
    pub evidence: String,
    pub status: String,
    pub ip_address: Option<String>,
    pub created_at: String,
    pub resolved_at: Option<String>,
    pub resolved_by: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub id: i64,
    pub user_id: i64,
    pub receipt_type: String,
    pub reference_id: i64,
    pub amount: i64,
    pub currency: String,
    pub description: String,
    pub metadata: String,
    pub receipt_hash: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendingLimit {
    pub user_id: i64,
    pub daily_limit: i64,
    pub monthly_limit: i64,
    pub daily_spent: i64,
    pub monthly_spent: i64,
    pub last_reset_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftGift {
    pub id: i64,
    pub user_id: i64,
    pub gift_id: i64,
    pub gift_record_id: i64,
    pub token_id: String,
    pub unlocked: bool,
    pub minted_at: String,
}

// ═══════════════════════════════════════════
// PHASE 13: SOCIAL + MODERATION STRUCTS
// ═══════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRecord {
    pub blocked_user_id: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: i64,
    pub reporter_id: i64,
    pub target_type: String, // user | moment | message | host | agency
    pub target_id: i64,
    pub category: String, // spam | nsfw | scam | harassment | violence | fraud | other
    pub description: String,
    pub status: String, // pending | reviewed | actioned | dismissed
    pub created_at: String,
    pub reviewed_at: Option<String>,
    pub reviewed_by: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationBadge {
    pub id: i64,
    pub user_id: i64,
    pub badge_type: String, // email_verified | identity_verified | agency | host | staff
    pub granted_at: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rating {
    pub id: i64,
    pub rater_id: i64,
    pub score: f64, // 1.0 – 5.0
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentFlag {
    pub id: i64,
    pub target_type: String, // moment | message | user | host
    pub target_id: i64,
    pub flag_type: String, // nsfw | spam | scam | abuse | other
    pub source: String,    // auto | manual
    pub severity: f64,     // 0.0 – 1.0
    pub description: String,
    pub status: String, // pending | reviewed | actioned | dismissed
    pub created_at: String,
    pub resolved_at: Option<String>,
    pub resolved_by: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModQueueItem {
    pub id: i64,
    pub item_type: String, // report | content_flag | appeal | user
    pub reference_id: i64,
    pub severity: f64,
    pub status: String, // pending | reviewed | actioned | dismissed
    pub notes: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Appeal {
    pub id: i64,
    pub user_id: i64,
    pub target_type: String, // ban | shadow_ban | content_flag
    pub target_id: i64,
    pub reason: String,
    pub status: String, // open | approved | rejected
    pub created_at: String,
    pub reviewed_at: Option<String>,
    pub reviewed_by: Option<i64>,
    pub admin_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowBan {
    pub user_id: i64,
    pub banned_at: String,
    pub reason: String,
    pub banned_until: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationSummary {
    pub user_id: i64,
    pub rating_avg: f64,
    pub rating_count: i64,
}

// ═══════════════════════════════════════════
// PHASE 8: WEBRTC + STREAMING STRUCTS
// ═══════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallRecord {
    pub call_id: String,
    pub room_id: String,
    pub host_id: i64,
    pub call_type: String, // p2p | flash | duo | group | live
    pub participants: Vec<i64>,
    pub status: String, // ringing | active | ended
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_secs: i64,
    pub screen_share: bool,
    pub screen_share_user: Option<i64>,
    pub recording: bool,
    #[serde(default)]
    pub recording_encrypted: bool,
    pub billed: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySample {
    pub call_id: String,
    pub user_id: i64,
    pub ts: String,
    pub bitrate_kbps: f64,
    pub packet_loss_pct: f64,
    pub rtt_ms: f64,
    pub resolution: String,
    pub simulcast_tier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallRecording {
    pub call_id: String,
    pub segment_id: i64,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub storage_key: String, // encrypted reference to the recording blob
    pub size_bytes: i64,
    pub encrypted: bool,
    pub status: String, // recording | finalized
}

// ═══════════════════════════════════════════
// INTEGRITY TYPES
// ═══════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct IntegrityReport {
    pub status: IntegrityStatus,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub enum IntegrityStatus {
    Ok,
    Repaired,
    Corrupted,
}

// ═══════════════════════════════════════════
// DATABASE
// ═══════════════════════════════════════════

pub struct Database {
    inner: RedbDatabase,
    #[allow(dead_code)]
    next_id: Mutex<i64>,
    db_path: std::path::PathBuf,
    write_lock: std::sync::Mutex<()>,
    wallet_lock: std::sync::Mutex<()>,
}

fn to_json<T: Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap_or_default()
}

fn now() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn today() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

fn this_month() -> String {
    chrono::Utc::now().format("%Y-%m").to_string()
}

#[allow(dead_code)]
impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let inner = RedbDatabase::create(path)?;
        let db = Self {
            inner,
            next_id: Mutex::new(1),
            db_path: std::path::PathBuf::from(path),
            write_lock: std::sync::Mutex::new(()),
            wallet_lock: std::sync::Mutex::new(()),
        };
        db.init_tables()?;
        db.seed_gift_catalog()?;
        Ok(db)
    }

    /// Open database with AES-256-GCM encryption at rest.
    /// `key` must be a 32-byte encryption key.
    #[allow(dead_code)]
    pub fn new_encrypted(path: &str, key: &[u8; 32]) -> Result<Self> {
        let backend = crate::encryption::EncryptedBackend::open(path, key)?;
        let inner = RedbDatabase::builder().create_with_backend(backend)?;
        let db = Self {
            inner,
            next_id: Mutex::new(1),
            db_path: std::path::PathBuf::from(path),
            write_lock: std::sync::Mutex::new(()),
            wallet_lock: std::sync::Mutex::new(()),
        };
        db.init_tables()?;
        db.seed_gift_catalog()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        let txn = self.inner.begin_write()?;
        // KV tables
        for t in [
            T_USER,
            T_WALLET,
            T_PROFILE,
            T_HOST,
            T_AGENCY,
            T_CHAT_SESSION,
            T_NOTIF_PREF,
            T_STAKE,
            T_SPENDING,
            T_COUNTER,
            T_I18N,
            IX_USER_BY_USERNAME,
            IX_USER_BY_EMAIL,
            IX_REFERRAL_CODE,
            T_TRUST,
            T_REPUTATION,
            T_CALL,
        ] {
            txn.open_table(t)?;
        }
        // Multimap tables
        for t in [
            MM_RECOVERY,
            MM_CONSENT,
            MM_DEVICE,
            MM_AGENCY_MEMBER,
            MM_TRANSACTION,
            MM_GIFT,
            MM_MOMENT,
            MM_MOMENT_LIKE,
            MM_MOMENT_COMMENT,
            MM_NOTIFICATION,
            MM_PUSH_TOKEN,
            MM_MSG,
            MM_CHAT_PARTICIPANT,
            MM_MATCH_QUEUE,
            MM_STAKING,
            MM_REFERRAL,
            MM_CALL_BILLING,
            MM_COMMISSION,
            MM_PAYOUT,
            MM_FRAUD,
            MM_RECEIPT,
            MM_NFT,
            MM_GIFT_CATALOG,
            IX_TX_USER,
            IX_GIFT_FROM,
            IX_GIFT_TO,
            IX_NOTIF_USER,
            IX_MSG_SESSION,
            IX_CHAT_USER,
            IX_NFT_USER,
            MM_BLOCK,
            MM_REPORT,
            MM_BADGE,
            MM_RATING,
            MM_CONTENT_FLAG,
            MM_MOD_QUEUE,
            MM_APPEAL,
            MM_SHADOW,
            MM_CALL_USER,
            MM_CALL_QUALITY,
            MM_CALL_RECORDING,
            MM_USER_ACTIVITY,
            MM_ANALYTICS_DAY,
        ] {
            txn.open_multimap_table(t)?;
        }
        txn.commit()?;
        Ok(())
    }

    fn next_seq(&self, table: &str) -> i64 {
        let _lock = self.write_lock.lock().unwrap_or_else(|e| e.into_inner());
        let txn = self.inner.begin_write().unwrap();
        let next = {
            let counters = txn.open_table(T_COUNTER).unwrap();
            let current = counters
                .get(table)
                .unwrap()
                .and_then(|v| v.value().parse::<i64>().ok())
                .unwrap_or(0);
            current + 1
        };
        {
            let mut counters = txn.open_table(T_COUNTER).unwrap();
            counters.insert(table, next.to_string().as_str()).unwrap();
        }
        let _ = txn.commit();
        next
    }

    fn get_json<T: for<'de> Deserialize<'de>>(
        &self,
        table: TableDefinition<&str, &str>,
        key: &str,
    ) -> Result<Option<T>> {
        let txn = self.inner.begin_read()?;
        let t = txn.open_table(table)?;
        match t.get(key)? {
            Some(v) => Ok(Some(serde_json::from_str(v.value())?)),
            None => Ok(None),
        }
    }

    fn put_json<T: Serialize>(
        &self,
        table: TableDefinition<&str, &str>,
        key: &str,
        val: &T,
    ) -> Result<()> {
        let _lock = self
            .write_lock
            .lock()
            .map_err(|_| anyhow::anyhow!("Write lock poisoned"))?;
        let txn = self.inner.begin_write()?;
        {
            let mut t = txn.open_table(table)?;
            t.insert(key, to_json(val).as_str())?;
        }
        txn.commit()?;
        Ok(())
    }

    fn mm_add(
        &self,
        table: MultimapTableDefinition<&str, &str>,
        key: &str,
        val: &str,
    ) -> Result<()> {
        let _lock = self
            .write_lock
            .lock()
            .map_err(|_| anyhow::anyhow!("Write lock poisoned"))?;
        let txn = self.inner.begin_write()?;
        {
            let mut t = txn.open_multimap_table(table)?;
            t.insert(key, val)?;
        }
        txn.commit()?;
        Ok(())
    }

    fn mm_get_all(
        &self,
        table: MultimapTableDefinition<&str, &str>,
        key: &str,
    ) -> Result<Vec<String>> {
        let txn = self.inner.begin_read()?;
        let t = txn.open_multimap_table(table)?;
        let iter = t.get(key)?;
        let mut results = Vec::new();
        for item in iter {
            results.push(item?.value().to_string());
        }
        Ok(results)
    }

    fn mm_count(&self, table: MultimapTableDefinition<&str, &str>, key: &str) -> Result<i64> {
        let txn = self.inner.begin_read()?;
        let t = txn.open_multimap_table(table)?;
        let iter = t.get(key)?;
        let mut count = 0;
        for _ in iter {
            count += 1;
        }
        Ok(count)
    }

    fn mm_remove_all(&self, table: MultimapTableDefinition<&str, &str>, key: &str) -> Result<()> {
        let _lock = self
            .write_lock
            .lock()
            .map_err(|_| anyhow::anyhow!("Write lock poisoned"))?;
        let txn = self.inner.begin_write()?;
        {
            let mut t = txn.open_multimap_table(table)?;
            t.remove_all(key)?;
        }
        txn.commit()?;
        Ok(())
    }

    fn mm_remove_one(
        &self,
        table: MultimapTableDefinition<&str, &str>,
        key: &str,
        val: &str,
    ) -> Result<bool> {
        let _lock = self
            .write_lock
            .lock()
            .map_err(|_| anyhow::anyhow!("Write lock poisoned"))?;
        let txn = self.inner.begin_write()?;
        let removed = {
            let mut t = txn.open_multimap_table(table)?;
            t.remove(key, val)?
        };
        txn.commit()?;
        Ok(removed)
    }

    fn mm_get_all_entries(
        &self,
        table: MultimapTableDefinition<&str, &str>,
    ) -> Result<Vec<(String, String)>> {
        let txn = self.inner.begin_read()?;
        let t = txn.open_multimap_table(table)?;
        let mut results = Vec::new();
        for entry in t.iter()? {
            let (k, mut vals) = entry?;
            for v in &mut vals {
                results.push((k.value().to_string(), v?.value().to_string()));
            }
        }
        Ok(results)
    }

    // ═══════════════════════════════════════════
    // USER FUNCTIONS
    // ═══════════════════════════════════════════

    pub fn create_user(&self, username: &str, email: &str, password: &str) -> Result<User> {
        if self.user_exists(username, email)? {
            anyhow::bail!("username or email already exists");
        }
        let password_hash = crate::security::password::hash_password(password)?;
        let id = self.next_seq("users");
        let user = User {
            id,
            username: username.to_string(),
            email: email.to_string(),
            password_hash,
            role: "user".to_string(),
            created_at: now(),
            failed_login_attempts: 0,
            locked_until: None,
            totp_secret: None,
            totp_enabled: false,
            kyc_level: 0,
            do_not_sell: false,
            region: "unknown".to_string(),
        };
        self.put_json(T_USER, &id.to_string(), &user)?;
        self.put_json(IX_USER_BY_USERNAME, username, &id.to_string())?;
        self.put_json(IX_USER_BY_EMAIL, email, &id.to_string())?;
        Ok(user)
    }

    pub fn find_user_by_username(&self, username: &str) -> Result<Option<User>> {
        match self.get_json::<String>(IX_USER_BY_USERNAME, username)? {
            Some(id_str) => {
                let id: i64 = id_str.parse()?;
                self.get_json(T_USER, &id.to_string())
            }
            None => Ok(None),
        }
    }

    pub fn find_user_by_id(&self, user_id: i64) -> Result<Option<User>> {
        self.get_json(T_USER, &user_id.to_string())
    }

    pub fn user_exists(&self, username: &str, email: &str) -> Result<bool> {
        let has_user = self
            .get_json::<String>(IX_USER_BY_USERNAME, username)?
            .is_some();
        let has_email = self.get_json::<String>(IX_USER_BY_EMAIL, email)?.is_some();
        Ok(has_user || has_email)
    }

    pub fn set_failed_attempts(&self, user_id: i64, attempts: i32) -> Result<()> {
        if let Some(mut user) = self.find_user_by_id(user_id)? {
            user.failed_login_attempts = attempts;
            self.put_json(T_USER, &user_id.to_string(), &user)?;
        }
        Ok(())
    }

    pub fn reset_failed_attempts(&self, user_id: i64) -> Result<()> {
        if let Some(mut user) = self.find_user_by_id(user_id)? {
            user.failed_login_attempts = 0;
            user.locked_until = None;
            self.put_json(T_USER, &user_id.to_string(), &user)?;
        }
        Ok(())
    }

    pub fn lock_account(&self, user_id: i64, until: &str) -> Result<()> {
        if let Some(mut user) = self.find_user_by_id(user_id)? {
            user.locked_until = Some(until.to_string());
            self.put_json(T_USER, &user_id.to_string(), &user)?;
        }
        Ok(())
    }

    pub fn set_user_role(&self, user_id: i64, role: &str) -> Result<()> {
        if let Some(mut user) = self.find_user_by_id(user_id)? {
            user.role = role.to_string();
            self.put_json(T_USER, &user_id.to_string(), &user)?;
        }
        Ok(())
    }

    pub fn set_totp_secret(&self, user_id: i64, secret: &str) -> Result<()> {
        if let Some(mut user) = self.find_user_by_id(user_id)? {
            user.totp_secret = Some(secret.to_string());
            self.put_json(T_USER, &user_id.to_string(), &user)?;
        }
        Ok(())
    }

    pub fn get_totp_secret(&self, user_id: i64) -> Result<Option<String>> {
        match self.find_user_by_id(user_id)? {
            Some(u) => Ok(u.totp_secret),
            None => Ok(None),
        }
    }

    pub fn enable_totp(&self, user_id: i64) -> Result<()> {
        if let Some(mut user) = self.find_user_by_id(user_id)? {
            user.totp_enabled = true;
            self.put_json(T_USER, &user_id.to_string(), &user)?;
        }
        Ok(())
    }

    pub fn disable_totp(&self, user_id: i64) -> Result<()> {
        if let Some(mut user) = self.find_user_by_id(user_id)? {
            user.totp_enabled = false;
            user.totp_secret = None;
            self.put_json(T_USER, &user_id.to_string(), &user)?;
        }
        Ok(())
    }

    pub fn store_recovery_codes(&self, user_id: i64, codes: &[(String, bool)]) -> Result<()> {
        self.mm_remove_all(MM_RECOVERY, &user_id.to_string())?;
        for (code_hash, used) in codes {
            let id = self.next_seq("recovery_codes");
            let rc = RecoveryCode {
                id,
                code_hash: code_hash.clone(),
                used: *used,
            };
            self.mm_add(MM_RECOVERY, &user_id.to_string(), &to_json(&rc))?;
        }
        Ok(())
    }

    pub fn get_recovery_codes(&self, user_id: i64) -> Result<Vec<RecoveryCode>> {
        let entries = self.mm_get_all(MM_RECOVERY, &user_id.to_string())?;
        entries
            .iter()
            .filter_map(|s| serde_json::from_str(s).ok())
            .collect::<Vec<_>>()
            .pipe(Ok)
    }

    pub fn mark_recovery_code_used(&self, user_id: i64, code_id: i64) -> Result<()> {
        let entries = self.mm_get_all(MM_RECOVERY, &user_id.to_string())?;
        self.mm_remove_all(MM_RECOVERY, &user_id.to_string())?;
        for entry in &entries {
            if let Ok(mut rc) = serde_json::from_str::<RecoveryCode>(entry) {
                if rc.id == code_id {
                    rc.used = true;
                }
                self.mm_add(MM_RECOVERY, &user_id.to_string(), &to_json(&rc))?;
            }
        }
        Ok(())
    }

    pub fn delete_recovery_codes(&self, user_id: i64) -> Result<()> {
        self.mm_remove_all(MM_RECOVERY, &user_id.to_string())
    }

    pub fn get_user_data(&self, user_id: i64) -> Result<serde_json::Value> {
        let user = self.find_user_by_id(user_id)?.unwrap_or_default();
        Ok(serde_json::json!({
            "user": {
                "id": user.id, "username": user.username, "email": user.email,
                "role": user.role, "created_at": user.created_at,
                "kyc_level": user.kyc_level, "do_not_sell": user.do_not_sell,
            },
            "consent_records": [],
            "devices": [],
        }))
    }

    pub fn delete_user_data(&self, user_id: i64) -> Result<()> {
        self.mm_remove_all(MM_RECOVERY, &user_id.to_string())?;
        self.mm_remove_all(MM_CONSENT, &user_id.to_string())?;
        self.mm_remove_all(MM_DEVICE, &user_id.to_string())?;
        let _lock = self
            .write_lock
            .lock()
            .map_err(|_| anyhow::anyhow!("Write lock poisoned"))?;
        let txn = self.inner.begin_write()?;
        {
            let mut t = txn.open_table(T_USER)?;
            t.remove(user_id.to_string().as_str())?;
        }
        txn.commit()?;
        Ok(())
    }

    pub fn record_consent(&self, user_id: i64, consent_type: &str, granted: bool) -> Result<()> {
        let id = self.next_seq("consent_records");
        let cr = ConsentRecord {
            id,
            consent_type: consent_type.to_string(),
            granted,
            created_at: now(),
        };
        self.mm_add(MM_CONSENT, &user_id.to_string(), &to_json(&cr))
    }

    pub fn get_consent_history(&self, user_id: i64) -> Result<Vec<ConsentRecord>> {
        self.mm_get_all(MM_CONSENT, &user_id.to_string())?
            .iter()
            .filter_map(|s| serde_json::from_str(s).ok())
            .collect::<Vec<_>>()
            .pipe(Ok)
    }

    pub fn set_do_not_sell(&self, user_id: i64, value: bool) -> Result<()> {
        if let Some(mut user) = self.find_user_by_id(user_id)? {
            user.do_not_sell = value;
            self.put_json(T_USER, &user_id.to_string(), &user)?;
        }
        Ok(())
    }

    pub fn set_kyc_level(&self, user_id: i64, level: i32) -> Result<()> {
        if let Some(mut user) = self.find_user_by_id(user_id)? {
            user.kyc_level = level;
            self.put_json(T_USER, &user_id.to_string(), &user)?;
        }
        Ok(())
    }

    pub fn store_device(&self, user_id: i64, fingerprint: &str, user_agent: &str) -> Result<()> {
        let device = serde_json::json!({
            "fingerprint": fingerprint, "user_agent": user_agent,
            "last_seen": now(), "created_at": now()
        });
        self.mm_add(MM_DEVICE, &user_id.to_string(), &to_json(&device))
    }

    pub fn health_check(&self) -> Result<()> {
        let _ = self.inner.begin_read()?;
        Ok(())
    }

    // ═══════════════════════════════════════════
    // INTEGRITY CHECK (Fix 4)
    // ═══════════════════════════════════════════

    #[allow(dead_code)]
    pub fn check_integrity(&self) -> Result<IntegrityReport> {
        let mut db = RedbDatabase::create(&self.db_path)?;
        let result = db.check_integrity()?;
        let (status, message) = match result {
            true => (
                IntegrityStatus::Ok,
                "Database integrity check passed".into(),
            ),
            false => (
                IntegrityStatus::Repaired,
                "Database was repaired during integrity check".into(),
            ),
        };
        Ok(IntegrityReport { status, message })
    }

    // ═══════════════════════════════════════════
    // BACKUP / SNAPSHOT (Fix 3)
    // ═══════════════════════════════════════════

    #[allow(dead_code)]
    pub fn compact(&self) -> Result<bool> {
        let mut db = RedbDatabase::create(&self.db_path)?;
        let performed = db.compact()?;
        Ok(performed)
    }

    #[allow(dead_code)]
    pub fn backup(&self, dest: impl AsRef<std::path::Path>) -> Result<u64> {
        let bytes_copied = std::fs::copy(&self.db_path, dest.as_ref())?;
        tracing::info!(
            src = %self.db_path.display(),
            dest = %dest.as_ref().display(),
            bytes = bytes_copied,
            "Database backup created"
        );
        Ok(bytes_copied)
    }

    #[allow(dead_code)]
    pub fn backup_with_compact(&self, dest: impl AsRef<std::path::Path>) -> Result<u64> {
        self.compact()?;
        self.backup(dest)
    }

    #[allow(dead_code)]
    pub fn db_path(&self) -> &std::path::Path {
        &self.db_path
    }

    pub fn user_count(&self) -> Result<i64> {
        let txn = self.inner.begin_read()?;
        let t = txn.open_table(T_USER)?;
        let mut count = 0;
        for _ in t.iter()? {
            count += 1;
        }
        Ok(count)
    }

    pub fn session_count(&self) -> Result<i64> {
        let txn = self.inner.begin_read()?;
        let t = txn.open_table(T_CHAT_SESSION)?;
        let mut count = 0;
        for _ in t.iter()? {
            count += 1;
        }
        Ok(count)
    }

    pub fn update_user_profile(
        &self,
        user_id: i64,
        display_name: &str,
        bio: &str,
        avatar_url: &str,
        country: &str,
    ) -> Result<()> {
        let profile = serde_json::json!({
            "user_id": user_id, "display_name": display_name, "bio": bio,
            "avatar_url": avatar_url, "country": country, "created_at": now()
        });
        self.put_json(T_PROFILE, &user_id.to_string(), &profile)
    }

    pub fn get_profile(&self, user_id: i64) -> Result<Option<serde_json::Value>> {
        self.get_json(T_PROFILE, &user_id.to_string())
    }

    pub fn search_users(&self, query: &str, limit: i64) -> Result<Vec<serde_json::Value>> {
        let txn = self.inner.begin_read()?;
        let t = txn.open_table(T_USER)?;
        let mut results = Vec::new();
        let pattern = query.to_lowercase();
        drop(t);
        drop(txn);
        let shadow_ids = self.active_shadow_ban_ids()?;
        let txn = self.inner.begin_read()?;
        let t = txn.open_table(T_USER)?;
        for entry in t.iter()? {
            let (_, v) = entry?;
            if let Ok(user) = serde_json::from_str::<User>(v.value()) {
                if shadow_ids.contains(&user.id) {
                    continue;
                }
                if user.username.to_lowercase().contains(&pattern) {
                    results.push(serde_json::json!({
                        "id": user.id, "username": user.username, "created_at": user.created_at
                    }));
                    if results.len() as i64 >= limit {
                        break;
                    }
                }
            }
        }
        Ok(results)
    }

    pub fn ban_user(&self, user_id: i64) -> Result<()> {
        self.lock_account(user_id, "2099-12-31T23:59:59Z")
    }

    pub fn unban_user(&self, user_id: i64) -> Result<()> {
        self.reset_failed_attempts(user_id)
    }

    pub fn list_users(&self, offset: i64, limit: i64) -> Result<Vec<serde_json::Value>> {
        let txn = self.inner.begin_read()?;
        let t = txn.open_table(T_USER)?;
        let mut results = Vec::new();
        let mut skipped = 0;
        for entry in t.iter()? {
            let (_, v) = entry?;
            if let Ok(user) = serde_json::from_str::<User>(v.value()) {
                if skipped < offset {
                    skipped += 1;
                    continue;
                }
                results.push(serde_json::json!({
                    "id": user.id, "username": user.username, "email": user.email,
                    "role": user.role, "created_at": user.created_at, "kyc_level": user.kyc_level,
                }));
                if results.len() as i64 >= limit {
                    break;
                }
            }
        }
        Ok(results)
    }

    // ═══════════════════════════════════════════
    // AGENCY
    // ═══════════════════════════════════════════

    pub fn create_agency(&self, owner_id: i64, name: &str, description: &str) -> Result<i64> {
        let id = self.next_seq("agencies");
        let agency = serde_json::json!({
            "id": id, "owner_id": owner_id, "name": name, "description": description, "created_at": now()
        });
        self.put_json(T_AGENCY, &id.to_string(), &agency)?;
        Ok(id)
    }

    pub fn get_agency(&self, agency_id: i64) -> Result<Option<serde_json::Value>> {
        self.get_json(T_AGENCY, &agency_id.to_string())
    }

    pub fn list_agencies(&self) -> Result<Vec<serde_json::Value>> {
        let txn = self.inner.begin_read()?;
        let t = txn.open_table(T_AGENCY)?;
        let mut results = Vec::new();
        for entry in t.iter()? {
            let (_, v) = entry?;
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(v.value()) {
                results.push(val);
            }
        }
        Ok(results)
    }

    pub fn add_agency_member(&self, agency_id: i64, user_id: i64, role: &str) -> Result<()> {
        let member = serde_json::json!({"user_id": user_id, "role": role, "joined_at": now()});
        self.mm_add(MM_AGENCY_MEMBER, &agency_id.to_string(), &to_json(&member))
    }

    pub fn get_agency_members(&self, agency_id: i64) -> Result<Vec<serde_json::Value>> {
        self.mm_get_all(MM_AGENCY_MEMBER, &agency_id.to_string())?
            .iter()
            .filter_map(|s| serde_json::from_str(s).ok())
            .collect::<Vec<_>>()
            .pipe(Ok)
    }

    // ═══════════════════════════════════════════
    // HOST
    // ═══════════════════════════════════════════

    pub fn create_host_profile(
        &self,
        user_id: i64,
        languages: &str,
        hourly_rate: i64,
    ) -> Result<()> {
        let host = serde_json::json!({
            "user_id": user_id, "languages": languages, "hourly_rate": hourly_rate,
            "available": false, "total_calls": 0, "rating": 0.0
        });
        self.put_json(T_HOST, &user_id.to_string(), &host)
    }

    pub fn get_host_profile(&self, user_id: i64) -> Result<Option<serde_json::Value>> {
        self.get_json(T_HOST, &user_id.to_string())
    }

    pub fn set_host_availability(&self, user_id: i64, available: bool) -> Result<()> {
        if let Some(mut host) = self.get_json::<serde_json::Value>(T_HOST, &user_id.to_string())? {
            host["available"] = serde_json::json!(available);
            self.put_json(T_HOST, &user_id.to_string(), &host)?;
        }
        Ok(())
    }

    pub fn list_hosts(&self, available_only: bool) -> Result<Vec<serde_json::Value>> {
        let txn = self.inner.begin_read()?;
        let t = txn.open_table(T_HOST)?;
        let mut results = Vec::new();
        for entry in t.iter()? {
            let (_, v) = entry?;
            if let Ok(host) = serde_json::from_str::<serde_json::Value>(v.value()) {
                if available_only && host["available"] != serde_json::json!(true) {
                    continue;
                }
                results.push(host);
            }
        }
        Ok(results)
    }

    // ═══════════════════════════════════════════
    // WALLET
    // ═══════════════════════════════════════════

    pub fn ensure_wallet(&self, user_id: i64) -> Result<()> {
        if self
            .get_json::<Wallet>(T_WALLET, &user_id.to_string())?
            .is_none()
        {
            let w = Wallet {
                user_id,
                balance: 0,
                frozen: false,
                created_at: now(),
                updated_at: now(),
            };
            self.put_json(T_WALLET, &user_id.to_string(), &w)?;
        }
        Ok(())
    }

    pub fn get_balance(&self, user_id: i64) -> Result<i64> {
        match self.get_json::<Wallet>(T_WALLET, &user_id.to_string())? {
            Some(w) => Ok(w.balance),
            None => Ok(0),
        }
    }

    pub fn deposit(&self, user_id: i64, amount: i64, description: &str) -> Result<i64> {
        let _lock = self
            .wallet_lock
            .lock()
            .map_err(|_| anyhow::anyhow!("Wallet lock poisoned"))?;
        self.ensure_wallet(user_id)?;
        let mut w = self
            .get_json::<Wallet>(T_WALLET, &user_id.to_string())?
            .unwrap();
        w.balance += amount;
        w.updated_at = now();
        self.put_json(T_WALLET, &user_id.to_string(), &w)?;
        let tx_id = self.next_seq("transactions");
        let tx = Transaction {
            id: tx_id,
            user_id,
            tx_type: "deposit".into(),
            amount,
            description: description.into(),
            target_user_id: None,
            created_at: now(),
        };
        self.mm_add(MM_TRANSACTION, &user_id.to_string(), &to_json(&tx))?;
        Ok(w.balance)
    }

    pub fn withdraw(&self, user_id: i64, amount: i64, description: &str) -> Result<i64> {
        let _lock = self
            .wallet_lock
            .lock()
            .map_err(|_| anyhow::anyhow!("Wallet lock poisoned"))?;
        let mut w = self
            .get_json::<Wallet>(T_WALLET, &user_id.to_string())?
            .unwrap_or(Wallet {
                user_id,
                balance: 0,
                frozen: false,
                created_at: now(),
                updated_at: now(),
            });
        if w.balance < amount {
            anyhow::bail!("Insufficient funds: {} < {}", w.balance, amount);
        }
        w.balance -= amount;
        w.updated_at = now();
        self.put_json(T_WALLET, &user_id.to_string(), &w)?;
        let tx_id = self.next_seq("transactions");
        let tx = Transaction {
            id: tx_id,
            user_id,
            tx_type: "withdraw".into(),
            amount,
            description: description.into(),
            target_user_id: None,
            created_at: now(),
        };
        self.mm_add(MM_TRANSACTION, &user_id.to_string(), &to_json(&tx))?;
        Ok(w.balance)
    }

    pub fn transfer(
        &self,
        from_user: i64,
        to_user: i64,
        amount: i64,
        description: &str,
    ) -> Result<()> {
        let _lock = self
            .wallet_lock
            .lock()
            .map_err(|_| anyhow::anyhow!("Wallet lock poisoned"))?;
        let mut from_w = self
            .get_json::<Wallet>(T_WALLET, &from_user.to_string())?
            .unwrap_or(Wallet {
                user_id: from_user,
                balance: 0,
                frozen: false,
                created_at: now(),
                updated_at: now(),
            });
        if from_w.balance < amount {
            anyhow::bail!("Insufficient funds: {} < {}", from_w.balance, amount);
        }
        from_w.balance -= amount;
        from_w.updated_at = now();
        self.put_json(T_WALLET, &from_user.to_string(), &from_w)?;

        self.ensure_wallet(to_user)?;
        let mut to_w = self
            .get_json::<Wallet>(T_WALLET, &to_user.to_string())?
            .unwrap();
        to_w.balance += amount;
        to_w.updated_at = now();
        self.put_json(T_WALLET, &to_user.to_string(), &to_w)?;

        let ts = now();
        let tx_out_id = self.next_seq("transactions");
        let tx_out = Transaction {
            id: tx_out_id,
            user_id: from_user,
            tx_type: "transfer_out".into(),
            amount,
            description: description.into(),
            target_user_id: Some(to_user),
            created_at: ts.clone(),
        };
        self.mm_add(MM_TRANSACTION, &from_user.to_string(), &to_json(&tx_out))?;

        let tx_in_id = self.next_seq("transactions");
        let tx_in = Transaction {
            id: tx_in_id,
            user_id: to_user,
            tx_type: "transfer_in".into(),
            amount,
            description: description.into(),
            target_user_id: Some(from_user),
            created_at: ts,
        };
        self.mm_add(MM_TRANSACTION, &to_user.to_string(), &to_json(&tx_in))?;
        Ok(())
    }

    pub fn get_transactions(&self, user_id: i64, limit: i64) -> Result<Vec<serde_json::Value>> {
        let mut txs: Vec<serde_json::Value> = self
            .mm_get_all(MM_TRANSACTION, &user_id.to_string())?
            .iter()
            .filter_map(|s| serde_json::from_str::<serde_json::Value>(s).ok())
            .collect();
        txs.sort_by(|a, b| b["id"].as_i64().cmp(&a["id"].as_i64()));
        txs.truncate(limit as usize);
        Ok(txs)
    }

    pub fn is_wallet_frozen(&self, user_id: i64) -> Result<bool> {
        match self.get_json::<Wallet>(T_WALLET, &user_id.to_string())? {
            Some(w) => Ok(w.frozen),
            None => Ok(false),
        }
    }

    pub fn freeze_wallet(&self, user_id: i64) -> Result<()> {
        self.ensure_wallet(user_id)?;
        if let Some(mut w) = self.get_json::<Wallet>(T_WALLET, &user_id.to_string())? {
            w.frozen = true;
            self.put_json(T_WALLET, &user_id.to_string(), &w)?;
        }
        Ok(())
    }

    pub fn unfreeze_wallet(&self, user_id: i64) -> Result<()> {
        if let Some(mut w) = self.get_json::<Wallet>(T_WALLET, &user_id.to_string())? {
            w.frozen = false;
            self.put_json(T_WALLET, &user_id.to_string(), &w)?;
        }
        Ok(())
    }

    // ═══════════════════════════════════════════
    // GIFT CATALOG + GIFTS
    // ═══════════════════════════════════════════

    pub fn get_gift_catalog(&self) -> Result<Vec<serde_json::Value>> {
        let txn = self.inner.begin_read()?;
        let t = txn.open_multimap_table(MM_GIFT_CATALOG)?;
        let mut results = Vec::new();
        for entry in t.iter()? {
            let (_, mut vals) = entry?;
            for v in &mut vals {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(v?.value()) {
                    results.push(val);
                }
            }
        }
        results.sort_by_key(|g| g["price"].as_i64().unwrap_or(0));
        Ok(results)
    }

    fn seed_gift_catalog(&self) -> Result<()> {
        let has_items = {
            let txn = self.inner.begin_read()?;
            let t = txn.open_multimap_table(MM_GIFT_CATALOG)?;
            t.iter()?.next().is_some()
        };
        if has_items {
            return Ok(());
        }
        let gifts = vec![
            (1, "Rose", "A single red rose", 10, "common"),
            (2, "Heart", "A golden heart", 50, "common"),
            (3, "Diamond Ring", "A sparkling diamond ring", 200, "rare"),
            (4, "Sports Car", "A virtual sports car", 500, "epic"),
            (5, "Yacht", "A luxury yacht", 1000, "legendary"),
            (
                6,
                "Private Island",
                "Your own virtual island",
                5000,
                "legendary",
            ),
        ];
        for (id, name, desc, price, rarity) in gifts {
            let item = serde_json::json!({"id": id, "name": name, "description": desc, "price": price, "rarity": rarity});
            self.mm_add(MM_GIFT_CATALOG, "catalog", &to_json(&item))?;
        }
        Ok(())
    }

    pub fn send_gift(&self, from_user: i64, to_user: i64, gift_id: i64) -> Result<i64> {
        let catalog = self.get_gift_catalog()?;
        let price = catalog
            .iter()
            .find(|g| g["id"].as_i64() == Some(gift_id))
            .map(|g| g["price"].as_i64().unwrap_or(0))
            .ok_or_else(|| anyhow::anyhow!("Gift not found"))?;

        let mut from_w = self
            .get_json::<Wallet>(T_WALLET, &from_user.to_string())?
            .unwrap();
        if from_w.balance < price {
            anyhow::bail!(
                "Insufficient funds for gift: {} < {}",
                from_w.balance,
                price
            );
        }
        from_w.balance -= price;
        from_w.updated_at = now();
        self.put_json(T_WALLET, &from_user.to_string(), &from_w)?;

        self.ensure_wallet(to_user)?;
        let mut to_w = self
            .get_json::<Wallet>(T_WALLET, &to_user.to_string())?
            .unwrap();
        to_w.balance += price;
        to_w.updated_at = now();
        self.put_json(T_WALLET, &to_user.to_string(), &to_w)?;

        let record_id = self.next_seq("gifts");
        let ts = now();
        let gift = GiftRecord {
            id: record_id,
            from_user_id: from_user,
            to_user_id: to_user,
            gift_id,
            price,
            created_at: ts.clone(),
        };
        self.mm_add(MM_GIFT, &to_user.to_string(), &to_json(&gift))?;
        self.mm_add(MM_GIFT, &format!("from_{}", from_user), &to_json(&gift))?;

        let tx_out_id = self.next_seq("transactions");
        let tx_out = Transaction {
            id: tx_out_id,
            user_id: from_user,
            tx_type: "gift_out".into(),
            amount: price,
            description: format!("Gift #{}", gift_id),
            target_user_id: Some(to_user),
            created_at: ts.clone(),
        };
        self.mm_add(MM_TRANSACTION, &from_user.to_string(), &to_json(&tx_out))?;

        let tx_in_id = self.next_seq("transactions");
        let tx_in = Transaction {
            id: tx_in_id,
            user_id: to_user,
            tx_type: "gift_in".into(),
            amount: price,
            description: format!("Gift #{}", gift_id),
            target_user_id: Some(from_user),
            created_at: ts,
        };
        self.mm_add(MM_TRANSACTION, &to_user.to_string(), &to_json(&tx_in))?;
        Ok(record_id)
    }

    pub fn get_received_gifts(&self, user_id: i64) -> Result<Vec<serde_json::Value>> {
        let entries = self.mm_get_all(MM_GIFT, &user_id.to_string())?;
        let catalog = self.get_gift_catalog()?;
        let mut results = Vec::new();
        for entry in &entries {
            if let Ok(gift) = serde_json::from_str::<GiftRecord>(entry) {
                let gname = catalog
                    .iter()
                    .find(|g| g["id"].as_i64() == Some(gift.gift_id))
                    .and_then(|g| g["name"].as_str().map(String::from))
                    .unwrap_or_default();
                let rarity = catalog
                    .iter()
                    .find(|g| g["id"].as_i64() == Some(gift.gift_id))
                    .and_then(|g| g["rarity"].as_str().map(String::from))
                    .unwrap_or_default();
                let from_user = {
                    let txn = self.inner.begin_read()?;
                    let t = txn.open_table(T_USER)?;
                    if let Some(v) = t.get(gift.from_user_id.to_string().as_str())? {
                        serde_json::from_str::<User>(v.value())
                            .ok()
                            .map(|u| u.username)
                            .unwrap_or_default()
                    } else {
                        String::new()
                    }
                };
                results.push(serde_json::json!({
                    "id": gift.id, "price": gift.price, "name": gname,
                    "rarity": rarity, "from_user": from_user, "created_at": gift.created_at
                }));
            }
        }
        Ok(results)
    }

    pub fn get_sent_gifts(&self, user_id: i64) -> Result<Vec<serde_json::Value>> {
        let entries = self.mm_get_all(MM_GIFT, &format!("from_{}", user_id))?;
        let catalog = self.get_gift_catalog()?;
        let mut results = Vec::new();
        for entry in &entries {
            if let Ok(gift) = serde_json::from_str::<GiftRecord>(entry) {
                let gname = catalog
                    .iter()
                    .find(|g| g["id"].as_i64() == Some(gift.gift_id))
                    .and_then(|g| g["name"].as_str().map(String::from))
                    .unwrap_or_default();
                let rarity = catalog
                    .iter()
                    .find(|g| g["id"].as_i64() == Some(gift.gift_id))
                    .and_then(|g| g["rarity"].as_str().map(String::from))
                    .unwrap_or_default();
                let to_user = self
                    .find_user_by_id(gift.to_user_id)?
                    .map(|u| u.username)
                    .unwrap_or_default();
                results.push(serde_json::json!({
                    "id": gift.id, "price": gift.price, "name": gname,
                    "rarity": rarity, "to_user": to_user, "created_at": gift.created_at
                }));
            }
        }
        Ok(results)
    }

    pub fn get_gift_stats(&self, user_id: i64) -> Result<serde_json::Value> {
        let sent_entries = self.mm_get_all(MM_GIFT, &format!("from_{}", user_id))?;
        let recv_entries = self.mm_get_all(MM_GIFT, &user_id.to_string())?;
        let total_spent: i64 = sent_entries
            .iter()
            .filter_map(|s| serde_json::from_str::<GiftRecord>(s).ok())
            .map(|g| g.price)
            .sum();
        let total_received: i64 = recv_entries
            .iter()
            .filter_map(|s| serde_json::from_str::<GiftRecord>(s).ok())
            .map(|g| g.price)
            .sum();
        Ok(serde_json::json!({
            "sent_count": sent_entries.len() as i64,
            "received_count": recv_entries.len() as i64,
            "total_spent": total_spent,
            "total_received": total_received,
            "rarest_gift_rarity": null,
        }))
    }

    pub fn mint_nft_gift(&self, user_id: i64, gift_id: i64, gift_record_id: i64) -> Result<i64> {
        let id = self.next_seq("nft_gifts");
        let token_id = format!(
            "YSH-NFT-{}-{}",
            gift_record_id,
            chrono::Utc::now().timestamp()
        );
        let nft = NftGift {
            id,
            user_id,
            gift_id,
            gift_record_id,
            token_id,
            unlocked: false,
            minted_at: now(),
        };
        self.mm_add(MM_NFT, &user_id.to_string(), &to_json(&nft))?;
        self.mm_add(IX_NFT_USER, &user_id.to_string(), &id.to_string())?;
        Ok(id)
    }

    pub fn get_nft_gifts(&self, user_id: i64) -> Result<Vec<serde_json::Value>> {
        self.mm_get_all(MM_NFT, &user_id.to_string())?
            .iter()
            .filter_map(|s| serde_json::from_str(s).ok())
            .collect::<Vec<_>>()
            .pipe(Ok)
    }

    // ═══════════════════════════════════════════
    // MOMENTS
    // ═══════════════════════════════════════════

    pub fn create_moment(
        &self,
        user_id: i64,
        content: &str,
        media_url: &str,
        media_type: &str,
    ) -> Result<i64> {
        let id = self.next_seq("moments");
        let m = Moment {
            id,
            user_id,
            content: content.into(),
            media_url: media_url.into(),
            media_type: media_type.into(),
            likes_count: 0,
            comments_count: 0,
            created_at: now(),
        };
        self.mm_add(MM_MOMENT, &user_id.to_string(), &to_json(&m))?;
        Ok(id)
    }

    pub fn get_moment_feed(
        &self,
        user_id: i64,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>> {
        let txn = self.inner.begin_read()?;
        let t = txn.open_multimap_table(MM_MOMENT)?;
        let mut all_moments: Vec<Moment> = Vec::new();
        for entry in t.iter()? {
            let (_, mut vals) = entry?;
            for v in &mut vals {
                if let Ok(m) = serde_json::from_str::<Moment>(v?.value()) {
                    all_moments.push(m);
                }
            }
        }
        all_moments.sort_by_key(|m| std::cmp::Reverse(m.id));
        drop(t);
        drop(txn);

        let blocked_ids: std::collections::HashSet<i64> = self
            .get_blocked_users(user_id)?
            .into_iter()
            .map(|r| r.blocked_user_id)
            .collect();
        let shadow_ids = self.active_shadow_ban_ids()?;
        let blocked_content: std::collections::HashSet<i64> = self
            .get_content_flags(Some("actioned"))?
            .into_iter()
            .filter(|f| f.target_type == "moment")
            .map(|f| f.target_id)
            .collect();

        let visible: Vec<Moment> = all_moments
            .into_iter()
            .filter(|m| !blocked_ids.contains(&m.user_id))
            .filter(|m| !shadow_ids.contains(&m.user_id))
            .filter(|m| !blocked_content.contains(&m.id))
            .collect();

        let likes_table = self.mm_get_all_entries(MM_MOMENT_LIKE)?;
        let results: Vec<serde_json::Value> = visible.into_iter().skip(offset as usize).take(limit as usize).map(|m| {
            let like_key = format!("{}_{}", user_id, m.id);
            let liked = likes_table.iter().any(|(k, _)| k == &like_key);
            let suffix = format!("_{}", m.id);
            let likes_count = likes_table.iter().filter(|(k, _)| k.ends_with(&suffix)).count() as i64;
            let username = self.find_user_by_id(m.user_id).ok().flatten().map(|u| u.username).unwrap_or_default();
            serde_json::json!({
                "id": m.id, "content": m.content, "media_url": m.media_url, "media_type": m.media_type,
                "username": username, "likes": likes_count, "comments": m.comments_count,
                "created_at": m.created_at, "liked": liked
            })
        }).collect();
        Ok(results)
    }

    pub fn like_moment(&self, _user_id: i64, moment_id: i64) -> Result<()> {
        let _lock = self
            .write_lock
            .lock()
            .map_err(|_| anyhow::anyhow!("Write lock poisoned"))?;
        let key = format!("{}_{}", _user_id, moment_id);
        let txn = self.inner.begin_write()?;
        {
            let mut t = txn.open_multimap_table(MM_MOMENT_LIKE)?;
            t.insert(key.as_str(), "1")?;
        }
        txn.commit()?;
        Ok(())
    }

    pub fn unlike_moment(&self, _user_id: i64, moment_id: i64) -> Result<()> {
        let _lock = self
            .write_lock
            .lock()
            .map_err(|_| anyhow::anyhow!("Write lock poisoned"))?;
        let key = format!("{}_{}", _user_id, moment_id);
        let txn = self.inner.begin_write()?;
        {
            let mut t = txn.open_multimap_table(MM_MOMENT_LIKE)?;
            t.remove(key.as_str(), "1")?;
        }
        txn.commit()?;
        Ok(())
    }

    pub fn comment_on_moment(&self, user_id: i64, moment_id: i64, content: &str) -> Result<i64> {
        let id = self.next_seq("moment_comments");
        let comment = serde_json::json!({"id": id, "user_id": user_id, "content": content, "created_at": now()});
        self.mm_add(
            MM_MOMENT_COMMENT,
            &moment_id.to_string(),
            &to_json(&comment),
        )?;
        Ok(id)
    }

    pub fn get_moment_comments(&self, moment_id: i64) -> Result<Vec<serde_json::Value>> {
        self.mm_get_all(MM_MOMENT_COMMENT, &moment_id.to_string())?
            .iter()
            .filter_map(|s| serde_json::from_str(s).ok())
            .collect::<Vec<_>>()
            .pipe(Ok)
    }

    pub fn delete_moment(&self, _user_id: i64, moment_id: i64) -> Result<bool> {
        let _lock = self
            .write_lock
            .lock()
            .map_err(|_| anyhow::anyhow!("Write lock poisoned"))?;
        let txn = self.inner.begin_write()?;
        let removed = {
            let mut t = txn.open_multimap_table(MM_MOMENT)?;
            t.remove_all(moment_id.to_string().as_str())?
                .next()
                .is_some()
        };
        txn.commit()?;
        Ok(removed)
    }

    // ═══════════════════════════════════════════
    // PLATFORM STATS
    // ═══════════════════════════════════════════

    fn count_kv_table(&self, table_def: TableDefinition<&str, &str>) -> Result<i64> {
        let txn = self.inner.begin_read()?;
        let t = txn.open_table(table_def)?;
        let mut c: i64 = 0;
        for _ in t.iter()? {
            c += 1;
        }
        Ok(c)
    }

    fn count_mm_table(&self, table_def: MultimapTableDefinition<&str, &str>) -> Result<i64> {
        let txn = self.inner.begin_read()?;
        let t = txn.open_multimap_table(table_def)?;
        let mut c: i64 = 0;
        for _ in t.iter()? {
            c += 1;
        }
        Ok(c)
    }

    pub fn platform_stats(&self) -> Result<serde_json::Value> {
        let users = self.user_count()?;
        let agencies = self.count_kv_table(T_AGENCY)?;
        let hosts = self.count_kv_table(T_HOST)?;
        let moments = self.count_mm_table(MM_MOMENT)?;
        let gifts = self.count_mm_table(MM_GIFT)?;
        let notifications = self.count_mm_table(MM_NOTIFICATION)?;
        Ok(serde_json::json!({
            "users": users,
            "agencies": agencies,
            "hosts": hosts,
            "moments": moments,
            "gifts": gifts,
            "total_transaction_volume": 0,
            "notifications": notifications,
        }))
    }

    // ═══════════════════════════════════════════
    // I18N OVERRIDES
    // ═══════════════════════════════════════════

    /// Stores a translation override (admin panel) keyed as `"{locale}::{key}"`.
    pub fn set_i18n_override(&self, locale: &str, key: &str, value: &str) -> Result<()> {
        let fk = format!("{locale}::{key}");
        let _lock = self
            .write_lock
            .lock()
            .map_err(|_| anyhow::anyhow!("Write lock poisoned"))?;
        let txn = self.inner.begin_write()?;
        {
            let mut t = txn.open_table(T_I18N)?;
            t.insert(fk.as_str(), value)?;
        }
        txn.commit()?;
        Ok(())
    }

    pub fn get_i18n_override(&self, locale: &str, key: &str) -> Result<Option<String>> {
        let fk = format!("{locale}::{key}");
        let txn = self.inner.begin_read()?;
        let t = txn.open_table(T_I18N)?;
        Ok(t.get(fk.as_str())?.map(|v| v.value().to_string()))
    }

    pub fn delete_i18n_override(&self, locale: &str, key: &str) -> Result<bool> {
        let fk = format!("{locale}::{key}");
        let _lock = self
            .write_lock
            .lock()
            .map_err(|_| anyhow::anyhow!("Write lock poisoned"))?;
        let txn = self.inner.begin_write()?;
        let removed = {
            let mut t = txn.open_table(T_I18N)?;
            let existed = t.get(fk.as_str())?.is_some();
            t.remove(fk.as_str())?;
            existed
        };
        txn.commit()?;
        Ok(removed)
    }

    pub fn list_i18n_overrides(&self) -> Result<Vec<(String, String)>> {
        let txn = self.inner.begin_read()?;
        let t = txn.open_table(T_I18N)?;
        let mut out = Vec::new();
        for item in t.iter()? {
            let (k, v) = item?;
            out.push((k.value().to_string(), v.value().to_string()));
        }
        Ok(out)
    }

    // ═══════════════════════════════════════════
    // NOTIFICATIONS
    // ═══════════════════════════════════════════

    pub fn create_notification(
        &self,
        user_id: i64,
        ntype: &str,
        title: &str,
        body: &str,
        data: &str,
        channel: &str,
    ) -> Result<i64> {
        let id = self.next_seq("notifications");
        let n = Notification {
            id,
            user_id,
            ntype: ntype.into(),
            title: title.into(),
            body: body.into(),
            data: data.into(),
            read: false,
            channel: channel.into(),
            status: "pending".into(),
            retries: 0,
            created_at: now(),
            sent_at: None,
        };
        self.mm_add(MM_NOTIFICATION, &user_id.to_string(), &to_json(&n))?;
        self.mm_add(
            IX_NOTIF_USER,
            &user_id.to_string(),
            &to_json(&serde_json::json!({"id": id, "read": false})),
        )?;
        Ok(id)
    }

    pub fn get_notifications(&self, user_id: i64, limit: i64) -> Result<Vec<serde_json::Value>> {
        self.mm_get_all(MM_NOTIFICATION, &user_id.to_string())?
            .iter()
            .filter_map(|s| serde_json::from_str::<serde_json::Value>(s).ok())
            .take(limit as usize)
            .collect::<Vec<_>>()
            .pipe(Ok)
    }

    pub fn get_unread_count(&self, user_id: i64) -> Result<i64> {
        let entries = self.mm_get_all(MM_NOTIFICATION, &user_id.to_string())?;
        let count = entries
            .iter()
            .filter_map(|s| serde_json::from_str::<Notification>(s).ok())
            .filter(|n| !n.read)
            .count();
        Ok(count as i64)
    }

    pub fn mark_notification_read(&self, _user_id: i64, notification_id: i64) -> Result<bool> {
        let entries = self.mm_get_all(MM_NOTIFICATION, &_user_id.to_string())?;
        for entry in &entries {
            if let Ok(mut n) = serde_json::from_str::<Notification>(entry)
                && n.id == notification_id
            {
                n.read = true;
                self.mm_remove_one(MM_NOTIFICATION, &_user_id.to_string(), entry)?;
                self.mm_add(MM_NOTIFICATION, &_user_id.to_string(), &to_json(&n))?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn mark_all_read(&self, user_id: i64) -> Result<usize> {
        let entries = self.mm_get_all(MM_NOTIFICATION, &user_id.to_string())?;
        let mut count = 0;
        for entry in &entries {
            if let Ok(mut n) = serde_json::from_str::<Notification>(entry)
                && !n.read
            {
                n.read = true;
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn get_notification_preference(&self, user_id: i64) -> Result<serde_json::Value> {
        match self.get_json::<serde_json::Value>(T_NOTIF_PREF, &user_id.to_string())? {
            Some(prefs) => Ok(prefs),
            None => {
                let default = serde_json::json!({
                    "email_enabled": true, "push_enabled": true, "in_app_enabled": true,
                    "email_gifts": true, "email_calls": true, "email_moments": true, "email_marketing": false,
                    "push_gifts": true, "push_calls": true, "push_moments": true,
                    "quiet_hours_start": null, "quiet_hours_end": null,
                });
                Ok(default)
            }
        }
    }

    pub fn update_notification_preference(
        &self,
        user_id: i64,
        field: &str,
        value: bool,
    ) -> Result<()> {
        let mut prefs = self.get_notification_preference(user_id)?;
        prefs[field] = serde_json::json!(value);
        self.put_json(T_NOTIF_PREF, &user_id.to_string(), &prefs)
    }

    pub fn update_quiet_hours(&self, user_id: i64, start: &str, end: &str) -> Result<()> {
        let mut prefs = self.get_notification_preference(user_id)?;
        prefs["quiet_hours_start"] = serde_json::json!(start);
        prefs["quiet_hours_end"] = serde_json::json!(end);
        self.put_json(T_NOTIF_PREF, &user_id.to_string(), &prefs)
    }

    pub fn register_push_token(&self, user_id: i64, token: &str, platform: &str) -> Result<()> {
        let t = serde_json::json!({"token": token, "platform": platform, "active": true, "created_at": now()});
        self.mm_add(MM_PUSH_TOKEN, &user_id.to_string(), &to_json(&t))
    }

    pub fn get_push_tokens(&self, user_id: i64) -> Result<Vec<serde_json::Value>> {
        self.mm_get_all(MM_PUSH_TOKEN, &user_id.to_string())?
            .iter()
            .filter_map(|s| serde_json::from_str::<serde_json::Value>(s).ok())
            .filter(|t| t["active"] == serde_json::json!(true))
            .collect::<Vec<_>>()
            .pipe(Ok)
    }

    pub fn deactivate_push_token(&self, user_id: i64, token: &str) -> Result<bool> {
        let entries = self.mm_get_all(MM_PUSH_TOKEN, &user_id.to_string())?;
        self.mm_remove_all(MM_PUSH_TOKEN, &user_id.to_string())?;
        let mut found = false;
        for entry in &entries {
            if let Ok(mut t) = serde_json::from_str::<serde_json::Value>(entry) {
                if t["token"].as_str() == Some(token) {
                    t["active"] = serde_json::json!(false);
                    found = true;
                }
                self.mm_add(MM_PUSH_TOKEN, &user_id.to_string(), &to_json(&t))?;
            }
        }
        Ok(found)
    }

    // ═══════════════════════════════════════════
    // CHAT
    // ═══════════════════════════════════════════

    pub fn create_chat_session(&self, session_type: &str, user_ids: &[i64]) -> Result<i64> {
        let id = self.next_seq("chat_sessions");
        let s = ChatSession {
            id,
            session_type: session_type.into(),
            created_at: now(),
            updated_at: now(),
        };
        self.put_json(T_CHAT_SESSION, &id.to_string(), &s)?;
        for uid in user_ids {
            self.mm_add(MM_CHAT_PARTICIPANT, &id.to_string(), &uid.to_string())?;
            self.mm_add(IX_CHAT_USER, &uid.to_string(), &id.to_string())?;
        }
        Ok(id)
    }

    pub fn find_direct_session(&self, user_a: i64, user_b: i64) -> Result<Option<i64>> {
        let sessions_a = self.mm_get_all(IX_CHAT_USER, &user_a.to_string())?;
        let sessions_b = self.mm_get_all(IX_CHAT_USER, &user_b.to_string())?;
        for sa in &sessions_a {
            for sb in &sessions_b {
                if sa == sb {
                    let sid: i64 = sa.parse().unwrap_or(0);
                    if let Some(session) =
                        self.get_json::<ChatSession>(T_CHAT_SESSION, &sid.to_string())?
                        && session.session_type == "direct"
                    {
                        return Ok(Some(sid));
                    }
                }
            }
        }
        Ok(None)
    }

    pub fn get_user_sessions(&self, user_id: i64) -> Result<Vec<serde_json::Value>> {
        let session_ids = self.mm_get_all(IX_CHAT_USER, &user_id.to_string())?;
        let mut results = Vec::new();
        for sid_str in session_ids {
            let sid: i64 = sid_str.parse().unwrap_or(0);
            if let Some(session) = self.get_json::<ChatSession>(T_CHAT_SESSION, &sid.to_string())? {
                let participants = self.get_session_participants(sid)?;
                results.push(serde_json::json!({
                    "session_id": session.id, "type": session.session_type,
                    "created_at": session.created_at, "updated_at": session.updated_at,
                    "participants": participants,
                }));
            }
        }
        Ok(results)
    }

    pub fn get_session_participants(&self, session_id: i64) -> Result<Vec<serde_json::Value>> {
        let user_ids = self.mm_get_all(MM_CHAT_PARTICIPANT, &session_id.to_string())?;
        let mut results = Vec::new();
        for uid_str in user_ids {
            let uid: i64 = uid_str.parse().unwrap_or(0);
            let username = self
                .find_user_by_id(uid)?
                .map(|u| u.username)
                .unwrap_or_default();
            results.push(serde_json::json!({"user_id": uid, "username": username}));
        }
        Ok(results)
    }

    pub fn send_message(
        &self,
        session_id: i64,
        sender_id: i64,
        content: &str,
        msg_type: &str,
        encrypted: bool,
    ) -> Result<i64> {
        let id = self.next_seq("messages");
        let m = Message {
            id,
            session_id,
            sender_id,
            content: content.into(),
            msg_type: msg_type.into(),
            encrypted,
            read: false,
            created_at: now(),
        };
        self.mm_add(MM_MSG, &session_id.to_string(), &to_json(&m))?;
        self.mm_add(IX_MSG_SESSION, &session_id.to_string(), &id.to_string())?;
        Ok(id)
    }

    pub fn get_messages(
        &self,
        session_id: i64,
        limit: i64,
        before_id: Option<i64>,
    ) -> Result<Vec<serde_json::Value>> {
        let entries = self.mm_get_all(MM_MSG, &session_id.to_string())?;
        let mut messages: Vec<Message> = entries
            .iter()
            .filter_map(|s| serde_json::from_str(s).ok())
            .collect();
        messages.sort_by_key(|m| std::cmp::Reverse(m.id));
        if let Some(before) = before_id {
            messages.retain(|m| m.id < before);
        }
        messages.truncate(limit as usize);
        messages.into_iter().map(|m| Ok(serde_json::json!({
            "id": m.id, "sender_id": m.sender_id, "username": self.find_user_by_id(m.sender_id)?.map(|u| u.username).unwrap_or_default(),
            "content": m.content, "type": m.msg_type, "encrypted": m.encrypted, "read": m.read, "created_at": m.created_at
        }))).collect::<Result<Vec<_>>>()
    }

    pub fn mark_messages_read(&self, session_id: i64, user_id: i64) -> Result<usize> {
        let entries = self.mm_get_all(MM_MSG, &session_id.to_string())?;
        let mut count = 0;
        self.mm_remove_all(MM_MSG, &session_id.to_string())?;
        for entry in &entries {
            if let Ok(mut m) = serde_json::from_str::<Message>(entry) {
                if m.sender_id != user_id && !m.read {
                    m.read = true;
                    count += 1;
                }
                self.mm_add(MM_MSG, &session_id.to_string(), &to_json(&m))?;
            }
        }
        Ok(count)
    }

    pub fn get_unread_message_count(&self, user_id: i64) -> Result<i64> {
        let session_ids = self.mm_get_all(IX_CHAT_USER, &user_id.to_string())?;
        let mut total = 0;
        for sid_str in session_ids {
            let sid: i64 = sid_str.parse().unwrap_or(0);
            let entries = self.mm_get_all(MM_MSG, &sid.to_string())?;
            for entry in &entries {
                if let Ok(m) = serde_json::from_str::<Message>(entry)
                    && m.sender_id != user_id
                    && !m.read
                {
                    total += 1;
                }
            }
        }
        Ok(total)
    }

    // ═══════════════════════════════════════════
    // MATCHING
    // ═══════════════════════════════════════════

    pub fn enqueue_match(&self, user_id: i64, mode: &str, preferences: &str) -> Result<i64> {
        self.mm_remove_all(MM_MATCH_QUEUE, &user_id.to_string())?;
        let entry = serde_json::json!({"user_id": user_id, "mode": mode, "preferences": preferences, "status": "waiting", "queued_at": now()});
        self.mm_add(MM_MATCH_QUEUE, &user_id.to_string(), &to_json(&entry))?;
        Ok(user_id)
    }

    pub fn dequeue_match(&self, user_id: i64) -> Result<bool> {
        self.mm_remove_all(MM_MATCH_QUEUE, &user_id.to_string())?;
        Ok(true)
    }

    pub fn find_match(&self, exclude_user_id: i64, mode: &str) -> Result<Option<i64>> {
        let entries = self.mm_get_all_entries(MM_MATCH_QUEUE)?;
        for (uid_str, val) in &entries {
            if uid_str == &exclude_user_id.to_string() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<serde_json::Value>(val)
                && entry["mode"].as_str() == Some(mode)
                && entry["status"].as_str() == Some("waiting")
            {
                return Ok(Some(uid_str.parse().unwrap_or(0)));
            }
        }
        Ok(None)
    }

    pub fn find_random_match(&self, exclude_user_id: i64) -> Result<Option<i64>> {
        let entries = self.mm_get_all_entries(MM_MATCH_QUEUE)?;
        let candidates: Vec<i64> = entries
            .iter()
            .filter_map(|(k, v)| {
                if k == &exclude_user_id.to_string() {
                    return None;
                }
                let entry: serde_json::Value = serde_json::from_str(v).ok()?;
                if entry["status"].as_str() == Some("waiting") {
                    k.parse().ok()
                } else {
                    None
                }
            })
            .collect();
        if candidates.is_empty() {
            return Ok(None);
        }
        let idx = rand::rng().random_range(0usize..candidates.len());
        Ok(Some(candidates[idx]))
    }

    pub fn complete_match(&self, _queue_id: i64) -> Result<()> {
        Ok(())
    }

    pub fn get_queue_size(&self) -> Result<i64> {
        let entries = self.mm_get_all_entries(MM_MATCH_QUEUE)?;
        let count = entries
            .iter()
            .filter(|(_, v)| {
                serde_json::from_str::<serde_json::Value>(v)
                    .ok()
                    .and_then(|e| e["status"].as_str().map(|s| s == "waiting"))
                    .unwrap_or(false)
            })
            .count();
        Ok(count as i64)
    }

    pub fn get_pending_match_count(&self) -> Result<i64> {
        self.get_queue_size()
    }

    // ═══════════════════════════════════════════
    // PHASE 9: STAKING
    // ═══════════════════════════════════════════

    pub fn stake(&self, user_id: i64, amount: i64, apy_rate: f64, unlock_days: i64) -> Result<i64> {
        let mut w = self
            .get_json::<Wallet>(T_WALLET, &user_id.to_string())?
            .unwrap();
        if w.balance < amount {
            anyhow::bail!("Insufficient funds for staking: {} < {}", w.balance, amount);
        }
        w.balance -= amount;
        w.updated_at = now();
        self.put_json(T_WALLET, &user_id.to_string(), &w)?;

        let id = self.next_seq("staking");
        let ts = now();
        let unlocks = chrono::Utc::now() + chrono::Duration::days(unlock_days);
        let stake = Staking {
            id,
            user_id,
            amount,
            apy_rate,
            status: "active".into(),
            staked_at: ts.clone(),
            unlocks_at: unlocks.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            rewards_earned: 0,
            last_reward_calc: ts.clone(),
        };
        self.mm_add(MM_STAKING, &user_id.to_string(), &to_json(&stake))?;

        let tx_id = self.next_seq("transactions");
        let tx = Transaction {
            id: tx_id,
            user_id,
            tx_type: "stake".into(),
            amount,
            description: format!("Staked {} YSH for {} days", amount, unlock_days),
            target_user_id: None,
            created_at: ts,
        };
        self.mm_add(MM_TRANSACTION, &user_id.to_string(), &to_json(&tx))?;
        Ok(id)
    }

    pub fn unstake(&self, user_id: i64, stake_id: i64) -> Result<i64> {
        let entries = self.mm_get_all(MM_STAKING, &user_id.to_string())?;
        let mut found: Option<Staking> = None;
        for entry in &entries {
            if let Ok(s) = serde_json::from_str::<Staking>(entry)
                && s.id == stake_id
                && s.status == "active"
            {
                found = Some(s);
                break;
            }
        }
        let stake = found.ok_or_else(|| anyhow::anyhow!("Stake not found"))?;
        let now_str = now();
        if stake.unlocks_at > now_str {
            anyhow::bail!(
                "Staking lock not yet expired, unlocks at: {}",
                stake.unlocks_at
            );
        }
        let total = stake.amount + stake.rewards_earned;

        self.mm_remove_all(MM_STAKING, &user_id.to_string())?;
        for entry in &entries {
            if let Ok(mut s) = serde_json::from_str::<Staking>(entry) {
                if s.id == stake_id {
                    s.status = "withdrawn".into();
                    s.amount = 0;
                    s.rewards_earned = 0;
                }
                self.mm_add(MM_STAKING, &user_id.to_string(), &to_json(&s))?;
            }
        }

        let mut w = self
            .get_json::<Wallet>(T_WALLET, &user_id.to_string())?
            .unwrap();
        w.balance += total;
        w.updated_at = now();
        self.put_json(T_WALLET, &user_id.to_string(), &w)?;

        let tx_id = self.next_seq("transactions");
        let tx = Transaction {
            id: tx_id,
            user_id,
            tx_type: "unstake".into(),
            amount: total,
            description: format!("Unstaked + rewards from #{}", stake_id),
            target_user_id: None,
            created_at: now(),
        };
        self.mm_add(MM_TRANSACTION, &user_id.to_string(), &to_json(&tx))?;
        Ok(total)
    }

    pub fn claim_staking_rewards(&self, user_id: i64, stake_id: i64) -> Result<i64> {
        let entries = self.mm_get_all(MM_STAKING, &user_id.to_string())?;
        let mut rewards = 0i64;
        self.mm_remove_all(MM_STAKING, &user_id.to_string())?;
        for entry in &entries {
            if let Ok(mut s) = serde_json::from_str::<Staking>(entry) {
                if s.id == stake_id && s.status == "active" {
                    rewards = s.rewards_earned;
                    s.rewards_earned = 0;
                    s.last_reward_calc = now();
                }
                self.mm_add(MM_STAKING, &user_id.to_string(), &to_json(&s))?;
            }
        }
        if rewards <= 0 {
            anyhow::bail!("No rewards to claim");
        }

        let mut w = self
            .get_json::<Wallet>(T_WALLET, &user_id.to_string())?
            .unwrap();
        w.balance += rewards;
        w.updated_at = now();
        self.put_json(T_WALLET, &user_id.to_string(), &w)?;

        let tx_id = self.next_seq("transactions");
        let tx = Transaction {
            id: tx_id,
            user_id,
            tx_type: "staking_reward".into(),
            amount: rewards,
            description: format!("Staking reward from #{}", stake_id),
            target_user_id: None,
            created_at: now(),
        };
        self.mm_add(MM_TRANSACTION, &user_id.to_string(), &to_json(&tx))?;
        Ok(rewards)
    }

    pub fn get_staking_positions(&self, user_id: i64) -> Result<Vec<serde_json::Value>> {
        self.mm_get_all(MM_STAKING, &user_id.to_string())?
            .iter()
            .filter_map(|s| serde_json::from_str::<serde_json::Value>(s).ok())
            .collect::<Vec<_>>()
            .pipe(Ok)
    }

    pub fn calculate_staking_rewards(&self) -> Result<usize> {
        let entries = self.mm_get_all_entries(MM_STAKING)?;
        let mut updated = 0;
        for (uid, val) in &entries {
            if let Ok(mut s) = serde_json::from_str::<Staking>(val)
                && s.status == "active"
            {
                let daily_reward = ((s.amount as f64 * s.apy_rate) / 365.0) as i64;
                if daily_reward > 0 {
                    s.rewards_earned += daily_reward;
                    s.last_reward_calc = now();
                    self.mm_remove_all(MM_STAKING, uid)?;
                    self.mm_add(MM_STAKING, uid, &to_json(&s))?;
                    updated += 1;
                }
            }
        }
        Ok(updated)
    }

    pub fn get_staking_stats(&self) -> Result<serde_json::Value> {
        let entries = self.mm_get_all_entries(MM_STAKING)?;
        let mut total_staked = 0i64;
        let mut total_rewards = 0i64;
        let mut active = 0i64;
        for (_, val) in &entries {
            if let Ok(s) = serde_json::from_str::<Staking>(val)
                && s.status == "active"
            {
                total_staked += s.amount;
                total_rewards += s.rewards_earned;
                active += 1;
            }
        }
        Ok(
            serde_json::json!({"total_staked": total_staked, "total_rewards_pending": total_rewards, "active_positions": active}),
        )
    }

    // ═══════════════════════════════════════════
    // PHASE 9: REFERRALS
    // ═══════════════════════════════════════════

    pub fn create_referral(&self, referrer_id: i64, referred_id: i64, code: &str) -> Result<i64> {
        let id = self.next_seq("referrals");
        let r = Referral {
            id,
            referrer_id,
            referred_id,
            code: code.into(),
            status: "pending".into(),
            referred_at: now(),
            first_purchase_at: None,
            total_earned: 0,
        };
        self.mm_add(MM_REFERRAL, &referrer_id.to_string(), &to_json(&r))?;
        self.put_json(IX_REFERRAL_CODE, code, &referrer_id.to_string())?;
        Ok(id)
    }

    pub fn find_referral_by_code(&self, code: &str) -> Result<Option<(i64, i64)>> {
        match self.get_json::<String>(IX_REFERRAL_CODE, code)? {
            Some(referrer_id_str) => {
                let referrer_id: i64 = referrer_id_str.parse()?;
                let entries = self.mm_get_all(MM_REFERRAL, &referrer_id.to_string())?;
                for entry in &entries {
                    if let Ok(r) = serde_json::from_str::<Referral>(entry)
                        && r.code == code
                    {
                        return Ok(Some((referrer_id, r.referred_id)));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn complete_referral(&self, referred_id: i64) -> Result<()> {
        let all_entries = self.mm_get_all_entries(MM_REFERRAL)?;
        for (referrer_key, val) in &all_entries {
            if let Ok(mut r) = serde_json::from_str::<Referral>(val)
                && r.referred_id == referred_id
                && r.status == "pending"
            {
                r.status = "completed".into();
                r.first_purchase_at = Some(now());
                self.mm_remove_one(MM_REFERRAL, referrer_key, val)?;
                self.mm_add(MM_REFERRAL, referrer_key, &to_json(&r))?;
            }
        }
        Ok(())
    }

    pub fn add_referral_earning(&self, referrer_id: i64, amount: i64) -> Result<()> {
        let entries = self.mm_get_all(MM_REFERRAL, &referrer_id.to_string())?;
        self.mm_remove_all(MM_REFERRAL, &referrer_id.to_string())?;
        for entry in &entries {
            if let Ok(mut r) = serde_json::from_str::<Referral>(entry) {
                r.total_earned += amount;
                self.mm_add(MM_REFERRAL, &referrer_id.to_string(), &to_json(&r))?;
            }
        }
        Ok(())
    }

    pub fn get_referral_stats(&self, user_id: i64) -> Result<serde_json::Value> {
        let entries = self.mm_get_all(MM_REFERRAL, &user_id.to_string())?;
        let referrals: Vec<Referral> = entries
            .iter()
            .filter_map(|s| serde_json::from_str(s).ok())
            .collect();
        let total_referred = referrals.len() as i64;
        let completed = referrals.iter().filter(|r| r.status == "completed").count() as i64;
        let total_earned = referrals.iter().map(|r| r.total_earned).sum::<i64>();
        let code = referrals.first().map(|r| r.code.clone());
        Ok(
            serde_json::json!({"referral_code": code, "total_referred": total_referred, "completed": completed, "total_earned": total_earned}),
        )
    }

    // ═══════════════════════════════════════════
    // PHASE 9: CALL BILLING
    // ═══════════════════════════════════════════

    pub fn start_call_billing(
        &self,
        caller_id: i64,
        host_id: i64,
        call_type: &str,
        cost_per_min: i64,
    ) -> Result<i64> {
        let id = self.next_seq("call_billing");
        let cb = CallBilling {
            id,
            caller_id,
            host_id,
            call_type: call_type.into(),
            started_at: now(),
            ended_at: None,
            duration_secs: 0,
            cost_per_min,
            total_cost: 0,
            host_earnings: 0,
            platform_fee: 0,
            status: "active".into(),
            paid: false,
        };
        self.mm_add(MM_CALL_BILLING, &host_id.to_string(), &to_json(&cb))?;
        Ok(id)
    }

    pub fn end_call_billing(&self, call_id: i64) -> Result<(i64, i64, i64)> {
        let entries = self.mm_get_all_entries(MM_CALL_BILLING)?;
        for (host_key, val) in &entries {
            if let Ok(mut cb) = serde_json::from_str::<CallBilling>(val)
                && cb.id == call_id
                && cb.status == "active"
            {
                let now_str = now();
                let start =
                    chrono::NaiveDateTime::parse_from_str(&cb.started_at, "%Y-%m-%dT%H:%M:%SZ")
                        .unwrap_or_default();
                let end = chrono::NaiveDateTime::parse_from_str(&now_str, "%Y-%m-%dT%H:%M:%SZ")
                    .unwrap_or_default();
                let duration_secs = (end - start).num_seconds().max(0);
                let duration_mins = ((duration_secs + 59) / 60).max(1);
                let total_cost = duration_mins * cb.cost_per_min;
                let host_earnings = (total_cost as f64 * 0.70) as i64;
                let platform_fee = total_cost - host_earnings;

                cb.ended_at = Some(now_str);
                cb.duration_secs = duration_secs;
                cb.total_cost = total_cost;
                cb.host_earnings = host_earnings;
                cb.platform_fee = platform_fee;
                cb.status = "completed".into();

                self.mm_remove_one(MM_CALL_BILLING, host_key, val)?;
                self.mm_add(MM_CALL_BILLING, host_key, &to_json(&cb))?;
                return Ok((total_cost, host_earnings, platform_fee));
            }
        }
        anyhow::bail!("Call not found")
    }

    pub fn get_host_call_stats(&self, host_id: i64) -> Result<serde_json::Value> {
        let entries = self.mm_get_all(MM_CALL_BILLING, &host_id.to_string())?;
        let calls: Vec<CallBilling> = entries
            .iter()
            .filter_map(|s| serde_json::from_str(s).ok())
            .collect();
        let total_calls = calls.iter().filter(|c| c.status == "completed").count() as i64;
        let total_earnings = calls
            .iter()
            .filter(|c| c.status == "completed")
            .map(|c| c.host_earnings)
            .sum::<i64>();
        let total_duration = calls
            .iter()
            .filter(|c| c.status == "completed")
            .map(|c| c.duration_secs)
            .sum::<i64>();
        Ok(
            serde_json::json!({"total_calls": total_calls, "total_earnings": total_earnings, "total_duration_secs": total_duration}),
        )
    }

    pub fn finalize_call_payment(&self, call_id: i64) -> Result<serde_json::Value> {
        let entries = self.mm_get_all_entries(MM_CALL_BILLING)?;
        for (host_key, val) in &entries {
            if let Ok(mut cb) = serde_json::from_str::<CallBilling>(val)
                && cb.id == call_id
                && cb.status == "completed"
                && !cb.paid
            {
                let balance =
                    self.withdraw(cb.caller_id, cb.total_cost, &format!("Call #{}", call_id))?;
                self.deposit(
                    cb.host_id,
                    cb.host_earnings,
                    &format!("Call #{} earnings", call_id),
                )?;
                cb.paid = true;
                self.mm_remove_one(MM_CALL_BILLING, host_key, val)?;
                self.mm_add(MM_CALL_BILLING, host_key, &to_json(&cb))?;
                return Ok(serde_json::json!({
                    "call_id": call_id,
                    "caller_balance": balance,
                    "total_cost": cb.total_cost,
                    "host_earnings": cb.host_earnings,
                    "platform_fee": cb.platform_fee,
                }));
            }
        }
        anyhow::bail!("No completed unpaid billing found for call {}", call_id)
    }

    pub fn find_active_call_billing(&self, caller_id: i64) -> Result<Option<CallBilling>> {
        let entries = self.mm_get_all_entries(MM_CALL_BILLING)?;
        for (_host_key, val) in &entries {
            if let Ok(cb) = serde_json::from_str::<CallBilling>(val)
                && cb.caller_id == caller_id
                && cb.status == "in_progress"
            {
                return Ok(Some(cb));
            }
        }
        Ok(None)
    }

    // ═══════════════════════════════════════════
    // PHASE 8: WEBRTC CALL RECORDS
    // ═══════════════════════════════════════════

    pub fn create_call_record(
        &self,
        call_id: &str,
        room_id: &str,
        host_id: i64,
        call_type: &str,
        participants: &[i64],
    ) -> Result<()> {
        let rec = CallRecord {
            call_id: call_id.into(),
            room_id: room_id.into(),
            host_id,
            call_type: call_type.into(),
            participants: participants.to_vec(),
            status: "ringing".into(),
            started_at: now(),
            ended_at: None,
            duration_secs: 0,
            screen_share: false,
            screen_share_user: None,
            recording: false,
            recording_encrypted: false,
            billed: 0,
        };
        self.put_json(T_CALL, call_id, &rec)?;
        self.mm_add(MM_CALL_USER, &host_id.to_string(), call_id)?;
        for p in participants {
            self.mm_add(MM_CALL_USER, &p.to_string(), call_id)?;
        }
        Ok(())
    }

    pub fn update_call_record(
        &self,
        call_id: &str,
        f: impl FnOnce(&mut CallRecord),
    ) -> Result<bool> {
        let mut rec = match self.get_json::<CallRecord>(T_CALL, call_id)? {
            Some(r) => r,
            None => return Ok(false),
        };
        f(&mut rec);
        self.put_json(T_CALL, call_id, &rec)?;
        Ok(true)
    }

    pub fn join_call(&self, call_id: &str, user_id: i64) -> Result<bool> {
        self.update_call_record(call_id, |rec| {
            if rec.status != "ended" && !rec.participants.contains(&user_id) {
                rec.participants.push(user_id);
                if rec.status == "ringing" {
                    rec.status = "active".into();
                }
            }
        })?;
        self.mm_add(MM_CALL_USER, &user_id.to_string(), call_id)?;
        Ok(true)
    }

    pub fn leave_call(&self, call_id: &str, user_id: i64) -> Result<()> {
        self.update_call_record(call_id, |rec| rec.participants.retain(|p| *p != user_id))?;
        Ok(())
    }

    pub fn set_call_screen_share(&self, call_id: &str, user_id: i64, active: bool) -> Result<bool> {
        self.update_call_record(call_id, |rec| {
            rec.screen_share = active;
            rec.screen_share_user = if active { Some(user_id) } else { None };
        })
    }

    pub fn set_call_recording(&self, call_id: &str, active: bool, encrypted: bool) -> Result<bool> {
        self.update_call_record(call_id, |rec| {
            rec.recording = active;
            rec.recording_encrypted = encrypted;
        })
    }

    pub fn end_call_record(&self, call_id: &str) -> Result<i64> {
        let mut rec = match self.get_json::<CallRecord>(T_CALL, call_id)? {
            Some(r) => r,
            None => anyhow::bail!("Call {} not found", call_id),
        };
        if rec.status == "ended" {
            return Ok(rec.duration_secs);
        }
        let now_str = now();
        let start = chrono::NaiveDateTime::parse_from_str(&rec.started_at, "%Y-%m-%dT%H:%M:%SZ")
            .unwrap_or_default();
        let end = chrono::NaiveDateTime::parse_from_str(&now_str, "%Y-%m-%dT%H:%M:%SZ")
            .unwrap_or_default();
        rec.duration_secs = (end - start).num_seconds().max(0) as i64;
        rec.ended_at = Some(now_str);
        rec.status = "ended".into();
        rec.recording = false;
        self.put_json(T_CALL, call_id, &rec)?;
        Ok(rec.duration_secs)
    }

    pub fn get_call_record(&self, call_id: &str) -> Result<Option<CallRecord>> {
        self.get_json::<CallRecord>(T_CALL, call_id)
    }

    pub fn get_call_history(&self, user_id: i64, limit: i64) -> Result<Vec<serde_json::Value>> {
        let entries = self.mm_get_all(MM_CALL_USER, &user_id.to_string())?;
        let mut recs: Vec<CallRecord> = Vec::new();
        for call_id in entries {
            if let Some(rec) = self.get_json::<CallRecord>(T_CALL, &call_id)? {
                recs.push(rec);
            }
        }
        recs.sort_by(|a, b| {
            b.started_at
                .cmp(&a.started_at)
                .then_with(|| b.call_id.cmp(&a.call_id))
        });
        let out: Vec<serde_json::Value> = recs
            .into_iter()
            .take(limit as usize)
            .map(|r| serde_json::to_value(r).unwrap_or_default())
            .collect();
        Ok(out)
    }

    pub fn get_call_stats(&self) -> Result<serde_json::Value> {
        let txn = self.inner.begin_read()?;
        let t = txn.open_table(T_CALL)?;
        let mut total = 0;
        let mut active = 0;
        let mut duration_sum: i64 = 0;
        for entry in t.iter()? {
            let (_, v) = entry?;
            total += 1;
            if let Ok(rec) = serde_json::from_str::<CallRecord>(v.value()) {
                if rec.status != "ended" {
                    active += 1;
                }
                duration_sum += rec.duration_secs;
            }
        }
        Ok(serde_json::json!({
            "total_calls": total,
            "active_calls": active,
            "total_duration_secs": duration_sum,
        }))
    }

    // ═══════════════════════════════════════════
    // PHASE 8: QUALITY METRICS
    // ═══════════════════════════════════════════

    #[allow(clippy::too_many_arguments)]
    pub fn add_quality_sample(
        &self,
        call_id: &str,
        user_id: i64,
        bitrate_kbps: f64,
        packet_loss_pct: f64,
        rtt_ms: f64,
        resolution: &str,
        simulcast_tier: &str,
    ) -> Result<()> {
        let sample = QualitySample {
            call_id: call_id.into(),
            user_id,
            ts: now(),
            bitrate_kbps,
            packet_loss_pct,
            rtt_ms,
            resolution: resolution.into(),
            simulcast_tier: simulcast_tier.into(),
        };
        self.mm_add(MM_CALL_QUALITY, call_id, &to_json(&sample))
    }

    pub fn get_quality_metrics(&self, call_id: &str) -> Result<Vec<QualitySample>> {
        let entries = self.mm_get_all(MM_CALL_QUALITY, call_id)?;
        Ok(entries
            .iter()
            .filter_map(|s| serde_json::from_str(s).ok())
            .collect())
    }

    pub fn aggregate_quality(&self, call_id: &str) -> Result<serde_json::Value> {
        let samples = self.get_quality_metrics(call_id)?;
        if samples.is_empty() {
            return Ok(serde_json::json!({"samples": 0}));
        }
        let n = samples.len() as f64;
        let bitrate: f64 = samples.iter().map(|s| s.bitrate_kbps).sum::<f64>() / n;
        let loss: f64 = samples.iter().map(|s| s.packet_loss_pct).sum::<f64>() / n;
        let rtt: f64 = samples.iter().map(|s| s.rtt_ms).sum::<f64>() / n;
        let simulcast_tiers_used: std::collections::HashSet<String> =
            samples.iter().map(|s| s.simulcast_tier.clone()).collect();
        Ok(serde_json::json!({
            "samples": samples.len(),
            "avg_bitrate_kbps": (bitrate * 10.0).round() / 10.0,
            "avg_packet_loss_pct": (loss * 100.0).round() / 100.0,
            "avg_rtt_ms": (rtt * 10.0).round() / 10.0,
            "simulcast_tiers_used": simulcast_tiers_used,
        }))
    }

    // ═══════════════════════════════════════════
    // PHASE 8: CALL RECORDINGS (opt-in, encrypted)
    // ═══════════════════════════════════════════

    pub fn start_call_recording(
        &self,
        call_id: &str,
        storage_key: &str,
        encrypted: bool,
        size_bytes: i64,
    ) -> Result<i64> {
        let segment_id = self.next_seq("call_recordings");
        let rec = CallRecording {
            call_id: call_id.into(),
            segment_id,
            started_at: now(),
            ended_at: None,
            storage_key: storage_key.into(),
            size_bytes,
            encrypted,
            status: "recording".into(),
        };
        self.mm_add(MM_CALL_RECORDING, call_id, &to_json(&rec))?;
        self.set_call_recording(call_id, true, encrypted)?;
        Ok(segment_id)
    }

    pub fn finalize_call_recording(&self, call_id: &str, segment_id: i64) -> Result<()> {
        let entries = self.mm_get_all(MM_CALL_RECORDING, call_id)?;
        let mut recs: Vec<CallRecording> = entries
            .iter()
            .filter_map(|s| serde_json::from_str::<CallRecording>(s).ok())
            .collect();
        let matched = recs
            .iter_mut()
            .find(|r| r.segment_id == segment_id && r.status == "recording");
        match matched {
            Some(rec) => {
                rec.ended_at = Some(now());
                rec.status = "finalized".into();
            }
            None => anyhow::bail!("Recording segment {} not found", segment_id),
        }
        self.mm_remove_all(MM_CALL_RECORDING, call_id)?;
        for rec in &recs {
            self.mm_add(MM_CALL_RECORDING, call_id, &to_json(rec))?;
        }
        // keep the call flagged as recording only while a segment is active
        self.set_call_recording(call_id, false, false)?;
        Ok(())
    }

    pub fn list_call_recordings(&self, call_id: &str) -> Result<Vec<CallRecording>> {
        let entries = self.mm_get_all(MM_CALL_RECORDING, call_id)?;
        Ok(entries
            .iter()
            .filter_map(|s| serde_json::from_str(s).ok())
            .collect())
    }

    // Random available peer for flash calls (not self, not in an active call).
    pub fn find_random_peer(&self, exclude_user_id: i64) -> Result<Option<i64>> {
        let txn = self.inner.begin_read()?;
        let t = txn.open_table(T_USER)?;
        let mut candidates: Vec<i64> = Vec::new();
        for entry in t.iter()? {
            let (k, _) = entry?;
            if let Ok(uid) = k.value().parse::<i64>()
                && uid != exclude_user_id
            {
                candidates.push(uid);
            }
        }
        drop(t);
        drop(txn);
        let busy: std::collections::HashSet<i64> = self
            .mm_get_all_entries(MM_CALL_USER)?
            .into_iter()
            .filter_map(|(k, _)| k.parse::<i64>().ok())
            .collect();
        candidates.retain(|c| !busy.contains(c));
        if candidates.is_empty() {
            return Ok(None);
        }
        let idx = rand::rng().random_range(0usize..candidates.len());
        Ok(Some(candidates[idx]))
    }

    // ═══════════════════════════════════════════
    // PHASE 9: COMMISSIONS
    // ═══════════════════════════════════════════

    pub fn create_commission(
        &self,
        user_id: i64,
        source_user_id: i64,
        source_tx_id: Option<i64>,
        tier: i32,
        percentage: f64,
        amount: i64,
    ) -> Result<i64> {
        let id = self.next_seq("commissions");
        let c = Commission {
            id,
            user_id,
            source_user_id,
            source_tx_id,
            tier,
            percentage,
            amount,
            status: "pending".into(),
            created_at: now(),
            paid_at: None,
        };
        self.mm_add(MM_COMMISSION, &user_id.to_string(), &to_json(&c))?;
        Ok(id)
    }

    pub fn get_user_commissions(
        &self,
        user_id: i64,
        status: Option<&str>,
    ) -> Result<Vec<serde_json::Value>> {
        self.mm_get_all(MM_COMMISSION, &user_id.to_string())?
            .iter()
            .filter_map(|s| serde_json::from_str::<serde_json::Value>(s).ok())
            .filter(|c| status.is_none_or(|s| c["status"].as_str() == Some(s)))
            .collect::<Vec<_>>()
            .pipe(Ok)
    }

    pub fn get_commission_summary(&self, user_id: i64) -> Result<serde_json::Value> {
        let entries = self.mm_get_all(MM_COMMISSION, &user_id.to_string())?;
        let commissions: Vec<Commission> = entries
            .iter()
            .filter_map(|s| serde_json::from_str(s).ok())
            .collect();
        let total_earned = commissions.iter().map(|c| c.amount).sum::<i64>();
        let pending = commissions
            .iter()
            .filter(|c| c.status == "pending")
            .map(|c| c.amount)
            .sum::<i64>();
        let paid = commissions
            .iter()
            .filter(|c| c.status == "paid")
            .map(|c| c.amount)
            .sum::<i64>();
        let referrals_count = commissions
            .iter()
            .map(|c| c.source_user_id)
            .collect::<std::collections::HashSet<_>>()
            .len() as i64;
        Ok(
            serde_json::json!({"total_earned": total_earned, "pending": pending, "paid": paid, "referrals_count": referrals_count}),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn distribute_commissions(
        &self,
        source_user_id: i64,
        _source_tx_id: Option<i64>,
        total_amount: i64,
        tier1_user_id: Option<i64>,
        tier2_user_id: Option<i64>,
        tier3_user_id: Option<i64>,
        tier4_user_id: Option<i64>,
    ) -> Result<()> {
        let tiers: Vec<(Option<i64>, f64)> = vec![
            (tier1_user_id, 0.40),
            (tier2_user_id, 0.20),
            (tier3_user_id, 0.10),
            (tier4_user_id, 0.05),
        ];
        for (i, (uid, pct)) in tiers.into_iter().enumerate() {
            if let Some(uid) = uid {
                if uid == source_user_id {
                    continue;
                }
                let amount = (total_amount as f64 * pct) as i64;
                if amount > 0 {
                    self.create_commission(uid, source_user_id, None, (i + 1) as i32, pct, amount)?;
                    self.deposit(
                        uid,
                        amount,
                        &format!("Commission tier {} from user #{}", i + 1, source_user_id),
                    )?;
                }
            }
        }
        Ok(())
    }

    pub fn pay_pending_commissions(&self, user_id: i64) -> Result<i64> {
        let entries = self.mm_get_all(MM_COMMISSION, &user_id.to_string())?;
        let total: i64 = entries
            .iter()
            .filter_map(|s| serde_json::from_str::<Commission>(s).ok())
            .filter(|c| c.status == "pending")
            .map(|c| c.amount)
            .sum();
        if total > 0 {
            self.mm_remove_all(MM_COMMISSION, &user_id.to_string())?;
            for entry in &entries {
                if let Ok(mut c) = serde_json::from_str::<Commission>(entry) {
                    if c.status == "pending" {
                        c.status = "paid".into();
                        c.paid_at = Some(now());
                    }
                    self.mm_add(MM_COMMISSION, &user_id.to_string(), &to_json(&c))?;
                }
            }
        }
        Ok(total)
    }

    // ═══════════════════════════════════════════
    // PHASE 9: PAYOUTS
    // ═══════════════════════════════════════════

    pub fn request_payout(
        &self,
        user_id: i64,
        amount: i64,
        currency: &str,
        wallet_address: &str,
        network: &str,
    ) -> Result<i64> {
        let balance = self.get_balance(user_id)?;
        if balance < amount {
            anyhow::bail!("Insufficient balance for payout: {} < {}", balance, amount);
        }
        if amount < 10 {
            anyhow::bail!("Minimum payout is 10 YSH");
        }

        let mut w = self
            .get_json::<Wallet>(T_WALLET, &user_id.to_string())?
            .unwrap();
        w.balance -= amount;
        w.updated_at = now();
        self.put_json(T_WALLET, &user_id.to_string(), &w)?;

        let id = self.next_seq("payouts");
        let p = Payout {
            id,
            user_id,
            amount,
            currency: currency.into(),
            wallet_address: wallet_address.into(),
            network: network.into(),
            status: "pending".into(),
            tx_hash: None,
            requested_at: now(),
            processed_at: None,
            admin_id: None,
            notes: String::new(),
        };
        self.mm_add(MM_PAYOUT, &user_id.to_string(), &to_json(&p))?;

        let tx_id = self.next_seq("transactions");
        let tx = Transaction {
            id: tx_id,
            user_id,
            tx_type: "payout_request".into(),
            amount,
            description: format!("Payout {} {} to {}", amount, currency, wallet_address),
            target_user_id: None,
            created_at: now(),
        };
        self.mm_add(MM_TRANSACTION, &user_id.to_string(), &to_json(&tx))?;
        Ok(id)
    }

    pub fn get_user_payouts(&self, user_id: i64) -> Result<Vec<serde_json::Value>> {
        self.mm_get_all(MM_PAYOUT, &user_id.to_string())?
            .iter()
            .filter_map(|s| serde_json::from_str(s).ok())
            .collect::<Vec<_>>()
            .pipe(Ok)
    }

    pub fn process_payout(
        &self,
        payout_id: i64,
        admin_id: i64,
        tx_hash: &str,
        approved: bool,
    ) -> Result<()> {
        let entries = self.mm_get_all_entries(MM_PAYOUT)?;
        for (user_key, val) in &entries {
            if let Ok(mut p) = serde_json::from_str::<Payout>(val)
                && p.id == payout_id
            {
                p.status = if approved {
                    "completed".into()
                } else {
                    "rejected".into()
                };
                p.tx_hash = Some(tx_hash.to_string());
                p.processed_at = Some(now());
                p.admin_id = Some(admin_id);
                p.notes = if approved {
                    "Approved".into()
                } else {
                    "Rejected".into()
                };
                self.mm_remove_all(MM_PAYOUT, user_key)?;
                self.mm_add(MM_PAYOUT, user_key, &to_json(&p))?;
                if !approved {
                    let mut w = self.get_json::<Wallet>(T_WALLET, user_key)?.unwrap();
                    w.balance += p.amount;
                    self.put_json(T_WALLET, user_key, &w)?;
                }
                return Ok(());
            }
        }
        anyhow::bail!("Payout not found")
    }

    pub fn get_pending_payouts(&self) -> Result<Vec<serde_json::Value>> {
        let entries = self.mm_get_all_entries(MM_PAYOUT)?;
        entries.iter().filter_map(|(_, val)| {
            let p: Payout = serde_json::from_str(val).ok()?;
            if p.status == "pending" {
                let username = self.find_user_by_id(p.user_id).ok().flatten().map(|u| u.username).unwrap_or_default();
                Some(serde_json::json!({
                    "id": p.id, "user_id": p.user_id, "username": username,
                    "amount": p.amount, "currency": p.currency,
                    "wallet_address": p.wallet_address, "network": p.network, "requested_at": p.requested_at
                }))
            } else { None }
        }).collect::<Vec<_>>().pipe(Ok)
    }

    // ═══════════════════════════════════════════
    // PHASE 9: FRAUD
    // ═══════════════════════════════════════════

    pub fn create_fraud_alert(
        &self,
        user_id: Option<i64>,
        alert_type: &str,
        severity: &str,
        description: &str,
        evidence: &str,
        ip_address: Option<&str>,
    ) -> Result<i64> {
        let id = self.next_seq("fraud_alerts");
        let a = FraudAlert {
            id,
            user_id,
            alert_type: alert_type.into(),
            severity: severity.into(),
            description: description.into(),
            evidence: evidence.into(),
            status: "open".into(),
            ip_address: ip_address.map(String::from),
            created_at: now(),
            resolved_at: None,
            resolved_by: None,
        };
        self.mm_add(MM_FRAUD, "all", &to_json(&a))?;
        Ok(id)
    }

    pub fn get_fraud_alerts(&self, status: Option<&str>) -> Result<Vec<serde_json::Value>> {
        self.mm_get_all(MM_FRAUD, "all")?
            .iter()
            .filter_map(|s| serde_json::from_str::<serde_json::Value>(s).ok())
            .filter(|a| status.is_none_or(|s| a["status"].as_str() == Some(s)))
            .collect::<Vec<_>>()
            .pipe(Ok)
    }

    pub fn resolve_fraud_alert(&self, alert_id: i64, resolver_id: i64) -> Result<()> {
        let entries = self.mm_get_all(MM_FRAUD, "all")?;
        self.mm_remove_all(MM_FRAUD, "all")?;
        for entry in &entries {
            if let Ok(mut a) = serde_json::from_str::<FraudAlert>(entry) {
                if a.id == alert_id {
                    a.status = "resolved".into();
                    a.resolved_at = Some(now());
                    a.resolved_by = Some(resolver_id);
                }
                self.mm_add(MM_FRAUD, "all", &to_json(&a))?;
            }
        }
        Ok(())
    }

    pub fn check_velocity(
        &self,
        user_id: i64,
        tx_type: &str,
        window_secs: i64,
    ) -> Result<(i64, i64)> {
        let entries = self.mm_get_all(MM_TRANSACTION, &user_id.to_string())?;
        let cutoff = chrono::Utc::now() - chrono::Duration::seconds(window_secs);
        let mut count = 0i64;
        let mut total_amount = 0i64;
        for entry in &entries {
            if let Ok(tx) = serde_json::from_str::<Transaction>(entry)
                && tx.tx_type == tx_type
                && let Ok(created) =
                    chrono::NaiveDateTime::parse_from_str(&tx.created_at, "%Y-%m-%dT%H:%M:%SZ")
                && created.and_utc() >= cutoff
            {
                count += 1;
                total_amount += tx.amount.abs();
            }
        }
        Ok((count, total_amount))
    }

    // ═══════════════════════════════════════════
    // PHASE 9: RECEIPTS
    // ═══════════════════════════════════════════

    #[allow(clippy::too_many_arguments)]
    pub fn create_receipt(
        &self,
        user_id: i64,
        receipt_type: &str,
        reference_id: i64,
        amount: i64,
        currency: &str,
        description: &str,
        metadata: &str,
    ) -> Result<i64> {
        let id = self.next_seq("receipts");
        let content = format!(
            "{}|{}|{}|{}|{}|{}",
            user_id, receipt_type, reference_id, amount, currency, description
        );
        let hash = blake3::hash(content.as_bytes()).to_hex().to_string();
        let r = Receipt {
            id,
            user_id,
            receipt_type: receipt_type.into(),
            reference_id,
            amount,
            currency: currency.into(),
            description: description.into(),
            metadata: metadata.into(),
            receipt_hash: hash,
            created_at: now(),
        };
        self.mm_add(MM_RECEIPT, &user_id.to_string(), &to_json(&r))?;
        Ok(id)
    }

    pub fn get_receipt(&self, receipt_id: i64) -> Result<Option<serde_json::Value>> {
        let entries = self.mm_get_all_entries(MM_RECEIPT)?;
        for (_, val) in &entries {
            if let Ok(r) = serde_json::from_str::<Receipt>(val)
                && r.id == receipt_id
            {
                return Ok(Some(serde_json::to_value(r)?));
            }
        }
        Ok(None)
    }

    pub fn get_user_receipts(&self, user_id: i64, limit: i64) -> Result<Vec<serde_json::Value>> {
        self.mm_get_all(MM_RECEIPT, &user_id.to_string())?
            .iter()
            .filter_map(|s| serde_json::from_str::<serde_json::Value>(s).ok())
            .take(limit as usize)
            .collect::<Vec<_>>()
            .pipe(Ok)
    }

    pub fn verify_receipt(&self, receipt_id: i64) -> Result<bool> {
        let entries = self.mm_get_all_entries(MM_RECEIPT)?;
        for (_, val) in &entries {
            if let Ok(r) = serde_json::from_str::<Receipt>(val)
                && r.id == receipt_id
            {
                let content = format!(
                    "{}|{}|{}|{}|{}|{}",
                    r.user_id, r.receipt_type, r.reference_id, r.amount, r.currency, r.description
                );
                let computed = blake3::hash(content.as_bytes()).to_hex().to_string();
                return Ok(computed == r.receipt_hash);
            }
        }
        Ok(false)
    }

    // ═══════════════════════════════════════════
    // PHASE 9: SPENDING LIMITS
    // ═══════════════════════════════════════════

    pub fn check_spending_limit(&self, user_id: i64, amount: i64) -> Result<(bool, String)> {
        let limits = self.get_spending_limits(user_id)?;
        let daily_limit = limits["daily_limit"].as_i64().unwrap_or(100000);
        let monthly_limit = limits["monthly_limit"].as_i64().unwrap_or(1000000);
        let daily_spent = limits["daily_spent"].as_i64().unwrap_or(0);
        let monthly_spent = limits["monthly_spent"].as_i64().unwrap_or(0);
        let last_reset = limits["last_reset_date"].as_str().unwrap_or("");

        let today_str = today();
        let this_month_str = this_month();

        let mut sl = SpendingLimit {
            user_id,
            daily_limit,
            monthly_limit,
            daily_spent: if last_reset != today_str {
                0
            } else {
                daily_spent
            },
            monthly_spent: if !last_reset.starts_with(&this_month_str) {
                0
            } else {
                monthly_spent
            },
            last_reset_date: today_str.clone(),
        };

        if sl.daily_spent + amount > sl.daily_limit {
            return Ok((
                false,
                format!(
                    "Daily limit exceeded: {} + {} > {}",
                    sl.daily_spent, amount, sl.daily_limit
                ),
            ));
        }
        if sl.monthly_spent + amount > sl.monthly_limit {
            return Ok((
                false,
                format!(
                    "Monthly limit exceeded: {} + {} > {}",
                    sl.monthly_spent, amount, sl.monthly_limit
                ),
            ));
        }

        sl.daily_spent += amount;
        sl.monthly_spent += amount;
        self.put_json(T_SPENDING, &user_id.to_string(), &sl)?;
        Ok((true, "OK".into()))
    }

    pub fn set_spending_limit(&self, user_id: i64, daily: i64, monthly: i64) -> Result<()> {
        let sl = SpendingLimit {
            user_id,
            daily_limit: daily,
            monthly_limit: monthly,
            daily_spent: 0,
            monthly_spent: 0,
            last_reset_date: today(),
        };
        self.put_json(T_SPENDING, &user_id.to_string(), &sl)
    }

    pub fn get_spending_limits(&self, user_id: i64) -> Result<serde_json::Value> {
        match self.get_json::<SpendingLimit>(T_SPENDING, &user_id.to_string())? {
            Some(sl) => Ok(serde_json::to_value(sl)?),
            None => Ok(serde_json::json!({
                "daily_limit": 100000, "monthly_limit": 1000000,
                "daily_spent": 0, "monthly_spent": 0, "last_reset_date": today()
            })),
        }
    }

    pub fn update_notification_status(&self, _notification_id: i64, _status: &str) -> Result<()> {
        Ok(())
    }
    pub fn increment_notification_retries(&self, _notification_id: i64) -> Result<i32> {
        Ok(0)
    }
    pub fn get_pending_notifications(&self, _limit: i64) -> Result<Vec<serde_json::Value>> {
        Ok(vec![])
    }

    // ═══════════════════════════════════════════
    // FASE 13: USER BLOCKS
    // ═══════════════════════════════════════════

    pub fn block_user(&self, blocker_id: i64, blocked_id: i64) -> Result<()> {
        if blocker_id == blocked_id {
            anyhow::bail!("Cannot block yourself");
        }
        if self.find_user_by_id(blocked_id)?.is_none() {
            anyhow::bail!("Target user not found");
        }
        let rec = BlockRecord {
            blocked_user_id: blocked_id,
            created_at: now(),
        };
        self.mm_add(MM_BLOCK, &blocker_id.to_string(), &to_json(&rec))
    }

    pub fn unblock_user(&self, blocker_id: i64, blocked_id: i64) -> Result<bool> {
        let entries = self.mm_get_all(MM_BLOCK, &blocker_id.to_string())?;
        for entry in &entries {
            if let Ok(rec) = serde_json::from_str::<BlockRecord>(entry)
                && rec.blocked_user_id == blocked_id
            {
                return self.mm_remove_one(MM_BLOCK, &blocker_id.to_string(), entry);
            }
        }
        Ok(false)
    }

    pub fn get_blocked_users(&self, blocker_id: i64) -> Result<Vec<BlockRecord>> {
        let mut recs: Vec<BlockRecord> = self
            .mm_get_all(MM_BLOCK, &blocker_id.to_string())?
            .iter()
            .filter_map(|s| serde_json::from_str(s).ok())
            .collect();
        recs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(recs)
    }

    pub fn is_blocked(&self, viewer_id: i64, author_id: i64) -> Result<bool> {
        Ok(self
            .get_blocked_users(viewer_id)?
            .iter()
            .any(|r| r.blocked_user_id == author_id))
    }

    // ═══════════════════════════════════════════
    // FASE 13: RATINGS + REPUTATION
    // ═══════════════════════════════════════════

    pub fn rate_user(&self, rater_id: i64, rated_id: i64, score: f64) -> Result<()> {
        if rater_id == rated_id {
            anyhow::bail!("Cannot rate yourself");
        }
        if !(1.0..=5.0).contains(&score) {
            anyhow::bail!("Score must be between 1 and 5");
        }
        if self.find_user_by_id(rated_id)?.is_none() {
            anyhow::bail!("Target user not found");
        }
        let entries = self.mm_get_all(MM_RATING, &rated_id.to_string())?;
        self.mm_remove_all(MM_RATING, &rated_id.to_string())?;
        let mut rewrote = false;
        for entry in &entries {
            if let Ok(mut r) = serde_json::from_str::<Rating>(entry) {
                if r.rater_id == rater_id {
                    r.score = score;
                    r.created_at = now();
                    rewrote = true;
                }
                self.mm_add(MM_RATING, &rated_id.to_string(), &to_json(&r))?;
            }
        }
        if !rewrote {
            let id = self.next_seq("user_ratings");
            let r = Rating {
                id,
                rater_id,
                score,
                created_at: now(),
            };
            self.mm_add(MM_RATING, &rated_id.to_string(), &to_json(&r))?;
        }
        self.recompute_reputation(rated_id)
    }

    fn recompute_reputation(&self, user_id: i64) -> Result<()> {
        let entries = self.mm_get_all(MM_RATING, &user_id.to_string())?;
        let ratings: Vec<Rating> = entries
            .iter()
            .filter_map(|s| serde_json::from_str(s).ok())
            .collect();
        let avg = if ratings.is_empty() {
            0.0
        } else {
            ratings.iter().map(|r| r.score).sum::<f64>() / ratings.len() as f64
        };
        let summary = ReputationSummary {
            user_id,
            rating_avg: (avg * 10.0).round() / 10.0,
            rating_count: ratings.len() as i64,
        };
        self.put_json(T_REPUTATION, &user_id.to_string(), &summary)
    }

    pub fn get_reputation(&self, user_id: i64) -> Result<ReputationSummary> {
        Ok(self
            .get_json::<ReputationSummary>(T_REPUTATION, &user_id.to_string())?
            .unwrap_or(ReputationSummary {
                user_id,
                rating_avg: 0.0,
                rating_count: 0,
            }))
    }

    pub fn get_ratings(&self, user_id: i64) -> Result<Vec<Rating>> {
        Ok(self
            .mm_get_all(MM_RATING, &user_id.to_string())?
            .iter()
            .filter_map(|s| serde_json::from_str(s).ok())
            .collect())
    }

    // ═══════════════════════════════════════════
    // FASE 13: VERIFICATION BADGES
    // ═══════════════════════════════════════════

    pub fn grant_badge(&self, user_id: i64, badge_type: &str) -> Result<i64> {
        if self.find_user_by_id(user_id)?.is_none() {
            anyhow::bail!("User not found");
        }
        let existing = self.get_user_badges(user_id)?;
        if existing.iter().any(|b| b.badge_type == badge_type) {
            anyhow::bail!("Badge already granted");
        }
        let id = self.next_seq("verification_badges");
        let b = VerificationBadge {
            id,
            user_id,
            badge_type: badge_type.into(),
            granted_at: now(),
            active: true,
        };
        self.mm_add(MM_BADGE, &user_id.to_string(), &to_json(&b))?;
        Ok(id)
    }

    pub fn revoke_badge(&self, user_id: i64, badge_type: &str) -> Result<bool> {
        let entries = self.mm_get_all(MM_BADGE, &user_id.to_string())?;
        self.mm_remove_all(MM_BADGE, &user_id.to_string())?;
        let mut found = false;
        for entry in &entries {
            if let Ok(mut b) = serde_json::from_str::<VerificationBadge>(entry) {
                if b.badge_type == badge_type && b.active {
                    b.active = false;
                    found = true;
                }
                self.mm_add(MM_BADGE, &user_id.to_string(), &to_json(&b))?;
            }
        }
        Ok(found)
    }

    pub fn get_user_badges(&self, user_id: i64) -> Result<Vec<VerificationBadge>> {
        Ok(self
            .mm_get_all(MM_BADGE, &user_id.to_string())?
            .iter()
            .filter_map(|s| serde_json::from_str(s).ok())
            .filter(|b: &VerificationBadge| b.active)
            .collect())
    }

    pub fn has_badge(&self, user_id: i64, badge_type: &str) -> Result<bool> {
        Ok(self
            .get_user_badges(user_id)?
            .iter()
            .any(|b| b.badge_type == badge_type))
    }

    // ═══════════════════════════════════════════
    // FASE 13: USER REPORTS
    // ═══════════════════════════════════════════

    pub fn create_report(
        &self,
        reporter_id: i64,
        target_type: &str,
        target_id: i64,
        category: &str,
        description: &str,
    ) -> Result<i64> {
        let id = self.next_seq("reports");
        let r = Report {
            id,
            reporter_id,
            target_type: target_type.into(),
            target_id,
            category: category.into(),
            description: description.into(),
            status: "pending".into(),
            created_at: now(),
            reviewed_at: None,
            reviewed_by: None,
        };
        self.mm_add(MM_REPORT, "all", &to_json(&r))?;
        let severity = report_severity(category);
        self.enqueue_moderation_item(
            "report",
            id,
            severity,
            format!("Report #{}: {} ({})", id, category, target_type),
        )?;
        Ok(id)
    }

    pub fn get_reports(&self, status: Option<&str>) -> Result<Vec<Report>> {
        let mut reports: Vec<Report> = self
            .mm_get_all(MM_REPORT, "all")?
            .iter()
            .filter_map(|s| serde_json::from_str(s).ok())
            .collect();
        if let Some(status) = status {
            reports.retain(|r| r.status == status);
        }
        reports.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(reports)
    }

    pub fn resolve_report(&self, report_id: i64, resolver_id: i64, status: &str) -> Result<()> {
        let entries = self.mm_get_all(MM_REPORT, "all")?;
        self.mm_remove_all(MM_REPORT, "all")?;
        for entry in &entries {
            if let Ok(mut r) = serde_json::from_str::<Report>(entry) {
                if r.id == report_id {
                    r.status = status.into();
                    r.reviewed_at = Some(now());
                    r.reviewed_by = Some(resolver_id);
                }
                self.mm_add(MM_REPORT, "all", &to_json(&r))?;
            }
        }
        Ok(())
    }

    pub fn get_user_reports(&self, reporter_id: i64) -> Result<Vec<Report>> {
        let mut reports: Vec<Report> = self
            .mm_get_all(MM_REPORT, "all")?
            .iter()
            .filter_map(|s| serde_json::from_str(s).ok())
            .filter(|r: &Report| r.reporter_id == reporter_id)
            .collect();
        reports.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(reports)
    }

    pub fn count_open_reports_for(&self, target_type: &str, target_id: i64) -> Result<i64> {
        let reports = self.get_reports(None)?;
        Ok(reports
            .iter()
            .filter(|r| {
                r.target_type == target_type && r.target_id == target_id && r.status != "dismissed"
            })
            .count() as i64)
    }

    pub fn distinct_reporters_for(&self, target_type: &str, target_id: i64) -> Result<i64> {
        let reports = self.get_reports(None)?;
        let reporters: std::collections::HashSet<i64> = reports
            .iter()
            .filter(|r| {
                r.target_type == target_type && r.target_id == target_id && r.status != "dismissed"
            })
            .map(|r| r.reporter_id)
            .collect();
        Ok(reporters.len() as i64)
    }

    // ═══════════════════════════════════════════
    // FASE 13: CONTENT FLAGS (auto + manual)
    // ═══════════════════════════════════════════

    pub fn flag_content(
        &self,
        flag_type: &str,
        source: &str,
        target_type: &str,
        target_id: i64,
        severity: f64,
        description: &str,
    ) -> Result<i64> {
        let id = self.next_seq("content_flags");
        let f = ContentFlag {
            id,
            target_type: target_type.into(),
            target_id,
            flag_type: flag_type.into(),
            source: source.into(),
            severity,
            description: description.into(),
            status: "pending".into(),
            created_at: now(),
            resolved_at: None,
            resolved_by: None,
        };
        self.mm_add(MM_CONTENT_FLAG, "all", &to_json(&f))?;
        self.enqueue_moderation_item(
            "content_flag",
            id,
            severity,
            format!("Content flag #{}: {} on {}", id, flag_type, target_type),
        )?;
        Ok(id)
    }

    pub fn get_content_flags(&self, status: Option<&str>) -> Result<Vec<ContentFlag>> {
        let mut flags: Vec<ContentFlag> = self
            .mm_get_all(MM_CONTENT_FLAG, "all")?
            .iter()
            .filter_map(|s| serde_json::from_str(s).ok())
            .collect();
        if let Some(status) = status {
            flags.retain(|f| f.status == status);
        }
        flags.sort_by(|a, b| {
            b.severity
                .partial_cmp(&a.severity)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.created_at.cmp(&a.created_at))
        });
        Ok(flags)
    }

    pub fn resolve_content_flag(&self, flag_id: i64, resolver_id: i64, status: &str) -> Result<()> {
        let entries = self.mm_get_all(MM_CONTENT_FLAG, "all")?;
        self.mm_remove_all(MM_CONTENT_FLAG, "all")?;
        for entry in &entries {
            if let Ok(mut f) = serde_json::from_str::<ContentFlag>(entry) {
                if f.id == flag_id {
                    f.status = status.into();
                    f.resolved_at = Some(now());
                    f.resolved_by = Some(resolver_id);
                }
                self.mm_add(MM_CONTENT_FLAG, "all", &to_json(&f))?;
            }
        }
        Ok(())
    }

    pub fn is_content_blocked(&self, target_type: &str, target_id: i64) -> Result<bool> {
        Ok(self
            .get_content_flags(Some("actioned"))?
            .iter()
            .any(|f| f.target_type == target_type && f.target_id == target_id))
    }

    // ═══════════════════════════════════════════
    // FASE 13: MODERATION QUEUE (priority by severity)
    // ═══════════════════════════════════════════

    pub fn enqueue_moderation_item(
        &self,
        item_type: &str,
        reference_id: i64,
        severity: f64,
        notes: impl Into<String>,
    ) -> Result<i64> {
        let id = self.next_seq("mod_queue");
        let item = ModQueueItem {
            id,
            item_type: item_type.into(),
            reference_id,
            severity,
            status: "pending".into(),
            notes: notes.into(),
            created_at: now(),
        };
        self.mm_add(MM_MOD_QUEUE, "queue", &to_json(&item))?;
        Ok(id)
    }

    pub fn get_moderation_queue(&self, status: Option<&str>) -> Result<Vec<ModQueueItem>> {
        let mut items: Vec<ModQueueItem> = self
            .mm_get_all(MM_MOD_QUEUE, "queue")?
            .iter()
            .filter_map(|s| serde_json::from_str(s).ok())
            .collect();
        if let Some(status) = status {
            items.retain(|i| i.status == status);
        }
        items.sort_by(|a, b| {
            b.severity
                .partial_cmp(&a.severity)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.created_at.cmp(&a.created_at))
        });
        Ok(items)
    }

    pub fn resolve_moderation_item(&self, item_id: i64, status: &str) -> Result<()> {
        let entries = self.mm_get_all(MM_MOD_QUEUE, "queue")?;
        self.mm_remove_all(MM_MOD_QUEUE, "queue")?;
        for entry in &entries {
            if let Ok(mut i) = serde_json::from_str::<ModQueueItem>(entry) {
                if i.id == item_id {
                    i.status = status.into();
                }
                self.mm_add(MM_MOD_QUEUE, "queue", &to_json(&i))?;
            }
        }
        Ok(())
    }

    pub fn pending_moderation_count(&self) -> Result<i64> {
        Ok(self.get_moderation_queue(Some("pending"))?.len() as i64)
    }

    // ═══════════════════════════════════════════
    // FASE 13: SHADOW BANS
    // ═══════════════════════════════════════════

    pub fn shadow_ban_user(
        &self,
        user_id: i64,
        reason: &str,
        duration_secs: Option<i64>,
    ) -> Result<()> {
        if self.is_shadow_banned(user_id)? {
            return Ok(());
        }
        let banned_until = duration_secs.map(|secs| {
            (chrono::Utc::now() + chrono::Duration::seconds(secs))
                .format("%Y-%m-%dT%H:%M:%SZ")
                .to_string()
        });
        let sb = ShadowBan {
            user_id,
            banned_at: now(),
            reason: reason.into(),
            banned_until,
        };
        self.mm_add(MM_SHADOW, "all", &to_json(&sb))
    }

    pub fn unshadow_ban_user(&self, user_id: i64) -> Result<bool> {
        let entries = self.mm_get_all(MM_SHADOW, "all")?;
        self.mm_remove_all(MM_SHADOW, "all")?;
        let mut found = false;
        for entry in &entries {
            if let Ok(sb) = serde_json::from_str::<ShadowBan>(entry) {
                if sb.user_id == user_id {
                    found = true;
                    continue;
                }
                self.mm_add(MM_SHADOW, "all", &to_json(&sb))?;
            }
        }
        Ok(found)
    }

    pub fn get_shadow_bans(&self) -> Result<Vec<ShadowBan>> {
        Ok(self
            .mm_get_all(MM_SHADOW, "all")?
            .iter()
            .filter_map(|s| serde_json::from_str(s).ok())
            .collect())
    }

    pub fn active_shadow_ban_ids(&self) -> Result<std::collections::HashSet<i64>> {
        let now_str = now();
        Ok(self
            .get_shadow_bans()?
            .into_iter()
            .filter(|sb| match &sb.banned_until {
                Some(until) => until > &now_str,
                None => true,
            })
            .map(|sb| sb.user_id)
            .collect())
    }

    pub fn is_shadow_banned(&self, user_id: i64) -> Result<bool> {
        let now_str = now();
        Ok(self.get_shadow_bans()?.iter().any(|sb| {
            sb.user_id == user_id
                && match &sb.banned_until {
                    Some(until) => until > &now_str,
                    None => true,
                }
        }))
    }

    // ═══════════════════════════════════════════
    // FASE 13: APPEALS
    // ═══════════════════════════════════════════

    pub fn create_appeal(
        &self,
        user_id: i64,
        target_type: &str,
        target_id: i64,
        reason: &str,
    ) -> Result<i64> {
        let id = self.next_seq("appeals");
        let a = Appeal {
            id,
            user_id,
            target_type: target_type.into(),
            target_id,
            reason: reason.into(),
            status: "open".into(),
            created_at: now(),
            reviewed_at: None,
            reviewed_by: None,
            admin_notes: String::new(),
        };
        self.mm_add(MM_APPEAL, "all", &to_json(&a))?;
        self.enqueue_moderation_item(
            "appeal",
            id,
            0.7,
            format!("Appeal #{}: {} #{}", id, target_type, target_id),
        )?;
        Ok(id)
    }

    pub fn get_user_appeals(&self, user_id: i64) -> Result<Vec<Appeal>> {
        let mut appeals: Vec<Appeal> = self
            .mm_get_all(MM_APPEAL, "all")?
            .iter()
            .filter_map(|s| serde_json::from_str(s).ok())
            .filter(|a: &Appeal| a.user_id == user_id)
            .collect();
        appeals.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(appeals)
    }

    pub fn get_appeals(&self, status: Option<&str>) -> Result<Vec<Appeal>> {
        let mut appeals: Vec<Appeal> = self
            .mm_get_all(MM_APPEAL, "all")?
            .iter()
            .filter_map(|s| serde_json::from_str(s).ok())
            .collect();
        if let Some(status) = status {
            appeals.retain(|a| a.status == status);
        }
        appeals.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(appeals)
    }

    pub fn resolve_appeal(
        &self,
        appeal_id: i64,
        resolver_id: i64,
        approved: bool,
        admin_notes: &str,
    ) -> Result<()> {
        let entries = self.mm_get_all(MM_APPEAL, "all")?;
        self.mm_remove_all(MM_APPEAL, "all")?;
        for entry in &entries {
            if let Ok(mut a) = serde_json::from_str::<Appeal>(entry) {
                if a.id == appeal_id && a.status == "open" {
                    a.status = if approved {
                        "approved".into()
                    } else {
                        "rejected".into()
                    };
                    a.reviewed_at = Some(now());
                    a.reviewed_by = Some(resolver_id);
                    a.admin_notes = admin_notes.to_string();
                    if approved {
                        match a.target_type.as_str() {
                            "shadow_ban" => {
                                let _ = self.unshadow_ban_user(a.user_id)?;
                                let _ = a.target_id;
                            }
                            "ban" => {
                                self.unban_user(a.user_id)?;
                                let _ = a.target_id;
                            }
                            "content_flag" => {
                                self.resolve_content_flag(a.target_id, resolver_id, "dismissed")?;
                            }
                            _ => {}
                        }
                    }
                }
                self.mm_add(MM_APPEAL, "all", &to_json(&a))?;
            }
        }
        Ok(())
    }

    // ═══════════════════════════════════════════
    // FASE 13: TRUST SCORE
    // ═══════════════════════════════════════════

    pub fn compute_trust_score(&self, user_id: i64) -> Result<f64> {
        let cfg = crate::config::settings::default_trust();
        let mut score = cfg.starting_score;
        if !cfg.enabled {
            return Ok(score);
        }

        if let Some(u) = self.find_user_by_id(user_id)? {
            if let Ok(created) =
                chrono::NaiveDateTime::parse_from_str(&u.created_at, "%Y-%m-%dT%H:%M:%SZ")
            {
                let age_days = (chrono::Utc::now() - created.and_utc()).num_days();
                score += (age_days as f64 / 30.0).min(1.0) * cfg.account_age_bonus_max;
            }
            let badge_count = self.get_user_badges(user_id)?.len() as f64;
            score += badge_count * cfg.badge_bonus;
            if u.kyc_level >= 2 {
                score += cfg.badge_bonus;
            }
        }

        let reports = self.get_reports(None)?;
        let report_count = reports
            .iter()
            .filter(|r| {
                r.target_type == "user" && r.target_id == user_id && r.status != "dismissed"
            })
            .count() as f64;
        score -= report_count * cfg.report_penalty;

        let flags = self.get_content_flags(None)?;
        let flag_count = flags
            .iter()
            .filter(|f| {
                f.target_type == "user" && f.target_id == user_id && f.status != "dismissed"
            })
            .count() as f64;
        score -= flag_count * cfg.flag_penalty;

        if self.is_shadow_banned(user_id)? {
            score -= cfg.shadow_ban_penalty;
        }

        if let Some(u) = self.find_user_by_id(user_id)?
            && let Some(l) = &u.locked_until
            && l.as_str() >= "2099-01-01"
        {
            score -= cfg.ban_penalty;
        }

        let score = score.clamp(0.0, 100.0);
        let stored = serde_json::json!({ "updated_at": now() });
        let _ = stored;
        self.put_json(
            T_TRUST,
            &user_id.to_string(),
            &serde_json::json!({
                "user_id": user_id,
                "score": (score * 10.0).round() / 10.0,
                "updated_at": now(),
            }),
        )?;
        Ok(score)
    }

    pub fn get_trust_score(&self, user_id: i64) -> Result<serde_json::Value> {
        let score = self.compute_trust_score(user_id)?;
        Ok(serde_json::json!({
            "user_id": user_id,
            "score": (score * 10.0).round() / 10.0,
            "level": trust_level(score),
        }))
    }

    // ═══════════════════════════════════════════
    // FASE 14: BACKGROUND JOBS (db side)
    // ═══════════════════════════════════════════

    /// Records a user action for analytics (activity feed, DAU/MAU).
    pub fn log_activity(&self, user_id: i64, action: &str) -> Result<()> {
        let day = today();
        let key = format!("{}::{}", day, user_id);
        for existing in self.mm_get_all(MM_USER_ACTIVITY, &key)? {
            if let Ok(ev) = serde_json::from_str::<serde_json::Value>(&existing)
                && ev["action"] == action
            {
                return Ok(());
            }
        }
        let entry = serde_json::json!({ "user_id": user_id, "action": action, "ts": now() });
        self.mm_add(MM_USER_ACTIVITY, &key, &to_json(&entry))
    }

    /// Sets the geographic region for a user (geo analytics).
    pub fn set_user_region(&self, user_id: i64, region: &str) -> Result<()> {
        let mut user = self
            .find_user_by_id(user_id)?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;
        user.region = region.to_string();
        self.put_json(T_USER, &user_id.to_string(), &user)
    }

    /// Test/tooling helper: backdate a stake's interest reference point.
    pub fn set_staking_recalc(&self, stake_id: i64, last_reward_calc: &str) -> Result<()> {
        let entries = self.mm_get_all_entries(MM_STAKING)?;
        for (user_key, val) in &entries {
            if let Ok(mut s) = serde_json::from_str::<Staking>(val)
                && s.id == stake_id
            {
                s.last_reward_calc = last_reward_calc.to_string();
                self.mm_remove_one(MM_STAKING, user_key, val)?;
                self.mm_add(MM_STAKING, user_key, &to_json(&s))?;
                return Ok(());
            }
        }
        anyhow::bail!("Stake not found")
    }

    /// Test/tooling helper: backdate every pending moderation item.
    pub fn age_moderation_items(&self, secs_back: i64) -> Result<i64> {
        let mut aged = 0i64;
        for val in self.mm_get_all(MM_MOD_QUEUE, "queue")? {
            if let Ok(mut item) = serde_json::from_str::<ModQueueItem>(&val) {
                let shifted = (chrono::Utc::now() - chrono::Duration::seconds(secs_back))
                    .format("%Y-%m-%dT%H:%M:%SZ")
                    .to_string();
                item.created_at = shifted;
                self.mm_remove_one(MM_MOD_QUEUE, "queue", &val)?;
                self.mm_add(MM_MOD_QUEUE, "queue", &to_json(&item))?;
                aged += 1;
            }
        }
        Ok(aged)
    }

    /// Test/tooling helper: backdate a match queue entry.
    pub fn age_match_queue(&self, secs_back: i64) -> Result<i64> {
        let mut aged = 0i64;
        for (key, val) in self.mm_get_all_entries(MM_MATCH_QUEUE)? {
            if let Ok(mut m) = serde_json::from_str::<serde_json::Value>(&val) {
                let shifted = (chrono::Utc::now() - chrono::Duration::seconds(secs_back))
                    .format("%Y-%m-%dT%H:%M:%SZ")
                    .to_string();
                m["queued_at"] = serde_json::Value::String(shifted);
                self.mm_remove_one(MM_MATCH_QUEUE, &key, &val)?;
                self.mm_add(MM_MATCH_QUEUE, &key, &to_json(&m))?;
                aged += 1;
            }
        }
        Ok(aged)
    }

    /// Test/tooling helper: backdate activity records by N days (key + ts).
    pub fn age_activity(&self, days_back: i64) -> Result<i64> {
        let mut aged = 0i64;
        for (key, val) in self.mm_get_all_entries(MM_USER_ACTIVITY)? {
            let shifted_day = (chrono::Utc::now().date_naive() - chrono::Duration::days(days_back))
                .format("%Y-%m-%d")
                .to_string();
            let shifted_ts = (chrono::Utc::now() - chrono::Duration::days(days_back))
                .format("%Y-%m-%dT%H:%M:%SZ")
                .to_string();
            let uid = key.split("::").nth(1).unwrap_or("0");
            self.mm_remove_one(MM_USER_ACTIVITY, &key, &val)?;
            if let Ok(mut ev) = serde_json::from_str::<serde_json::Value>(&val) {
                ev["ts"] = serde_json::Value::String(shifted_ts);
                self.mm_add(
                    MM_USER_ACTIVITY,
                    &format!("{}::{}", shifted_day, uid),
                    &to_json(&ev),
                )?;
                aged += 1;
            }
        }
        Ok(aged)
    }

    /// PAYOUT WORKER: auto-disburses pending payouts (status pending -> completed
    /// with an auto-generated hash). Returns processed count and total amount.
    pub fn auto_process_payouts(&self) -> Result<serde_json::Value> {
        let entries = self.mm_get_all_entries(MM_PAYOUT)?;
        let mut processed = 0i64;
        let mut total = 0i64;
        for (user_key, val) in &entries {
            if let Ok(mut p) = serde_json::from_str::<Payout>(val)
                && p.status == "pending"
            {
                p.status = "completed".into();
                p.tx_hash = Some(format!("0xauto{:08x}", p.id));
                p.processed_at = Some(now());
                p.notes = "Automated disbursement".into();
                self.mm_remove_one(MM_PAYOUT, user_key, val)?;
                self.mm_add(MM_PAYOUT, user_key, &to_json(&p))?;
                processed += 1;
                total += p.amount;
            }
        }
        Ok(serde_json::json!({ "processed": processed, "total_amount": total }))
    }

    /// STAKING WORKER: accrues compounding interest on active stakes based on
    /// elapsed time since the last calculation. Rewards are credited to the stake
    /// and paid out at withdrawal.
    pub fn compute_staking_interest(&self) -> Result<serde_json::Value> {
        let entries = self.mm_get_all_entries(MM_STAKING)?;
        let now_utc = chrono::Utc::now();
        let mut updated = 0i64;
        let mut total_interest = 0i64;
        for (user_key, val) in &entries {
            if let Ok(mut s) = serde_json::from_str::<Staking>(val) {
                if s.status != "active" || s.amount <= 0 {
                    continue;
                }
                let last = chrono::DateTime::parse_from_rfc3339(&s.last_reward_calc)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or(now_utc);
                let days = (now_utc - last).num_seconds() as f64 / 86400.0;
                if days < 0.5 {
                    continue; // cooldown to avoid rounding noise
                }
                let interest =
                    ((s.amount as f64) * (s.apy_rate / 100.0) * (days / 365.0)).floor() as i64;
                if interest > 0 {
                    s.rewards_earned += interest;
                    s.last_reward_calc = now();
                    self.mm_remove_one(MM_STAKING, user_key, val)?;
                    self.mm_add(MM_STAKING, user_key, &to_json(&s))?;
                    updated += 1;
                    total_interest += interest;
                }
            }
        }
        Ok(serde_json::json!({ "updated": updated, "total_interest": total_interest }))
    }

    /// MODERATION WORKER: auto-resolves stale pending moderation items. Items
    /// older than `auto_resolve_secs` with severity >= `action_above` are
    /// actioned; severity <= `dismiss_below` are dismissed.
    pub fn auto_resolve_moderation(
        &self,
        auto_resolve_secs: i64,
        dismiss_below: f64,
        action_above: f64,
    ) -> Result<serde_json::Value> {
        let entries = self.mm_get_all(MM_MOD_QUEUE, "queue")?;
        let now_utc = chrono::Utc::now();
        let mut dismissed = 0i64;
        let mut actioned = 0i64;
        for val in &entries {
            if let Ok(mut item) = serde_json::from_str::<ModQueueItem>(val) {
                if item.status != "pending" {
                    continue;
                }
                let age = chrono::DateTime::parse_from_rfc3339(&item.created_at)
                    .map(|dt| (now_utc - dt.with_timezone(&chrono::Utc)).num_seconds())
                    .unwrap_or(0);
                if age < auto_resolve_secs {
                    continue;
                }
                if item.severity >= action_above {
                    item.status = "actioned".into();
                    actioned += 1;
                } else if item.severity <= dismiss_below {
                    item.status = "dismissed".into();
                    dismissed += 1;
                } else {
                    continue; // keep for human moderator review
                }
                self.mm_remove_one(MM_MOD_QUEUE, "queue", val)?;
                self.mm_add(MM_MOD_QUEUE, "queue", &to_json(&item))?;
            }
        }
        Ok(serde_json::json!({ "dismissed": dismissed, "actioned": actioned }))
    }

    /// CLEANUP WORKER: removes stale match queue entries (>10 min waiting),
    /// expired call quality samples, and activity data older than retention.
    pub fn cleanup_expired(
        &self,
        analytics_retention_days: i64,
        quality_retention_days: i64,
    ) -> Result<serde_json::Value> {
        let now_utc = chrono::Utc::now();
        let mut removed = 0i64;

        let match_entries = self.mm_get_all_entries(MM_MATCH_QUEUE)?;
        for (key, val) in &match_entries {
            if let Ok(m) = serde_json::from_str::<serde_json::Value>(val)
                && m["status"] == "waiting"
                && m["queued_at"]
                    .as_str()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|c| (now_utc - c.with_timezone(&chrono::Utc)).num_seconds() > 600)
                    .unwrap_or(false)
            {
                self.mm_remove_one(MM_MATCH_QUEUE, key, val)?;
                removed += 1;
            }
        }

        let quality_entries = self.mm_get_all_entries(MM_CALL_QUALITY)?;
        for (key, val) in &quality_entries {
            if let Ok(q) = serde_json::from_str::<serde_json::Value>(val)
                && q["ts"]
                    .as_str()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|t| {
                        (now_utc - t.with_timezone(&chrono::Utc)).num_seconds()
                            > quality_retention_days * 86400
                    })
                    .unwrap_or(false)
            {
                self.mm_remove_one(MM_CALL_QUALITY, key, val)?;
                removed += 1;
            }
        }

        let retention_secs = analytics_retention_days * 86400;
        let activity_entries = self.mm_get_all_entries(MM_USER_ACTIVITY)?;
        for (key, _) in &activity_entries {
            if let Some(day) = key.split("::").next()
                && let Ok(d) = chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d")
            {
                let then = d.and_hms_opt(0, 0, 0).unwrap().and_utc();
                if (now_utc - then).num_seconds() > retention_secs {
                    self.mm_remove_all(MM_USER_ACTIVITY, key)?;
                    removed += 1;
                }
            }
        }

        Ok(serde_json::json!({ "removed": removed }))
    }

    /// NOTIFICATION WORKER: advances in-app notifications to `sent` immediately.
    /// Delivery channels (email/push) bump their retry budget; once exhausted
    /// the notification moves to `failed`.
    pub fn flush_pending_notifications(&self) -> Result<serde_json::Value> {
        let entries = self.mm_get_all_entries(MM_NOTIFICATION)?;
        let mut sent = 0i64;
        let mut failed = 0i64;
        let mut pending = 0i64;
        for (user_key, val) in &entries {
            if let Ok(mut n) = serde_json::from_str::<Notification>(val) {
                if n.status != "pending" {
                    continue;
                }
                if n.channel == "inapp" {
                    n.status = "sent".into();
                    n.sent_at = Some(now());
                    n.retries += 1;
                    sent += 1;
                } else {
                    n.retries += 1;
                    if n.retries >= 3 {
                        n.status = "failed".into();
                        failed += 1;
                    } else {
                        pending += 1;
                    }
                }
                self.mm_remove_one(MM_NOTIFICATION, user_key, val)?;
                self.mm_add(MM_NOTIFICATION, user_key, &to_json(&n))?;
            }
        }
        Ok(serde_json::json!({ "sent": sent, "failed": failed, "pending": pending }))
    }

    /// ANALYTICS WORKER: computes and persists a daily snapshot (DAU, MAU,
    /// new users, revenue, calls, gifts, messages).
    pub fn compute_analytics_snapshot(&self) -> Result<serde_json::Value> {
        let day = today();
        let mut day_users: HashSet<i64> = HashSet::new();
        let mut mau_users: HashSet<i64> = HashSet::new();
        let now_utc = chrono::Utc::now();
        let mau_window = now_utc - chrono::Duration::days(30);

        for (key, val) in self.mm_get_all_entries(MM_USER_ACTIVITY)? {
            if let Ok(ev) = serde_json::from_str::<serde_json::Value>(&val) {
                let uid = ev["user_id"].as_i64().unwrap_or(-1);
                if key.starts_with(&format!("{}::", day)) {
                    day_users.insert(uid);
                }
                let ts = ev["ts"]
                    .as_str()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|t| t.with_timezone(&chrono::Utc));
                if let Some(t) = ts
                    && t >= mau_window
                {
                    mau_users.insert(uid);
                }
            }
        }

        let dau = day_users.len() as i64;
        let mau = mau_users.len() as i64;
        let new_users = self.count_signups_on(&day)?;
        let transactions = self.sum_transactions_on(&day)?;
        let gifts = self.sum_gifts_on(&day)?;
        let calls = self.count_calls_on(&day)?;
        let platform_fees = self.sum_call_fees_range(&day, &day)?;
        let messages = self.count_messages_on(&day)?;

        let snap = serde_json::json!({
            "date": day, "dau": dau, "mau": mau, "new_users": new_users,
            "transactions": transactions, "gifts": gifts, "calls": calls,
            "platform_fees": platform_fees, "messages": messages,
            "computed_at": now(),
        });
        self.mm_add(MM_ANALYTICS_DAY, &day, &to_json(&snap))?;
        Ok(snap)
    }

    // ═══════════════════════════════════════════
    // FASE 15: ANALYTICS (db side)
    // ═══════════════════════════════════════════

    pub fn db_size(&self) -> Result<u64> {
        let meta = std::fs::metadata(&self.db_path)?;
        Ok(meta.len())
    }

    /// Daily analytics snapshots persisted by the analytics worker.
    pub fn list_analytics_snapshots(&self, limit: i64) -> Result<Vec<serde_json::Value>> {
        let mut rows: Vec<serde_json::Value> = self
            .mm_get_all_entries(MM_ANALYTICS_DAY)?
            .iter()
            .filter_map(|(_, v)| serde_json::from_str::<serde_json::Value>(v).ok())
            .filter(|v| v["date"].is_string())
            .collect();
        rows.sort_by(|a, b| b["date"].as_str().cmp(&a["date"].as_str()));
        rows.truncate(limit.max(1) as usize);
        Ok(rows)
    }

    /// Real-time counters that live in the DB.
    pub fn realtime_db_metrics(&self) -> Result<serde_json::Value> {
        let mut active_calls = 0i64;
        for (_, val) in self.mm_get_all_entries(MM_CALL_BILLING)? {
            if let Ok(cb) = serde_json::from_str::<CallBilling>(&val)
                && cb.status == "active"
            {
                active_calls += 1;
            }
        }
        let pending_reports = self.count_reports_with_status("pending")?;
        let pending_payouts = self
            .mm_get_all_entries(MM_PAYOUT)?
            .iter()
            .filter(|(_, v)| {
                serde_json::from_str::<Payout>(v)
                    .map(|p| p.status == "pending")
                    .unwrap_or(false)
            })
            .count() as i64;
        Ok(serde_json::json!({
            "active_calls": active_calls,
            "pending_reports": pending_reports,
            "pending_payouts": pending_payouts,
        }))
    }

    /// USER ANALYTICS: per-day DAU/new signups over the range, plus rolling MAU,
    /// retention and churn (today vs yesterday).
    pub fn get_user_analytics(&self, days: i64) -> Result<serde_json::Value> {
        let today_utc = chrono::Utc::now().date_naive();
        let mut series: Vec<serde_json::Value> = Vec::new();
        let mut day_to_users: BTreeMap<String, HashSet<i64>> = BTreeMap::new();

        for (key, val) in self.mm_get_all_entries(MM_USER_ACTIVITY)? {
            if let Some((day, _)) = key.split_once("::")
                && let Ok(ev) = serde_json::from_str::<serde_json::Value>(&val)
            {
                let uid = ev["user_id"].as_i64().unwrap_or(-1);
                day_to_users.entry(day.to_string()).or_default().insert(uid);
            }
        }

        for offset in (0..days).rev() {
            let day = (today_utc - chrono::Duration::days(offset))
                .format("%Y-%m-%d")
                .to_string();
            let dau = day_to_users.get(&day).map(|s| s.len()).unwrap_or(0) as i64;
            let new_users = self.count_signups_on(&day)?;
            series.push(serde_json::json!({ "date": day, "dau": dau, "new_users": new_users, "retention": null }));
        }

        // retention: users active yesterday who are also active today.
        if series.len() >= 2 {
            let prev = &series[series.len() - 2]["date"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let curr = &series[series.len() - 1]["date"]
                .as_str()
                .unwrap_or("")
                .to_string();
            if let (Some(a), Some(b)) = (day_to_users.get(prev), day_to_users.get(curr)) {
                let retained = a.intersection(b).count() as f64;
                if !a.is_empty() {
                    let retention = retained / a.len() as f64;
                    if let Some(v) = series.last_mut()
                        && let Some(obj) = v.as_object_mut()
                    {
                        obj.insert(
                            "retention".into(),
                            serde_json::json!((retention * 100.0).round() / 100.0),
                        );
                        obj.insert(
                            "churn".into(),
                            serde_json::json!(((1.0 - retention) * 100.0).round() / 100.0),
                        );
                    }
                }
            }
        }

        let mau: usize = day_to_users
            .values()
            .flatten()
            .collect::<HashSet<_>>()
            .len();
        Ok(serde_json::json!({
            "days": series,
            "summary": {
                "mau": mau,
                "total_dau_last_day": series.last().and_then(|s| s["dau"].as_i64()).unwrap_or(0),
            }
        }))
    }

    /// REVENUE ANALYTICS: MRR proxy, ARPU, LTV and gift/call economy totals.
    pub fn get_revenue_analytics(&self, days: i64) -> Result<serde_json::Value> {
        let from = (chrono::Utc::now().date_naive() - chrono::Duration::days(days - 1))
            .format("%Y-%m-%d")
            .to_string();
        let to = today();
        let transactions = self.sum_transactions_range(&from, &to)?;
        let gifts = self.sum_gifts_range(&from, &to)?;
        let platform_fees = self.sum_call_fees_range(&from, &to)?;
        let users = self.count_users()?;
        let active = self.active_users_last(days)?;
        let arpu = if active > 0 {
            (transactions + gifts) / active
        } else {
            0
        };
        Ok(serde_json::json!({
            "range_days": days,
            "transactions": transactions,
            "gifts": gifts,
            "calls_fees": platform_fees,
            "gross_volume": transactions + gifts + platform_fees,
            "arpu": arpu,
            "ltv": arpu * 12,
            "active_users": active,
            "total_users": users,
        }))
    }

    /// AGENCY PERFORMANCE: revenue + calls attributed to each agency's members.
    pub fn get_agency_performance(&self) -> Result<Vec<serde_json::Value>> {
        let mut member_tx: BTreeMap<i64, i64> = BTreeMap::new();
        for (_, val) in self.mm_get_all_entries(MM_TRANSACTION)? {
            if let Ok(t) = serde_json::from_str::<Transaction>(&val) {
                *member_tx.entry(t.user_id).or_insert(0) += t.amount.abs();
            }
        }
        let mut host_calls: BTreeMap<i64, i64> = BTreeMap::new();
        for (_, val) in self.mm_get_all_entries(MM_CALL_BILLING)? {
            if let Ok(cb) = serde_json::from_str::<CallBilling>(&val)
                && cb.status == "completed"
            {
                *host_calls.entry(cb.host_id).or_insert(0) += 1;
            }
        }

        let mut out = Vec::new();
        for agency in self.list_agencies()? {
            let id = agency["id"].as_i64().unwrap_or(0);
            let members = self.get_agency_members(id)?;
            let mut member_revenue = 0i64;
            let mut member_calls = 0i64;
            for m in &members {
                let uid = m["user_id"].as_i64().unwrap_or(0);
                member_revenue += member_tx.get(&uid).copied().unwrap_or(0);
                member_calls += host_calls.get(&uid).copied().unwrap_or(0);
            }
            out.push(serde_json::json!({
                "agency_id": id,
                "name": agency["name"].as_str().unwrap_or(""),
                "members": members.len(),
                "member_revenue": member_revenue,
                "member_calls": member_calls,
            }));
        }
        Ok(out)
    }

    /// HOST PERFORMANCE: leaderboard of hosts by completed calls + earnings.
    pub fn get_host_leaderboard(&self, limit: i64) -> Result<Vec<serde_json::Value>> {
        let mut per_host: BTreeMap<i64, (i64, i64)> = BTreeMap::new();
        for (_, val) in self.mm_get_all_entries(MM_CALL_BILLING)? {
            if let Ok(cb) = serde_json::from_str::<CallBilling>(&val)
                && cb.status == "completed"
            {
                let e = per_host.entry(cb.host_id).or_insert((0, 0));
                e.0 += 1;
                e.1 += cb.host_earnings;
            }
        }
        let mut rows: Vec<serde_json::Value> = per_host.iter().map(|(uid, (calls, earned))| {
            serde_json::json!({ "host_id": uid, "calls": calls, "earnings": earned })
        }).collect();
        rows.sort_by(|a, b| {
            b["earnings"]
                .as_i64()
                .unwrap_or(0)
                .cmp(&a["earnings"].as_i64().unwrap_or(0))
        });
        rows.truncate(limit.max(1) as usize);
        Ok(rows)
    }

    /// GEOGRAPHIC DISTRIBUTION: user base grouped by region.
    pub fn get_geo_distribution(&self) -> Result<serde_json::Value> {
        let mut regions: BTreeMap<String, i64> = BTreeMap::new();
        let txn = self.inner.begin_read()?;
        let t = txn.open_table(T_USER)?;
        for entry in t.iter()? {
            let (_, v) = entry?;
            if let Ok(u) = serde_json::from_str::<serde_json::Value>(v.value()) {
                let region = u["region"].as_str().unwrap_or("unknown").to_string();
                *regions.entry(region).or_insert(0) += 1;
            }
        }
        let total: i64 = regions.values().sum();
        let distribution: Vec<serde_json::Value> = regions
            .iter()
            .map(|(r, c)| {
                let pct = if total > 0 {
                    (*c as f64 / total as f64 * 100.0) as i64
                } else {
                    0
                };
                serde_json::json!({ "region": r, "users": c, "pct": pct })
            })
            .collect();
        Ok(serde_json::json!({ "total_users": total, "distribution": distribution }))
    }

    /// MODERATION METRICS: aggregate counts across the moderation pipeline.
    pub fn get_moderation_metrics(&self) -> Result<serde_json::Value> {
        let mut queue: BTreeMap<String, i64> = BTreeMap::new();
        for val in self.mm_get_all(MM_MOD_QUEUE, "queue")? {
            if let Ok(i) = serde_json::from_str::<ModQueueItem>(&val) {
                *queue.entry(i.status).or_insert(0) += 1;
            }
        }
        let mut reports: BTreeMap<String, i64> = BTreeMap::new();
        for val in self.mm_get_all(MM_REPORT, "all")? {
            if let Ok(r) = serde_json::from_str::<Report>(&val) {
                *reports.entry(r.status).or_insert(0) += 1;
            }
        }
        let mut appeals: BTreeMap<String, i64> = BTreeMap::new();
        for val in self.mm_get_all(MM_APPEAL, "all")? {
            if let Ok(a) = serde_json::from_str::<Appeal>(&val) {
                *appeals.entry(a.status).or_insert(0) += 1;
            }
        }
        let mut shadow_bans = 0i64;
        for val in self.mm_get_all(MM_SHADOW, "all")? {
            if let Ok(sb) = serde_json::from_str::<ShadowBan>(&val) {
                if let Some(until) = &sb.banned_until {
                    if let Ok(u) = chrono::DateTime::parse_from_rfc3339(until)
                        && chrono::Utc::now() < u.with_timezone(&chrono::Utc)
                    {
                        shadow_bans += 1;
                    }
                } else {
                    shadow_bans += 1;
                }
            }
        }
        Ok(serde_json::json!({
            "queue": queue,
            "reports": reports,
            "appeals": appeals,
            "active_shadow_bans": shadow_bans,
        }))
    }

    fn count_signups_on(&self, day: &str) -> Result<i64> {
        let txn = self.inner.begin_read()?;
        let t = txn.open_table(T_USER)?;
        let mut count = 0i64;
        for entry in t.iter()? {
            let (_, v) = entry?;
            if let Ok(u) = serde_json::from_str::<serde_json::Value>(v.value())
                && u["created_at"]
                    .as_str()
                    .map(|s| s.starts_with(day))
                    .unwrap_or(false)
            {
                count += 1;
            }
        }
        Ok(count)
    }

    fn count_users(&self) -> Result<i64> {
        let txn = self.inner.begin_read()?;
        let t = txn.open_table(T_USER)?;
        Ok(t.len()? as i64)
    }

    fn active_users_last(&self, days: i64) -> Result<i64> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
        let mut users: HashSet<i64> = HashSet::new();
        for (key, val) in self.mm_get_all_entries(MM_USER_ACTIVITY)? {
            let _ = key;
            if let Ok(ev) = serde_json::from_str::<serde_json::Value>(&val)
                && let Some(ts) = ev["ts"]
                    .as_str()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                && ts.with_timezone(&chrono::Utc) >= cutoff
            {
                users.insert(ev["user_id"].as_i64().unwrap_or(-1));
            }
        }
        Ok(users.len() as i64)
    }

    fn sum_transactions_on(&self, day: &str) -> Result<i64> {
        let mut total = 0i64;
        for (_, val) in self.mm_get_all_entries(MM_TRANSACTION)? {
            if let Ok(t) = serde_json::from_str::<Transaction>(&val)
                && t.created_at.starts_with(day)
            {
                total += t.amount.abs();
            }
        }
        Ok(total)
    }

    pub fn sum_transactions_range(&self, from: &str, to: &str) -> Result<i64> {
        let mut total = 0i64;
        for (_, val) in self.mm_get_all_entries(MM_TRANSACTION)? {
            if let Ok(t) = serde_json::from_str::<Transaction>(&val)
                && t.created_at.len() >= 10
            {
                let day = &t.created_at[..10];
                if day >= from && day <= to {
                    total += t.amount.abs();
                }
            }
        }
        Ok(total)
    }

    fn sum_gifts_on(&self, day: &str) -> Result<i64> {
        let mut total = 0i64;
        for (key, val) in self.mm_get_all_entries(MM_GIFT)? {
            if key.parse::<i64>().is_err() {
                continue; // skip "from_" copies to avoid double counting
            }
            if let Ok(g) = serde_json::from_str::<GiftRecord>(&val)
                && g.created_at.starts_with(day)
            {
                total += g.price;
            }
        }
        Ok(total)
    }

    pub fn sum_gifts_range(&self, from: &str, to: &str) -> Result<i64> {
        let mut total = 0i64;
        for (key, val) in self.mm_get_all_entries(MM_GIFT)? {
            if key.parse::<i64>().is_err() {
                continue;
            }
            if let Ok(g) = serde_json::from_str::<GiftRecord>(&val)
                && g.created_at.len() >= 10
            {
                let day = &g.created_at[..10];
                if day >= from && day <= to {
                    total += g.price;
                }
            }
        }
        Ok(total)
    }

    fn count_calls_on(&self, day: &str) -> Result<i64> {
        let mut count = 0i64;
        for (_, val) in self.mm_get_all_entries(MM_CALL_BILLING)? {
            if let Ok(cb) = serde_json::from_str::<CallBilling>(&val)
                && cb.started_at.starts_with(day)
            {
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn sum_call_fees_range(&self, from: &str, to: &str) -> Result<i64> {
        let mut total = 0i64;
        for (_, val) in self.mm_get_all_entries(MM_CALL_BILLING)? {
            if let Ok(cb) = serde_json::from_str::<CallBilling>(&val)
                && cb.status == "completed"
                && cb.started_at.len() >= 10
            {
                let day = &cb.started_at[..10];
                if day >= from && day <= to {
                    total += cb.platform_fee;
                }
            }
        }
        Ok(total)
    }

    fn count_messages_on(&self, day: &str) -> Result<i64> {
        let mut count = 0i64;
        for (_, val) in self.mm_get_all_entries(MM_MSG)? {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&val)
                && v["created_at"]
                    .as_str()
                    .map(|s| s.starts_with(day))
                    .unwrap_or(false)
            {
                count += 1;
            }
        }
        Ok(count)
    }

    fn count_reports_with_status(&self, status: &str) -> Result<i64> {
        let mut count = 0i64;
        for val in self.mm_get_all(MM_REPORT, "all")? {
            if let Ok(r) = serde_json::from_str::<Report>(&val)
                && r.status == status
            {
                count += 1;
            }
        }
        Ok(count)
    }
}

// Helper trait
trait Pipe {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
        Self: Sized;
}

impl<T> Pipe for T {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        f(self)
    }
}

fn report_severity(category: &str) -> f64 {
    match category {
        "scam" | "fraud" => 0.9,
        "nsfw" | "violence" => 0.8,
        "harassment" => 0.6,
        "spam" => 0.4,
        _ => 0.3,
    }
}

fn trust_level(score: f64) -> &'static str {
    if score >= 80.0 {
        "excellent"
    } else if score >= 60.0 {
        "good"
    } else if score >= 40.0 {
        "watch"
    } else {
        "restricted"
    }
}

// Default impl for User
