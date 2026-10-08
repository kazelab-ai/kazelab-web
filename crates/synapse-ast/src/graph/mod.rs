//! Bidirectional Call Graph & Module Dependency Mesh.

pub mod bridges;

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallGraphEdge {
    pub caller_id: String,
    pub callee_id: String,
    pub call_count: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CallGraph {
    pub forward_calls: HashMap<String, HashSet<String>>,
    pub reverse_calls: HashMap<String, HashSet<String>>,
}

impl CallGraph {
    pub fn new() -> Self {
        Self {
            forward_calls: HashMap::new(),
            reverse_calls: HashMap::new(),
        }
    }

    pub fn record_call(&mut self, caller: &str, callee: &str) {
        self.forward_calls
            .entry(caller.to_string())
            .or_default()
            .insert(callee.to_string());

        self.reverse_calls
            .entry(callee.to_string())
            .or_default()
            .insert(caller.to_string());
    }

    pub fn get_callees(&self, caller: &str) -> Vec<String> {
        self.forward_calls
            .get(caller)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn get_callers(&self, callee: &str) -> Vec<String> {
        self.reverse_calls
            .get(callee)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default()
    }
}
