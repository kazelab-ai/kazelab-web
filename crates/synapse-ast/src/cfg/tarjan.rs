//! Tarjan's Strongly Connected Components (SCC) and Dominator Tree Graph Analysis.
//! Computes immediate dominators, dominance frontiers, and condensation DAGs for CFG optimization.

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct DirectedGraph {
    pub adjacency: HashMap<usize, Vec<usize>>,
    pub reverse_adjacency: HashMap<usize, Vec<usize>>,
    pub all_nodes: HashSet<usize>,
}

impl DirectedGraph {
    pub fn new() -> Self {
        Self {
            adjacency: HashMap::new(),
            reverse_adjacency: HashMap::new(),
            all_nodes: HashSet::new(),
        }
    }

    pub fn add_edge(&mut self, u: usize, v: usize) {
        self.all_nodes.insert(u);
        self.all_nodes.insert(v);
        self.adjacency.entry(u).or_default().push(v);
        self.reverse_adjacency.entry(v).or_default().push(u);
    }

    pub fn successors(&self, u: usize) -> &[usize] {
        self.adjacency.get(&u).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn predecessors(&self, u: usize) -> &[usize] {
        self.reverse_adjacency.get(&u).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Computes Strongly Connected Components using Tarjan's linear-time DFS algorithm.
    pub fn tarjan_scc(&self) -> Vec<Vec<usize>> {
        let mut index = 0;
        let mut stack = Vec::new();
        let mut on_stack = HashSet::new();
        let mut indices = HashMap::new();
        let mut lowlink = HashMap::new();
        let mut sccs = Vec::new();

        for &node in &self.all_nodes {
            if !indices.contains_key(&node) {
                self.strongconnect(
                    node,
                    &mut index,
                    &mut stack,
                    &mut on_stack,
                    &mut indices,
                    &mut lowlink,
                    &mut sccs,
                );
            }
        }

        sccs
    }

    fn strongconnect(
        &self,
        v: usize,
        index: &mut usize,
        stack: &mut Vec<usize>,
        on_stack: &mut HashSet<usize>,
        indices: &mut HashMap<usize, usize>,
        lowlink: &mut HashMap<usize, usize>,
        sccs: &mut Vec<Vec<usize>>,
    ) {
        indices.insert(v, *index);
        lowlink.insert(v, *index);
        *index += 1;
        stack.push(v);
        on_stack.insert(v);

        for &w in self.successors(v) {
            if !indices.contains_key(&w) {
                self.strongconnect(w, index, stack, on_stack, indices, lowlink, sccs);
                let w_low = lowlink[&w];
                let v_low = lowlink.get_mut(&v).unwrap();
                *v_low = (*v_low).min(w_low);
            } else if on_stack.contains(&w) {
                let w_idx = indices[&w];
                let v_low = lowlink.get_mut(&v).unwrap();
                *v_low = (*v_low).min(w_idx);
            }
        }

        if lowlink[&v] == indices[&v] {
            let mut current_scc = Vec::new();
            while let Some(w) = stack.pop() {
                on_stack.remove(&w);
                current_scc.push(w);
                if w == v {
                    break;
                }
            }
            sccs.push(current_scc);
        }
    }

    /// Computes Dominance Frontiers for SSA Phi-node placement (Cytron et al. 1991).
    pub fn compute_dominance_frontiers(
        &self,
        idom: &HashMap<usize, usize>,
    ) -> HashMap<usize, HashSet<usize>> {
        let mut df: HashMap<usize, HashSet<usize>> = HashMap::new();
        for &node in &self.all_nodes {
            df.insert(node, HashSet::new());
        }

        for &b in &self.all_nodes {
            let preds = self.predecessors(b);
            if preds.len() >= 2 {
                for &p in preds {
                    let mut runner = p;
                    let b_idom = idom.get(&b).copied();
                    while Some(runner) != b_idom {
                        df.entry(runner).or_default().insert(b);
                        if let Some(&next_idom) = idom.get(&runner) {
                            if next_idom == runner {
                                break;
                            }
                            runner = next_idom;
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        df
    }
}
