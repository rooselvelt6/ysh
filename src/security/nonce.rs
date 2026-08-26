use std::sync::atomic::{AtomicU64, Ordering};
use zeroize::Zeroize;

pub struct NonceGenerator {
    counter: AtomicU64,
    base: [u8; 4],
}

impl NonceGenerator {
    pub fn new() -> Self {
        let mut base = [0u8; 4];
        use rand_core::OsRng;
        use rand_core::RngCore;
        OsRng.fill_bytes(&mut base);
        Self {
            counter: AtomicU64::new(0),
            base,
        }
    }

    pub fn next(&self) -> [u8; 12] {
        let count = self.counter.fetch_add(1, Ordering::SeqCst);
        let mut nonce = [0u8; 12];
        nonce[..4].copy_from_slice(&self.base);
        nonce[4..].copy_from_slice(&count.to_le_bytes());
        nonce
    }

    pub fn current_counter(&self) -> u64 {
        self.counter.load(Ordering::SeqCst)
    }
}

impl Default for NonceGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for NonceGenerator {
    fn drop(&mut self) {
        self.base.zeroize();
    }
}
