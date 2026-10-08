//! Asynchronous Thread Pool and Worker Stealing Task Queue.

pub mod backpressure;
pub mod circuit_breaker;

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;

pub struct WorkerStealingPool {
    active_workers: AtomicUsize,
    is_shutdown: AtomicBool,
    notify: Arc<Notify>,
}

impl WorkerStealingPool {
    pub fn new(workers: usize) -> Self {
        Self {
            active_workers: AtomicUsize::new(workers),
            is_shutdown: AtomicBool::new(false),
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn active_workers(&self) -> usize {
        self.active_workers.load(Ordering::Relaxed)
    }

    pub fn shutdown(&self) {
        self.is_shutdown.store(true, Ordering::Release);
        self.notify.notify_waiters();
    }

    pub fn is_active(&self) -> bool {
        !self.is_shutdown.load(Ordering::Acquire)
    }
}
