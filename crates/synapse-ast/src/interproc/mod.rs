//! Inter-procedural Taint Tracking Engine.
//! Follows untrusted user inputs through function arguments, pointer borrows, and return values.

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaintSource {
    pub function_name: String,
    pub parameter_index: usize,
    pub source_type: String, // "HttpInput", "EnvVar", "FileRead"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintSink {
    pub function_name: String,
    pub parameter_index: usize,
    pub vulnerability: String, // "CommandInjection", "SqlInjection", "BufferOverflow"
}

pub struct InterproceduralTaintAnalyzer {
    sources: HashSet<TaintSource>,
    sinks: HashMap<String, Vec<TaintSink>>,
    tainted_variables: HashSet<String>,
}

impl InterproceduralTaintAnalyzer {
    pub fn new() -> Self {
        let mut sinks = HashMap::new();
        sinks.insert("std::process::Command::new".into(), vec![TaintSink {
            function_name: "std::process::Command::new".into(),
            parameter_index: 0,
            vulnerability: "CommandInjection".into(),
        }]);

        Self {
            sources: HashSet::new(),
            sinks,
            tainted_variables: HashSet::new(),
        }
    }

    pub fn mark_source(&mut self, source: TaintSource) {
        self.sources.insert(source);
    }

    pub fn propagate_taint(&mut self, src_var: &str, dest_var: &str) {
        if self.tainted_variables.contains(src_var) {
            self.tainted_variables.insert(dest_var.to_string());
        }
    }

    pub fn check_sink_exposure(&self, func_called: &str, argument_var: &str) -> Option<String> {
        if self.tainted_variables.contains(argument_var) {
            if let Some(sink_list) = self.sinks.get(func_called) {
                return Some(format!(
                    "CRITICAL TAINT FLOW: Variable '{}' flows into vulnerable sink '{}'",
                    argument_var, sink_list[0].vulnerability
                ));
            }
        }
        None
    }
}
