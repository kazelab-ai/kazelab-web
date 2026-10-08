//! Semantic AST diff and patch generator.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstDiffEntry {
    pub symbol_name: String,
    pub change_type: String, // "Added", "Modified", "Removed"
    pub old_span: Option<(usize, usize)>,
    pub new_span: Option<(usize, usize)>,
}

pub struct SemanticDiffer;

impl SemanticDiffer {
    pub fn compute_symbol_diff(old_symbols: &[String], new_symbols: &[String]) -> Vec<AstDiffEntry> {
        let mut diffs = Vec::new();
        for sym in new_symbols {
            if !old_symbols.contains(sym) {
                diffs.push(AstDiffEntry {
                    symbol_name: sym.clone(),
                    change_type: "Added".into(),
                    old_span: None,
                    new_span: Some((1, 10)),
                });
            }
        }
        for sym in old_symbols {
            if !new_symbols.contains(sym) {
                diffs.push(AstDiffEntry {
                    symbol_name: sym.clone(),
                    change_type: "Removed".into(),
                    old_span: Some((1, 10)),
                    new_span: None,
                });
            }
        }
        diffs
    }
}
