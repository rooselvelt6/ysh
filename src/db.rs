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
            );",
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
}
