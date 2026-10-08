//! Differential Fuzzing and Mutation-Based Test Case Synthesis Harness.
//! Generates boundary inputs and compares execution outputs against reference models to uncover edge-case defects.

use std::collections::HashSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzMutationSeed {
    pub raw_bytes: Vec<u8>,
    pub execution_energy: u32,
    pub coverage_hash: u64,
}

pub struct DifferentialFuzzHarness {
    corpus: Vec<FuzzMutationSeed>,
    discovered_coverage: HashSet<u64>,
    total_mutations: usize,
    crashes_found: usize,
}

impl DifferentialFuzzHarness {
    pub fn new() -> Self {
        Self {
            corpus: Vec::new(),
            discovered_coverage: HashSet::new(),
            total_mutations: 0,
            crashes_found: 0,
        }
    }

    pub fn seed(&mut self, initial_payload: &[u8]) {
        let h = Self::hash_coverage(initial_payload);
        self.discovered_coverage.insert(h);
        self.corpus.push(FuzzMutationSeed {
            raw_bytes: initial_payload.to_vec(),
            execution_energy: 100,
            coverage_hash: h,
        });
    }

    /// Mutates input buffer using bit-flipping, byte-shuffling, and boundary integer injection.
    pub fn mutate(&mut self, input: &[u8]) -> Vec<u8> {
        self.total_mutations += 1;
        let mut mutated = input.to_vec();
        if mutated.is_empty() {
            return vec![0x41];
        }

        let idx = self.total_mutations % mutated.len();
        match self.total_mutations % 4 {
            0 => {
                // Bit flip
                mutated[idx] ^= 1 << (self.total_mutations % 8);
            }
            1 => {
                // Byte replacement with interesting boundary value (0, 255, 127)
                let interesting_bytes = [0x00, 0xFF, 0x7F, 0x80, 0x0A, 0x0D];
                mutated[idx] = interesting_bytes[self.total_mutations % interesting_bytes.len()];
            }
            2 => {
                // Insertion
                mutated.insert(idx, 0x20);
            }
            _ => {
                // Deletion
                if mutated.len() > 1 {
                    mutated.remove(idx);
                }
            }
        }
        mutated
    }

    /// Compares two implementations (primary vs oracle/reference) on identical input.
    pub fn evaluate_differential<F1, F2>(
        &mut self,
        input: &[u8],
        mut target_fn: F1,
        mut oracle_fn: F2,
    ) -> Result<bool, String>
    where
        F1: FnMut(&[u8]) -> Result<Vec<u8>, String>,
        F2: FnMut(&[u8]) -> Result<Vec<u8>, String>,
    {
        let target_res = target_fn(input);
        let oracle_res = oracle_fn(input);

        match (target_res, oracle_res) {
            (Ok(out1), Ok(out2)) => {
                if out1 != out2 {
                    self.crashes_found += 1;
                    return Err(format!(
                        "Differential divergence detected! Target output: {:?}, Oracle output: {:?}",
                        out1, out2
                    ));
                }
                let cov = Self::hash_coverage(&out1);
                if self.discovered_coverage.insert(cov) {
                    self.corpus.push(FuzzMutationSeed {
                        raw_bytes: input.to_vec(),
                        execution_energy: 50,
                        coverage_hash: cov,
                    });
                }
                Ok(true)
            }
            (Err(e1), Err(e2)) if e1 == e2 => Ok(true),
            (r1, r2) => {
                self.crashes_found += 1;
                Err(format!(
                    "Discrepancy in execution status: target={:?}, oracle={:?}",
                    r1, r2
                ))
            }
        }
    }

    fn hash_coverage(data: &[u8]) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        for &b in data {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    }

    pub fn stats(&self) -> (usize, usize, usize) {
        (self.corpus.len(), self.total_mutations, self.crashes_found)
    }
}
