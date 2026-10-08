//! Dynamic Memory Defragmenter and Page Compaction Engine for PagedAttention Blocks.
//! Migrates fragmented token KV blocks into contiguous memory ranges to maximize GPU memory throughput.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CompactionPlan {
    pub migrations: Vec<(usize, usize)>, // (source_block_id, target_block_id)
    pub blocks_freed: usize,
    pub fragmentation_before: f64,
    pub fragmentation_after: f64,
}

pub struct KvMemoryDefragmenter;

impl KvMemoryDefragmenter {
    /// Computes an optimal compaction plan using the Two-Finger sliding algorithm.
    pub fn plan_compaction(
        block_occupancy: &HashMap<usize, usize>, // block_id -> occupied_slots_count
        capacity_per_block: usize,
        _total_blocks: usize,
    ) -> CompactionPlan {
        let mut occupied_blocks: Vec<(usize, usize)> = block_occupancy
            .iter()
            .map(|(&id, &cnt)| (id, cnt))
            .collect();

        // Sort descending by block ID for trailing block reclamation
        occupied_blocks.sort_by_key(|b| b.0);

        let total_occupied_slots: usize = occupied_blocks.iter().map(|b| b.1).sum();
        let min_required_blocks = (total_occupied_slots + capacity_per_block - 1) / capacity_per_block;

        let frag_before = if !occupied_blocks.is_empty() {
            1.0 - (total_occupied_slots as f64 / (occupied_blocks.len() * capacity_per_block) as f64)
        } else {
            0.0
        };

        let mut migrations = Vec::new();
        let mut left = 0;
        let mut right = occupied_blocks.len().saturating_sub(1);

        while left < right && occupied_blocks.len() > min_required_blocks {
            // Find free slot in left block
            let left_free = capacity_per_block.saturating_sub(occupied_blocks[left].1);
            if left_free == 0 {
                left += 1;
                continue;
            }

            let right_slots = occupied_blocks[right].1;
            if right_slots <= left_free {
                // Completely migrate right block into left block
                migrations.push((occupied_blocks[right].0, occupied_blocks[left].0));
                occupied_blocks[left].1 += right_slots;
                right = right.saturating_sub(1);
            } else {
                left += 1;
            }
        }

        let blocks_freed = migrations.len();
        let remaining_blocks = occupied_blocks.len().saturating_sub(blocks_freed);
        let frag_after = if remaining_blocks > 0 {
            1.0 - (total_occupied_slots as f64 / (remaining_blocks * capacity_per_block) as f64)
        } else {
            0.0
        };

        CompactionPlan {
            migrations,
            blocks_freed,
            fragmentation_before: frag_before.max(0.0),
            fragmentation_after: frag_after.max(0.0),
        }
    }
}
