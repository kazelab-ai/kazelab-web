pub mod dfa;
pub mod dominator;
pub mod loop_analyzer;
pub mod tarjan;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicBlock {
    pub block_id: usize,
    pub instructions: Vec<String>,
    pub successors: Vec<usize>,
    pub is_terminal: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ControlFlowGraph {
    pub function_name: String,
    pub blocks: HashMap<usize, BasicBlock>,
    pub entry_block_id: usize,
}

impl ControlFlowGraph {
    pub fn new(function_name: &str) -> Self {
        Self {
            function_name: function_name.to_string(),
            blocks: HashMap::new(),
            entry_block_id: 0,
        }
    }

    pub fn add_block(&mut self, block: BasicBlock) {
        self.blocks.insert(block.block_id, block);
    }

    pub fn compute_cyclomatic_complexity(&self) -> usize {
        let edges: usize = self.blocks.values().map(|b| b.successors.len()).sum();
        let nodes = self.blocks.len();
        if nodes == 0 {
            1
        } else if edges >= nodes {
            edges - nodes + 2
        } else {
            1
        }
    }
}
