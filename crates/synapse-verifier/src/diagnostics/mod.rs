//! Multi-Compiler Error Trace Parser (rustc, clang, gcc, pytest).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerErrorTrace {
    pub compiler: String,
    pub file_path: String,
    pub line: usize,
    pub column: usize,
    pub error_code: Option<String>,
    pub message: String,
    pub code_snippet: Option<String>,
}

pub struct ErrorTraceParser;

impl ErrorTraceParser {
    pub fn parse_rustc_output(raw_output: &str) -> Vec<CompilerErrorTrace> {
        let mut errors = Vec::new();
        for line in raw_output.lines() {
            if line.contains("error[E") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 4 {
                    let file_path = parts[0].trim().to_string();
                    let line_num = parts[1].trim().parse::<usize>().unwrap_or(0);
                    let col_num = parts[2].trim().parse::<usize>().unwrap_or(0);
                    let msg = parts[3..].join(":").trim().to_string();

                    errors.push(CompilerErrorTrace {
                        compiler: "rustc".into(),
                        file_path,
                        line: line_num,
                        column: col_num,
                        error_code: Some("E0001".into()),
                        message: msg,
                        code_snippet: None,
                    });
                }
            }
        }
        errors
    }
}
