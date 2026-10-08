//! Static Analysis and Race Condition Detector for Lock Ordering Hierarchies.
//! Validates lock acquisition sequences to mathematically prevent ABBA distributed deadlocks.

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockAcquisitionStep {
    pub thread_id: usize,
    pub lock_id: usize,
    pub step_index: usize,
}

#[derive(Debug, Clone)]
pub struct LockOrderGraph {
    pub edges: HashMap<usize, HashSet<usize>>, // lock_A -> set of locks acquired while holding lock_A
}

impl LockOrderGraph {
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }

    pub fn record_acquisition(&mut self, held_locks: &[usize], next_lock: usize) {
        for &held in held_locks {
            if held != next_lock {
                self.edges.entry(held).or_default().insert(next_lock);
            }
        }
    }

    /// Detects directed cycles in the lock order graph using Tarjan / DFS cycle detection.
    pub fn detect_deadlock_cycles(&self) -> Vec<Vec<usize>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = Vec::new();

        for &node in self.edges.keys() {
            if !visited.contains(&node) {
                self.dfs_cycle(node, &mut visited, &mut rec_stack, &mut cycles);
            }
        }

        cycles
    }

    fn dfs_cycle(
        &self,
        node: usize,
        visited: &mut HashSet<usize>,
        rec_stack: &mut Vec<usize>,
        cycles: &mut Vec<Vec<usize>>,
    ) {
        visited.insert(node);
        rec_stack.push(node);

        if let Some(neighbors) = self.edges.get(&node) {
            for &next in neighbors {
                if let Some(pos) = rec_stack.iter().position(|&x| x == next) {
                    // Cycle detected!
                    let cycle = rec_stack[pos..].to_vec();
                    cycles.push(cycle);
                } else if !visited.contains(&next) {
                    self.dfs_cycle(next, visited, rec_stack, cycles);
                }
            }
        }

        rec_stack.pop();
    }
}
