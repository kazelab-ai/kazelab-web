pub mod eviction;
pub mod tiering;
pub mod defrag;

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::RwLock;

pub const PAGE_BLOCK_SIZE: usize = 16; // 16 tokens per physical block

#[derive(Debug, Clone)]
pub struct PhysicalBlock {
    pub block_id: usize,
    pub token_ids: [u32; PAGE_BLOCK_SIZE],
    pub ref_count: usize,
}

pub struct PagedAttentionBlockManager {
    free_blocks: RwLock<Vec<usize>>,
    block_table: RwLock<HashMap<usize, PhysicalBlock>>,
    total_capacity: usize,
    allocated_blocks_count: AtomicUsize,
}

impl PagedAttentionBlockManager {
    pub fn new(total_blocks: usize) -> Self {
        let mut free = Vec::with_capacity(total_blocks);
        for id in (0..total_blocks).rev() {
            free.push(id);
        }

        Self {
            free_blocks: RwLock::new(free),
            block_table: RwLock::new(HashMap::new()),
            total_capacity: total_blocks,
            allocated_blocks_count: AtomicUsize::new(0),
        }
    }

    pub async fn allocate_block(&self, tokens: &[u32]) -> Result<usize, &'static str> {
        let mut free = self.free_blocks.write().await;
        if let Some(block_id) = free.pop() {
            let mut block_data = [0u32; PAGE_BLOCK_SIZE];
            let copy_len = tokens.len().min(PAGE_BLOCK_SIZE);
            block_data[..copy_len].copy_from_slice(&tokens[..copy_len]);

            let physical_block = PhysicalBlock {
                block_id,
                token_ids: block_data,
                ref_count: 1,
            };

            let mut table = self.block_table.write().await;
            table.insert(block_id, physical_block);
            self.allocated_blocks_count.fetch_add(1, Ordering::Relaxed);
            Ok(block_id)
        } else {
            Err("Out of physical KV cache blocks (Out of memory)")
        }
    }

    pub async fn share_block_prefix(&self, block_id: usize) -> Result<(), &'static str> {
        let mut table = self.block_table.write().await;
        if let Some(block) = table.get_mut(&block_id) {
            block.ref_count += 1;
            Ok(())
        } else {
            Err("Block ID not found in page table")
        }
    }

    pub async fn free_block(&self, block_id: usize) {
        let mut table = self.block_table.write().await;
        if let Some(block) = table.get_mut(&block_id) {
            block.ref_count = block.ref_count.saturating_sub(1);
            if block.ref_count == 0 {
                table.remove(&block_id);
                let mut free = self.free_blocks.write().await;
                free.push(block_id);
                self.allocated_blocks_count.fetch_sub(1, Ordering::Relaxed);
            }
        }
    }

    pub fn usage_percentage(&self) -> f64 {
        let allocated = self.allocated_blocks_count.load(Ordering::Relaxed);
        (allocated as f64) / (self.total_capacity as f64) * 100.0
    }
}
