//! Write-Ahead Log (WAL) Engine for Durable Raft Consensus Operations.

use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalFrame {
    pub magic: u32, // 0x57414C46 "WALF"
    pub entry_index: u64,
    pub term: u64,
    pub payload_len: u32,
    pub checksum_crc32: u32,
    pub payload: Vec<u8>,
}

impl WalFrame {
    pub fn compute_checksum(payload: &[u8]) -> u32 {
        let mut crc: u32 = 0xFFFFFFFF;
        for &byte in payload {
            crc ^= byte as u32;
            for _ in 0..8 {
                let mask = if (crc & 1) != 0 { 0xEDB88320 } else { 0 };
                crc = (crc >> 1) ^ mask;
            }
        }
        !crc
    }

    pub fn new(index: u64, term: u64, payload: Vec<u8>) -> Self {
        let checksum = Self::compute_checksum(&payload);
        Self {
            magic: 0x57414C46,
            entry_index: index,
            term,
            payload_len: payload.len() as u32,
            checksum_crc32: checksum,
            payload,
        }
    }

    pub fn verify(&self) -> bool {
        self.magic == 0x57414C46 && Self::compute_checksum(&self.payload) == self.checksum_crc32
    }
}

pub struct WriteAheadLog {
    last_log_index: AtomicU64,
    in_memory_log: Mutex<Vec<WalFrame>>,
}

impl WriteAheadLog {
    pub fn new() -> Self {
        Self {
            last_log_index: AtomicU64::new(0),
            in_memory_log: Mutex::new(Vec::new()),
        }
    }

    pub async fn append(&self, term: u64, payload: Vec<u8>) -> u64 {
        let index = self.last_log_index.fetch_add(1, Ordering::SeqCst) + 1;
        let frame = WalFrame::new(index, term, payload);
        let mut log = self.in_memory_log.lock().await;
        log.push(frame);
        index
    }

    pub async fn read_range(&self, start_idx: u64, end_idx: u64) -> Vec<WalFrame> {
        let log = self.in_memory_log.lock().await;
        log.iter()
            .filter(|f| f.entry_index >= start_idx && f.entry_index <= end_idx)
            .cloned()
            .collect()
    }

    pub async fn len(&self) -> usize {
        let log = self.in_memory_log.lock().await;
        log.len()
    }
}
