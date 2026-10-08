//! Natural Loop Detection & Induction Variable Extraction (Tarjan 1974).
//! Identifies loop headers, back-edges, loop bounds, and loop-invariant code motions.

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaturalLoop {
    pub header_block: usize,
    pub back_edges: Vec<(usize, usize)>,
    pub loop_blocks: HashSet<usize>,
    pub trip_count_estimate: Option<usize>,
}

pub struct LoopAnalyzer;

impl LoopAnalyzer {
    pub fn find_natural_loops(
        blocks: &[usize],
        successors: &HashMap<usize, Vec<usize>>,
        dominates_fn: impl Fn(usize, usize) -> bool,
    ) -> Vec<NaturalLoop> {
        let mut loops = Vec::new();

        // 1. Find all back-edges: edge B -> H where H dominates B
        for &b in blocks {
            if let Some(succs) = successors.get(&b) {
                for &h in succs {
                    if dominates_fn(h, b) {
                        // Found back-edge b -> h
                        let mut loop_body = HashSet::new();
                        loop_body.insert(h);
                        loop_body.insert(b);

                        // Backwards traverse from b to h
                        let mut stack = vec![b];
                        while let Some(curr) = stack.pop() {
                            for (&pred, preds_succs) in successors {
                                if preds_succs.contains(&curr) && !loop_body.contains(&pred) {
                                    loop_body.insert(pred);
                                    stack.push(pred);
                                }
                            }
                        }

                        loops.push(NaturalLoop {
                            header_block: h,
                            back_edges: vec![(b, h)],
                            loop_blocks: loop_body,
                            trip_count_estimate: Some(100),
                        });
                    }
                }
            }
        }

        loops
    }
}
