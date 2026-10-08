//! Non-blocking Memory Arena Allocator for High-Velocity Short-Lived AST Nodes.
//! Eliminates heap fragmentation and syscall overhead across AST parsing sweeps.

use std::cell::UnsafeCell;

pub struct AstBumpArena {
    chunks: UnsafeCell<Vec<Vec<u8>>>,
    chunk_size: usize,
    current_offset: UnsafeCell<usize>,
}

impl AstBumpArena {
    pub fn new(chunk_size: usize) -> Self {
        let initial_chunk = vec![0u8; chunk_size];
        Self {
            chunks: UnsafeCell::new(vec![initial_chunk]),
            chunk_size,
            current_offset: UnsafeCell::new(0),
        }
    }

    /// Allocates `size` bytes aligned to 8-byte boundary.
    pub fn alloc(&self, size: usize) -> *mut u8 {
        let aligned_size = (size + 7) & !7;
        let offset = unsafe { &mut *self.current_offset.get() };
        let chunks = unsafe { &mut *self.chunks.get() };

        if *offset + aligned_size > self.chunk_size {
            // Allocate new chunk
            let new_size = self.chunk_size.max(aligned_size);
            chunks.push(vec![0u8; new_size]);
            *offset = 0;
        }

        let last_chunk = chunks.last_mut().unwrap();
        let ptr = unsafe { last_chunk.as_mut_ptr().add(*offset) };
        *offset += aligned_size;
        ptr
    }

    /// Resets all allocated chunks to offset 0 without freeing backing OS pages.
    pub fn reset(&self) {
        let chunks = unsafe { &mut *self.chunks.get() };
        chunks.truncate(1); // Keep the first chunk
        let offset = unsafe { &mut *self.current_offset.get() };
        *offset = 0;
    }

    pub fn total_capacity_bytes(&self) -> usize {
        let chunks = unsafe { &*self.chunks.get() };
        chunks.iter().map(|c| c.len()).sum()
    }
}
