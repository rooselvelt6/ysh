use crate::config::settings::DdosRateLimit;
use crate::middleware::ip_blocklist::IpBlocklist;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

struct WindowCounter {
    count: u64,
    window_start: Instant,
}

pub struct PerIpRateLimiter {
    counters: DashMap<String, WindowCounter>,
    ip_blocklist: Arc<IpBlocklist>,
    rate_limit: DdosRateLimit,
}

impl PerIpRateLimiter {
    pub fn new(rate_limit: DdosRateLimit, ip_blocklist: Arc<IpBlocklist>) -> Arc<Self> {
        Arc::new(Self {
            counters: DashMap::new(),
            ip_blocklist,
            rate_limit,
        })
    }

    fn max_for_path(&self, path: &str) -> u64 {
        if path.contains("/login") || path.contains("/register") || path.contains("/2fa") || path.contains("/recovery") {
            self.rate_limit.auth_max_per_minute
        } else if path.contains("/admin") {
            self.rate_limit.admin_max_per_minute
        } else {
            self.rate_limit.api_max_per_minute
        }
    }

    pub fn check(&self, ip: &str, path: &str) -> RateLimitResponse {
        if self.ip_blocklist.is_blocked(ip) {
            return RateLimitResponse { allowed: false, remaining: 0, retry_after_secs: 60 };
        }

        let max = self.max_for_path(path);
        let key = format!("{}:{}", ip, path);

        let entry = self.counters
            .entry(key.clone())
            .or_insert_with(|| WindowCounter {
                count: 0,
                window_start: Instant::now(),
            });

        let mut entry = entry;
        if entry.window_start.elapsed() > Duration::from_secs(60) {
            entry.count = 0;
            entry.window_start = Instant::now();
        }

        entry.count += 1;
        let count = entry.count;
        let remaining = if count <= max { max - count } else { 0 };
        let allowed = count <= max;

        if !allowed {
            let _ = self.ip_blocklist.record_error(ip);
            let retry = entry.window_start.elapsed().as_secs();
            let retry_after = if retry < 60 { 60 - retry } else { 1 };
            RateLimitResponse { allowed: false, remaining: 0, retry_after_secs: retry_after }
        } else {
            RateLimitResponse { allowed: true, remaining, retry_after_secs: 0 }
        }
    }
}

pub struct RateLimitResponse {
    pub allowed: bool,
    #[allow(dead_code)]
    pub remaining: u64,
    #[allow(dead_code)]
    pub retry_after_secs: u64,
}
