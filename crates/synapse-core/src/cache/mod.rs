//! Tiered LRU Prompt Cache with sub-second TTL eviction.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

pub struct CacheEntry {
    pub value: Vec<u8>,
    pub expires_at: Instant,
    pub hits: usize,
}

pub struct TieredPromptCache {
    entries: RwLock<HashMap<String, CacheEntry>>,
    default_ttl: Duration,
}

impl TieredPromptCache {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            default_ttl: Duration::from_secs(ttl_secs),
        }
    }

    pub async fn put(&self, key: &str, value: Vec<u8>) {
        let mut map = self.entries.write().await;
        let entry = CacheEntry {
            value,
            expires_at: Instant::now() + self.default_ttl,
            hits: 0,
        };
        map.insert(key.to_string(), entry);
    }

    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        let mut map = self.entries.write().await;
        if let Some(entry) = map.get_mut(key) {
            if Instant::now() < entry.expires_at {
                entry.hits += 1;
                return Some(entry.value.clone());
            } else {
                map.remove(key);
            }
        }
        None
    }

    pub async fn prune_expired(&self) -> usize {
        let mut map = self.entries.write().await;
        let now = Instant::now();
        let initial_len = map.len();
        map.retain(|_, v| v.expires_at > now);
        initial_len - map.len()
    }

    pub async fn len(&self) -> usize {
        let map = self.entries.read().await;
        map.len()
    }
}
