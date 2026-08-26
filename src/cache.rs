use anyhow::Result;
use sled::Db;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct Cache {
    db: Db,
}

impl Cache {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let db = sled::Config::new()
            .path(path)
            .cache_capacity(64 * 1024 * 1024)
            .flush_every_ms(Some(1000))
            .open()?;
        Ok(Self { db })
    }

    pub fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        let entry = RawEntry {
            value: value.to_vec(),
            expires_at: None,
        };
        self.db.insert(key.as_bytes(), serialize(&entry)?)?;
        self.db.flush()?;
        Ok(())
    }

    pub fn set_with_ttl(&self, key: &str, value: &[u8], ttl: Duration) -> Result<()> {
        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs()
            + ttl.as_secs();
        let entry = RawEntry {
            value: value.to_vec(),
            expires_at: Some(expires_at),
        };
        self.db.insert(key.as_bytes(), serialize(&entry)?)?;
        self.db.flush()?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        match self.db.get(key.as_bytes())? {
            Some(data) => {
                let entry = deserialize(&data)?;
                if let Some(expires_at) = entry.expires_at {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)?
                        .as_secs();
                    if now > expires_at {
                        self.db.remove(key.as_bytes())?;
                        return Ok(None);
                    }
                }
                Ok(Some(entry.value))
            }
            None => Ok(None),
        }
    }

    pub fn get_string(&self, key: &str) -> Result<Option<String>> {
        match self.get(key)? {
            Some(bytes) => Ok(Some(String::from_utf8(bytes)?)),
            None => Ok(None),
        }
    }

    pub fn set_string(&self, key: &str, value: &str) -> Result<()> {
        self.set(key, value.as_bytes())
    }

    pub fn set_string_with_ttl(&self, key: &str, value: &str, ttl: Duration) -> Result<()> {
        self.set_with_ttl(key, value.as_bytes(), ttl)
    }

    pub fn delete(&self, key: &str) -> Result<bool> {
        let existed = self.db.remove(key.as_bytes())?.is_some();
        self.db.flush()?;
        Ok(existed)
    }

    #[allow(dead_code)]
    pub fn exists(&self, key: &str) -> Result<bool> {
        match self.db.get(key.as_bytes())? {
            Some(data) => {
                let entry = deserialize(&data)?;
                if let Some(expires_at) = entry.expires_at {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)?
                        .as_secs();
                    if now > expires_at {
                        self.db.remove(key.as_bytes())?;
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            None => Ok(false),
        }
    }

    #[allow(dead_code)]
    pub fn increment(&self, key: &str) -> Result<u64> {
        let new_val = self
            .db
            .fetch_and_update(key.as_bytes(), |old| {
                let count = old
                    .and_then(|v| deserialize(v).ok())
                    .and_then(|e| {
                        if let Some(expires_at) = e.expires_at {
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            if now > expires_at {
                                return None;
                            }
                        }
                        String::from_utf8(e.value).ok()?.parse::<u64>().ok()
                    })
                    .unwrap_or(0)
                    + 1;
                let entry = RawEntry {
                    value: count.to_string().into_bytes(),
                    expires_at: None,
                };
                serialize(&entry).ok()
            })?
            .map(|v| {
                deserialize(&v)
                    .ok()
                    .and_then(|e| String::from_utf8(e.value).ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0)
            })
            .unwrap_or(0);
        self.db.flush()?;
        Ok(new_val)
    }

    pub fn increment_with_ttl(&self, key: &str, ttl: Duration) -> Result<u64> {
        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs()
            + ttl.as_secs();
        let new_val = self
            .db
            .fetch_and_update(key.as_bytes(), |old| {
                let count = old
                    .and_then(|v| deserialize(v).ok())
                    .and_then(|e| {
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        if let Some(exp) = e.expires_at {
                            if now > exp {
                                return None;
                            }
                        }
                        String::from_utf8(e.value).ok()?.parse::<u64>().ok()
                    })
                    .unwrap_or(0)
                    + 1;
                let entry = RawEntry {
                    value: count.to_string().into_bytes(),
                    expires_at: Some(expires_at),
                };
                serialize(&entry).ok()
            })?
            .map(|v| {
                deserialize(&v)
                    .ok()
                    .and_then(|e| String::from_utf8(e.value).ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0)
            })
            .unwrap_or(0);
        self.db.flush()?;
        Ok(new_val)
    }

    pub fn health_check(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }

    pub fn stats(&self) -> CacheStats {
        let mut stats = CacheStats::default();
        for item in self.db.iter() {
            if let Ok((_key, val)) = item {
                stats.total_entries += 1;
                stats.total_bytes += val.len() as u64;
            }
        }
        stats
    }
}

#[derive(Debug, Default)]
pub struct CacheStats {
    pub total_entries: u64,
    pub total_bytes: u64,
}

struct RawEntry {
    value: Vec<u8>,
    expires_at: Option<u64>,
}

fn serialize(entry: &RawEntry) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&(entry.value.len() as u32).to_le_bytes());
    buf.extend_from_slice(&entry.value);
    match entry.expires_at {
        Some(t) => {
            buf.push(1);
            buf.extend_from_slice(&t.to_le_bytes());
        }
        None => {
            buf.push(0);
        }
    }
    Ok(buf)
}

