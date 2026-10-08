//! Sharded Concurrent Striped Hash Map for Multi-Core Thread Contention Elimination.
//! Partitions keys across $2^k$ independent lock stripes to prevent lock convoys under heavy actor concurrency.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use tokio::sync::RwLock;

pub struct StripedCacheShard<V> {
    entries: RwLock<HashMap<String, V>>,
}

pub struct StripedConcurrentCache<V: Clone> {
    shards: Vec<StripedCacheShard<V>>,
    shard_mask: usize,
}

impl<V: Clone> StripedConcurrentCache<V> {
    /// Creates a new striped concurrent cache with `num_shards` power-of-two partitions (e.g. 16 or 32).
    pub fn new(num_shards: usize) -> Self {
        let rounded_shards = num_shards.next_power_of_two();
        let mut shards = Vec::with_capacity(rounded_shards);
        for _ in 0..rounded_shards {
            shards.push(StripedCacheShard {
                entries: RwLock::new(HashMap::new()),
            });
        }
        Self {
            shards,
            shard_mask: rounded_shards - 1,
        }
    }

    #[inline(always)]
    fn get_shard_index(&self, key: &str) -> usize {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) & self.shard_mask
    }

    pub async fn insert(&self, key: &str, value: V) {
        let shard_idx = self.get_shard_index(key);
        let mut guard = self.shards[shard_idx].entries.write().await;
        guard.insert(key.to_string(), value);
    }

    pub async fn get(&self, key: &str) -> Option<V> {
        let shard_idx = self.get_shard_index(key);
        let guard = self.shards[shard_idx].entries.read().await;
        guard.get(key).cloned()
    }

    pub async fn remove(&self, key: &str) -> Option<V> {
        let shard_idx = self.get_shard_index(key);
        let mut guard = self.shards[shard_idx].entries.write().await;
        guard.remove(key)
    }

    pub async fn total_entries(&self) -> usize {
        let mut sum = 0;
        for shard in &self.shards {
            let guard = shard.entries.read().await;
            sum += guard.len();
        }
        sum
    }

    pub fn shard_count(&self) -> usize {
        self.shards.len()
    }
}
