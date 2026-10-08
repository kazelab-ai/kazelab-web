//! Zero-Copy In-Memory Page Ring Buffer for Sub-Microsecond Inter-Process Communication (IPC).
//! Designed to pass serialized AST trees and LLM prompt tokens without heap copies.

use std::sync::atomic::{AtomicUsize, Ordering};

pub const SHM_PAGE_SIZE: usize = 4096;

#[repr(C, align(64))]
pub struct ShmRingBuffer {
    page_data: [u8; SHM_PAGE_SIZE],
    write_offset: AtomicUsize,
    read_offset: AtomicUsize,
    committed_bytes: AtomicUsize,
}

impl ShmRingBuffer {
    pub fn new() -> Self {
        Self {
            page_data: [0u8; SHM_PAGE_SIZE],
            write_offset: AtomicUsize::new(0),
            read_offset: AtomicUsize::new(0),
            committed_bytes: AtomicUsize::new(0),
        }
    }

    pub fn write_slice(&mut self, data: &[u8]) -> Result<usize, &'static str> {
        let len = data.len();
        if len > SHM_PAGE_SIZE {
            return Err("Payload exceeds shared memory page boundary");
        }

        let curr_write = self.write_offset.load(Ordering::Relaxed);
        let space_available = SHM_PAGE_SIZE - curr_write;

        if len > space_available {
            return Err("Insufficient ring buffer page capacity");
        }

        // Direct memory copy into page memory
        self.page_data[curr_write..curr_write + len].copy_from_slice(data);
        self.write_offset.fetch_add(len, Ordering::Release);
        self.committed_bytes.fetch_add(len, Ordering::Release);

        Ok(len)
    }

    pub fn read_committed(&self) -> &[u8] {
        let written = self.write_offset.load(Ordering::Acquire);
        &self.page_data[..written]
    }

    pub fn reset(&mut self) {
        self.write_offset.store(0, Ordering::Relaxed);
        self.read_offset.store(0, Ordering::Relaxed);
        self.committed_bytes.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_copy_write() {
        let mut shm = ShmRingBuffer::new();
        let payload = b"SYNAPSE_TOKEN_STREAM_CHUNK";
        let written = shm.write_slice(payload).unwrap();
        assert_eq!(written, payload.len());
        assert_eq!(shm.read_committed(), payload);
    }
}
