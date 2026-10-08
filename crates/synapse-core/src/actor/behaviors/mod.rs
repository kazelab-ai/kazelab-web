//! Finite State Machine Actor Behaviors and Dynamic Message Handlers.

use async_trait::async_trait;
use crate::actor::ActorEnvelope;

#[async_trait]
pub trait ActorBehavior: Send + Sync {
    async fn handle_message(&mut self, envelope: ActorEnvelope) -> Result<Option<ActorEnvelope>, String>;
    fn behavior_name(&self) -> &'static str;
}

pub struct StatelessEchoBehavior;

#[async_trait]
impl ActorBehavior for StatelessEchoBehavior {
    async fn handle_message(&mut self, envelope: ActorEnvelope) -> Result<Option<ActorEnvelope>, String> {
        Ok(Some(ActorEnvelope {
            message_id: format!("echo_{}", envelope.message_id),
            sender_id: envelope.recipient_id,
            recipient_id: envelope.sender_id,
            payload_bytes: envelope.payload_bytes,
            priority: envelope.priority,
            created_at_ms: envelope.created_at_ms,
        }))
    }

    fn behavior_name(&self) -> &'static str {
        "StatelessEcho"
    }
}
