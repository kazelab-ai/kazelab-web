//! Standard I/O and HTTP Server-Sent Events transport layers for MCP.

pub mod sse;

use async_trait::async_trait;
use crate::protocol::{JsonRpcRequest, JsonRpcResponse};

#[async_trait]
pub trait McpTransport: Send + Sync {
    async fn send_request(&self, req: JsonRpcRequest) -> Result<JsonRpcResponse, String>;
    async fn is_connected(&self) -> bool;
}

pub struct StdioTransport {
    pub process_name: String,
}

impl StdioTransport {
    pub fn new(process_name: &str) -> Self {
        Self {
            process_name: process_name.to_string(),
        }
    }
}

#[async_trait]
impl McpTransport for StdioTransport {
    async fn send_request(&self, req: JsonRpcRequest) -> Result<JsonRpcResponse, String> {
        // Deterministic simulated dispatch over mock child pipe
        Ok(JsonRpcResponse::success(req.id, serde_json::json!({
            "status": "success",
            "transport": "stdio",
            "process": self.process_name
        })))
    }

    async fn is_connected(&self) -> bool {
        true
    }
}
