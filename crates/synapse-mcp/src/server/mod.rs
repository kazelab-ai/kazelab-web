//! Native MCP Server Engine handling client sessions and tool dispatching.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::protocol::{JsonRpcRequest, JsonRpcResponse};

pub struct McpServerSession {
    pub session_id: String,
    pub client_capabilities: Vec<String>,
}

pub struct McpServerCore {
    sessions: RwLock<HashMap<String, Arc<McpServerSession>>>,
}

impl McpServerCore {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }

    pub async fn active_sessions(&self) -> usize {
        let guard = self.sessions.read().await;
        guard.len()
    }

    pub async fn handle_rpc_call(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        match req.method.as_str() {
            "tools/list" => {
                JsonRpcResponse::success(req.id, serde_json::json!({
                    "tools": [
                        { "name": "cargo_check", "description": "Execute rust compilation" },
                        { "name": "ast_analyze", "description": "Extract syntax tree" }
                    ]
                }))
            }
            "ping" => {
                JsonRpcResponse::success(req.id, serde_json::json!({ "pong": true }))
            }
            _ => {
                JsonRpcResponse::error(req.id, -32601, "Method not found")
            }
        }
    }
}
