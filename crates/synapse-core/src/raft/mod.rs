use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RaftRole {
    Follower,
    Candidate,
    Leader,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaftLogEntry {
    pub term: u64,
    pub index: u64,
    pub payload_command: String,
    pub target_actor_id: String,
}

pub struct RaftNodeState {
    pub node_id: String,
    pub role: RwLock<RaftRole>,
    pub current_term: AtomicU64,
    pub voted_for: RwLock<Option<String>>,
    pub log: RwLock<Vec<RaftLogEntry>>,
    pub commit_index: AtomicU64,
    pub last_applied: AtomicU64,
    pub peers: Vec<String>,
}

impl RaftNodeState {
    pub fn new(node_id: &str, peers: Vec<String>) -> Self {
        Self {
            node_id: node_id.to_string(),
            role: RwLock::new(RaftRole::Follower),
            current_term: AtomicU64::new(0),
            voted_for: RwLock::new(None),
            log: RwLock::new(Vec::new()),
            commit_index: AtomicU64::new(0),
            last_applied: AtomicU64::new(0),
            peers,
        }
    }

    pub async fn start_election(&self) -> bool {
        let mut role = self.role.write().await;
        *role = RaftRole::Candidate;
        let _new_term = self.current_term.fetch_add(1, Ordering::SeqCst) + 1;
        let mut voted = self.voted_for.write().await;
        *voted = Some(self.node_id.clone());

        let total_nodes = self.peers.len() + 1;
        let quorum = (total_nodes / 2) + 1;
        let votes_granted = 1; // Votes for self

        if votes_granted >= quorum {
            *role = RaftRole::Leader;
            true
        } else {
            false
        }
    }

    pub async fn append_entry(&self, command: &str, target_actor: &str) -> Result<u64, &'static str> {
        let role = self.role.read().await;
        if *role != RaftRole::Leader {
            return Err("Only leader can append log entries");
        }

        let mut log = self.log.write().await;
        let index = (log.len() as u64) + 1;
        let term = self.current_term.load(Ordering::Relaxed);

        log.push(RaftLogEntry {
            term,
            index,
            payload_command: command.to_string(),
            target_actor_id: target_actor.to_string(),
        });

        self.commit_index.store(index, Ordering::Release);
        Ok(index)
    }

    pub async fn is_leader(&self) -> bool {
        *self.role.read().await == RaftRole::Leader
    }
}
