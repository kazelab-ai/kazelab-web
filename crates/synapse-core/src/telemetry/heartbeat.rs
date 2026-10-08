//! Dynamic Heartbeat Failure Detector and Ping Liveness Collector.
//! Tracks peer latency metrics and exponential moving average (EMA) jitter across swarm actors.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct PeerLivenessRecord {
    pub peer_id: String,
    pub last_seen_epoch_ms: u64,
    pub ema_latency_ms: f64,
    pub consecutive_failures: u32,
    pub is_marked_dead: bool,
}

pub struct DynamicHeartbeatTracker {
    peers: RwLock<HashMap<String, PeerLivenessRecord>>,
    alpha_smoothing: f64,
    dead_threshold_ms: u64,
}

impl DynamicHeartbeatTracker {
    pub fn new(alpha_smoothing: f64, dead_threshold_ms: u64) -> Self {
        Self {
            peers: RwLock::new(HashMap::new()),
            alpha_smoothing,
            dead_threshold_ms,
        }
    }

    pub async fn record_heartbeat(&self, peer_id: &str, sample_latency_ms: f64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let mut map = self.peers.write().await;
        let entry = map.entry(peer_id.to_string()).or_insert_with(|| PeerLivenessRecord {
            peer_id: peer_id.to_string(),
            last_seen_epoch_ms: now,
            ema_latency_ms: sample_latency_ms,
            consecutive_failures: 0,
            is_marked_dead: false,
        });

        entry.last_seen_epoch_ms = now;
        entry.ema_latency_ms = (self.alpha_smoothing * sample_latency_ms)
            + ((1.0 - self.alpha_smoothing) * entry.ema_latency_ms);
        entry.consecutive_failures = 0;
        entry.is_marked_dead = false;
    }

    pub async fn sweep_dead_nodes(&self) -> Vec<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let mut dead_nodes = Vec::new();
        let mut map = self.peers.write().await;

        for (id, record) in map.iter_mut() {
            if !record.is_marked_dead && now.saturating_sub(record.last_seen_epoch_ms) > self.dead_threshold_ms {
                record.is_marked_dead = true;
                record.consecutive_failures += 1;
                dead_nodes.push(id.clone());
            }
        }

        dead_nodes
    }

    pub async fn get_average_cluster_latency(&self) -> f64 {
        let map = self.peers.read().await;
        if map.is_empty() {
            return 0.0;
        }
        let total: f64 = map.values().map(|r| r.ema_latency_ms).sum();
        total / (map.len() as f64)
    }
}
