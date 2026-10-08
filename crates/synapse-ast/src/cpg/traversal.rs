//! Gremlin-style Declarative Graph Traversal Pipeline for Code Property Graph Queries.
//! Enables complex interprocedural taint, slice, and dominator queries across AST, CFG, and PDG edges.

use crate::cpg::{CodePropertyGraph, CpgEdgeType, CpgNode};
use std::collections::HashSet;

#[derive(Clone)]
pub struct CpgTraversal<'a> {
    graph: &'a CodePropertyGraph,
    current_nodes: Vec<usize>,
}

impl<'a> CpgTraversal<'a> {
    pub fn new(graph: &'a CodePropertyGraph, start_nodes: Vec<usize>) -> Self {
        Self {
            graph,
            current_nodes: start_nodes,
        }
    }

    pub fn from_all(graph: &'a CodePropertyGraph) -> Self {
        let all_ids: Vec<usize> = graph.nodes.keys().cloned().collect();
        Self {
            graph,
            current_nodes: all_ids,
        }
    }

    pub fn has_label(mut self, label: &str) -> Self {
        self.current_nodes.retain(|&id| {
            if let Some(node) = self.graph.nodes.get(&id) {
                node.label == label
            } else {
                false
            }
        });
        self
    }

    pub fn code_contains(mut self, substring: &str) -> Self {
        self.current_nodes.retain(|&id| {
            if let Some(node) = self.graph.nodes.get(&id) {
                node.code_snippet.contains(substring)
            } else {
                false
            }
        });
        self
    }

    pub fn out_step(self, edge_type: CpgEdgeType) -> Self {
        let mut next_nodes = Vec::new();
        let mut visited = HashSet::new();

        for current_id in &self.current_nodes {
            for edge in &self.graph.edges {
                if edge.from_node == *current_id && edge.edge_type == edge_type {
                    if visited.insert(edge.to_node) {
                        next_nodes.push(edge.to_node);
                    }
                }
            }
        }

        Self {
            graph: self.graph,
            current_nodes: next_nodes,
        }
    }

    pub fn in_step(self, edge_type: CpgEdgeType) -> Self {
        let mut prev_nodes = Vec::new();
        let mut visited = HashSet::new();

        for current_id in &self.current_nodes {
            for edge in &self.graph.edges {
                if edge.to_node == *current_id && edge.edge_type == edge_type {
                    if visited.insert(edge.from_node) {
                        prev_nodes.push(edge.from_node);
                    }
                }
            }
        }

        Self {
            graph: self.graph,
            current_nodes: prev_nodes,
        }
    }

    pub fn out_ast(self) -> Self {
        self.out_step(CpgEdgeType::AstChild)
    }

    pub fn in_ast(self) -> Self {
        self.in_step(CpgEdgeType::AstChild)
    }

    pub fn cfg_succ(self) -> Self {
        self.out_step(CpgEdgeType::CfgFlow)
    }

    pub fn cfg_pred(self) -> Self {
        self.in_step(CpgEdgeType::CfgFlow)
    }

    pub fn ddg_uses(self) -> Self {
        self.out_step(CpgEdgeType::DataDependence)
    }

    pub fn ddg_defs(self) -> Self {
        self.in_step(CpgEdgeType::DataDependence)
    }

    pub fn reachable_forward(self, edge_type: CpgEdgeType, max_depth: usize) -> Self {
        let mut frontier = self.current_nodes.clone();
        let mut all_reachable = HashSet::new();

        for &n in &frontier {
            all_reachable.insert(n);
        }

        for _ in 0..max_depth {
            let mut next_frontier = Vec::new();
            for &curr in &frontier {
                for edge in &self.graph.edges {
                    if edge.from_node == curr && edge.edge_type == edge_type {
                        if all_reachable.insert(edge.to_node) {
                            next_frontier.push(edge.to_node);
                        }
                    }
                }
            }
            if next_frontier.is_empty() {
                break;
            }
            frontier = next_frontier;
        }

        Self {
            graph: self.graph,
            current_nodes: all_reachable.into_iter().collect(),
        }
    }

    pub fn reachable_backward(self, edge_type: CpgEdgeType, max_depth: usize) -> Self {
        let mut frontier = self.current_nodes.clone();
        let mut all_reachable = HashSet::new();

        for &n in &frontier {
            all_reachable.insert(n);
        }

        for _ in 0..max_depth {
            let mut next_frontier = Vec::new();
            for &curr in &frontier {
                for edge in &self.graph.edges {
                    if edge.to_node == curr && edge.edge_type == edge_type {
                        if all_reachable.insert(edge.from_node) {
                            next_frontier.push(edge.from_node);
                        }
                    }
                }
            }
            if next_frontier.is_empty() {
                break;
            }
            frontier = next_frontier;
        }

        Self {
            graph: self.graph,
            current_nodes: all_reachable.into_iter().collect(),
        }
    }

    pub fn to_ids(self) -> Vec<usize> {
        self.current_nodes
    }

    pub fn to_nodes(self) -> Vec<CpgNode> {
        self.current_nodes
            .into_iter()
            .filter_map(|id| self.graph.nodes.get(&id).cloned())
            .collect()
    }
}
