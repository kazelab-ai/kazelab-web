//! Hardware-Assisted Hardware Performance Counters (HPC) Profiler.
//! Measures CPU branch mispredictions, cache misses (L1/LLC), and instruction retirement cycles.

use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HpcHardwareProfile {
    pub total_cpu_cycles: u64,
    pub instructions_retired: u64,
    pub instructions_per_cycle: f64,
    pub branch_instructions: u64,
    pub branch_mispredictions: u64,
    pub branch_miss_rate: f64,
    pub l1_data_cache_misses: u64,
    pub llc_cache_misses: u64,
}

pub struct HardwareProfileMonitor {
    cycles: AtomicU64,
    instructions: AtomicU64,
    branches: AtomicU64,
    branch_misses: AtomicU64,
    l1_misses: AtomicU64,
    llc_misses: AtomicU64,
}

impl HardwareProfileMonitor {
    pub fn new() -> Self {
        Self {
            cycles: AtomicU64::new(0),
            instructions: AtomicU64::new(0),
            branches: AtomicU64::new(0),
            branch_misses: AtomicU64::new(0),
            l1_misses: AtomicU64::new(0),
            llc_misses: AtomicU64::new(0),
        }
    }

    pub fn record_sample(
        &self,
        cycles: u64,
        instructions: u64,
        branches: u64,
        branch_misses: u64,
        l1_misses: u64,
        llc_misses: u64,
    ) {
        self.cycles.fetch_add(cycles, Ordering::Relaxed);
        self.instructions.fetch_add(instructions, Ordering::Relaxed);
        self.branches.fetch_add(branches, Ordering::Relaxed);
        self.branch_misses.fetch_add(branch_misses, Ordering::Relaxed);
        self.l1_misses.fetch_add(l1_misses, Ordering::Relaxed);
        self.llc_misses.fetch_add(llc_misses, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> HpcHardwareProfile {
        let c = self.cycles.load(Ordering::Relaxed);
        let i = self.instructions.load(Ordering::Relaxed);
        let b = self.branches.load(Ordering::Relaxed);
        let bm = self.branch_misses.load(Ordering::Relaxed);
        let l1 = self.l1_misses.load(Ordering::Relaxed);
        let llc = self.llc_misses.load(Ordering::Relaxed);

        let ipc = if c > 0 { (i as f64) / (c as f64) } else { 0.0 };
        let b_rate = if b > 0 { (bm as f64) / (b as f64) * 100.0 } else { 0.0 };

        HpcHardwareProfile {
            total_cpu_cycles: c,
            instructions_retired: i,
            instructions_per_cycle: ipc,
            branch_instructions: b,
            branch_mispredictions: bm,
            branch_miss_rate: b_rate,
            l1_data_cache_misses: l1,
            llc_cache_misses: llc,
        }
    }
}
