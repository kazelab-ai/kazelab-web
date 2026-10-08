//! Symbolic Execution and Path Constraint Evaluation.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolicPathConstraint {
    pub variable: String,
    pub operator: String,
    pub bound_value: i64,
}

pub struct SymbolicEvaluator;

impl SymbolicEvaluator {
    pub fn is_path_feasible(constraints: &[SymbolicPathConstraint]) -> bool {
        let mut min_val = i64::MIN;
        let mut max_val = i64::MAX;

        for c in constraints {
            if c.operator == ">=" && c.bound_value > min_val {
                min_val = c.bound_value;
            } else if c.operator == "<=" && c.bound_value < max_val {
                max_val = c.bound_value;
            }
        }

        min_val <= max_val
    }
}
