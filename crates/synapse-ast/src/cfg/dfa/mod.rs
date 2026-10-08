//! Dataflow Analysis (DFA) Framework: Reaching Definitions and Live Variable Analysis.

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default)]
pub struct LivenessAnalysisResult {
    pub in_sets: HashMap<usize, HashSet<String>>,
    pub out_sets: HashMap<usize, HashSet<String>>,
}

pub struct DataflowEngine;

impl DataflowEngine {
    pub fn compute_live_variables(
        blocks: &[usize],
        uses: &HashMap<usize, HashSet<String>>,
        defs: &HashMap<usize, HashSet<String>>,
    ) -> LivenessAnalysisResult {
        let mut in_sets = HashMap::new();
        let mut out_sets = HashMap::new();

        for &block in blocks {
            in_sets.insert(block, HashSet::new());
            out_sets.insert(block, HashSet::new());
        }

        // Fixed-point backward iteration
        let mut changed = true;
        let mut iterations = 0;
        while changed && iterations < 50 {
            changed = false;
            iterations += 1;

            for &b in blocks.iter().rev() {
                let use_b = uses.get(&b).cloned().unwrap_or_default();
                let def_b = defs.get(&b).cloned().unwrap_or_default();
                let out_b: HashSet<String> = out_sets.get(&b).cloned().unwrap_or_default();

                // IN[B] = USE[B] U (OUT[B] - DEF[B])
                let mut new_in = use_b;
                for var in &out_b {
                    if !def_b.contains(var) {
                        new_in.insert(var.clone());
                    }
                }

                if in_sets.get(&b) != Some(&new_in) {
                    in_sets.insert(b, new_in);
                    changed = true;
                }
            }
        }

        LivenessAnalysisResult { in_sets, out_sets }
    }
}
