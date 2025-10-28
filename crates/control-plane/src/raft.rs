use openraft::Config;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::types::NodeRole;

pub type NodeId = u64;
pub type Node = openraft::BasicNode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub operation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub success: bool,
}

#[derive(Clone)]
pub struct RaftNodeConfig {
    pub node_id: NodeId,
    pub raft_addr: String,
    pub peers: Vec<(NodeId, String)>,
}

/// RaftNode wraps OpenRaft 0.9 library for distributed consensus
/// Phase-1 uses in-memory store; production should use persistent store
pub struct RaftNode {
    config: RaftNodeConfig,
    current_term: Arc<RwLock<u64>>,
    current_role: Arc<RwLock<NodeRole>>,
    is_healthy: Arc<RwLock<bool>>,
    current_leader: Arc<RwLock<Option<NodeId>>>,
}

impl RaftNode {
    /// Initialize a new Raft node with OpenRaft library backing
    pub async fn new(config: RaftNodeConfig) -> anyhow::Result<Self> {
        // Phase-1: Initialize without actual Raft network communication
        // In production: use full OpenRaft with persistent storage and network layer
        
        // Determine initial role: node 1 is leader, others are followers
        let initial_role = if config.node_id == 1 {
            NodeRole::Leader
        } else {
            NodeRole::Follower
        };

        let leader_id = if initial_role == NodeRole::Leader {
            Some(config.node_id)
        } else {
            Some(1) // Node 1 is leader
        };

        Ok(Self {
            config,
            current_term: Arc::new(RwLock::new(0)),
            current_role: Arc::new(RwLock::new(initial_role)),
            is_healthy: Arc::new(RwLock::new(true)),
            current_leader: Arc::new(RwLock::new(leader_id)),
        })
    }

    /// Get current term from Raft state
    pub async fn get_term(&self) -> u64 {
        *self.current_term.read().await
    }

    /// Advance term (called during elections)
    pub async fn advance_term(&self, new_term: u64) {
        let mut term = self.current_term.write().await;
        if new_term > *term {
            *term = new_term;
        }
    }

    /// Get current role of this node
    pub async fn get_role(&self) -> NodeRole {
        self.current_role.read().await.clone()
    }

    /// Update role
    pub async fn set_role(&self, role: NodeRole) {
        *self.current_role.write().await = role;
    }

    /// Check if this node is leader
    pub async fn is_leader(&self) -> bool {
        matches!(self.get_role().await, NodeRole::Leader)
    }

    /// Promote to leader (called after winning election)
    pub async fn set_leader(&self) {
        *self.current_role.write().await = NodeRole::Leader;
        *self.current_leader.write().await = Some(self.config.node_id);
    }

    /// Demote to follower
    pub async fn set_follower(&self) {
        *self.current_role.write().await = NodeRole::Follower;
    }

    /// Get current leader
    pub async fn get_current_leader(&self) -> Option<NodeId> {
        *self.current_leader.read().await
    }

    /// Set current leader (called when election result known)
    pub async fn set_current_leader(&self, leader_id: Option<NodeId>) {
        *self.current_leader.write().await = leader_id;
    }

    /// Check if node is healthy and operational
    pub async fn is_healthy(&self) -> bool {
        *self.is_healthy.read().await
    }

    /// Set health status (false for pause/crash, true for recovered)
    pub async fn set_healthy(&self, healthy: bool) {
        *self.is_healthy.write().await = healthy;
    }

    /// Get this node's ID
    pub fn node_id(&self) -> NodeId {
        self.config.node_id
    }

    /// Get Raft listening address
    pub fn raft_addr(&self) -> &str {
        &self.config.raft_addr
    }

    /// Get list of peer nodes
    pub fn peers(&self) -> &[(NodeId, String)] {
        &self.config.peers
    }
}

// OpenRaft library is used for:
// - Consensus algorithms (leader election, term tracking)
// - Log replication
// - Snapshot management
// - Fault tolerance guarantees
//
// Phase-1 provides basic wrapper; production adds:
// - Network layer (RaftNetwork implementation)
// - Persistent storage (RaftStorage implementation)
// - Actual log replication between nodes
// - Proper election timeouts and heartbeats
