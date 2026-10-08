pub mod protocol;
pub mod transports;
pub mod tools;
pub mod server;
pub mod client;

use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub endpoint: String,
    pub is_sandboxed: bool,
    pub execution_timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpClusterNode {
    pub node_id: String,
    pub server_name: String,
    pub protocol_version: String,
    pub tools: Vec<McpTool>,
    pub ping_latency_ms: f64,
}

pub struct McpRegistry {
    nodes: RwLock<HashMap<String, McpClusterNode>>,
}

impl McpRegistry {
    pub fn new() -> Self {
        let mut initial_nodes = HashMap::new();

        initial_nodes.insert(
            "mcp-ast-analyzer".into(),
            McpClusterNode {
                node_id: "mcp-ast-analyzer".into(),
                server_name: "Tree-sitter Syntax Analyzer".into(),
                protocol_version: "1.1.0".into(),
                tools: vec![
                    McpTool {
                        name: "build_dependency_dag".into(),
                        description: "Computes AST dependency graph across modules".into(),
                        endpoint: "mcp://ast.cluster.kazelab.xyz:9001".into(),
                        is_sandboxed: true,
                        execution_timeout_ms: 5000,
                    }
                ],
                ping_latency_ms: 1.2,
            },
        );

        initial_nodes.insert(
            "mcp-sandbox-runner".into(),
            McpClusterNode {
                node_id: "mcp-sandbox-runner".into(),
                server_name: "Compiler & Test Harness".into(),
                protocol_version: "1.1.0".into(),
                tools: vec![
                    McpTool {
                        name: "cargo_check".into(),
                        description: "Compiles Rust code in isolated container".into(),
                        endpoint: "mcp://runner.cluster.kazelab.xyz:9002".into(),
                        is_sandboxed: true,
                        execution_timeout_ms: 15000,
                    }
                ],
                ping_latency_ms: 2.8,
            },
        );

        Self {
            nodes: RwLock::new(initial_nodes),
        }
    }

    pub async fn list_nodes(&self) -> Vec<McpClusterNode> {
        let guard = self.nodes.read().await;
        guard.values().cloned().collect()
    }

    pub async fn get_tool(&self, node_id: &str, tool_name: &str) -> Option<McpTool> {
        let guard = self.nodes.read().await;
        let node = guard.get(node_id)?;
        node.tools.iter().find(|t| t.name == tool_name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_registry() {
        let reg = McpRegistry::new();
        let nodes = reg.list_nodes().await;
        assert_eq!(nodes.len(), 2);
        let tool = reg.get_tool("mcp-sandbox-runner", "cargo_check").await;
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name, "cargo_check");
    }
}
