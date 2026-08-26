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
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
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
            created_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        };
        Ok(user)
    }

    pub fn find_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT id, username, email, password_hash, role, created_at FROM users WHERE username = ?1")?;
        let mut rows = stmt.query_map(params![username], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                email: row.get(2)?,
                password_hash: row.get(3)?,
                role: row.get(4)?,
                created_at: row.get(5)?,
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
}
