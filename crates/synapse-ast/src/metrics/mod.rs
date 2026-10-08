//! Maintainability and Halstead Code Complexity Metrics.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetricsReport {
    pub total_lines: usize,
    pub comment_lines: usize,
    pub blank_lines: usize,
    pub cyclomatic_complexity: usize,
    pub maintainability_index: f64,
}

pub struct MetricsCalculator;

impl MetricsCalculator {
    pub fn analyze(source: &str, cyclomatic: usize) -> CodeMetricsReport {
        let mut total = 0;
        let mut comment = 0;
        let mut blank = 0;

        for line in source.lines() {
            total += 1;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                blank += 1;
            } else if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                comment += 1;
            }
        }

        let code_lines = total - blank - comment;
        let mi = 171.0 - 5.2 * (cyclomatic as f64).ln().max(0.0) - 0.23 * (code_lines as f64);

        CodeMetricsReport {
            total_lines: total,
            comment_lines: comment,
            blank_lines: blank,
            cyclomatic_complexity: cyclomatic,
            maintainability_index: mi.clamp(0.0, 100.0),
        }
    }
}
