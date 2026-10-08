//! Conflict-Free Replicated Data Types (CRDT) for Multi-Agent Concurrent Collaborative Code Synthesis.
//! Implements State-based LWW-Element-Set (Last-Write-Wins) and Observed-Remove Map.

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LwwEntry<T> {
    pub value: T,
    pub timestamp_ms: u64,
    pub node_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LwwElementSet<T: std::hash::Hash + Eq + Clone> {
    add_set: HashMap<T, LwwEntry<()>>,
    remove_set: HashMap<T, LwwEntry<()>>,
}

impl<T: std::hash::Hash + Eq + Clone> LwwElementSet<T> {
    pub fn new() -> Self {
        Self {
            add_set: HashMap::new(),
            remove_set: HashMap::new(),
        }
    }

    pub fn add(&mut self, element: T, timestamp_ms: u64, node_id: &str) {
        let entry = LwwEntry {
            value: (),
            timestamp_ms,
            node_id: node_id.to_string(),
        };
        if let Some(existing) = self.add_set.get(&element) {
            if timestamp_ms >= existing.timestamp_ms {
                self.add_set.insert(element, entry);
            }
        } else {
            self.add_set.insert(element, entry);
        }
    }

    pub fn remove(&mut self, element: T, timestamp_ms: u64, node_id: &str) {
        let entry = LwwEntry {
            value: (),
            timestamp_ms,
            node_id: node_id.to_string(),
        };
        if let Some(existing) = self.remove_set.get(&element) {
            if timestamp_ms >= existing.timestamp_ms {
                self.remove_set.insert(element, entry);
            }
        } else {
            self.remove_set.insert(element, entry);
        }
    }

    pub fn contains(&self, element: &T) -> bool {
        if let Some(add_entry) = self.add_set.get(element) {
            if let Some(rem_entry) = self.remove_set.get(element) {
                // Bias towards Add on timestamp ties
                add_entry.timestamp_ms >= rem_entry.timestamp_ms
            } else {
                true
            }
        } else {
            false
        }
    }

    /// Merges another LwwElementSet replica into this one (Lattice Join operator $\sqcup$).
    pub fn merge(&mut self, other: &LwwElementSet<T>) {
        for (elem, other_entry) in &other.add_set {
            if let Some(local_entry) = self.add_set.get(elem) {
                if other_entry.timestamp_ms > local_entry.timestamp_ms {
                    self.add_set.insert(elem.clone(), other_entry.clone());
                }
            } else {
                self.add_set.insert(elem.clone(), other_entry.clone());
            }
        }

        for (elem, other_entry) in &other.remove_set {
            if let Some(local_entry) = self.remove_set.get(elem) {
                if other_entry.timestamp_ms > local_entry.timestamp_ms {
                    self.remove_set.insert(elem.clone(), other_entry.clone());
                }
            } else {
                self.remove_set.insert(elem.clone(), other_entry.clone());
            }
        }
    }

    pub fn elements(&self) -> HashSet<T> {
        let mut set = HashSet::new();
        for elem in self.add_set.keys() {
            if self.contains(elem) {
                set.insert(elem.clone());
            }
        }
        set
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtDocumentBuffer {
    pub file_path: String,
    lines: HashMap<usize, LwwEntry<String>>,
}

impl CrdtDocumentBuffer {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
            lines: HashMap::new(),
        }
    }

    pub fn update_line(&mut self, line_idx: usize, content: &str, timestamp_ms: u64, node_id: &str) {
        let entry = LwwEntry {
            value: content.to_string(),
            timestamp_ms,
            node_id: node_id.to_string(),
        };

        if let Some(existing) = self.lines.get(&line_idx) {
            if timestamp_ms > existing.timestamp_ms {
                self.lines.insert(line_idx, entry);
            }
        } else {
            self.lines.insert(line_idx, entry);
        }
    }

    pub fn merge_with(&mut self, other: &CrdtDocumentBuffer) {
        for (idx, other_entry) in &other.lines {
            if let Some(local_entry) = self.lines.get(idx) {
                if other_entry.timestamp_ms > local_entry.timestamp_ms {
                    self.lines.insert(*idx, other_entry.clone());
                }
            } else {
                self.lines.insert(*idx, other_entry.clone());
            }
        }
    }

    pub fn render_content(&self) -> String {
        let mut sorted_keys: Vec<usize> = self.lines.keys().cloned().collect();
        sorted_keys.sort();
        let mut out = String::new();
        for k in sorted_keys {
            if let Some(entry) = self.lines.get(&k) {
                out.push_str(&entry.value);
                out.push('\n');
            }
        }
        out
    }
}
