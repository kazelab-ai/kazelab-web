//! Topological Tarjan Condensation and Strongly Connected Component DAG Shrinker.
//! Reduces general cyclic dependency graphs into strict Directed Acyclic Graphs (DAGs) for compilation ordering.

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct CondensedDag {
    pub scc_components: Vec<Vec<usize>>,
    pub scc_adjacency: HashMap<usize, HashSet<usize>>, // scc_id -> target_scc_ids
}

impl CondensedDag {
    /// Builds a condensed DAG where each node is an SCC component.
    pub fn build(original_nodes: &HashSet<usize>, sccs: Vec<Vec<usize>>, original_edges: &HashMap<usize, Vec<usize>>) -> Self {
        let mut node_to_scc: HashMap<usize, usize> = HashMap::new();
        for (scc_id, component) in sccs.iter().enumerate() {
            for &node in component {
                node_to_scc.insert(node, scc_id);
            }
        }

        let mut scc_adjacency: HashMap<usize, HashSet<usize>> = HashMap::new();
        for &u in original_nodes {
            if let Some(&u_scc) = node_to_scc.get(&u) {
                if let Some(neighbors) = original_edges.get(&u) {
                    for &v in neighbors {
                        if let Some(&v_scc) = node_to_scc.get(&v) {
                            if u_scc != v_scc {
                                scc_adjacency.entry(u_scc).or_default().insert(v_scc);
                            }
                        }
                    }
                }
            }
        }

        Self {
            scc_components: sccs,
            scc_adjacency,
        }
    }

    /// Computes reverse topological ordering over the condensed DAG.
    pub fn topological_sort(&self) -> Vec<usize> {
        let mut in_degree: HashMap<usize, usize> = HashMap::new();
        let total_sccs = self.scc_components.len();

        for i in 0..total_sccs {
            in_degree.insert(i, 0);
        }

        for targets in self.scc_adjacency.values() {
            for &t in targets {
                *in_degree.entry(t).or_insert(0) += 1;
            }
        }

        let mut queue: Vec<usize> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut order = Vec::new();

        while let Some(curr) = queue.pop() {
            order.push(curr);
            if let Some(neighbors) = self.scc_adjacency.get(&curr) {
                for &next in neighbors {
                    if let Some(deg) = in_degree.get_mut(&next) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            queue.push(next);
                        }
                    }
                }
            }
        }

        order
    }
}
