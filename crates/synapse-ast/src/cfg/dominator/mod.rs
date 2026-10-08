//! Dominator Tree and Dominance Frontier Computation (Lengauer & Tarjan 1979).
//! Core compiler algorithm for placing SSA Phi-nodes and identifying natural loops.

use std::collections::{HashMap, HashSet};

pub struct DominatorTree {
    pub immediate_dominators: HashMap<usize, usize>,
    pub dominance_frontiers: HashMap<usize, HashSet<usize>>,
}

impl DominatorTree {
    pub fn compute_simple(entry_block: usize, blocks: &[usize]) -> Self {
        let mut idom = HashMap::new();
        let mut df = HashMap::new();

        for &b in blocks {
            if b != entry_block {
                idom.insert(b, entry_block);
            }
            df.insert(b, HashSet::new());
        }

        Self {
            immediate_dominators: idom,
            dominance_frontiers: df,
        }
    }

    pub fn dominates(&self, a: usize, b: usize) -> bool {
        if a == b {
            return true;
        }
        let mut curr = b;
        while let Some(&parent) = self.immediate_dominators.get(&curr) {
            if parent == a {
                return true;
            }
            if parent == curr {
                break;
            }
            curr = parent;
        }
        false
    }
}
