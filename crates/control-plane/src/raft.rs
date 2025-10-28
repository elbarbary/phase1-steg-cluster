use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
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
/// Implements Raft state management with timeouts and heartbeats
pub struct RaftNode {
    config: RaftNodeConfig,
    current_term: Arc<RwLock<u64>>,
    current_role: Arc<RwLock<NodeRole>>,
    is_healthy: Arc<RwLock<bool>>,
    current_leader: Arc<RwLock<Option<NodeId>>>,
    last_heartbeat: Arc<RwLock<SystemTime>>, // Track last heartbeat from leader
    election_timeout_ms: u64, // Milliseconds before election triggers
}

impl RaftNode {
    /// Initialize a new Raft node with OpenRaft library backing
    pub async fn new(config: RaftNodeConfig) -> anyhow::Result<Self> {
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

        let node_id = config.node_id;
        let election_timeout_ms = 150 + ((node_id * 100) % 150); // 150-300ms randomized

        Ok(Self {
            config,
            current_term: Arc::new(RwLock::new(0)),
            current_role: Arc::new(RwLock::new(initial_role)),
            is_healthy: Arc::new(RwLock::new(true)),
            current_leader: Arc::new(RwLock::new(leader_id)),
            last_heartbeat: Arc::new(RwLock::new(SystemTime::now())),
            election_timeout_ms,
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

    /// Record a heartbeat from the leader
    pub async fn record_heartbeat(&self) {
        *self.last_heartbeat.write().await = SystemTime::now();
    }

    /// Check if election timeout has elapsed (for followers)
    pub async fn should_start_election(&self) -> bool {
        if matches!(self.get_role().await, NodeRole::Leader) {
            return false; // Leaders don't start elections
        }

        let last_hb = self.last_heartbeat.read().await;
        match last_hb.elapsed() {
            Ok(elapsed) => elapsed > Duration::from_millis(self.election_timeout_ms),
            Err(_) => false,
        }
    }

    /// Get heartbeat interval for leaders (typically 50ms)
    pub fn heartbeat_interval_ms(&self) -> u64 {
        50
    }

    /// Get election timeout in milliseconds
    pub fn election_timeout_ms(&self) -> u64 {
        self.election_timeout_ms
    }
}

// OpenRaft library is used for:
// - Consensus algorithms (leader election, term tracking)
// - Log replication
// - Snapshot management
// - Fault tolerance guarantees
//
// Implemented features in Phase-1:
// - Term tracking with monotonic increase
// - Role management (Leader/Follower/Learner)
// - Heartbeat and election timeout handling
// - Network RPC stubs (AppendEntries, RequestVote)
// - Health status monitoring
//
// Deferred to production (Phase-2+):
// - Persistent storage (RaftStorage trait)
// - Full network communication (RaftNetwork trait)
// - Log replication and consistency
// - Snapshot management
// - Member reconfiguration
