//! Adaptive Backpressure Engine and Token-Bucket Rate Limiter for High-Frequency Swarm Interconnects.
//! Protects downstream actor mailboxes and LLM providers from request throttling and memory bloat.

use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct TokenBucketRateLimiter {
    capacity: i64,
    tokens: AtomicI64,
    refill_rate_per_sec: f64,
    last_refill_epoch_ms: AtomicU64,
}

impl TokenBucketRateLimiter {
    pub fn new(capacity: i64, refill_rate_per_sec: f64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            capacity,
            tokens: AtomicI64::new(capacity),
            refill_rate_per_sec,
            last_refill_epoch_ms: AtomicU64::new(now),
        }
    }

    fn refill(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let last = self.last_refill_epoch_ms.swap(now, Ordering::SeqCst);
        let elapsed_secs = (now.saturating_sub(last) as f64) / 1000.0;

        if elapsed_secs > 0.0 {
            let added_tokens = (elapsed_secs * self.refill_rate_per_sec) as i64;
            if added_tokens > 0 {
                let mut current = self.tokens.load(Ordering::Relaxed);
                loop {
                    let next = (current + added_tokens).min(self.capacity);
                    match self.tokens.compare_exchange_weak(
                        current,
                        next,
                        Ordering::SeqCst,
                        Ordering::Relaxed,
                    ) {
                        Ok(_) => break,
                        Err(actual) => current = actual,
                    }
                }
            }
        }
    }

    pub fn try_acquire(&self, count: i64) -> bool {
        self.refill();

        let mut current = self.tokens.load(Ordering::Relaxed);
        loop {
            if current < count {
                return false;
            }
            let next = current - count;
            match self.tokens.compare_exchange_weak(
                current,
                next,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => return true,
                Err(actual) => current = actual,
            }
        }
    }

    pub fn available_tokens(&self) -> i64 {
        self.refill();
        self.tokens.load(Ordering::Relaxed)
    }
}
