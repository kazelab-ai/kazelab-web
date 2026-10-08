//! Weakest Precondition (WP) Calculus (Edsger W. Dijkstra 1975).
//! Computes exact precondition formulas needed to guarantee postconditions without loop unrolling.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WpFormula {
    pub raw_predicate: String,
}

impl WpFormula {
    pub fn new(pred: &str) -> Self {
        Self {
            raw_predicate: pred.to_string(),
        }
    }

    pub fn and(&self, other: &Self) -> Self {
        Self {
            raw_predicate: format!("({} AND {})", self.raw_predicate, other.raw_predicate),
        }
    }

    pub fn implies(&self, other: &Self) -> Self {
        Self {
            raw_predicate: format!("({} => {})", self.raw_predicate, other.raw_predicate),
        }
    }
}

pub struct WeakestPreconditionEngine;

impl WeakestPreconditionEngine {
    pub fn wp_assignment(var: &str, expr: &str, postcondition: &WpFormula) -> WpFormula {
        // wp(x := E, Q) = Q[E/x]
        let substituted = postcondition.raw_predicate.replace(var, expr);
        WpFormula::new(&substituted)
    }

    pub fn wp_if(cond: &str, wp_then: &WpFormula, wp_else: &WpFormula) -> WpFormula {
        // wp(if B then S1 else S2, Q) = (B => wp(S1, Q)) /\ (~B => wp(S2, Q))
        let not_cond = format!("NOT({})", cond);
        let branch_then = WpFormula::new(cond).implies(wp_then);
        let branch_else = WpFormula::new(&not_cond).implies(wp_else);
        branch_then.and(&branch_else)
    }
}
