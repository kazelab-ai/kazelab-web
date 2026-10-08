//! High-resolution metrics, cycle latency, and token efficiency statistics.

pub mod heartbeat;

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SwarmTelemetrySnapshot {
    pub total_messages_dispatched: usize,
    pub total_tokens_processed: u64,
    pub cached_tokens_saved: u64,
    pub average_cycle_duration_ms: f64,
    pub self_healing_invocations: usize,
    pub active_cluster_nodes: usize,
}

pub struct TelemetryCollector {
    dispatched_counter: AtomicUsize,
    tokens_processed: AtomicU64,
    cached_tokens: AtomicU64,
    total_latency_micros: AtomicU64,
    self_healing_counter: AtomicUsize,
}

impl TelemetryCollector {
    pub fn new() -> Self {
        Self {
            dispatched_counter: AtomicUsize::new(0),
            tokens_processed: AtomicU64::new(0),
            cached_tokens: AtomicU64::new(0),
            total_latency_micros: AtomicU64::new(0),
            self_healing_counter: AtomicUsize::new(0),
        }
    }

    pub fn record_dispatch(&self, tokens: u64, cached: u64, latency_micros: u64) {
        self.dispatched_counter.fetch_add(1, Ordering::Relaxed);
        self.tokens_processed.fetch_add(tokens, Ordering::Relaxed);
        self.cached_tokens.fetch_add(cached, Ordering::Relaxed);
        self.total_latency_micros.fetch_add(latency_micros, Ordering::Relaxed);
    }

    pub fn record_self_healing_attempt(&self) {
        self.self_healing_counter.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self, active_nodes: usize) -> SwarmTelemetrySnapshot {
        let count = self.dispatched_counter.load(Ordering::Relaxed);
        let total_lat = self.total_latency_micros.load(Ordering::Relaxed);
        let avg_ms = if count > 0 {
            (total_lat as f64) / (count as f64) / 1000.0
        } else {
            0.0
        };

        SwarmTelemetrySnapshot {
            total_messages_dispatched: count,
            total_tokens_processed: self.tokens_processed.load(Ordering::Relaxed),
            cached_tokens_saved: self.cached_tokens.load(Ordering::Relaxed),
            average_cycle_duration_ms: avg_ms,
            self_healing_invocations: self.self_healing_counter.load(Ordering::Relaxed),
            active_cluster_nodes: active_nodes,
        }
    }
}
