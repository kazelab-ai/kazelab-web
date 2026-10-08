//! High-throughput micro-benchmark for zero-copy SHM and ring buffer execution.

use std::time::Instant;
use synapse_core::ring::RingBuffer;
use synapse_core::shm::ShmRingBuffer;

fn main() {
    println!("=== KazeLab SynapseFlow High-Throughput Benchmarks ===");

    // Benchmark 1: Lock-Free Ring Buffer 1,000,000 Operations
    let mut ring = RingBuffer::new(1024);
    let start = Instant::now();
    let iters = 1_000_000;
    for i in 0..iters {
        let _ = ring.push(i);
        let _ = ring.pop();
    }
    let elapsed = start.elapsed();
    println!(
        "Ring Buffer: {} ops in {:?} ({:.2} Mops/sec)",
        iters,
        elapsed,
        (iters as f64) / elapsed.as_secs_f64() / 1_000_000.0
    );

    // Benchmark 2: Zero-Copy Shared Memory Page Throughput
    let mut shm = ShmRingBuffer::new();
    let payload = [0xAAu8; 1024];
    let shm_start = Instant::now();
    let shm_iters = 500_000;
    for _ in 0..shm_iters {
        let _ = shm.write_slice(&payload);
        shm.reset();
    }
    let shm_elapsed = shm_start.elapsed();
    let mb_transferred = (shm_iters as f64 * 1024.0) / (1024.0 * 1024.0);
    println!(
        "Zero-Copy SHM: {:.2} MB transferred in {:?} ({:.2} GB/sec throughput)",
        mb_transferred,
        shm_elapsed,
        (mb_transferred / 1024.0) / shm_elapsed.as_secs_f64()
    );
}
