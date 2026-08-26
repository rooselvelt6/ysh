use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

struct BlockEntry {
    blocked_at: Instant,
    duration: Duration,
}

pub struct IpBlocklist {
    blocked: DashMap<String, BlockEntry>,
    error_counts: DashMap<String, (AtomicU64, Instant)>,
    threshold: u64,
    window: Duration,
    block_duration: Duration,
    max_size: usize,
}

impl IpBlocklist {
    pub fn new(threshold: u64, window_secs: u64, block_secs: u64, max_size: usize) -> Arc<Self> {
        Arc::new(Self {
            blocked: DashMap::new(),
            error_counts: DashMap::new(),
            threshold,
            window: Duration::from_secs(window_secs),
            block_duration: Duration::from_secs(block_secs),
            max_size,
        })
    }

    pub fn is_blocked(&self, ip: &str) -> bool {
        if let Some(entry) = self.blocked.get(ip) {
            if entry.blocked_at.elapsed() < entry.duration {
                return true;
            }
            drop(entry);
            self.blocked.remove(ip);
        }
        false
    }

    pub fn record_error(&self, ip: &str) -> bool {
        let should_block = {
            let entry = self.error_counts
                .entry(ip.to_string())
                .or_insert_with(|| (AtomicU64::new(0), Instant::now()));

            let (count, first_seen) = entry.value();
            let expired = first_seen.elapsed() > self.window;
            let current_count = count.load(Ordering::Relaxed);

            if expired {
                drop(entry);
                self.error_counts.insert(ip.to_string(), (AtomicU64::new(1), Instant::now()));
                return false;
            }

            let new_count = current_count + 1;
            count.store(new_count, Ordering::Relaxed);
            new_count >= self.threshold
        };

        if should_block {
            self.error_counts.remove(ip);
            if self.blocked.len() < self.max_size {
                self.blocked.insert(ip.to_string(), BlockEntry {
                    blocked_at: Instant::now(),
                    duration: self.block_duration,
                });
                tracing::warn!("Auto-blocked IP {} for {}s", ip, self.block_duration.as_secs());
            }
            return true;
        }
        false
    }

    #[allow(dead_code)]
    pub fn block_ip(&self, ip: &str, duration: Duration) {
        if self.blocked.len() < self.max_size {
            self.blocked.insert(ip.to_string(), BlockEntry {
                blocked_at: Instant::now(),
                duration,
            });
            tracing::info!("Manually blocked IP {} for {}s", ip, duration.as_secs());
        }
    }

    #[allow(dead_code)]
    pub fn unblock_ip(&self, ip: &str) -> bool {
        self.blocked.remove(ip).is_some()
    }

    pub fn blocked_count(&self) -> usize {
        self.blocked.len()
    }

    #[allow(dead_code)]
    pub fn cleanup_expired(&self) {
        self.blocked.retain(|_, entry| entry.blocked_at.elapsed() < entry.duration);
        self.error_counts.retain(|_, (count, first_seen)| {
            first_seen.elapsed() < self.window && count.load(Ordering::Relaxed) > 0
        });
    }
}
