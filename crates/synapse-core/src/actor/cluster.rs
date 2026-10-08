//! SWIM (Structured Weakly-Consistent Infection-Style Process Group Membership Protocol) Engine.
//! Provides O(1) failure detection, heartbeat probing, and gossip-based state dissemination.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberStatus {
    Alive,
    Suspect,
    Dead,
    Left,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberNode {
    pub node_id: String,
    pub address: String,
    pub port: u16,
    pub incarnation: u64,
    pub status: MemberStatus,
    pub last_heartbeat_epoch_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwimMessage {
    Ping { sender_id: String, sequence: u64 },
    Ack { sender_id: String, sequence: u64 },
    PingReq { sender_id: String, target_id: String, sequence: u64 },
    Suspect { node_id: String, incarnation: u64 },
    Alive { node_id: String, incarnation: u64 },
    Dead { node_id: String, incarnation: u64 },
}

pub struct SwimCluster {
    pub local_node_id: String,
    pub local_address: String,
    pub local_port: u16,
    incarnation: AtomicU64,
    members: RwLock<HashMap<String, MemberNode>>,
    sequence_counter: AtomicU64,
    suspect_timeout_ms: u64,
}

impl SwimCluster {
    pub fn new(local_node_id: &str, address: &str, port: u16, suspect_timeout_ms: u64) -> Self {
        let mut initial_members = HashMap::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        initial_members.insert(
            local_node_id.to_string(),
            MemberNode {
                node_id: local_node_id.to_string(),
                address: address.to_string(),
                port,
                incarnation: 0,
                status: MemberStatus::Alive,
                last_heartbeat_epoch_ms: now,
            },
        );

        Self {
            local_node_id: local_node_id.to_string(),
            local_address: address.to_string(),
            local_port: port,
            incarnation: AtomicU64::new(0),
            members: RwLock::new(initial_members),
            sequence_counter: AtomicU64::new(0),
            suspect_timeout_ms,
        }
    }

    pub fn next_sequence(&self) -> u64 {
        self.sequence_counter.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub async fn join_member(&self, node_id: &str, address: &str, port: u16) {
        let mut members = self.members.write().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        members.insert(
            node_id.to_string(),
            MemberNode {
                node_id: node_id.to_string(),
                address: address.to_string(),
                port,
                incarnation: 0,
                status: MemberStatus::Alive,
                last_heartbeat_epoch_ms: now,
            },
        );
    }

    pub async fn handle_message(&self, msg: SwimMessage) -> Option<SwimMessage> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        match msg {
            SwimMessage::Ping { sender_id, sequence } => {
                let mut members = self.members.write().await;
                if let Some(m) = members.get_mut(&sender_id) {
                    m.last_heartbeat_epoch_ms = now;
                    if m.status == MemberStatus::Suspect {
                        m.status = MemberStatus::Alive;
                    }
                }
                Some(SwimMessage::Ack {
                    sender_id: self.local_node_id.clone(),
                    sequence,
                })
            }
            SwimMessage::Ack { sender_id, .. } => {
                let mut members = self.members.write().await;
                if let Some(m) = members.get_mut(&sender_id) {
                    m.last_heartbeat_epoch_ms = now;
                    m.status = MemberStatus::Alive;
                }
                None
            }
            SwimMessage::Suspect { node_id, incarnation } => {
                if node_id == self.local_node_id {
                    // Refute suspicion by incrementing our incarnation number
                    let next_inc = self.incarnation.fetch_add(1, Ordering::SeqCst) + 1;
                    return Some(SwimMessage::Alive {
                        node_id: self.local_node_id.clone(),
                        incarnation: next_inc,
                    });
                } else {
                    let mut members = self.members.write().await;
                    if let Some(m) = members.get_mut(&node_id) {
                        if incarnation >= m.incarnation && m.status == MemberStatus::Alive {
                            m.status = MemberStatus::Suspect;
                            m.incarnation = incarnation;
                        }
                    }
                }
                None
            }
            SwimMessage::Alive { node_id, incarnation } => {
                let mut members = self.members.write().await;
                if let Some(m) = members.get_mut(&node_id) {
                    if incarnation > m.incarnation {
                        m.status = MemberStatus::Alive;
                        m.incarnation = incarnation;
                        m.last_heartbeat_epoch_ms = now;
                    }
                }
                None
            }
            SwimMessage::Dead { node_id, incarnation } => {
                let mut members = self.members.write().await;
                if let Some(m) = members.get_mut(&node_id) {
                    if incarnation >= m.incarnation {
                        m.status = MemberStatus::Dead;
                    }
                }
                None
            }
            SwimMessage::PingReq { target_id, sequence, .. } => {
                // Forward ping request
                Some(SwimMessage::Ping {
                    sender_id: target_id,
                    sequence,
                })
            }
        }
    }

    pub async fn check_timeouts(&self) -> Vec<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let mut suspects = Vec::new();
        let mut members = self.members.write().await;

        for (id, node) in members.iter_mut() {
            if id == &self.local_node_id {
                continue;
            }
            if node.status == MemberStatus::Alive && now.saturating_sub(node.last_heartbeat_epoch_ms) > self.suspect_timeout_ms {
                node.status = MemberStatus::Suspect;
                suspects.push(id.clone());
            } else if node.status == MemberStatus::Suspect && now.saturating_sub(node.last_heartbeat_epoch_ms) > self.suspect_timeout_ms * 2 {
                node.status = MemberStatus::Dead;
            }
        }

        suspects
    }

    pub async fn get_alive_members(&self) -> Vec<MemberNode> {
        let members = self.members.read().await;
        members
            .values()
            .filter(|m| m.status == MemberStatus::Alive)
            .cloned()
            .collect()
    }
}
