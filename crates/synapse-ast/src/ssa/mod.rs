//! Static Single Assignment (SSA) Form Representation & Phi Node Insertion (Cytron et al. 1991).
//! Transforms mutable variable assignments into immutable versioned values for formal compiler reasoning.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsaVariable {
    pub name: String,
    pub version: usize,
}

impl SsaVariable {
    pub fn display_name(&self) -> String {
        format!("{}_{}", self.name, self.version)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsaPhiNode {
    pub target_var: SsaVariable,
    pub incoming_operands: Vec<SsaVariable>,
    pub basic_block_id: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsaInstruction {
    pub target: SsaVariable,
    pub opcode: String, // "ADD", "LOAD", "STORE", "CALL"
    pub operands: Vec<SsaVariable>,
}

pub struct SsaTransformer {
    version_counters: HashMap<String, usize>,
}

impl SsaTransformer {
    pub fn new() -> Self {
        Self {
            version_counters: HashMap::new(),
        }
    }

    pub fn assign_new_version(&mut self, base_name: &str) -> SsaVariable {
        let counter = self.version_counters.entry(base_name.to_string()).or_insert(0);
        *counter += 1;
        SsaVariable {
            name: base_name.to_string(),
            version: *counter,
        }
    }

    pub fn insert_phi_node(&mut self, var_name: &str, incoming: Vec<SsaVariable>, block_id: usize) -> SsaPhiNode {
        let target = self.assign_new_version(var_name);
        SsaPhiNode {
            target_var: target,
            incoming_operands: incoming,
            basic_block_id: block_id,
        }
    }
}
