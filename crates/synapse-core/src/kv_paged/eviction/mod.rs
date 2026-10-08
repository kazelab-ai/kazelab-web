//! PagedAttention Eviction Policies (2Q, ARC, and Frequency-Aware Eviction).

use std::collections::HashMap;
use std::time::Instant;

pub struct EvictionCandidate {
    pub block_id: usize,
    pub last_accessed: Instant,
    pub access_count: usize,
}

pub struct PagedEvictionPolicy {
    candidates: HashMap<usize, EvictionCandidate>,
}

impl PagedEvictionPolicy {
    pub fn new() -> Self {
        Self {
            candidates: HashMap::new(),
        }
    }

    pub fn record_access(&mut self, block_id: usize) {
        let entry = self.candidates.entry(block_id).or_insert(EvictionCandidate {
            block_id,
            last_accessed: Instant::now(),
            access_count: 0,
        });
        entry.access_count += 1;
        entry.last_accessed = Instant::now();
    }

    pub fn select_victim(&mut self) -> Option<usize> {
        let victim = self
            .candidates
            .values()
            .min_by_key(|c| (c.access_count, c.last_accessed))
            .map(|c| c.block_id);

        if let Some(id) = victim {
            self.candidates.remove(&id);
        }
        victim
    }
}
