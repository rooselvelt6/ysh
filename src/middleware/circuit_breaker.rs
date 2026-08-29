use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct CircuitBreaker {
    is_open: Arc<AtomicBool>,
    failure_count: Arc<std::sync::atomic::AtomicU32>,
    last_failure: Arc<std::sync::Mutex<Option<Instant>>>,
    failure_threshold: u32,
    recovery_timeout: Duration,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            is_open: Arc::new(AtomicBool::new(false)),
            failure_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
            last_failure: Arc::new(std::sync::Mutex::new(None)),
            failure_threshold,
            recovery_timeout,
        }
    }

    pub fn is_available(&self) -> bool {
        if !self.is_open.load(Ordering::SeqCst) {
            return true;
        }

        let last = self.last_failure.lock().unwrap();
        if let Some(failure_time) = *last
            && failure_time.elapsed() >= self.recovery_timeout
        {
            self.is_open.store(false, Ordering::SeqCst);
            self.failure_count.store(0, Ordering::SeqCst);
            return true;
        }
        false
    }

    pub fn record_success(&self) {
        self.failure_count.store(0, Ordering::SeqCst);
        self.is_open.store(false, Ordering::SeqCst);
    }

    pub fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        if count >= self.failure_threshold {
            self.is_open.store(true, Ordering::SeqCst);
            *self.last_failure.lock().unwrap() = Some(Instant::now());
            tracing::warn!(
                "Circuit breaker OPEN after {} failures",
                self.failure_threshold
            );
        }
    }
}
