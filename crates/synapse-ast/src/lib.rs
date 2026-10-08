//! Semantic AST Graph, Dependency DAG, and Symbol Mesh Extractor.

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolNode {
    pub symbol_id: String,
    pub name: String,
    pub kind: String, // "struct", "fn", "trait", "enum"
    pub file_path: String,
    pub line_span: (usize, usize),
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyDag {
    pub nodes: HashMap<String, SymbolNode>,
}

impl DependencyDag {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn insert_symbol(&mut self, symbol: SymbolNode) {
        self.nodes.insert(symbol.symbol_id.clone(), symbol);
    }

    pub fn find_dependents(&self, target_id: &str) -> Vec<String> {
        let mut dependents = Vec::new();
        for (id, node) in &self.nodes {
            if node.dependencies.iter().any(|dep| dep == target_id) {
                dependents.push(id.clone());
            }
        }
        dependents
    }

    pub fn topological_order(&self) -> Vec<String> {
        let mut order = Vec::new();
        let mut visited = HashSet::new();

        for id in self.nodes.keys() {
            Self::dfs(id, &self.nodes, &mut visited, &mut order);
        }

        order
    }

    fn dfs(
        current: &str,
        nodes: &HashMap<String, SymbolNode>,
        visited: &mut HashSet<String>,
        order: &mut Vec<String>,
    ) {
        if visited.contains(current) {
            return;
        }
        visited.insert(current.to_string());

        if let Some(node) = nodes.get(current) {
            for dep in &node.dependencies {
                Self::dfs(dep, nodes, visited, order);
            }
        }

        order.push(current.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dag_topological() {
        let mut dag = DependencyDag::new();
        dag.insert_symbol(SymbolNode {
            symbol_id: "core::engine".into(),
            name: "Engine".into(),
            kind: "struct".into(),
            file_path: "src/engine.rs".into(),
            line_span: (1, 50),
            dependencies: vec!["core::actor".into()],
        });
        dag.insert_symbol(SymbolNode {
            symbol_id: "core::actor".into(),
            name: "Actor".into(),
            kind: "trait".into(),
            file_path: "src/actor.rs".into(),
            line_span: (1, 20),
            dependencies: vec![],
        });

        let deps = dag.find_dependents("core::actor");
        assert_eq!(deps, vec!["core::engine"]);
    }
}
