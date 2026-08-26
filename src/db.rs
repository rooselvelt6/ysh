use anyhow::Result;
use rusqlite::{params, Connection};
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: String,
    pub failed_login_attempts: i32,
    pub locked_until: Option<String>,
    pub totp_enabled: bool,
    pub kyc_level: i32,
    pub do_not_sell: bool,
}

#[derive(Debug, Clone)]
pub struct RecoveryCode {
    pub id: i64,
    pub code_hash: String,
    pub used: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConsentRecord {
    pub id: i64,
    pub consent_type: String,
    pub granted: bool,
    pub created_at: String,
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'user',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                failed_login_attempts INTEGER NOT NULL DEFAULT 0,
                locked_until TEXT,
                totp_secret TEXT,
                totp_enabled INTEGER NOT NULL DEFAULT 0,
                kyc_level INTEGER NOT NULL DEFAULT 0,
                do_not_sell INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS recovery_codes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                code_hash TEXT NOT NULL,
                used INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS consent_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                consent_type TEXT NOT NULL,
                granted INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS devices (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                fingerprint TEXT NOT NULL,
                user_agent TEXT,
                last_seen TEXT NOT NULL DEFAULT (datetime('now')),
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS profiles (
                user_id INTEGER PRIMARY KEY,
                display_name TEXT NOT NULL DEFAULT '',
                bio TEXT NOT NULL DEFAULT '',
                avatar_url TEXT NOT NULL DEFAULT '',
                country TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS agencies (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                owner_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (owner_id) REFERENCES users(id)
            );

            CREATE TABLE IF NOT EXISTS agency_members (
                agency_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,
                role TEXT NOT NULL DEFAULT 'host',
                joined_at TEXT NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (agency_id, user_id),
                FOREIGN KEY (agency_id) REFERENCES agencies(id) ON DELETE CASCADE,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS hosts (
                user_id INTEGER PRIMARY KEY,
                languages TEXT NOT NULL DEFAULT 'en',
                hourly_rate INTEGER NOT NULL DEFAULT 0,
                available INTEGER NOT NULL DEFAULT 0,
                total_calls INTEGER NOT NULL DEFAULT 0,
                rating REAL NOT NULL DEFAULT 0.0,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS wallets (
                user_id INTEGER PRIMARY KEY,
                balance INTEGER NOT NULL DEFAULT 0,
                frozen INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS transactions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                tx_type TEXT NOT NULL,
                amount INTEGER NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                target_user_id INTEGER,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (user_id) REFERENCES users(id)
            );

            CREATE TABLE IF NOT EXISTS gift_catalog (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                price INTEGER NOT NULL,
                rarity TEXT NOT NULL DEFAULT 'common'
            );

            CREATE TABLE IF NOT EXISTS gifts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                from_user_id INTEGER NOT NULL,
                to_user_id INTEGER NOT NULL,
                gift_id INTEGER NOT NULL,
                price INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (from_user_id) REFERENCES users(id),
                FOREIGN KEY (to_user_id) REFERENCES users(id),
                FOREIGN KEY (gift_id) REFERENCES gift_catalog(id)
            );

            CREATE TABLE IF NOT EXISTS moments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                content TEXT NOT NULL DEFAULT '',
                media_url TEXT NOT NULL DEFAULT '',
                media_type TEXT NOT NULL DEFAULT 'text',
                likes_count INTEGER NOT NULL DEFAULT 0,
                comments_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS moment_likes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                moment_id INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(user_id, moment_id),
                FOREIGN KEY (user_id) REFERENCES users(id),
                FOREIGN KEY (moment_id) REFERENCES moments(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS moment_comments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                moment_id INTEGER NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (user_id) REFERENCES users(id),
                FOREIGN KEY (moment_id) REFERENCES moments(id) ON DELETE CASCADE
            );

            INSERT OR IGNORE INTO gift_catalog (name, description, price, rarity) VALUES
                ('Rose', 'A single red rose', 10, 'common'),
                ('Heart', 'A golden heart', 50, 'common'),
                ('Diamond Ring', 'A sparkling diamond ring', 200, 'rare'),
                ('Sports Car', 'A virtual sports car', 500, 'epic'),
                ('Yacht', 'A luxury yacht', 1000, 'legendary'),
                ('Private Island', 'Your own virtual island', 5000, 'legendary');

            CREATE TABLE IF NOT EXISTS notifications (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                ntype TEXT NOT NULL,
                title TEXT NOT NULL,
                body TEXT NOT NULL,
                data TEXT NOT NULL DEFAULT '{}',
                read INTEGER NOT NULL DEFAULT 0,
                channel TEXT NOT NULL DEFAULT 'in_app',
                status TEXT NOT NULL DEFAULT 'pending',
                retries INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                sent_at TEXT,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS notification_preferences (
                user_id INTEGER PRIMARY KEY,
                email_enabled INTEGER NOT NULL DEFAULT 1,
                push_enabled INTEGER NOT NULL DEFAULT 1,
                in_app_enabled INTEGER NOT NULL DEFAULT 1,
                email_gifts INTEGER NOT NULL DEFAULT 1,
                email_calls INTEGER NOT NULL DEFAULT 1,
                email_moments INTEGER NOT NULL DEFAULT 1,
                email_marketing INTEGER NOT NULL DEFAULT 0,
                push_gifts INTEGER NOT NULL DEFAULT 1,
                push_calls INTEGER NOT NULL DEFAULT 1,
                push_moments INTEGER NOT NULL DEFAULT 1,
                quiet_hours_start TEXT,
                quiet_hours_end TEXT,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS push_tokens (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                token TEXT NOT NULL,
                platform TEXT NOT NULL DEFAULT 'web',
                active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS chat_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_type TEXT NOT NULL DEFAULT 'direct',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS chat_participants (
                session_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,
                joined_at TEXT NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (session_id, user_id),
                FOREIGN KEY (session_id) REFERENCES chat_sessions(id) ON DELETE CASCADE,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                sender_id INTEGER NOT NULL,
                content TEXT NOT NULL,
                msg_type TEXT NOT NULL DEFAULT 'text',
                encrypted INTEGER NOT NULL DEFAULT 0,
                read INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (session_id) REFERENCES chat_sessions(id) ON DELETE CASCADE,
                FOREIGN KEY (sender_id) REFERENCES users(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS matching_queue (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                mode TEXT NOT NULL DEFAULT 'random',
                preferences TEXT NOT NULL DEFAULT '{}',
                status TEXT NOT NULL DEFAULT 'waiting',
                queued_at TEXT NOT NULL DEFAULT (datetime('now')),
                matched_at TEXT,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id, created_at);
            CREATE INDEX IF NOT EXISTS idx_messages_read ON messages(session_id, sender_id, read);
            CREATE INDEX IF NOT EXISTS idx_notifications_user ON notifications(user_id, read);
            CREATE INDEX IF NOT EXISTS idx_transactions_user ON transactions(user_id, created_at);
            CREATE INDEX IF NOT EXISTS idx_moments_feed ON moments(user_id, created_at);
            CREATE INDEX IF NOT EXISTS idx_matching_queue ON matching_queue(status, mode);
            CREATE INDEX IF NOT EXISTS idx_chat_participants ON chat_participants(user_id);

            -- ═══════════════════════════════════════════
            -- PHASE 9: ECONOMY + CRYPTO PAYMENTS
            -- ═══════════════════════════════════════════

            CREATE TABLE IF NOT EXISTS staking (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                amount INTEGER NOT NULL,
                apy_rate REAL NOT NULL DEFAULT 0.05,
                status TEXT NOT NULL DEFAULT 'active',
                staked_at TEXT NOT NULL DEFAULT (datetime('now')),
                unlocks_at TEXT NOT NULL,
                rewards_earned INTEGER NOT NULL DEFAULT 0,
                last_reward_calc TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS referrals (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                referrer_id INTEGER NOT NULL,
                referred_id INTEGER NOT NULL UNIQUE,
                code TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                referred_at TEXT NOT NULL DEFAULT (datetime('now')),
                first_purchase_at TEXT,
                total_earned INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (referrer_id) REFERENCES users(id),
                FOREIGN KEY (referred_id) REFERENCES users(id)
            );

            CREATE TABLE IF NOT EXISTS call_billing (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                caller_id INTEGER NOT NULL,
                host_id INTEGER NOT NULL,
                call_type TEXT NOT NULL DEFAULT 'video',
                started_at TEXT NOT NULL,
                ended_at TEXT,
                duration_secs INTEGER NOT NULL DEFAULT 0,
                cost_per_min INTEGER NOT NULL DEFAULT 0,
                total_cost INTEGER NOT NULL DEFAULT 0,
                host_earnings INTEGER NOT NULL DEFAULT 0,
                platform_fee INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'active',
                FOREIGN KEY (caller_id) REFERENCES users(id),
                FOREIGN KEY (host_id) REFERENCES users(id)
            );

            CREATE TABLE IF NOT EXISTS commissions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                source_user_id INTEGER NOT NULL,
                source_tx_id INTEGER,
                tier INTEGER NOT NULL DEFAULT 1,
                percentage REAL NOT NULL DEFAULT 0.10,
                amount INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                paid_at TEXT,
                FOREIGN KEY (user_id) REFERENCES users(id),
                FOREIGN KEY (source_user_id) REFERENCES users(id)
            );

            CREATE TABLE IF NOT EXISTS payouts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                amount INTEGER NOT NULL,
                currency TEXT NOT NULL DEFAULT 'USDT',
                wallet_address TEXT NOT NULL,
                network TEXT NOT NULL DEFAULT 'TRC20',
                status TEXT NOT NULL DEFAULT 'pending',
                tx_hash TEXT,
                requested_at TEXT NOT NULL DEFAULT (datetime('now')),
                processed_at TEXT,
                admin_id INTEGER,
                notes TEXT NOT NULL DEFAULT '',
                FOREIGN KEY (user_id) REFERENCES users(id)
            );

            CREATE TABLE IF NOT EXISTS fraud_alerts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER,
                alert_type TEXT NOT NULL,
                severity TEXT NOT NULL DEFAULT 'medium',
                description TEXT NOT NULL DEFAULT '',
                evidence TEXT NOT NULL DEFAULT '{}',
                status TEXT NOT NULL DEFAULT 'open',
                ip_address TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                resolved_at TEXT,
                resolved_by INTEGER
            );

            CREATE TABLE IF NOT EXISTS receipts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                receipt_type TEXT NOT NULL,
                reference_id INTEGER NOT NULL,
                amount INTEGER NOT NULL,
                currency TEXT NOT NULL DEFAULT 'YSH',
                description TEXT NOT NULL DEFAULT '',
                metadata TEXT NOT NULL DEFAULT '{}',
                receipt_hash TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (user_id) REFERENCES users(id)
            );

            CREATE TABLE IF NOT EXISTS spending_limits (
                user_id INTEGER PRIMARY KEY,
                daily_limit INTEGER NOT NULL DEFAULT 100000,
                monthly_limit INTEGER NOT NULL DEFAULT 1000000,
                daily_spent INTEGER NOT NULL DEFAULT 0,
                monthly_spent INTEGER NOT NULL DEFAULT 0,
                last_reset_date TEXT NOT NULL DEFAULT (date('now')),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS nft_gifts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                gift_id INTEGER NOT NULL,
                gift_record_id INTEGER NOT NULL,
                token_id TEXT NOT NULL,
                unlocked INTEGER NOT NULL DEFAULT 0,
                minted_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (user_id) REFERENCES users(id),
                FOREIGN KEY (gift_id) REFERENCES gift_catalog(id),
                FOREIGN KEY (gift_record_id) REFERENCES gifts(id)
            );

            CREATE INDEX IF NOT EXISTS idx_staking_user ON staking(user_id, status);
            CREATE INDEX IF NOT EXISTS idx_referrals_code ON referrals(code);
            CREATE INDEX IF NOT EXISTS idx_call_billing_host ON call_billing(host_id);
            CREATE INDEX IF NOT EXISTS idx_commissions_user ON commissions(user_id, status);
            CREATE INDEX IF NOT EXISTS idx_payouts_user ON payouts(user_id, status);
            CREATE INDEX IF NOT EXISTS idx_fraud_alerts ON fraud_alerts(status, severity);
            CREATE INDEX IF NOT EXISTS idx_receipts_user ON receipts(user_id, created_at);",
        )?;
        Ok(())
    }

    pub fn create_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO users (username, email, password_hash) VALUES (?1, ?2, ?3)",
            params![username, email, password_hash],
        )?;
        let id = conn.last_insert_rowid();
        let user = User {
            id,
            username: username.to_string(),
            email: email.to_string(),
            password_hash: password_hash.to_string(),
            role: "user".to_string(),
            created_at: chrono::Utc::now()
                .format("%Y-%m-%dT%H:%M:%SZ")
                .to_string(),
            failed_login_attempts: 0,
            locked_until: None,
            totp_enabled: false,
            kyc_level: 0,
            do_not_sell: false,
        };
        Ok(user)
    }

    pub fn find_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, username, email, password_hash, role, created_at,
                    failed_login_attempts, locked_until, totp_enabled,
                    kyc_level, do_not_sell
             FROM users WHERE username = ?1",
        )?;
        let mut rows = stmt.query_map(params![username], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                email: row.get(2)?,
                password_hash: row.get(3)?,
                role: row.get(4)?,
                created_at: row.get(5)?,
                failed_login_attempts: row.get(6)?,
                locked_until: row.get(7)?,
                totp_enabled: row.get::<_, i32>(8)? != 0,
                kyc_level: row.get(9)?,
                do_not_sell: row.get::<_, i32>(10)? != 0,
            })
        })?;
        match rows.next() {
            Some(user) => Ok(Some(user?)),
            None => Ok(None),
        }
    }

    pub fn find_user_by_id(&self, user_id: i64) -> Result<Option<User>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, username, email, password_hash, role, created_at,
                    failed_login_attempts, locked_until, totp_enabled,
                    kyc_level, do_not_sell
             FROM users WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![user_id], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                email: row.get(2)?,
                password_hash: row.get(3)?,
                role: row.get(4)?,
                created_at: row.get(5)?,
                failed_login_attempts: row.get(6)?,
                locked_until: row.get(7)?,
                totp_enabled: row.get::<_, i32>(8)? != 0,
                kyc_level: row.get(9)?,
                do_not_sell: row.get::<_, i32>(10)? != 0,
            })
        })?;
        match rows.next() {
            Some(user) => Ok(Some(user?)),
            None => Ok(None),
        }
    }

    pub fn user_exists(&self, username: &str, email: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT COUNT(*) FROM users WHERE username = ?1 OR email = ?2")?;
        let count: i64 = stmt.query_row(params![username, email], |row| row.get(0))?;
        Ok(count > 0)
    }

    pub fn set_failed_attempts(&self, user_id: i64, attempts: i32) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE users SET failed_login_attempts = ?1 WHERE id = ?2",
            params![attempts, user_id],
        )?;
        Ok(())
    }

    pub fn reset_failed_attempts(&self, user_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE users SET failed_login_attempts = 0, locked_until = NULL WHERE id = ?1",
            params![user_id],
        )?;
        Ok(())
    }

    pub fn lock_account(&self, user_id: i64, until: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE users SET locked_until = ?1 WHERE id = ?2",
            params![until, user_id],
        )?;
        Ok(())
    }

    pub fn set_totp_secret(&self, user_id: i64, secret: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE users SET totp_secret = ?1 WHERE id = ?2",
            params![secret, user_id],
        )?;
        Ok(())
    }

    pub fn get_totp_secret(&self, user_id: i64) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT totp_secret FROM users WHERE id = ?1")?;
        let mut rows = stmt.query_map(params![user_id], |row| row.get::<_, Option<String>>(0))?;
        match rows.next() {
            Some(r) => Ok(r?),
            None => Ok(None),
        }
    }

    pub fn enable_totp(&self, user_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE users SET totp_enabled = 1 WHERE id = ?1",
            params![user_id],
        )?;
        Ok(())
    }

    pub fn disable_totp(&self, user_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE users SET totp_enabled = 0, totp_secret = NULL WHERE id = ?1",
            params![user_id],
        )?;
        Ok(())
    }

    pub fn store_recovery_codes(&self, user_id: i64, codes: &[(String, bool)]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        for (code_hash, used) in codes {
            conn.execute(
                "INSERT INTO recovery_codes (user_id, code_hash, used) VALUES (?1, ?2, ?3)",
                params![user_id, code_hash, *used as i32],
            )?;
        }
        Ok(())
    }

    pub fn get_recovery_codes(&self, user_id: i64) -> Result<Vec<RecoveryCode>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, code_hash, used FROM recovery_codes WHERE user_id = ?1 ORDER BY id",
        )?;
        let rows = stmt.query_map(params![user_id], |row| {
            Ok(RecoveryCode {
                id: row.get(0)?,
                code_hash: row.get(1)?,
                used: row.get::<_, i32>(2)? != 0,
            })
        })?;
        let mut codes = Vec::new();
        for row in rows {
            codes.push(row?);
        }
        Ok(codes)
    }

    pub fn mark_recovery_code_used(&self, code_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE recovery_codes SET used = 1 WHERE id = ?1",
            params![code_id],
        )?;
        Ok(())
    }

    pub fn delete_recovery_codes(&self, user_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM recovery_codes WHERE user_id = ?1",
            params![user_id],
        )?;
        Ok(())
    }

    pub fn get_user_data(&self, user_id: i64) -> Result<serde_json::Value> {
        let conn = self.conn.lock().unwrap();

        let user = {
            let mut stmt = conn.prepare(
                "SELECT id, username, email, role, created_at, kyc_level, do_not_sell FROM users WHERE id = ?1",
            )?;
            let mut rows = stmt.query_map(params![user_id], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "username": row.get::<_, String>(1)?,
                    "email": row.get::<_, String>(2)?,
                    "role": row.get::<_, String>(3)?,
                    "created_at": row.get::<_, String>(4)?,
                    "kyc_level": row.get::<_, i32>(5)?,
                    "do_not_sell": row.get::<_, i32>(6)? != 0,
                }))
            })?;
            rows.next().transpose()?.unwrap_or_default()
        };

        let consent_records = {
            let mut stmt = conn.prepare(
                "SELECT consent_type, granted, created_at FROM consent_records WHERE user_id = ?1 ORDER BY id",
            )?;
            let rows = stmt.query_map(params![user_id], |row| {
                Ok(serde_json::json!({
                    "consent_type": row.get::<_, String>(0)?,
                    "granted": row.get::<_, i32>(1)? != 0,
                    "created_at": row.get::<_, String>(2)?,
                }))
            })?;
            rows.filter_map(|r| r.ok()).collect::<Vec<_>>()
        };

        let devices = {
            let mut stmt = conn.prepare(
                "SELECT fingerprint, user_agent, last_seen, created_at FROM devices WHERE user_id = ?1 ORDER BY id",
            )?;
            let rows = stmt.query_map(params![user_id], |row| {
                Ok(serde_json::json!({
                    "fingerprint": row.get::<_, String>(0)?,
                    "user_agent": row.get::<_, Option<String>>(1)?,
                    "last_seen": row.get::<_, String>(2)?,
                    "created_at": row.get::<_, String>(3)?,
                }))
            })?;
            rows.filter_map(|r| r.ok()).collect::<Vec<_>>()
        };

        Ok(serde_json::json!({
            "user": user,
            "consent_records": consent_records,
            "devices": devices,
        }))
    }

    pub fn delete_user_data(&self, user_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM recovery_codes WHERE user_id = ?1", params![user_id])?;
        conn.execute("DELETE FROM consent_records WHERE user_id = ?1", params![user_id])?;
        conn.execute("DELETE FROM devices WHERE user_id = ?1", params![user_id])?;
        conn.execute("DELETE FROM users WHERE id = ?1", params![user_id])?;
        Ok(())
    }

    pub fn record_consent(
        &self,
        user_id: i64,
        consent_type: &str,
        granted: bool,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO consent_records (user_id, consent_type, granted) VALUES (?1, ?2, ?3)",
            params![user_id, consent_type, granted as i32],
        )?;
        Ok(())
    }

    pub fn get_consent_history(&self, user_id: i64) -> Result<Vec<ConsentRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, consent_type, granted, created_at FROM consent_records WHERE user_id = ?1 ORDER BY id",
        )?;
        let rows = stmt.query_map(params![user_id], |row| {
            Ok(ConsentRecord {
                id: row.get(0)?,
                consent_type: row.get(1)?,
                granted: row.get::<_, i32>(2)? != 0,
                created_at: row.get(3)?,
            })
        })?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    pub fn set_do_not_sell(&self, user_id: i64, value: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE users SET do_not_sell = ?1 WHERE id = ?2",
            params![value as i32, user_id],
        )?;
        Ok(())
    }

    pub fn set_kyc_level(&self, user_id: i64, level: i32) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE users SET kyc_level = ?1 WHERE id = ?2",
            params![level, user_id],
        )?;
        Ok(())
    }

    pub fn store_device(
        &self,
        user_id: i64,
        fingerprint: &str,
        user_agent: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let exists: bool = conn.query_row(
            "SELECT COUNT(*) FROM devices WHERE user_id = ?1 AND fingerprint = ?2",
            params![user_id, fingerprint],
            |row| row.get::<_, i64>(0),
        )? > 0;

        if exists {
            conn.execute(
                "UPDATE devices SET last_seen = datetime('now') WHERE user_id = ?1 AND fingerprint = ?2",
                params![user_id, fingerprint],
            )?;
        } else {
            conn.execute(
                "INSERT INTO devices (user_id, fingerprint, user_agent) VALUES (?1, ?2, ?3)",
                params![user_id, fingerprint, user_agent],
            )?;
        }
        Ok(())
    }

    pub fn health_check(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("SELECT 1")?;
        Ok(())
    }

    pub fn user_count(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
        Ok(count)
    }

    pub fn session_count(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM devices",
            [],
            |row| row.get(0),
        )?;
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
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO profiles (user_id, display_name, bio, avatar_url, country)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(user_id) DO UPDATE SET
                display_name=excluded.display_name, bio=excluded.bio,
                avatar_url=excluded.avatar_url, country=excluded.country",
            params![user_id, display_name, bio, avatar_url, country],
        )?;
        Ok(())
    }

    pub fn get_profile(&self, user_id: i64) -> Result<Option<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT display_name, bio, avatar_url, country, created_at
             FROM profiles WHERE user_id = ?1",
        )?;
        let mut rows = stmt.query_map(params![user_id], |row| {
            Ok(serde_json::json!({
                "display_name": row.get::<_, String>(0)?,
                "bio": row.get::<_, String>(1)?,
                "avatar_url": row.get::<_, String>(2)?,
                "country": row.get::<_, String>(3)?,
                "created_at": row.get::<_, String>(4)?,
            }))
        })?;
        match rows.next() {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }

    pub fn search_users(&self, query: &str, limit: i64) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, username, created_at FROM users
             WHERE username LIKE ?1 LIMIT ?2",
        )?;
        let pattern = format!("%{}%", query);
        let rows = stmt.query_map(params![pattern, limit], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "username": row.get::<_, String>(1)?,
                "created_at": row.get::<_, String>(2)?,
            }))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn ban_user(&self, user_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE users SET locked_until = '2099-12-31T23:59:59Z' WHERE id = ?1",
            params![user_id],
        )?;
        Ok(())
    }

    pub fn unban_user(&self, user_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE users SET locked_until = NULL WHERE id = ?1",
            params![user_id],
        )?;
        Ok(())
    }

    pub fn list_users(&self, offset: i64, limit: i64) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, username, email, role, created_at, kyc_level
             FROM users ORDER BY id LIMIT ?1 OFFSET ?2",
        )?;
        let rows = stmt.query_map(params![limit, offset], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "username": row.get::<_, String>(1)?,
                "email": row.get::<_, String>(2)?,
                "role": row.get::<_, String>(3)?,
                "created_at": row.get::<_, String>(4)?,
                "kyc_level": row.get::<_, i32>(5)?,
            }))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn create_agency(
        &self,
        owner_id: i64,
        name: &str,
        description: &str,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO agencies (owner_id, name, description) VALUES (?1, ?2, ?3)",
            params![owner_id, name, description],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_agency(&self, agency_id: i64) -> Result<Option<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT a.id, a.name, a.description, a.owner_id, a.created_at,
                    u.username as owner_name,
                    (SELECT COUNT(*) FROM agency_members WHERE agency_id = a.id) as member_count
             FROM agencies a
             JOIN users u ON u.id = a.owner_id
             WHERE a.id = ?1",
        )?;
        let mut rows = stmt.query_map(params![agency_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "name": row.get::<_, String>(1)?,
                "description": row.get::<_, String>(2)?,
                "owner_id": row.get::<_, i64>(3)?,
                "created_at": row.get::<_, String>(4)?,
                "owner_name": row.get::<_, String>(5)?,
                "member_count": row.get::<_, i64>(6)?,
            }))
        })?;
        match rows.next() {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }

    pub fn list_agencies(&self) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT a.id, a.name, a.description, u.username as owner_name,
                    (SELECT COUNT(*) FROM agency_members WHERE agency_id = a.id) as member_count
             FROM agencies a JOIN users u ON u.id = a.owner_id ORDER BY a.id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "name": row.get::<_, String>(1)?,
                "description": row.get::<_, String>(2)?,
                "owner_name": row.get::<_, String>(3)?,
                "member_count": row.get::<_, i64>(4)?,
            }))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn add_agency_member(
        &self,
        agency_id: i64,
        user_id: i64,
        role: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO agency_members (agency_id, user_id, role) VALUES (?1, ?2, ?3)",
            params![agency_id, user_id, role],
        )?;
        Ok(())
    }

    pub fn get_agency_members(&self, agency_id: i64) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT am.user_id, u.username, am.role, am.joined_at
             FROM agency_members am JOIN users u ON u.id = am.user_id
             WHERE am.agency_id = ?1 ORDER BY am.joined_at",
        )?;
        let rows = stmt.query_map(params![agency_id], |row| {
            Ok(serde_json::json!({
                "user_id": row.get::<_, i64>(0)?,
                "username": row.get::<_, String>(1)?,
                "role": row.get::<_, String>(2)?,
                "joined_at": row.get::<_, String>(3)?,
            }))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn create_host_profile(
        &self,
        user_id: i64,
        languages: &str,
        hourly_rate: i64,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO hosts (user_id, languages, hourly_rate)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(user_id) DO UPDATE SET
                languages=excluded.languages, hourly_rate=excluded.hourly_rate",
            params![user_id, languages, hourly_rate],
        )?;
        Ok(())
    }

    pub fn get_host_profile(&self, user_id: i64) -> Result<Option<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT h.languages, h.hourly_rate, h.available, h.total_calls, h.rating,
                    u.username
             FROM hosts h JOIN users u ON u.id = h.user_id
             WHERE h.user_id = ?1",
        )?;
        let mut rows = stmt.query_map(params![user_id], |row| {
            Ok(serde_json::json!({
                "languages": row.get::<_, String>(0)?,
                "hourly_rate": row.get::<_, i64>(1)?,
                "available": row.get::<_, i32>(2)? != 0,
                "total_calls": row.get::<_, i64>(3)?,
                "rating": row.get::<_, f64>(4)?,
                "username": row.get::<_, String>(5)?,
            }))
        })?;
        match rows.next() {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }

    pub fn set_host_availability(&self, user_id: i64, available: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE hosts SET available = ?1 WHERE user_id = ?2",
            params![available as i32, user_id],
        )?;
        Ok(())
    }

    pub fn list_hosts(&self, available_only: bool) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let query = if available_only {
            "SELECT h.user_id, u.username, h.languages, h.hourly_rate, h.rating
             FROM hosts h JOIN users u ON u.id = h.user_id
             WHERE h.available = 1 ORDER BY h.rating DESC"
        } else {
            "SELECT h.user_id, u.username, h.languages, h.hourly_rate, h.rating
             FROM hosts h JOIN users u ON u.id = h.user_id
             ORDER BY h.rating DESC"
        };
        let mut stmt = conn.prepare(query)?;
        let rows = stmt.query_map([], |row| {
            Ok(serde_json::json!({
                "user_id": row.get::<_, i64>(0)?,
                "username": row.get::<_, String>(1)?,
                "languages": row.get::<_, String>(2)?,
                "hourly_rate": row.get::<_, i64>(3)?,
                "rating": row.get::<_, f64>(4)?,
            }))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_balance(&self, user_id: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let balance: i64 = conn.query_row(
            "SELECT COALESCE(balance, 0) FROM wallets WHERE user_id = ?1",
            params![user_id],
            |row| row.get(0),
        ).unwrap_or(0);
        Ok(balance)
    }

    pub fn ensure_wallet(&self, user_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO wallets (user_id, balance) VALUES (?1, 0)",
            params![user_id],
        )?;
        Ok(())
    }

    pub fn deposit(&self, user_id: i64, amount: i64, description: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE wallets SET balance = balance + ?1 WHERE user_id = ?2",
            params![amount, user_id],
        )?;
        conn.execute(
            "INSERT INTO transactions (user_id, tx_type, amount, description)
             VALUES (?1, 'deposit', ?2, ?3)",
            params![user_id, amount, description],
        )?;
        let balance: i64 = conn.query_row(
            "SELECT balance FROM wallets WHERE user_id = ?1",
            params![user_id],
            |row| row.get(0),
        )?;
        Ok(balance)
    }

    pub fn withdraw(&self, user_id: i64, amount: i64, description: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let balance: i64 = conn.query_row(
            "SELECT balance FROM wallets WHERE user_id = ?1",
            params![user_id],
            |row| row.get(0),
        )?;
        if balance < amount {
            anyhow::bail!("Insufficient funds: {} < {}", balance, amount);
        }
        conn.execute(
            "UPDATE wallets SET balance = balance - ?1 WHERE user_id = ?2",
            params![amount, user_id],
        )?;
        conn.execute(
            "INSERT INTO transactions (user_id, tx_type, amount, description)
             VALUES (?1, 'withdraw', ?2, ?3)",
            params![user_id, amount, description],
        )?;
        Ok(balance - amount)
    }

    pub fn transfer(
        &self,
        from_user: i64,
        to_user: i64,
        amount: i64,
        description: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let balance: i64 = conn.query_row(
            "SELECT balance FROM wallets WHERE user_id = ?1",
            params![from_user],
            |row| row.get(0),
        )?;
        if balance < amount {
            anyhow::bail!("Insufficient funds: {} < {}", balance, amount);
        }
        conn.execute(
            "UPDATE wallets SET balance = balance - ?1 WHERE user_id = ?2",
            params![amount, from_user],
        )?;
        conn.execute(
            "UPDATE wallets SET balance = balance + ?1 WHERE user_id = ?2",
            params![amount, to_user],
        )?;
        conn.execute(
            "INSERT INTO transactions (user_id, tx_type, amount, description, target_user_id)
             VALUES (?1, 'transfer_out', ?2, ?3, ?4)",
            params![from_user, amount, description, to_user],
        )?;
        conn.execute(
            "INSERT INTO transactions (user_id, tx_type, amount, description, target_user_id)
             VALUES (?1, 'transfer_in', ?2, ?3, ?4)",
            params![to_user, amount, description, from_user],
        )?;
        Ok(())
    }

    pub fn get_transactions(
        &self,
        user_id: i64,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, tx_type, amount, description, target_user_id, created_at
             FROM transactions WHERE user_id = ?1 ORDER BY id DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![user_id, limit], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "type": row.get::<_, String>(1)?,
                "amount": row.get::<_, i64>(2)?,
                "description": row.get::<_, String>(3)?,
                "target_user_id": row.get::<_, Option<i64>>(4)?,
                "created_at": row.get::<_, String>(5)?,
            }))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_gift_catalog(&self) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, price, rarity FROM gift_catalog ORDER BY price",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "name": row.get::<_, String>(1)?,
                "description": row.get::<_, String>(2)?,
                "price": row.get::<_, i64>(3)?,
                "rarity": row.get::<_, String>(4)?,
            }))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn send_gift(
        &self,
        from_user: i64,
        to_user: i64,
        gift_id: i64,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let price: i64 = conn.query_row(
            "SELECT price FROM gift_catalog WHERE id = ?1",
            params![gift_id],
            |row| row.get(0),
        )?;
        let balance: i64 = conn.query_row(
            "SELECT balance FROM wallets WHERE user_id = ?1",
            params![from_user],
            |row| row.get(0),
        )?;
        if balance < price {
            anyhow::bail!("Insufficient funds for gift: {} < {}", balance, price);
        }
        conn.execute(
            "UPDATE wallets SET balance = balance - ?1 WHERE user_id = ?2",
            params![price, from_user],
        )?;
        conn.execute(
            "UPDATE wallets SET balance = balance + ?1 WHERE user_id = ?2",
            params![price, to_user],
        )?;
        conn.execute(
            "INSERT INTO gifts (from_user_id, to_user_id, gift_id, price)
             VALUES (?1, ?2, ?3, ?4)",
            params![from_user, to_user, gift_id, price],
        )?;
        let gift_id_inserted = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO transactions (user_id, tx_type, amount, description, target_user_id)
             VALUES (?1, 'gift_out', ?2, ?3, ?4)",
            params![from_user, price, format!("Gift #{}", gift_id), to_user],
        )?;
        conn.execute(
            "INSERT INTO transactions (user_id, tx_type, amount, description, target_user_id)
             VALUES (?1, 'gift_in', ?2, ?3, ?4)",
            params![to_user, price, format!("Gift #{}", gift_id), from_user],
        )?;
        Ok(gift_id_inserted)
    }

    pub fn get_received_gifts(&self, user_id: i64) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT g.id, g.price, gc.name, gc.rarity, u.username as from_user, g.created_at
             FROM gifts g
             JOIN gift_catalog gc ON gc.id = g.gift_id
             JOIN users u ON u.id = g.from_user_id
             WHERE g.to_user_id = ?1 ORDER BY g.id DESC",
        )?;
        let rows = stmt.query_map(params![user_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "price": row.get::<_, i64>(1)?,
                "name": row.get::<_, String>(2)?,
                "rarity": row.get::<_, String>(3)?,
                "from_user": row.get::<_, String>(4)?,
                "created_at": row.get::<_, String>(5)?,
            }))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn create_moment(
        &self,
        user_id: i64,
        content: &str,
        media_url: &str,
        media_type: &str,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO moments (user_id, content, media_url, media_type)
             VALUES (?1, ?2, ?3, ?4)",
            params![user_id, content, media_url, media_type],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_moment_feed(
        &self,
        user_id: i64,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT m.id, m.content, m.media_url, m.media_type,
                    u.username, m.likes_count, m.comments_count, m.created_at,
                    CASE WHEN ml.id IS NOT NULL THEN 1 ELSE 0 END as liked
             FROM moments m
             JOIN users u ON u.id = m.user_id
             LEFT JOIN moment_likes ml ON ml.moment_id = m.id AND ml.user_id = ?1
             ORDER BY m.id DESC LIMIT ?2 OFFSET ?3",
        )?;
        let rows = stmt.query_map(params![user_id, limit, offset], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "content": row.get::<_, String>(1)?,
                "media_url": row.get::<_, String>(2)?,
                "media_type": row.get::<_, String>(3)?,
                "username": row.get::<_, String>(4)?,
                "likes": row.get::<_, i64>(5)?,
                "comments": row.get::<_, i64>(6)?,
                "created_at": row.get::<_, String>(7)?,
                "liked": row.get::<_, i32>(8)? != 0,
            }))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn like_moment(&self, user_id: i64, moment_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let exists: bool = conn.query_row(
            "SELECT COUNT(*) FROM moment_likes WHERE user_id = ?1 AND moment_id = ?2",
            params![user_id, moment_id],
            |row| row.get::<_, i64>(0),
        )? > 0;
        if exists {
            return Ok(());
        }
        conn.execute(
            "INSERT INTO moment_likes (user_id, moment_id) VALUES (?1, ?2)",
            params![user_id, moment_id],
        )?;
        conn.execute(
            "UPDATE moments SET likes_count = likes_count + 1 WHERE id = ?1",
            params![moment_id],
        )?;
        Ok(())
    }

    pub fn unlike_moment(&self, user_id: i64, moment_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let deleted = conn.execute(
            "DELETE FROM moment_likes WHERE user_id = ?1 AND moment_id = ?2",
            params![user_id, moment_id],
        )?;
        if deleted > 0 {
            conn.execute(
                "UPDATE moments SET likes_count = MAX(0, likes_count - 1) WHERE id = ?1",
                params![moment_id],
            )?;
        }
        Ok(())
    }

    pub fn comment_on_moment(
        &self,
        user_id: i64,
        moment_id: i64,
        content: &str,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO moment_comments (user_id, moment_id, content)
             VALUES (?1, ?2, ?3)",
            params![user_id, moment_id, content],
        )?;
        conn.execute(
            "UPDATE moments SET comments_count = comments_count + 1 WHERE id = ?1",
            params![moment_id],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_moment_comments(
        &self,
        moment_id: i64,
    ) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT mc.id, u.username, mc.content, mc.created_at
             FROM moment_comments mc
             JOIN users u ON u.id = mc.user_id
             WHERE mc.moment_id = ?1 ORDER BY mc.id",
        )?;
        let rows = stmt.query_map(params![moment_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "username": row.get::<_, String>(1)?,
                "content": row.get::<_, String>(2)?,
                "created_at": row.get::<_, String>(3)?,
            }))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn delete_moment(&self, user_id: i64, moment_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let deleted = conn.execute(
            "DELETE FROM moments WHERE id = ?1 AND user_id = ?2",
            params![moment_id, user_id],
        )?;
        Ok(deleted > 0)
    }

    pub fn platform_stats(&self) -> Result<serde_json::Value> {
        let conn = self.conn.lock().unwrap();
        let users: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0))?;
        let agencies: i64 = conn.query_row("SELECT COUNT(*) FROM agencies", [], |r| r.get(0))?;
        let hosts: i64 = conn.query_row("SELECT COUNT(*) FROM hosts", [], |r| r.get(0))?;
        let moments: i64 = conn.query_row("SELECT COUNT(*) FROM moments", [], |r| r.get(0))?;
        let gifts: i64 = conn.query_row("SELECT COUNT(*) FROM gifts", [], |r| r.get(0))?;
        let total_volume: i64 = conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions",
            [], |r| r.get(0),
        )?;
        let notifications: i64 = conn.query_row("SELECT COUNT(*) FROM notifications", [], |r| r.get(0))?;
        Ok(serde_json::json!({
            "users": users,
            "agencies": agencies,
            "hosts": hosts,
            "moments": moments,
            "gifts": gifts,
            "total_transaction_volume": total_volume,
            "notifications": notifications,
        }))
    }

    pub fn create_notification(
        &self,
        user_id: i64,
        ntype: &str,
        title: &str,
        body: &str,
        data: &str,
        channel: &str,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO notifications (user_id, ntype, title, body, data, channel)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![user_id, ntype, title, body, data, channel],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_notifications(&self, user_id: i64, limit: i64) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, ntype, title, body, data, read, channel, status, created_at
             FROM notifications WHERE user_id = ?1
             ORDER BY created_at DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![user_id, limit], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "type": row.get::<_, String>(1)?,
                "title": row.get::<_, String>(2)?,
                "body": row.get::<_, String>(3)?,
                "data": row.get::<_, String>(4)?,
                "read": row.get::<_, i32>(5)? != 0,
                "channel": row.get::<_, String>(6)?,
                "status": row.get::<_, String>(7)?,
                "created_at": row.get::<_, String>(8)?,
            }))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_unread_count(&self, user_id: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM notifications WHERE user_id = ?1 AND read = 0",
            params![user_id],
            |r| r.get(0),
        )?;
        Ok(count)
    }

    pub fn mark_notification_read(&self, user_id: i64, notification_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let updated = conn.execute(
            "UPDATE notifications SET read = 1 WHERE id = ?1 AND user_id = ?2",
            params![notification_id, user_id],
        )?;
        Ok(updated > 0)
    }

    pub fn mark_all_read(&self, user_id: i64) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let updated = conn.execute(
            "UPDATE notifications SET read = 1 WHERE user_id = ?1 AND read = 0",
            params![user_id],
        )?;
        Ok(updated as usize)
    }

    #[allow(dead_code)]
    pub fn update_notification_status(
        &self,
        notification_id: i64,
        status: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE notifications SET status = ?1, sent_at = datetime('now') WHERE id = ?2",
            params![status, notification_id],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn increment_notification_retries(&self, notification_id: i64) -> Result<i32> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE notifications SET retries = retries + 1 WHERE id = ?1",
            params![notification_id],
        )?;
        let retries: i32 = conn.query_row(
            "SELECT retries FROM notifications WHERE id = ?1",
            params![notification_id],
            |r| r.get(0),
        )?;
        Ok(retries)
    }

    #[allow(dead_code)]
    pub fn get_pending_notifications(&self, limit: i64) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT n.id, n.user_id, n.ntype, n.title, n.body, n.data, n.channel, n.retries,
                    u.email, u.username
             FROM notifications n
             JOIN users u ON n.user_id = u.id
             WHERE n.status = 'pending' AND n.retries < 3
             ORDER BY n.created_at ASC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "user_id": row.get::<_, i64>(1)?,
                "type": row.get::<_, String>(2)?,
                "title": row.get::<_, String>(3)?,
                "body": row.get::<_, String>(4)?,
                "data": row.get::<_, String>(5)?,
                "channel": row.get::<_, String>(6)?,
                "retries": row.get::<_, i32>(7)?,
                "email": row.get::<_, String>(8)?,
                "username": row.get::<_, String>(9)?,
            }))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_notification_preference(&self, user_id: i64) -> Result<serde_json::Value> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT email_enabled, push_enabled, in_app_enabled,
                    email_gifts, email_calls, email_moments, email_marketing,
                    push_gifts, push_calls, push_moments,
                    quiet_hours_start, quiet_hours_end
             FROM notification_preferences WHERE user_id = ?1",
            params![user_id],
            |row| {
                Ok(serde_json::json!({
                    "email_enabled": row.get::<_, i32>(0)? != 0,
                    "push_enabled": row.get::<_, i32>(1)? != 0,
                    "in_app_enabled": row.get::<_, i32>(2)? != 0,
                    "email_gifts": row.get::<_, i32>(3)? != 0,
                    "email_calls": row.get::<_, i32>(4)? != 0,
                    "email_moments": row.get::<_, i32>(5)? != 0,
                    "email_marketing": row.get::<_, i32>(6)? != 0,
                    "push_gifts": row.get::<_, i32>(7)? != 0,
                    "push_calls": row.get::<_, i32>(8)? != 0,
                    "push_moments": row.get::<_, i32>(9)? != 0,
                    "quiet_hours_start": row.get::<_, Option<String>>(10)?,
                    "quiet_hours_end": row.get::<_, Option<String>>(11)?,
                }))
            },
        );
        match result {
            Ok(prefs) => Ok(prefs),
            Err(_) => {
                conn.execute(
                    "INSERT OR IGNORE INTO notification_preferences (user_id) VALUES (?1)",
                    params![user_id],
                )?;
                Ok(serde_json::json!({
                    "email_enabled": true,
                    "push_enabled": true,
                    "in_app_enabled": true,
                    "email_gifts": true,
                    "email_calls": true,
                    "email_moments": true,
                    "email_marketing": false,
                    "push_gifts": true,
                    "push_calls": true,
                    "push_moments": true,
                    "quiet_hours_start": null,
                    "quiet_hours_end": null,
                }))
            }
        }
    }

    pub fn update_notification_preference(
        &self,
        user_id: i64,
        field: &str,
        value: bool,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO notification_preferences (user_id) VALUES (?1)",
            params![user_id],
        )?;
        let sql = format!(
            "UPDATE notification_preferences SET {} = ?1 WHERE user_id = ?2",
            field
        );
        conn.execute(&sql, params![value as i32, user_id])?;
        Ok(())
    }

    pub fn update_quiet_hours(
        &self,
        user_id: i64,
        start: &str,
        end: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO notification_preferences (user_id) VALUES (?1)",
            params![user_id],
        )?;
        conn.execute(
            "UPDATE notification_preferences SET quiet_hours_start = ?1, quiet_hours_end = ?2
             WHERE user_id = ?3",
            params![start, end, user_id],
        )?;
        Ok(())
    }

    pub fn register_push_token(
        &self,
        user_id: i64,
        token: &str,
        platform: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO push_tokens (user_id, token, platform) VALUES (?1, ?2, ?3)",
            params![user_id, token, platform],
        )?;
        Ok(())
    }

    pub fn get_push_tokens(&self, user_id: i64) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, token, platform, created_at FROM push_tokens
             WHERE user_id = ?1 AND active = 1",
        )?;
        let rows = stmt.query_map(params![user_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "token": row.get::<_, String>(1)?,
                "platform": row.get::<_, String>(2)?,
                "created_at": row.get::<_, String>(3)?,
            }))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn deactivate_push_token(&self, user_id: i64, token: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let updated = conn.execute(
            "UPDATE push_tokens SET active = 0 WHERE user_id = ?1 AND token = ?2",
            params![user_id, token],
        )?;
        Ok(updated > 0)
    }

    pub fn create_chat_session(&self, session_type: &str, user_ids: &[i64]) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO chat_sessions (session_type) VALUES (?1)",
            params![session_type],
        )?;
        let session_id = conn.last_insert_rowid();
        for uid in user_ids {
            conn.execute(
                "INSERT INTO chat_participants (session_id, user_id) VALUES (?1, ?2)",
                params![session_id, uid],
            )?;
        }
        Ok(session_id)
    }

    pub fn find_direct_session(&self, user_a: i64, user_b: i64) -> Result<Option<i64>> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT cp1.session_id
             FROM chat_participants cp1
             JOIN chat_participants cp2 ON cp1.session_id = cp2.session_id
             JOIN chat_sessions cs ON cs.id = cp1.session_id
             WHERE cp1.user_id = ?1 AND cp2.user_id = ?2 AND cs.session_type = 'direct'",
            params![user_a, user_b],
            |row| row.get::<_, i64>(0),
        );
        match result {
            Ok(id) => Ok(Some(id)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_user_sessions(&self, user_id: i64) -> Result<Vec<serde_json::Value>> {
        let sessions = {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT cs.id, cs.session_type, cs.created_at, cs.updated_at
                 FROM chat_sessions cs
                 JOIN chat_participants cp ON cp.session_id = cs.id
                 WHERE cp.user_id = ?1
                 ORDER BY cs.updated_at DESC",
            )?;
            let rows = stmt.query_map(params![user_id], |row| {
                let sid: i64 = row.get(0)?;
                Ok(serde_json::json!({
                    "session_id": sid,
                    "type": row.get::<_, String>(1)?,
                    "created_at": row.get::<_, String>(2)?,
                    "updated_at": row.get::<_, String>(3)?,
                }))
            })?;
            let mut sessions = Vec::new();
            for row in rows {
                sessions.push(row?);
            }
            sessions
        };
        let mut sessions = sessions;
        for s in &mut sessions {
            let sid = s["session_id"].as_i64().unwrap_or(0);
            let participants = self.get_session_participants(sid)?;
            s["participants"] = serde_json::json!(participants);
        }
        Ok(sessions)
    }

    pub fn get_session_participants(&self, session_id: i64) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT cp.user_id, u.username
             FROM chat_participants cp
             JOIN users u ON u.id = cp.user_id
             WHERE cp.session_id = ?1",
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(serde_json::json!({
                "user_id": row.get::<_, i64>(0)?,
                "username": row.get::<_, String>(1)?,
            }))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn send_message(
        &self,
        session_id: i64,
        sender_id: i64,
        content: &str,
        msg_type: &str,
        encrypted: bool,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO messages (session_id, sender_id, content, msg_type, encrypted)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![session_id, sender_id, content, msg_type, encrypted as i32],
        )?;
        conn.execute(
            "UPDATE chat_sessions SET updated_at = datetime('now') WHERE id = ?1",
            params![session_id],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_messages(
        &self,
        session_id: i64,
        limit: i64,
        before_id: Option<i64>,
    ) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        if let Some(before) = before_id {
            let mut stmt = conn.prepare(
                "SELECT m.id, m.sender_id, u.username, m.content, m.msg_type,
                        m.encrypted, m.read, m.created_at
                 FROM messages m
                 JOIN users u ON u.id = m.sender_id
                 WHERE m.session_id = ?1 AND m.id < ?2
                 ORDER BY m.id DESC LIMIT ?3",
            )?;
            let rows = stmt.query_map(params![session_id, before, limit], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "sender_id": row.get::<_, i64>(1)?,
                    "username": row.get::<_, String>(2)?,
                    "content": row.get::<_, String>(3)?,
                    "type": row.get::<_, String>(4)?,
                    "encrypted": row.get::<_, i32>(5)? != 0,
                    "read": row.get::<_, i32>(6)? != 0,
                    "created_at": row.get::<_, String>(7)?,
                }))
            })?;
            Ok(rows.filter_map(|r| r.ok()).collect())
        } else {
            let mut stmt = conn.prepare(
                "SELECT m.id, m.sender_id, u.username, m.content, m.msg_type,
                        m.encrypted, m.read, m.created_at
                 FROM messages m
                 JOIN users u ON u.id = m.sender_id
                 WHERE m.session_id = ?1
                 ORDER BY m.id DESC LIMIT ?2",
            )?;
            let rows = stmt.query_map(params![session_id, limit], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "sender_id": row.get::<_, i64>(1)?,
                    "username": row.get::<_, String>(2)?,
                    "content": row.get::<_, String>(3)?,
                    "type": row.get::<_, String>(4)?,
                    "encrypted": row.get::<_, i32>(5)? != 0,
                    "read": row.get::<_, i32>(6)? != 0,
                    "created_at": row.get::<_, String>(7)?,
                }))
            })?;
            Ok(rows.filter_map(|r| r.ok()).collect())
        }
    }

    pub fn mark_messages_read(&self, session_id: i64, user_id: i64) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let updated = conn.execute(
            "UPDATE messages SET read = 1 WHERE session_id = ?1 AND sender_id != ?2 AND read = 0",
            params![session_id, user_id],
        )?;
        Ok(updated as usize)
    }

    pub fn get_unread_message_count(&self, user_id: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM messages m
             JOIN chat_participants cp ON cp.session_id = m.session_id
             WHERE cp.user_id = ?1 AND m.sender_id != ?1 AND m.read = 0",
            params![user_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn enqueue_match(
        &self,
        user_id: i64,
        mode: &str,
        preferences: &str,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM matching_queue WHERE user_id = ?1 AND status = 'waiting'",
            params![user_id],
        )?;
        conn.execute(
            "INSERT INTO matching_queue (user_id, mode, preferences) VALUES (?1, ?2, ?3)",
            params![user_id, mode, preferences],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn dequeue_match(&self, user_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let updated = conn.execute(
            "DELETE FROM matching_queue WHERE user_id = ?1 AND status = 'waiting'",
            params![user_id],
        )?;
        Ok(updated > 0)
    }

    pub fn find_match(
        &self,
        exclude_user_id: i64,
        mode: &str,
    ) -> Result<Option<i64>> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT user_id FROM matching_queue
             WHERE status = 'waiting' AND mode = ?1 AND user_id != ?2
             ORDER BY queued_at ASC LIMIT 1",
            params![mode, exclude_user_id],
            |row| row.get::<_, i64>(0),
        );
        match result {
            Ok(uid) => Ok(Some(uid)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn find_random_match(&self, exclude_user_id: i64) -> Result<Option<i64>> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT user_id FROM matching_queue
             WHERE status = 'waiting' AND user_id != ?1
             ORDER BY RANDOM() LIMIT 1",
            params![exclude_user_id],
            |row| row.get::<_, i64>(0),
        );
        match result {
            Ok(uid) => Ok(Some(uid)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    #[allow(dead_code)]
    pub fn complete_match(&self, queue_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE matching_queue SET status = 'matched', matched_at = datetime('now') WHERE id = ?1",
            params![queue_id],
        )?;
        Ok(())
    }

    pub fn get_queue_size(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM matching_queue WHERE status = 'waiting'",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    #[allow(dead_code)]
    pub fn get_pending_match_count(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM matching_queue WHERE status = 'waiting'",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    // ═══════════════════════════════════════════
    // PHASE 9: STAKING
    // ═══════════════════════════════════════════

    pub fn stake(&self, user_id: i64, amount: i64, apy_rate: f64, unlock_days: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let balance: i64 = conn.query_row(
            "SELECT COALESCE(balance, 0) FROM wallets WHERE user_id = ?1",
            params![user_id], |r| r.get(0),
        )?;
        if balance < amount {
            anyhow::bail!("Insufficient funds for staking: {} < {}", balance, amount);
        }
        conn.execute(
            "UPDATE wallets SET balance = balance - ?1 WHERE user_id = ?2",
            params![amount, user_id],
        )?;
        conn.execute(
            "INSERT INTO staking (user_id, amount, apy_rate, unlocks_at)
             VALUES (?1, ?2, ?3, datetime('now', '+' || ?4 || ' days'))",
            params![user_id, amount, apy_rate, unlock_days],
        )?;
        let id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO transactions (user_id, tx_type, amount, description)
             VALUES (?1, 'stake', ?2, ?3)",
            params![user_id, amount, format!("Staked {} YSH for {} days", amount, unlock_days)],
        )?;
        Ok(id)
    }

    pub fn unstake(&self, user_id: i64, stake_id: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let row: (i64, String, i64) = conn.query_row(
            "SELECT amount, unlocks_at, rewards_earned FROM staking WHERE id = ?1 AND user_id = ?2 AND status = 'active'",
            params![stake_id, user_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        if row.1 > now {
            anyhow::bail!("Staking lock not yet expired, unlocks at: {}", row.1);
        }
        let total = row.0 + row.2;
        conn.execute(
            "UPDATE staking SET status = 'withdrawn', amount = 0, rewards_earned = 0 WHERE id = ?1",
            params![stake_id],
        )?;
        conn.execute(
            "UPDATE wallets SET balance = balance + ?1 WHERE user_id = ?2",
            params![total, user_id],
        )?;
        conn.execute(
            "INSERT INTO transactions (user_id, tx_type, amount, description)
             VALUES (?1, 'unstake', ?2, ?3)",
            params![user_id, total, format!("Unstaked + rewards from #{}", stake_id)],
        )?;
        Ok(total)
    }

    pub fn claim_staking_rewards(&self, user_id: i64, stake_id: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let rewards: i64 = conn.query_row(
            "SELECT rewards_earned FROM staking WHERE id = ?1 AND user_id = ?2 AND status = 'active'",
            params![stake_id, user_id], |r| r.get(0),
        )?;
        if rewards <= 0 {
            anyhow::bail!("No rewards to claim");
        }
        conn.execute(
            "UPDATE staking SET rewards_earned = 0, last_reward_calc = datetime('now') WHERE id = ?1",
            params![stake_id],
        )?;
        conn.execute(
            "UPDATE wallets SET balance = balance + ?1 WHERE user_id = ?2",
            params![rewards, user_id],
        )?;
        conn.execute(
            "INSERT INTO transactions (user_id, tx_type, amount, description)
             VALUES (?1, 'staking_reward', ?2, ?3)",
            params![user_id, rewards, format!("Staking reward from #{}", stake_id)],
        )?;
        Ok(rewards)
    }

    pub fn get_staking_positions(&self, user_id: i64) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, amount, apy_rate, status, staked_at, unlocks_at, rewards_earned
             FROM staking WHERE user_id = ?1 ORDER BY id DESC",
        )?;
        let rows = stmt.query_map(params![user_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "amount": row.get::<_, i64>(1)?,
                "apy_rate": row.get::<_, f64>(2)?,
                "status": row.get::<_, String>(3)?,
                "staked_at": row.get::<_, String>(4)?,
                "unlocks_at": row.get::<_, String>(5)?,
                "rewards_earned": row.get::<_, i64>(6)?,
            }))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    #[allow(dead_code)]
    pub fn calculate_staking_rewards(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, amount, apy_rate FROM staking WHERE status = 'active'",
        )?;
        let stakes: Vec<(i64, i64, f64)> = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?.filter_map(|r| r.ok()).collect();

        let mut updated = 0;
        for (id, amount, apy) in stakes {
            let daily_reward = ((amount as f64 * apy) / 365.0) as i64;
            if daily_reward > 0 {
                conn.execute(
                    "UPDATE staking SET rewards_earned = rewards_earned + ?1,
                     last_reward_calc = datetime('now') WHERE id = ?2",
                    params![daily_reward, id],
                )?;
                updated += 1;
            }
        }
        Ok(updated)
    }

    pub fn get_staking_stats(&self) -> Result<serde_json::Value> {
        let conn = self.conn.lock().unwrap();
        let total_staked: i64 = conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM staking WHERE status = 'active'",
            [], |r| r.get(0),
        )?;
        let total_rewards: i64 = conn.query_row(
            "SELECT COALESCE(SUM(rewards_earned), 0) FROM staking WHERE status = 'active'",
            [], |r| r.get(0),
        )?;
        let active_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM staking WHERE status = 'active'",
            [], |r| r.get(0),
        )?;
        Ok(serde_json::json!({
            "total_staked": total_staked,
            "total_rewards_pending": total_rewards,
            "active_positions": active_count,
        }))
    }

    // ═══════════════════════════════════════════
    // PHASE 9: REFERRALS
    // ═══════════════════════════════════════════

    pub fn create_referral(&self, referrer_id: i64, referred_id: i64, code: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO referrals (referrer_id, referred_id, code) VALUES (?1, ?2, ?3)",
            params![referrer_id, referred_id, code],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn find_referral_by_code(&self, code: &str) -> Result<Option<(i64, i64)>> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT referrer_id, referred_id FROM referrals WHERE code = ?1",
            params![code],
            |r| Ok((r.get(0)?, r.get(1)?)),
        );
        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    #[allow(dead_code)]
    pub fn complete_referral(&self, referred_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE referrals SET status = 'completed', first_purchase_at = datetime('now')
             WHERE referred_id = ?1 AND status = 'pending'",
            params![referred_id],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn add_referral_earning(&self, referrer_id: i64, amount: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE referrals SET total_earned = total_earned + ?1 WHERE referrer_id = ?2",
            params![amount, referrer_id],
        )?;
        Ok(())
    }

    pub fn get_referral_stats(&self, user_id: i64) -> Result<serde_json::Value> {
        let conn = self.conn.lock().unwrap();
        let total_referred: i64 = conn.query_row(
            "SELECT COUNT(*) FROM referrals WHERE referrer_id = ?1",
            params![user_id], |r| r.get(0),
        )?;
        let completed: i64 = conn.query_row(
            "SELECT COUNT(*) FROM referrals WHERE referrer_id = ?1 AND status = 'completed'",
            params![user_id], |r| r.get(0),
        )?;
        let total_earned: i64 = conn.query_row(
            "SELECT COALESCE(SUM(total_earned), 0) FROM referrals WHERE referrer_id = ?1",
            params![user_id], |r| r.get(0),
        )?;
        let code: Option<String> = conn.query_row(
            "SELECT code FROM referrals WHERE referrer_id = ?1 LIMIT 1",
            params![user_id], |r| r.get(0),
        ).ok();
        Ok(serde_json::json!({
            "referral_code": code,
            "total_referred": total_referred,
            "completed": completed,
            "total_earned": total_earned,
        }))
    }

    // ═══════════════════════════════════════════
    // PHASE 9: CALL BILLING
    // ═══════════════════════════════════════════

    #[allow(dead_code)]
    pub fn start_call_billing(
        &self, caller_id: i64, host_id: i64, call_type: &str, cost_per_min: i64,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO call_billing (caller_id, host_id, call_type, cost_per_min, started_at)
             VALUES (?1, ?2, ?3, ?4, datetime('now'))",
            params![caller_id, host_id, call_type, cost_per_min],
        )?;
        Ok(conn.last_insert_rowid())
    }

    #[allow(dead_code)]
    pub fn end_call_billing(&self, call_id: i64) -> Result<(i64, i64, i64)> {
        let conn = self.conn.lock().unwrap();
        let (cost_per_min, started_at): (i64, String) = conn.query_row(
            "SELECT cost_per_min, started_at FROM call_billing WHERE id = ?1 AND status = 'active'",
            params![call_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let now_str = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let start = chrono::NaiveDateTime::parse_from_str(&started_at, "%Y-%m-%dT%H:%M:%SZ")
            .unwrap_or(chrono::NaiveDateTime::default());
        let end = chrono::NaiveDateTime::parse_from_str(&now_str, "%Y-%m-%dT%H:%M:%SZ")
            .unwrap_or(chrono::NaiveDateTime::default());
        let duration_secs = (end - start).num_seconds().max(0) as i64;
        let duration_mins = ((duration_secs + 59) / 60).max(1);
        let total_cost = duration_mins * cost_per_min;
        let host_earnings = (total_cost as f64 * 0.70) as i64;
        let platform_fee = total_cost - host_earnings;

        conn.execute(
            "UPDATE call_billing SET ended_at = ?1, duration_secs = ?2,
             total_cost = ?3, host_earnings = ?4, platform_fee = ?5, status = 'completed'
             WHERE id = ?6",
            params![now_str, duration_secs, total_cost, host_earnings, platform_fee, call_id],
        )?;
        Ok((total_cost, host_earnings, platform_fee))
    }

    #[allow(dead_code)]
    pub fn get_host_call_stats(&self, host_id: i64) -> Result<serde_json::Value> {
        let conn = self.conn.lock().unwrap();
        let total_calls: i64 = conn.query_row(
            "SELECT COUNT(*) FROM call_billing WHERE host_id = ?1 AND status = 'completed'",
            params![host_id], |r| r.get(0),
        )?;
        let total_earnings: i64 = conn.query_row(
            "SELECT COALESCE(SUM(host_earnings), 0) FROM call_billing WHERE host_id = ?1 AND status = 'completed'",
            params![host_id], |r| r.get(0),
        )?;
        let total_duration: i64 = conn.query_row(
            "SELECT COALESCE(SUM(duration_secs), 0) FROM call_billing WHERE host_id = ?1 AND status = 'completed'",
            params![host_id], |r| r.get(0),
        )?;
        Ok(serde_json::json!({
            "total_calls": total_calls,
            "total_earnings": total_earnings,
            "total_duration_secs": total_duration,
        }))
    }

    // ═══════════════════════════════════════════
    // PHASE 9: COMMISSIONS (Multi-level)
    // ═══════════════════════════════════════════

    #[allow(dead_code)]
    pub fn create_commission(
        &self, user_id: i64, source_user_id: i64, source_tx_id: Option<i64>,
        tier: i32, percentage: f64, amount: i64,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO commissions (user_id, source_user_id, source_tx_id, tier, percentage, amount)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![user_id, source_user_id, source_tx_id, tier, percentage, amount],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_user_commissions(&self, user_id: i64, status: Option<&str>) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, source_user_id, tier, percentage, amount, status, created_at
             FROM commissions WHERE user_id = ?1 AND (?2 IS NULL OR status = ?2) ORDER BY id DESC",
        )?;
        let rows = stmt.query_map(params![user_id, status], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "source_user_id": row.get::<_, i64>(1)?,
                "tier": row.get::<_, i32>(2)?,
                "percentage": row.get::<_, f64>(3)?,
                "amount": row.get::<_, i64>(4)?,
                "status": row.get::<_, String>(5)?,
                "created_at": row.get::<_, String>(6)?,
            }))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_commission_summary(&self, user_id: i64) -> Result<serde_json::Value> {
        let conn = self.conn.lock().unwrap();
        let total_earned: i64 = conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM commissions WHERE user_id = ?1",
            params![user_id], |r| r.get(0),
        )?;
        let pending: i64 = conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM commissions WHERE user_id = ?1 AND status = 'pending'",
            params![user_id], |r| r.get(0),
        )?;
        let paid: i64 = conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM commissions WHERE user_id = ?1 AND status = 'paid'",
            params![user_id], |r| r.get(0),
        )?;
        let referrals_count: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT source_user_id) FROM commissions WHERE user_id = ?1",
            params![user_id], |r| r.get(0),
        )?;
        Ok(serde_json::json!({
            "total_earned": total_earned,
            "pending": pending,
            "paid": paid,
            "referrals_count": referrals_count,
        }))
    }

    #[allow(dead_code)]
    pub fn distribute_commissions(
        &self, source_user_id: i64, source_tx_id: Option<i64>, total_amount: i64,
        tier1_user_id: Option<i64>, tier2_user_id: Option<i64>,
        tier3_user_id: Option<i64>, tier4_user_id: Option<i64>,
    ) -> Result<()> {
        let tiers: Vec<(Option<i64>, f64)> = vec![
            (tier1_user_id, 0.40),
            (tier2_user_id, 0.20),
            (tier3_user_id, 0.10),
            (tier4_user_id, 0.05),
        ];
        for (i, (uid, pct)) in tiers.into_iter().enumerate() {
            if let Some(uid) = uid {
                if uid == source_user_id { continue; }
                let amount = (total_amount as f64 * pct) as i64;
                if amount > 0 {
                    self.create_commission(uid, source_user_id, source_tx_id, (i + 1) as i32, pct, amount)?;
                    self.deposit(uid, amount, &format!("Commission tier {} from user #{}", i + 1, source_user_id))?;
                }
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn pay_pending_commissions(&self, user_id: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let total: i64 = conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM commissions WHERE user_id = ?1 AND status = 'pending'",
            params![user_id], |r| r.get(0),
        )?;
        if total > 0 {
            conn.execute(
                "UPDATE commissions SET status = 'paid', paid_at = datetime('now')
                 WHERE user_id = ?1 AND status = 'pending'",
                params![user_id],
            )?;
        }
        Ok(total)
    }

    // ═══════════════════════════════════════════
    // PHASE 9: PAYOUTS
    // ═══════════════════════════════════════════

    pub fn request_payout(
        &self, user_id: i64, amount: i64, currency: &str,
        wallet_address: &str, network: &str,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let balance: i64 = conn.query_row(
            "SELECT COALESCE(balance, 0) FROM wallets WHERE user_id = ?1",
            params![user_id], |r| r.get(0),
        )?;
        if balance < amount {
            anyhow::bail!("Insufficient balance for payout: {} < {}", balance, amount);
        }
        if amount < 10 {
            anyhow::bail!("Minimum payout is 10 YSH");
        }
        conn.execute(
            "UPDATE wallets SET balance = balance - ?1 WHERE user_id = ?2",
            params![amount, user_id],
        )?;
        conn.execute(
            "INSERT INTO payouts (user_id, amount, currency, wallet_address, network)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![user_id, amount, currency, wallet_address, network],
        )?;
        let id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO transactions (user_id, tx_type, amount, description)
             VALUES (?1, 'payout_request', ?2, ?3)",
            params![user_id, amount, format!("Payout {} {} to {}", amount, currency, wallet_address)],
        )?;
        Ok(id)
    }

    pub fn get_user_payouts(&self, user_id: i64) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, amount, currency, wallet_address, network, status,
                    tx_hash, requested_at, processed_at
             FROM payouts WHERE user_id = ?1 ORDER BY id DESC",
        )?;
        let rows = stmt.query_map(params![user_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "amount": row.get::<_, i64>(1)?,
                "currency": row.get::<_, String>(2)?,
                "wallet_address": row.get::<_, String>(3)?,
                "network": row.get::<_, String>(4)?,
                "status": row.get::<_, String>(5)?,
                "tx_hash": row.get::<_, Option<String>>(6)?,
                "requested_at": row.get::<_, String>(7)?,
                "processed_at": row.get::<_, Option<String>>(8)?,
            }))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn process_payout(&self, payout_id: i64, admin_id: i64, tx_hash: &str, approved: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let status = if approved { "completed" } else { "rejected" };
        let note = if approved { "Approved" } else { "Rejected" };
        conn.execute(
            "UPDATE payouts SET status = ?1, tx_hash = ?2, processed_at = datetime('now'),
             admin_id = ?3, notes = ?4 WHERE id = ?5",
            params![status, tx_hash, admin_id, note, payout_id],
        )?;
        if !approved {
            let (user_id, amount): (i64, i64) = conn.query_row(
                "SELECT user_id, amount FROM payouts WHERE id = ?1",
                params![payout_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            conn.execute(
                "UPDATE wallets SET balance = balance + ?1 WHERE user_id = ?2",
                params![amount, user_id],
            )?;
        }
        Ok(())
    }

    pub fn get_pending_payouts(&self) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT p.id, p.user_id, u.username, p.amount, p.currency,
                    p.wallet_address, p.network, p.requested_at
             FROM payouts p JOIN users u ON u.id = p.user_id
             WHERE p.status = 'pending' ORDER BY p.requested_at ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "user_id": row.get::<_, i64>(1)?,
                "username": row.get::<_, String>(2)?,
                "amount": row.get::<_, i64>(3)?,
                "currency": row.get::<_, String>(4)?,
                "wallet_address": row.get::<_, String>(5)?,
                "network": row.get::<_, String>(6)?,
                "requested_at": row.get::<_, String>(7)?,
            }))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    // ═══════════════════════════════════════════
    // PHASE 9: FRAUD DETECTION
    // ═══════════════════════════════════════════

    pub fn create_fraud_alert(
        &self, user_id: Option<i64>, alert_type: &str, severity: &str,
        description: &str, evidence: &str, ip_address: Option<&str>,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO fraud_alerts (user_id, alert_type, severity, description, evidence, ip_address)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![user_id, alert_type, severity, description, evidence, ip_address],
        )?;
        Ok(conn.last_insert_rowid())
    }

    #[allow(dead_code)]
    pub fn get_fraud_alerts(&self, status: Option<&str>) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, user_id, alert_type, severity, description, status, created_at
             FROM fraud_alerts WHERE (?1 IS NULL OR status = ?1) ORDER BY id DESC",
        )?;
        let rows = stmt.query_map(params![status], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "user_id": row.get::<_, Option<i64>>(1)?,
                "alert_type": row.get::<_, String>(2)?,
                "severity": row.get::<_, String>(3)?,
                "description": row.get::<_, String>(4)?,
                "status": row.get::<_, String>(5)?,
                "created_at": row.get::<_, String>(6)?,
            }))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    #[allow(dead_code)]
    pub fn resolve_fraud_alert(&self, alert_id: i64, resolver_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE fraud_alerts SET status = 'resolved', resolved_at = datetime('now'),
             resolved_by = ?1 WHERE id = ?2",
            params![resolver_id, alert_id],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn check_velocity(&self, user_id: i64, tx_type: &str, window_secs: i64) -> Result<(i64, i64)> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transactions
             WHERE user_id = ?1 AND tx_type = ?2
             AND created_at > datetime('now', '-' || ?3 || ' seconds')",
            params![user_id, tx_type, window_secs],
            |r| r.get(0),
        )?;
        let total_amount: i64 = conn.query_row(
            "SELECT COALESCE(SUM(ABS(amount)), 0) FROM transactions
             WHERE user_id = ?1 AND tx_type = ?2
             AND created_at > datetime('now', '-' || ?3 || ' seconds')",
            params![user_id, tx_type, window_secs],
            |r| r.get(0),
        )?;
        Ok((count, total_amount))
    }

    // ═══════════════════════════════════════════
    // PHASE 9: RECEIPTS
    // ═══════════════════════════════════════════

    pub fn create_receipt(
        &self, user_id: i64, receipt_type: &str, reference_id: i64,
        amount: i64, currency: &str, description: &str, metadata: &str,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let receipt_content = format!("{}|{}|{}|{}|{}|{}", user_id, receipt_type, reference_id, amount, currency, description);
        let receipt_hash = blake3::hash(receipt_content.as_bytes()).to_hex().to_string();
        conn.execute(
            "INSERT INTO receipts (user_id, receipt_type, reference_id, amount, currency, description, metadata, receipt_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![user_id, receipt_type, reference_id, amount, currency, description, metadata, receipt_hash],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_receipt(&self, receipt_id: i64) -> Result<Option<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, user_id, receipt_type, reference_id, amount, currency,
                    description, metadata, receipt_hash, created_at
             FROM receipts WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![receipt_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "user_id": row.get::<_, i64>(1)?,
                "receipt_type": row.get::<_, String>(2)?,
                "reference_id": row.get::<_, i64>(3)?,
                "amount": row.get::<_, i64>(4)?,
                "currency": row.get::<_, String>(5)?,
                "description": row.get::<_, String>(6)?,
                "metadata": row.get::<_, String>(7)?,
                "receipt_hash": row.get::<_, String>(8)?,
                "created_at": row.get::<_, String>(9)?,
            }))
        })?;
        match rows.next() {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }

    pub fn get_user_receipts(&self, user_id: i64, limit: i64) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, receipt_type, reference_id, amount, currency, description, receipt_hash, created_at
             FROM receipts WHERE user_id = ?1 ORDER BY id DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![user_id, limit], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "type": row.get::<_, String>(1)?,
                "reference_id": row.get::<_, i64>(2)?,
                "amount": row.get::<_, i64>(3)?,
                "currency": row.get::<_, String>(4)?,
                "description": row.get::<_, String>(5)?,
                "receipt_hash": row.get::<_, String>(6)?,
                "created_at": row.get::<_, String>(7)?,
            }))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn verify_receipt(&self, receipt_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let (user_id, receipt_type, reference_id, amount, currency, description, stored_hash): (
            i64, String, i64, i64, String, String, String,
        ) = conn.query_row(
            "SELECT user_id, receipt_type, reference_id, amount, currency, description, receipt_hash
             FROM receipts WHERE id = ?1",
            params![receipt_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?)),
        )?;
        let content = format!("{}|{}|{}|{}|{}|{}", user_id, receipt_type, reference_id, amount, currency, description);
        let computed = blake3::hash(content.as_bytes()).to_hex().to_string();
        Ok(computed == stored_hash)
    }

    // ═══════════════════════════════════════════
    // PHASE 9: ENHANCED WALLET
    // ═══════════════════════════════════════════

    pub fn freeze_wallet(&self, user_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE wallets SET frozen = 1 WHERE user_id = ?1",
            params![user_id],
        )?;
        Ok(())
    }

    pub fn unfreeze_wallet(&self, user_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE wallets SET frozen = 0 WHERE user_id = ?1",
            params![user_id],
        )?;
        Ok(())
    }

    pub fn is_wallet_frozen(&self, user_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let frozen: i32 = conn.query_row(
            "SELECT COALESCE(frozen, 0) FROM wallets WHERE user_id = ?1",
            params![user_id], |r| r.get(0),
        ).unwrap_or(0);
        Ok(frozen != 0)
    }

    pub fn check_spending_limit(&self, user_id: i64, amount: i64) -> Result<(bool, String)> {
        let conn = self.conn.lock().unwrap();
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let month = chrono::Utc::now().format("%Y-%m").to_string();

        conn.execute(
            "INSERT OR IGNORE INTO spending_limits (user_id, last_reset_date) VALUES (?1, ?2)",
            params![user_id, today],
        )?;

        let (daily_limit, monthly_limit, daily_spent, monthly_spent, last_reset): (i64, i64, i64, i64, String) = conn.query_row(
            "SELECT daily_limit, monthly_limit, daily_spent, monthly_spent, last_reset_date
             FROM spending_limits WHERE user_id = ?1",
            params![user_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )?;

        if last_reset != today {
            conn.execute(
                "UPDATE spending_limits SET daily_spent = 0, last_reset_date = ?1 WHERE user_id = ?2",
                params![today, user_id],
            )?;
        }
        let month_prefix = &month;
        if !last_reset.starts_with(month_prefix) {
            conn.execute(
                "UPDATE spending_limits SET monthly_spent = 0 WHERE user_id = ?1",
                params![user_id],
            )?;
        }

        let daily_spent = if last_reset != today { 0 } else { daily_spent };
        if daily_spent + amount > daily_limit {
            return Ok((false, format!("Daily limit exceeded: {} + {} > {}", daily_spent, amount, daily_limit)));
        }
        if monthly_spent + amount > monthly_limit {
            return Ok((false, format!("Monthly limit exceeded: {} + {} > {}", monthly_spent, amount, monthly_limit)));
        }

        conn.execute(
            "UPDATE spending_limits SET daily_spent = daily_spent + ?1, monthly_spent = monthly_spent + ?1
             WHERE user_id = ?2",
            params![amount, user_id],
        )?;
        Ok((true, "OK".into()))
    }

    pub fn set_spending_limit(&self, user_id: i64, daily: i64, monthly: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO spending_limits (user_id, daily_limit, monthly_limit)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(user_id) DO UPDATE SET daily_limit = ?2, monthly_limit = ?3",
            params![user_id, daily, monthly],
        )?;
        Ok(())
    }

    pub fn get_spending_limits(&self, user_id: i64) -> Result<serde_json::Value> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT daily_limit, monthly_limit, daily_spent, monthly_spent
             FROM spending_limits WHERE user_id = ?1",
            params![user_id],
            |row| {
                Ok(serde_json::json!({
                    "daily_limit": row.get::<_, i64>(0)?,
                    "monthly_limit": row.get::<_, i64>(1)?,
                    "daily_spent": row.get::<_, i64>(2)?,
                    "monthly_spent": row.get::<_, i64>(3)?,
                }))
            },
        );
        match result {
            Ok(v) => Ok(v),
            Err(_) => Ok(serde_json::json!({
                "daily_limit": 100000,
                "monthly_limit": 1000000,
                "daily_spent": 0,
                "monthly_spent": 0,
            })),
        }
    }

    // ═══════════════════════════════════════════
    // PHASE 9: ENHANCED GIFTS
    // ═══════════════════════════════════════════

    pub fn get_sent_gifts(&self, user_id: i64) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT g.id, g.price, gc.name, gc.rarity, u.username as to_user, g.created_at
             FROM gifts g
             JOIN gift_catalog gc ON gc.id = g.gift_id
             JOIN users u ON u.id = g.to_user_id
             WHERE g.from_user_id = ?1 ORDER BY g.id DESC",
        )?;
        let rows = stmt.query_map(params![user_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "price": row.get::<_, i64>(1)?,
                "name": row.get::<_, String>(2)?,
                "rarity": row.get::<_, String>(3)?,
                "to_user": row.get::<_, String>(4)?,
                "created_at": row.get::<_, String>(5)?,
            }))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_gift_stats(&self, user_id: i64) -> Result<serde_json::Value> {
        let conn = self.conn.lock().unwrap();
        let sent_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM gifts WHERE from_user_id = ?1",
            params![user_id], |r| r.get(0),
        )?;
        let received_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM gifts WHERE to_user_id = ?1",
            params![user_id], |r| r.get(0),
        )?;
        let total_spent: i64 = conn.query_row(
            "SELECT COALESCE(SUM(price), 0) FROM gifts WHERE from_user_id = ?1",
            params![user_id], |r| r.get(0),
        )?;
        let total_received: i64 = conn.query_row(
            "SELECT COALESCE(SUM(price), 0) FROM gifts WHERE to_user_id = ?1",
            params![user_id], |r| r.get(0),
        )?;
        let rarest: Option<String> = conn.query_row(
            "SELECT gc.rarity FROM gifts g
             JOIN gift_catalog gc ON gc.id = g.gift_id
             WHERE g.to_user_id = ?1
             ORDER BY CASE gc.rarity WHEN 'legendary' THEN 1 WHEN 'epic' THEN 2 WHEN 'rare' THEN 3 ELSE 4 END
             LIMIT 1",
            params![user_id], |r| r.get(0),
        ).ok();
        Ok(serde_json::json!({
            "sent_count": sent_count,
            "received_count": received_count,
            "total_spent": total_spent,
            "total_received": total_received,
            "rarest_gift_rarity": rarest,
        }))
    }

    pub fn mint_nft_gift(&self, user_id: i64, gift_id: i64, gift_record_id: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let token_id = format!("YSH-NFT-{}-{}", gift_record_id, chrono::Utc::now().timestamp());
        conn.execute(
            "INSERT INTO nft_gifts (user_id, gift_id, gift_record_id, token_id)
             VALUES (?1, ?2, ?3, ?4)",
            params![user_id, gift_id, gift_record_id, token_id],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_nft_gifts(&self, user_id: i64) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT n.id, n.token_id, gc.name, gc.rarity, n.unlocked, n.minted_at
             FROM nft_gifts n
             JOIN gift_catalog gc ON gc.id = n.gift_id
             WHERE n.user_id = ?1 ORDER BY n.id DESC",
        )?;
        let rows = stmt.query_map(params![user_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "token_id": row.get::<_, String>(1)?,
                "name": row.get::<_, String>(2)?,
                "rarity": row.get::<_, String>(3)?,
                "unlocked": row.get::<_, i32>(4)? != 0,
                "minted_at": row.get::<_, String>(5)?,
            }))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
}
