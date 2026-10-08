//! Multi-Tier KV-Cache Memory Tiering (HBM -> DRAM -> Host SSD / NVMe Page Eviction).
//! Provides asynchronous page migration, cache locality tracking, and NVMe block spilling.

use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryTier {
    HighBandwidthMemory, // GPU HBM / SRAM
    HostDram,            // System DDR5 RAM
    NvmeDiskSpill,       // Local NVMe SSD Block
}

#[derive(Debug, Clone)]
pub struct TieredBlockMetadata {
    pub block_id: usize,
    pub current_tier: MemoryTier,
    pub access_counter: u64,
    pub last_access_epoch_ms: u64,
    pub dirty: bool,
}

pub struct TieredKvStorageManager {
    hbm_capacity_blocks: usize,
    dram_capacity_blocks: usize,
    tier_meta: RwLock<HashMap<usize, TieredBlockMetadata>>,
    dram_payloads: RwLock<HashMap<usize, Vec<u8>>>,
    disk_payloads: RwLock<HashMap<usize, Vec<u8>>>,
}

impl TieredKvStorageManager {
    pub fn new(hbm_capacity_blocks: usize, dram_capacity_blocks: usize) -> Self {
        Self {
            hbm_capacity_blocks,
            dram_capacity_blocks,
            tier_meta: RwLock::new(HashMap::new()),
            dram_payloads: RwLock::new(HashMap::new()),
            disk_payloads: RwLock::new(HashMap::new()),
        }
    }

    pub fn hbm_capacity(&self) -> usize {
        self.hbm_capacity_blocks
    }

    pub async fn register_block(&self, block_id: usize, tier: MemoryTier) {
        let mut meta = self.tier_meta.write().await;
        meta.insert(
            block_id,
            TieredBlockMetadata {
                block_id,
                current_tier: tier,
                access_counter: 1,
                last_access_epoch_ms: 0,
                dirty: false,
            },
        );
    }

    pub async fn touch_block(&self, block_id: usize) -> Option<MemoryTier> {
        let mut meta = self.tier_meta.write().await;
        if let Some(entry) = meta.get_mut(&block_id) {
            entry.access_counter += 1;
            Some(entry.current_tier)
        } else {
            None
        }
    }

    pub async fn demote_to_dram(&self, block_id: usize, payload: Vec<u8>) -> Result<(), &'static str> {
        let mut dram = self.dram_payloads.write().await;
        if dram.len() >= self.dram_capacity_blocks {
            return Err("DRAM tier exhausted");
        }
        dram.insert(block_id, payload);
        let mut meta = self.tier_meta.write().await;
        if let Some(entry) = meta.get_mut(&block_id) {
            entry.current_tier = MemoryTier::HostDram;
        }
        Ok(())
    }

    pub async fn demote_to_disk(&self, block_id: usize, payload: Vec<u8>) -> Result<(), &'static str> {
        let mut disk = self.disk_payloads.write().await;
        disk.insert(block_id, payload);
        let mut meta = self.tier_meta.write().await;
        if let Some(entry) = meta.get_mut(&block_id) {
            entry.current_tier = MemoryTier::NvmeDiskSpill;
        }
        Ok(())
    }

    pub async fn promote_to_hbm(&self, block_id: usize) -> Result<Option<Vec<u8>>, &'static str> {
        let mut meta = self.tier_meta.write().await;
        let entry = meta.get_mut(&block_id).ok_or("Block not found")?;

        match entry.current_tier {
            MemoryTier::HighBandwidthMemory => Ok(None),
            MemoryTier::HostDram => {
                let mut dram = self.dram_payloads.write().await;
                let data = dram.remove(&block_id);
                entry.current_tier = MemoryTier::HighBandwidthMemory;
                Ok(data)
            }
            MemoryTier::NvmeDiskSpill => {
                let mut disk = self.disk_payloads.write().await;
                let data = disk.remove(&block_id);
                entry.current_tier = MemoryTier::HighBandwidthMemory;
                Ok(data)
            }
        }
    }

    pub async fn tier_stats(&self) -> (usize, usize, usize) {
        let meta = self.tier_meta.read().await;
        let mut hbm = 0;
        let mut dram = 0;
        let mut disk = 0;
        for entry in meta.values() {
            match entry.current_tier {
                MemoryTier::HighBandwidthMemory => hbm += 1,
                MemoryTier::HostDram => dram += 1,
                MemoryTier::NvmeDiskSpill => disk += 1,
            }
        }
        (hbm, dram, disk)
    }
}
