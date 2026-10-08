//! Code Property Graph (CPG) Engine (Yamaguchi et al., IEEE S&P).
//! Unifies Abstract Syntax Tree (AST), Control Flow Graph (CFG), and Program Dependence Graph (PDG).

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CpgEdgeType {
    AstChild,
    CfgFlow,
    DataDependence,
    ControlDependence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpgNode {
    pub node_id: usize,
    pub label: String,
    pub code_snippet: String,
    pub line_number: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpgEdge {
    pub from_node: usize,
    pub to_node: usize,
    pub edge_type: CpgEdgeType,
}

pub struct CodePropertyGraph {
    pub nodes: HashMap<usize, CpgNode>,
    pub edges: Vec<CpgEdge>,
}

impl CodePropertyGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: CpgNode) {
        self.nodes.insert(node.node_id, node);
    }

    pub fn add_edge(&mut self, from: usize, to: usize, edge_type: CpgEdgeType) {
        self.edges.push(CpgEdge {
            from_node: from,
            to_node: to,
            edge_type,
        });
    }

    pub fn get_data_dependencies(&self, target_node: usize) -> Vec<usize> {
        self.edges
            .iter()
            .filter(|e| e.to_node == target_node && e.edge_type == CpgEdgeType::DataDependence)
            .map(|e| e.from_node)
            .collect()
    }
}
