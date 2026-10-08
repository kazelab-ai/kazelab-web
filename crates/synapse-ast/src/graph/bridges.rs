//! Tarjan Linear-Time Bridge and Articulation Point Connectivity Analysis in Call Graphs.
//! Identifies critical single-point-of-failure functions and bridges in inter-procedural architecture.

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CallEdge {
    pub caller_id: usize,
    pub callee_id: usize,
}

pub struct CallGraphBridgeAnalyzer {
    adj: HashMap<usize, Vec<usize>>,
    all_nodes: HashSet<usize>,
}

impl CallGraphBridgeAnalyzer {
    pub fn new() -> Self {
        Self {
            adj: HashMap::new(),
            all_nodes: HashSet::new(),
        }
    }

    pub fn add_call(&mut self, caller: usize, callee: usize) {
        self.all_nodes.insert(caller);
        self.all_nodes.insert(callee);
        self.adj.entry(caller).or_default().push(callee);
        self.adj.entry(callee).or_default().push(caller); // Undirected view for biconnectivity
    }

    /// Finds all bridge edges whose removal disconnects the call graph.
    pub fn find_critical_bridges(&self) -> Vec<CallEdge> {
        let mut bridges = Vec::new();
        let mut visited = HashSet::new();
        let mut tin = HashMap::new();
        let mut low = HashMap::new();
        let mut timer = 0;

        for &node in &self.all_nodes {
            if !visited.contains(&node) {
                self.dfs_bridge(
                    node,
                    None,
                    &mut timer,
                    &mut visited,
                    &mut tin,
                    &mut low,
                    &mut bridges,
                );
            }
        }

        bridges
    }

    fn dfs_bridge(
        &self,
        v: usize,
        p: Option<usize>,
        timer: &mut usize,
        visited: &mut HashSet<usize>,
        tin: &mut HashMap<usize, usize>,
        low: &mut HashMap<usize, usize>,
        bridges: &mut Vec<CallEdge>,
    ) {
        visited.insert(v);
        tin.insert(v, *timer);
        low.insert(v, *timer);
        *timer += 1;

        if let Some(neighbors) = self.adj.get(&v) {
            for &to in neighbors {
                if Some(to) == p {
                    continue;
                }
                if visited.contains(&to) {
                    let v_low = low.get_mut(&v).unwrap();
                    let to_tin = tin[&to];
                    *v_low = (*v_low).min(to_tin);
                } else {
                    self.dfs_bridge(to, Some(v), timer, visited, tin, low, bridges);
                    let to_low = low[&to];
                    let v_low = low.get_mut(&v).unwrap();
                    *v_low = (*v_low).min(to_low);
                    if to_low > tin[&v] {
                        bridges.push(CallEdge {
                            caller_id: v,
                            callee_id: to,
                        });
                    }
                }
            }
        }
    }

    /// Finds all articulation points (cut vertices) whose removal increases connected components.
    pub fn find_articulation_points(&self) -> HashSet<usize> {
        let mut cut_vertices = HashSet::new();
        let mut visited = HashSet::new();
        let mut tin = HashMap::new();
        let mut low = HashMap::new();
        let mut timer = 0;

        for &node in &self.all_nodes {
            if !visited.contains(&node) {
                self.dfs_cut(
                    node,
                    None,
                    &mut timer,
                    &mut visited,
                    &mut tin,
                    &mut low,
                    &mut cut_vertices,
                );
            }
        }

        cut_vertices
    }

    fn dfs_cut(
        &self,
        v: usize,
        p: Option<usize>,
        timer: &mut usize,
        visited: &mut HashSet<usize>,
        tin: &mut HashMap<usize, usize>,
        low: &mut HashMap<usize, usize>,
        cut_vertices: &mut HashSet<usize>,
    ) {
        visited.insert(v);
        tin.insert(v, *timer);
        low.insert(v, *timer);
        *timer += 1;
        let mut children = 0;

        if let Some(neighbors) = self.adj.get(&v) {
            for &to in neighbors {
                if Some(to) == p {
                    continue;
                }
                if visited.contains(&to) {
                    let v_low = low.get_mut(&v).unwrap();
                    let to_tin = tin[&to];
                    *v_low = (*v_low).min(to_tin);
                } else {
                    self.dfs_cut(to, Some(v), timer, visited, tin, low, cut_vertices);
                    let to_low = low[&to];
                    let v_low = low.get_mut(&v).unwrap();
                    *v_low = (*v_low).min(to_low);
                    if to_low >= tin[&v] && p.is_some() {
                        cut_vertices.insert(v);
                    }
                    children += 1;
                }
            }
        }

        if p.is_none() && children > 1 {
            cut_vertices.insert(v);
        }
    }
}
