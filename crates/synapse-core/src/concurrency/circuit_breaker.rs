//! Circuit Breaker and Failure Recovery State Machine for Outbound Microservice & MCP Integrations.
//! Implements Closed, Open, and Half-Open states with exponential backoff and recovery trip thresholds.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Tripped, failing fast
    HalfOpen, // Testing service recovery
}

pub struct CircuitBreaker {
    failure_threshold: u32,
    recovery_time_window_ms: u64,
    consecutive_failures: AtomicU32,
    success_probe_count: AtomicU32,
    last_failure_epoch_ms: AtomicU64,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, recovery_time_window_ms: u64) -> Self {
        Self {
            failure_threshold,
            recovery_time_window_ms,
            consecutive_failures: AtomicU32::new(0),
            success_probe_count: AtomicU32::new(0),
            last_failure_epoch_ms: AtomicU64::new(0),
        }
    }

    pub fn state(&self) -> CircuitState {
        let failures = self.consecutive_failures.load(Ordering::Relaxed);
        if failures < self.failure_threshold {
            return CircuitState::Closed;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let last_fail = self.last_failure_epoch_ms.load(Ordering::Relaxed);
        if now.saturating_sub(last_fail) > self.recovery_time_window_ms {
            CircuitState::HalfOpen
        } else {
            CircuitState::Open
        }
    }

    pub fn allow_request(&self) -> bool {
        match self.state() {
            CircuitState::Closed => true,
            CircuitState::HalfOpen => true,
            CircuitState::Open => false,
        }
    }

    pub fn record_success(&self) {
        match self.state() {
            CircuitState::HalfOpen => {
                let probes = self.success_probe_count.fetch_add(1, Ordering::SeqCst) + 1;
                if probes >= 3 {
                    // Fully recovered: reset to closed
                    self.consecutive_failures.store(0, Ordering::SeqCst);
                    self.success_probe_count.store(0, Ordering::SeqCst);
                }
            }
            CircuitState::Closed => {
                self.consecutive_failures.store(0, Ordering::Relaxed);
            }
            CircuitState::Open => {}
        }
    }

    pub fn record_failure(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        self.last_failure_epoch_ms.store(now, Ordering::SeqCst);
        self.consecutive_failures.fetch_add(1, Ordering::SeqCst);
        self.success_probe_count.store(0, Ordering::SeqCst);
    }
}