fn deserialize(data: &[u8]) -> Result<RawEntry> {
    if data.len() < 4 {
        anyhow::bail!("Cache entry too short");
    }
    let mut pos = 0;

    let val_len = u32::from_le_bytes(data[pos..pos + 4].try_into()?) as usize;
    pos += 4;

    if data.len() < pos + val_len + 1 {
        anyhow::bail!("Cache entry corrupted: val_len={} but only {} bytes remain", val_len, data.len() - pos);
    }

    let value = data[pos..pos + val_len].to_vec();
    pos += val_len;

    let has_expiry = data[pos];
    pos += 1;

    let expires_at = if has_expiry == 1 {
        if data.len() < pos + 8 {
            anyhow::bail!("Cache entry truncated: missing expiry bytes");
        }
        Some(u64::from_le_bytes(
            data[pos..pos + 8].try_into()?,
        ))
    } else {
        None
    };

    Ok(RawEntry {
        value,
        expires_at,
    })
}

pub struct SessionCache {
    cache: Cache,
}

impl SessionCache {
    pub fn new(cache: Cache) -> Self {
        Self { cache }
    }

    pub fn health_check(&self) -> Result<()> {
        self.cache.health_check()
    }

    #[allow(dead_code)]
    pub fn store_session(
        &self,
        session_id: &str,
        user_id: &str,
        ttl: Duration,
    ) -> Result<()> {
        self.cache
            .set_string_with_ttl(&format!("session:{}", session_id), user_id, ttl)
    }

    #[allow(dead_code)]
    pub fn get_session(&self, session_id: &str) -> Result<Option<String>> {
        self.cache.get_string(&format!("session:{}", session_id))
    }

    #[allow(dead_code)]
    pub fn destroy_session(&self, session_id: &str) -> Result<bool> {
        self.cache.delete(&format!("session:{}", session_id))
    }
}

pub struct RateLimitCache {
    cache: Cache,
}

impl RateLimitCache {
    pub fn new(cache: Cache) -> Self {
        Self { cache }
    }

    pub fn health_check(&self) -> Result<()> {
        self.cache.health_check()
    }

    pub fn check_rate_limit(
        &self,
        key: &str,
        max_requests: u64,
        window: Duration,
    ) -> Result<RateLimitResult> {
        let cache_key = format!("rl:{}", key);
        let current = self.cache.increment_with_ttl(&cache_key, window)?;
        Ok(RateLimitResult {
            allowed: current <= max_requests,
            remaining: if current <= max_requests {
                max_requests - current
            } else {
                0
            },
            limit: max_requests,
            retry_after: if current > max_requests {
                Some(window)
            } else {
                None
            },
        })
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub remaining: u64,
    pub limit: u64,
    pub retry_after: Option<Duration>,
}
