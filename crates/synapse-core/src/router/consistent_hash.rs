//! Consistent Hashing Ring with Virtual Nodes (Ketama Algorithm).
//! Distributes requests evenly across cluster nodes with minimal key re-mapping upon node churn.

use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

pub struct ConsistentHashRing {
    ring: BTreeMap<u64, String>, // token_hash -> node_id
    virtual_nodes_per_node: usize,
    nodes: Vec<String>,
}

impl ConsistentHashRing {
    pub fn new(virtual_nodes_per_node: usize) -> Self {
        Self {
            ring: BTreeMap::new(),
            virtual_nodes_per_node,
            nodes: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node_id: &str) {
        if !self.nodes.iter().any(|n| n == node_id) {
            self.nodes.push(node_id.to_string());
        }
        for v in 0..self.virtual_nodes_per_node {
            let vkey = format!("{}-vnode-{}", node_id, v);
            let h = Self::hash_key(&vkey);
            self.ring.insert(h, node_id.to_string());
        }
    }

    pub fn remove_node(&mut self, node_id: &str) {
        self.nodes.retain(|n| n != node_id);
        for v in 0..self.virtual_nodes_per_node {
            let vkey = format!("{}-vnode-{}", node_id, v);
            let h = Self::hash_key(&vkey);
            self.ring.remove(&h);
        }
    }

    /// Resolves target node for a given key by finding the successor token on the ring.
    pub fn get_node(&self, key: &str) -> Option<&str> {
        if self.ring.is_empty() {
            return None;
        }

        let h = Self::hash_key(key);

        // Find first token >= h, or wrap around to the beginning
        if let Some((_, node)) = self.ring.range(h..).next() {
            Some(node.as_str())
        } else if let Some((_, node)) = self.ring.iter().next() {
            Some(node.as_str())
        } else {
            None
        }
    }

    /// Returns N distinct physical replica nodes for redundancy replication.
    pub fn get_replicas(&self, key: &str, replica_count: usize) -> Vec<String> {
        if self.ring.is_empty() {
            return Vec::new();
        }

        let h = Self::hash_key(key);
        let mut result = Vec::new();

        // Chain range(h..) followed by range(..h) to cover the entire circle
        let iter = self.ring.range(h..).chain(self.ring.range(..h));

        for (_, node) in iter {
            if !result.contains(node) {
                result.push(node.clone());
                if result.len() >= replica_count || result.len() >= self.nodes.len() {
                    break;
                }
            }
        }

        result
    }

    fn hash_key(key: &str) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    pub fn total_nodes(&self) -> usize {
        self.nodes.len()
    }
}
