//! RAII Memory Invariant and Lifetime Verifier.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySafetyViolation {
    pub violation_type: String,
    pub description: String,
    pub severity: String,
    pub line_span: (usize, usize),
}

pub struct MemoryProver;

impl MemoryProver {
    pub fn verify_invariants(source_code: &str) -> Vec<MemorySafetyViolation> {
        let mut violations = Vec::new();

        for (idx, line) in source_code.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains("*mut ") || trimmed.contains("*const ") {
                violations.push(MemorySafetyViolation {
                    violation_type: "RawPointerDeclaration".into(),
                    description: "Raw pointer declared outside managed abstraction".into(),
                    severity: "HIGH".into(),
                    line_span: (idx + 1, idx + 1),
                });
            }
            if trimmed.contains("std::mem::forget") {
                violations.push(MemorySafetyViolation {
                    violation_type: "ResourceLeakPotential".into(),
                    description: "std::mem::forget explicitly called; may leak OS resource".into(),
                    severity: "CRITICAL".into(),
                    line_span: (idx + 1, idx + 1),
                });
            }
        }

        violations
    }
}
