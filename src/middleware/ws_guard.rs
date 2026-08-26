use crate::config::settings::DdosWs;
use crate::middleware::ip_blocklist::IpBlocklist;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub struct WsGuard {
    #[allow(dead_code)]
    connections_per_user: DashMap<i64, AtomicU32>,
    #[allow(dead_code)]
    message_counts: DashMap<String, (AtomicU32, Instant)>,
    #[allow(dead_code)]
    config: DdosWs,
    #[allow(dead_code)]
    ip_blocklist: Arc<IpBlocklist>,
}

impl WsGuard {
    pub fn new(config: DdosWs, ip_blocklist: Arc<IpBlocklist>) -> Arc<Self> {
        Arc::new(Self {
            connections_per_user: DashMap::new(),
            message_counts: DashMap::new(),
            config,
            ip_blocklist,
        })
    }

    #[allow(dead_code)]
    pub fn can_connect(&self, user_id: i64) -> bool {
        let entry = self.connections_per_user
            .entry(user_id)
            .or_insert_with(|| AtomicU32::new(0));
        let count = entry.load(Ordering::Relaxed);
        count < self.config.max_connections_per_user
    }

    #[allow(dead_code)]
    pub fn on_connect(&self, user_id: i64) {
        let entry = self.connections_per_user
            .entry(user_id)
            .or_insert_with(|| AtomicU32::new(0));
        entry.fetch_add(1, Ordering::Relaxed);
    }

    #[allow(dead_code)]
    pub fn on_disconnect(&self, user_id: i64) {
        if let Some(entry) = self.connections_per_user.get(&user_id) {
            let prev = entry.fetch_sub(1, Ordering::Relaxed);
            if prev <= 1 {
                drop(entry);
                self.connections_per_user.remove(&user_id);
            }
        }
    }

    #[allow(dead_code)]
    pub fn check_message_rate(&self, user_id: i64) -> bool {
        let key = format!("ws:{}", user_id);

        {
            let entry = self.message_counts
                .entry(key.clone())
                .or_insert_with(|| (AtomicU32::new(0), Instant::now()));

            let (count, start) = entry.value();
            if start.elapsed().as_secs() >= 1 {
                count.store(1, Ordering::Relaxed);
                drop(entry);
                let _ = self.message_counts.insert(key, (AtomicU32::new(1), Instant::now()));
                return true;
            }

            let current = count.fetch_add(1, Ordering::Relaxed) + 1;
            current <= self.config.message_rate_per_second
        }
    }

    #[allow(dead_code)]
    pub fn max_message_size(&self) -> usize {
        self.config.max_message_size_bytes
    }

    #[allow(dead_code)]
    pub fn heartbeat_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.config.heartbeat_timeout_secs)
    }

    #[allow(dead_code)]
    pub fn cleanup(&self) {
        self.message_counts.retain(|_, (count, start)| {
            start.elapsed().as_secs() < 60 && count.load(Ordering::Relaxed) > 0
        });
    }
}
