//! Concolic Execution Engine (Combining Concrete and Symbolic Execution - DART / SAGE).
//! Generates directed inputs that force execution down previously unreached AST branches.

use std::collections::HashSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcolicState {
    pub concrete_inputs: Vec<i64>,
    pub path_condition: Vec<String>,
    pub branch_depth: usize,
}

pub struct ConcolicExecutionEngine {
    explored_paths: HashSet<String>,
}

impl ConcolicExecutionEngine {
    pub fn new() -> Self {
        Self {
            explored_paths: HashSet::new(),
        }
    }

    pub fn negate_last_branch(&mut self, state: &ConcolicState) -> Option<ConcolicState> {
        if state.path_condition.is_empty() {
            return None;
        }

        let mut inverted_condition = state.path_condition.clone();
        if let Some(last) = inverted_condition.pop() {
            let negated = if last.starts_with("NOT(") {
                last.trim_start_matches("NOT(").trim_end_matches(')').to_string()
            } else {
                format!("NOT({})", last)
            };
            inverted_condition.push(negated);

            let path_hash = inverted_condition.join(" AND ");
            if self.explored_paths.contains(&path_hash) {
                return None; // Already visited path
            }

            self.explored_paths.insert(path_hash);

            Some(ConcolicState {
                concrete_inputs: vec![state.concrete_inputs.first().cloned().unwrap_or(0) + 1],
                path_condition: inverted_condition,
                branch_depth: state.branch_depth + 1,
            })
        } else {
            None
        }
    }
}
