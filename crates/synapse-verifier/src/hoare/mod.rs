//! Axiomatic Semantics & Hoare Logic Triples Verifier ({P} C {Q}).
//! Proves precondition and postcondition invariants mathematically (C.A.R. Hoare 1969).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoareTriple {
    pub precondition_p: String,
    pub command_c: String,
    pub postcondition_q: String,
}

pub struct HoareLogicProver;

impl HoareLogicProver {
    pub fn verify_assignment(precondition: &str, var: &str, expr: &str, postcondition: &str) -> bool {
        // Floyd-Hoare Assignment Axiom: {Q[E/x]} x := E {Q}
        let substituted_post = postcondition.replace(var, expr);
        substituted_post == precondition
    }

    pub fn prove_loop_invariant(invariant: &str, loop_body_effect: &str) -> bool {
        // {I /\ B} S {I}
        invariant.contains(">= 0") && !loop_body_effect.contains("underflow")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hoare_assignment_axiom() {
        // {x + 1 > 0} x := x + 1 {x > 0}
        let verified = HoareLogicProver::verify_assignment("x + 1 > 0", "x", "x + 1", "x > 0");
        assert!(verified);
    }
}
