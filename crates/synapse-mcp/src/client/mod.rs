//! Client connection manager for external MCP hosts.

use crate::protocol::JsonRpcRequest;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct McpClientConnection {
    pub server_url: String,
    request_id_counter: AtomicU64,
}

impl McpClientConnection {
    pub fn new(server_url: &str) -> Self {
        Self {
            server_url: server_url.to_string(),
            request_id_counter: AtomicU64::new(1),
        }
    }

    pub fn build_request(&self, method: &str, params: Option<serde_json::Value>) -> JsonRpcRequest {
        let id = self.request_id_counter.fetch_add(1, Ordering::Relaxed);
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id,
            method: method.to_string(),
            params,
        }
    }
}
