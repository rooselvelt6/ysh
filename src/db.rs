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
            CREATE INDEX IF NOT EXISTS idx_chat_participants ON chat_participants(user_id);",
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
}
