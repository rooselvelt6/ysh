# Plan: 5 Robustness/Security Improvements for YSH (redb)

## Context
User requested resolution of 5 concerns regarding the redb database layer:
1. Upgrade from redb 2.x to 4.x
2. Write queue for concurrency safety
3. Backup/snapshot automation
4. Integrity check on startup
5. Encryption at rest

## Current State
- redb 2.6.3, Rust 2024 edition, 177 tests passing, 0 warnings, 0 errors
- `Database` struct wraps `redb::Database` behind `Arc<Mutex<redb::Database>>`
- 40 table definitions (10 `TableDefinition` + 30 `MultimapTableDefinition`)
- All tests use `tempfile::NamedTempFile` for isolation

---

## Fix 1: Upgrade redb 2.x → 4.x

### Changes
- `Cargo.toml`: `redb = "2"` → `redb = "4"`
- API changes to handle:
  - `Durability::Paranoid` removed (we don't use it — no change needed)
  - `AccessGuardMut` renamed to `AccessGuardMutInPlace` (we don't use it — no change needed)
  - `begin_write()` now blocks instead of panicking (actually better for us)
  - `check_integrity()` exists and returns `Result<bool, DatabaseError>`
  - `compact()` is available
  - `Durability` is per-transaction, not per-database
  - `StorageBackend` trait available with `create_with_backend()` on Builder
- **Breaking**: redb v2 databases use v2 format, v4 only supports v3+. Since we use `tempfile` for tests and create fresh DBs, this is fine. New DBs default to v3 format.

### Verification
- `cargo build` — 0 errors
- `cargo test` — all 177 pass
- `cargo clippy` — 0 warnings

---

## Fix 2: Write Queue for Concurrency

### Problem
redb only allows 1 write transaction at a time. `begin_write()` blocks if another is in progress. Under heavy load this causes thread starvation and potential deadlocks.

### Solution
Add a write queue using `tokio::sync::mpsc` bounded channel:

```rust
struct WriteRequest {
    // closure that performs the write transaction
    operation: Box<dyn FnOnce(&redb::Database) -> Result<(), YshError> + Send>,
    response: oneshot::Sender<Result<(), YshError>>,
}
```

Changes to `src/db.rs`:
- Add `write_tx: mpsc::Sender<WriteRequest>` to `Database` struct
- Spawn a background task that drains the channel and executes writes sequentially
- Replace direct `self.inner.lock().begin_write()` calls with `self.write_tx.send(...)` pattern
- Keep read methods unchanged (reads are already concurrent via `begin_read()`)
- Add `Database::write(&self, op) -> Result<()>` convenience method

### Verification
- `cargo test` — all pass
- Concurrent write test: spawn multiple tasks writing simultaneously, verify no panics

---

## Fix 3: Backup/Snapshot Automation

### Solution
Add to `src/db.rs`:

```rust
impl Database {
    /// Create a backup of the database to the given path
    pub fn backup(&self, dest: impl AsRef<Path>) -> Result<(), YshError> {
        // compact first, then copy the file
        // Uses fs::copy on the locked database
    }

    /// Run compaction to reclaim space
    pub fn compact(&self) -> Result<(), YshError> {
        // Wraps redb's compact()
    }
}
```

Add to `src/config/settings.rs`:
```rust
pub struct BackupConfig {
    pub enabled: bool,
    pub interval_secs: u64,      // how often to backup
    pub backup_dir: String,      // where to store backups
    pub max_backups: usize,      // rotation count
    pub compact_before_backup: bool,
}
```

Add a background tokio task in `src/server.rs` (or new `src/backup.rs`) that:
1. Runs every `interval_secs`
2. Calls `compact()` if configured
3. Calls `backup()` to a timestamped file
4. Rotates old backups beyond `max_backups`

Config in `config/default.toml`:
```toml
[backup]
enabled = false
interval_secs = 3600
backup_dir = "./backups"
max_backups = 7
compact_before_backup = true
```

### Verification
- Unit test: create DB, write data, backup, open backup, verify data
- Integration test: backup + restore cycle

---

## Fix 4: Integrity Check on Startup

### Solution
Add to `src/db.rs`:

```rust
impl Database {
    /// Check database integrity. Returns Ok(true) if OK, Ok(false) if repaired, Err if corrupted.
    pub fn check_integrity(&self) -> Result<IntegrityReport, YshError> {
        // Wraps redb's check_integrity()
        // Returns detailed report
    }
}

pub struct IntegrityReport {
    pub status: IntegrityStatus,
    pub message: String,
}

pub enum IntegrityStatus {
    Ok,
    Repaired,
    Corrupted,
}
```

Call `check_integrity()` on startup in `Database::new()` or in server initialization.

Config:
```toml
[integrity]
check_on_startup = true
auto_repair = true
```

If integrity check fails:
- Log critical error
- If `auto_repair` is true, attempt repair via `check_integrity()`
- If repair fails, return error and refuse to start (prevents data corruption propagation)

### Verification
- Test: corrupt a temp file, verify detection
- Test: normal DB passes integrity check

---

## Fix 5: Encryption at Rest

### Solution
Implement a custom `StorageBackend` wrapper using AES-256-GCM:

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

struct EncryptedBackend {
    inner: File,
    cipher: Aes256Gcm,
    page_size: usize, // redb default: 4096
}
```

Page-level encryption:
- Each page is encrypted independently with AES-256-GCM
- Nonce = 8-byte page_number ++ 4-byte counter (unique per page)
- Each write generates a fresh nonce
- `read()` decrypts; `write()` encrypts
- Master key stored in a separate keyfile or environment variable

Changes:
- New file: `src/db/encryption.rs` (or `src/crypto_backend.rs`)
- `EncryptedBackend` implements `StorageBackend` trait
- `Builder::create_with_backend(EncryptedBackend::new(path, key)?)`
- Config:
```toml
[encryption]
enabled = false
key_env = "YSH_DB_KEY"      # env var holding hex key
key_file = "./keyfile"       # OR key file path
algorithm = "AES-256-GCM"
```

Key management:
- On first run with encryption enabled, generate random 32-byte key, save to key_file
- On subsequent runs, load key from key_file or env var
- If key is lost, data is irrecoverable (by design)

### Verification
- Test: write data with encryption, verify raw file is unreadable
- Test: read back with same key, verify data integrity
- Test: wrong key fails to open

---

## Implementation Order
1. **Fix 1**: Upgrade redb 2→4 (foundation, everything else depends on this)
2. **Fix 4**: Integrity check (simple, uses redb's built-in)
3. **Fix 3**: Backup (simple file operations)
4. **Fix 2**: Write queue (adds complexity, needs careful testing)
5. **Fix 5**: Encryption (most complex, new subsystem)

## Files to Modify
- `Cargo.toml` — redb version bump, possibly new deps
- `src/db.rs` — all fixes touch this (integrity, backup, write queue, encryption backend)
- `src/config/settings.rs` — BackupConfig, IntegrityConfig, EncryptionConfig
- `src/config/loader.rs` — load new configs
- `config/default.toml` — new config sections
- `src/lib.rs` — new module exports
- `tests/db_tests.rs` — new tests for each fix

## New Files
- `src/db/encryption.rs` — EncryptedBackend (StorageBackend impl)
- `src/backup.rs` — backup scheduler (optional, could be in db.rs)

## Verification
After all 5 fixes:
```bash
cargo build 2>&1 | grep -E "error|warning" | wc -l  # expect 0
cargo test 2>&1 | tail -5  # expect all pass, count > 177
cargo clippy 2>&1 | grep -E "warning|error" | wc -l  # expect 0
```
