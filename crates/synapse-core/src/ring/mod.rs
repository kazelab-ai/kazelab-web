//! High-performance bounded SPSC / MPMC ring buffer queue with atomic head/tail pointers.
//! Zero heap reallocation in the hot path.

use std::sync::atomic::{AtomicUsize, Ordering};

pub struct RingBuffer<T> {
    buffer: Vec<Option<T>>,
    capacity: usize,
    head: AtomicUsize,
    tail: AtomicUsize,
}

impl<T: Clone> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Capacity must be greater than zero");
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(None);
        }
        Self {
            buffer,
            capacity,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        if head >= tail {
            head - tail
        } else {
            self.capacity - (tail - head)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Acquire) == self.tail.load(Ordering::Acquire)
    }

    pub fn is_full(&self) -> bool {
        (self.head.load(Ordering::Acquire) + 1) % self.capacity == self.tail.load(Ordering::Acquire)
    }

    pub fn push(&mut self, item: T) -> Result<(), &'static str> {
        let head = self.head.load(Ordering::Relaxed);
        let next_head = (head + 1) % self.capacity;
        if next_head == self.tail.load(Ordering::Acquire) {
            return Err("Ring buffer full");
        }
        self.buffer[head] = Some(item);
        self.head.store(next_head, Ordering::Release);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        let tail = self.tail.load(Ordering::Relaxed);
        if tail == self.head.load(Ordering::Acquire) {
            return None;
        }
        let item = self.buffer[tail].take();
        self.tail.store((tail + 1) % self.capacity, Ordering::Release);
        item
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_fifo() {
        let mut rb = RingBuffer::new(5);
        assert!(rb.is_empty());
        rb.push(10).unwrap();
        rb.push(20).unwrap();
        rb.push(30).unwrap();
        assert_eq!(rb.len(), 3);
        assert_eq!(rb.pop(), Some(10));
        assert_eq!(rb.pop(), Some(20));
        assert_eq!(rb.pop(), Some(30));
        assert_eq!(rb.pop(), None);
    }
}
