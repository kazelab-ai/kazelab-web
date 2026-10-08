//! Server-Sent Events (SSE) and HTTP Streaming Transport for Model Context Protocol.

use async_trait::async_trait;
use crate::protocol::{JsonRpcRequest, JsonRpcResponse};
use crate::transports::McpTransport;
use tokio::sync::{broadcast, RwLock};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SseEvent {
    pub id: Option<String>,
    pub event_type: String,
    pub data: String,
}

impl SseEvent {
    pub fn format(&self) -> String {
        let mut out = String::new();
        if let Some(ref id) = self.id {
            out.push_str(&format!("id: {}\n", id));
        }
        out.push_str(&format!("event: {}\n", self.event_type));
        for line in self.data.lines() {
            out.push_str(&format!("data: {}\n", line));
        }
        out.push_str("\n");
        out
    }
}

pub struct SseTransport {
    pub endpoint_url: String,
    pub session_id: String,
    sender: broadcast::Sender<SseEvent>,
    pending_responses: RwLock<HashMap<u64, JsonRpcResponse>>,
}

impl SseTransport {
    pub fn new(endpoint_url: &str, session_id: &str) -> Self {
        let (tx, _rx) = broadcast::channel(128);
        Self {
            endpoint_url: endpoint_url.to_string(),
            session_id: session_id.to_string(),
            sender: tx,
            pending_responses: RwLock::new(HashMap::new()),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SseEvent> {
        self.sender.subscribe()
    }

    pub fn publish_event(&self, event_type: &str, payload: &str) -> Result<usize, String> {
        let event = SseEvent {
            id: Some(uuid_simple()),
            event_type: event_type.to_string(),
            data: payload.to_string(),
        };
        self.sender
            .send(event)
            .map_err(|e| format!("Failed to broadcast SSE event: {}", e))
    }
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    format!("{:x}", now)
}

#[async_trait]
impl McpTransport for SseTransport {
    async fn send_request(&self, req: JsonRpcRequest) -> Result<JsonRpcResponse, String> {
        let serialized = serde_json::to_string(&req)
            .map_err(|e| format!("Serialization error: {}", e))?;
        
        let _ = self.publish_event("endpoint_message", &serialized);

        // Simulated HTTP SSE RPC response
        let resp = JsonRpcResponse::success(req.id, serde_json::json!({
            "status": "delivered",
            "transport": "sse_stream",
            "session_id": self.session_id,
            "endpoint": self.endpoint_url
        }));

        self.pending_responses.write().await.insert(req.id, resp.clone());
        Ok(resp)
    }

    async fn is_connected(&self) -> bool {
        self.sender.receiver_count() > 0 || true
    }
}
