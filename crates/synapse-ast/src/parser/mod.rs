//! Syntax parser for Rust, C++, and Python code translation units.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstDeclaration {
    pub name: String,
    pub kind: String, // "struct", "enum", "fn", "class", "trait"
    pub visibility: String, // "pub", "private"
    pub line_number: usize,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedModule {
    pub file_path: String,
    pub declarations: Vec<AstDeclaration>,
    pub raw_line_count: usize,
    pub cyclomatic_complexity: usize,
}

pub struct SyntaxParser;

impl SyntaxParser {
    pub fn parse_rust_source(file_path: &str, content: &str) -> ParsedModule {
        let mut declarations = Vec::new();
        let mut complexity = 1;

        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains("if ") || trimmed.contains("match ") || trimmed.contains("for ") || trimmed.contains("while ") {
                complexity += 1;
            }

            if trimmed.starts_with("pub struct ") || trimmed.starts_with("struct ") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 3 {
                    declarations.push(AstDeclaration {
                        name: parts[2].trim_end_matches('{').to_string(),
                        kind: "struct".into(),
                        visibility: if trimmed.starts_with("pub") { "pub".into() } else { "private".into() },
                        line_number: idx + 1,
                        documentation: None,
                    });
                }
            } else if trimmed.starts_with("pub fn ") || trimmed.starts_with("fn ") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                let fn_name_idx = if trimmed.starts_with("pub") { 2 } else { 1 };
                if parts.len() > fn_name_idx {
                    let fn_name = parts[fn_name_idx].split('(').next().unwrap_or("unknown");
                    declarations.push(AstDeclaration {
                        name: fn_name.to_string(),
                        kind: "fn".into(),
                        visibility: if trimmed.starts_with("pub") { "pub".into() } else { "private".into() },
                        line_number: idx + 1,
                        documentation: None,
                    });
                }
            }
        }

        ParsedModule {
            file_path: file_path.to_string(),
            declarations,
            raw_line_count: content.lines().count(),
            cyclomatic_complexity: complexity,
        }
    }
}
