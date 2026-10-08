pub mod behaviors;
pub mod supervision;

use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ActorError {
    #[error("Actor mailbox is full")]
    MailboxFull,
    #[error("Actor panic or terminated: {0}")]
    Terminated(String),
    #[error("Supervision failure: {0}")]
    SupervisionFailed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorState {
    Initializing,
    Idle,
    Processing,
    Suspended,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorEnvelope {
    pub message_id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub payload_bytes: Vec<u8>,
    pub priority: u8,
    pub created_at_ms: u64,
}

pub struct ActorMailbox {
    tx: mpsc::Sender<ActorEnvelope>,
    rx: tokio::sync::Mutex<mpsc::Receiver<ActorEnvelope>>,
    capacity: usize,
}

impl ActorMailbox {
    pub fn new(capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(capacity);
        Self {
            tx,
            rx: tokio::sync::Mutex::new(rx),
            capacity,
        }
    }

    pub async fn send(&self, envelope: ActorEnvelope) -> Result<(), ActorError> {
        self.tx.send(envelope).await.map_err(|_| ActorError::MailboxFull)
    }

    pub async fn recv(&self) -> Option<ActorEnvelope> {
        let mut guard = self.rx.lock().await;
        guard.recv().await
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

pub struct SwarmActorNode {
    pub actor_id: String,
    pub role_name: String,
    pub state: RwLock<ActorState>,
    pub mailbox: Arc<ActorMailbox>,
}

impl SwarmActorNode {
    pub fn new(actor_id: &str, role_name: &str, mailbox_capacity: usize) -> Self {
        Self {
            actor_id: actor_id.to_string(),
            role_name: role_name.to_string(),
            state: RwLock::new(ActorState::Initializing),
            mailbox: Arc::new(ActorMailbox::new(mailbox_capacity)),
        }
    }

    pub async fn transition_state(&self, next_state: ActorState) {
        let mut state_guard = self.state.write().await;
        *state_guard = next_state;
    }

    pub async fn get_state(&self) -> ActorState {
        *self.state.read().await
    }
}

pub struct SupervisorRegistry {
    actors: RwLock<std::collections::HashMap<String, Arc<SwarmActorNode>>>,
}

impl SupervisorRegistry {
    pub fn new() -> Self {
        Self {
            actors: RwLock::new(std::collections::HashMap::new()),
        }
    }

    pub async fn register(&self, actor: Arc<SwarmActorNode>) {
        let mut map = self.actors.write().await;
        map.insert(actor.actor_id.clone(), actor);
    }

    pub async fn route_message(&self, envelope: ActorEnvelope) -> Result<(), ActorError> {
        let map = self.actors.read().await;
        if let Some(target) = map.get(&envelope.recipient_id) {
            target.mailbox.send(envelope).await
        } else {
            Err(ActorError::Terminated(format!("Target actor {} not found", envelope.recipient_id)))
        }
    }

    pub async fn active_actors_count(&self) -> usize {
        let map = self.actors.read().await;
        map.len()
    }
}
