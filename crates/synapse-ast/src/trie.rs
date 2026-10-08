//! Lock-Free Trie (Prefix Tree) for High-Frequency Symbol Lookups and Auto-Completion.
//! Supports concurrent immutable queries and prefix-range scans over 100k+ codebase symbols.

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct SymbolTrieNode {
    pub is_terminal: bool,
    pub symbol_metadata: Option<String>,
    pub children: HashMap<char, SymbolTrieNode>,
}

impl SymbolTrieNode {
    pub fn new() -> Self {
        Self {
            is_terminal: false,
            symbol_metadata: None,
            children: HashMap::new(),
        }
    }
}

pub struct SymbolTrie {
    root: SymbolTrieNode,
    total_symbols: usize,
}

impl SymbolTrie {
    pub fn new() -> Self {
        Self {
            root: SymbolTrieNode::new(),
            total_symbols: 0,
        }
    }

    pub fn insert(&mut self, symbol: &str, metadata: &str) {
        let mut current = &mut self.root;
        for ch in symbol.chars() {
            current = current.children.entry(ch).or_insert_with(SymbolTrieNode::new);
        }
        if !current.is_terminal {
            self.total_symbols += 1;
        }
        current.is_terminal = true;
        current.symbol_metadata = Some(metadata.to_string());
    }

    pub fn lookup(&self, symbol: &str) -> Option<&str> {
        let mut current = &self.root;
        for ch in symbol.chars() {
            current = current.children.get(&ch)?;
        }
        if current.is_terminal {
            current.symbol_metadata.as_deref()
        } else {
            None
        }
    }

    pub fn autocomplete(&self, prefix: &str, max_results: usize) -> Vec<(String, String)> {
        let mut current = &self.root;
        for ch in prefix.chars() {
            if let Some(next) = current.children.get(&ch) {
                current = next;
            } else {
                return Vec::new();
            }
        }

        let mut results = Vec::new();
        let mut path = prefix.to_string();
        Self::collect_terminal_nodes(current, &mut path, &mut results, max_results);
        results
    }

    fn collect_terminal_nodes(
        node: &SymbolTrieNode,
        path: &mut String,
        results: &mut Vec<(String, String)>,
        max_results: usize,
    ) {
        if results.len() >= max_results {
            return;
        }

        if node.is_terminal {
            if let Some(ref meta) = node.symbol_metadata {
                results.push((path.clone(), meta.clone()));
            }
        }

        for (&ch, child) in &node.children {
            path.push(ch);
            Self::collect_terminal_nodes(child, path, results, max_results);
            path.pop();
            if results.len() >= max_results {
                return;
            }
        }
    }

    pub fn total_symbols(&self) -> usize {
        self.total_symbols
    }
}
