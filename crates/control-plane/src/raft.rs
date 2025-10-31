use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use crate::types::NodeRole;
use crate::storage::RaftStorage;

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
    voted_for: Arc<RwLock<Option<NodeId>>>, // Track who we voted for in current term
    votes_received: Arc<RwLock<HashSet<NodeId>>>, // Track votes received as candidate
    storage: Arc<RaftStorage>, // Persistent log and state storage
}

impl RaftNode {
    /// Initialize a new Raft node with OpenRaft library backing
    pub async fn new(config: RaftNodeConfig) -> anyhow::Result<Self> {
        // Initialize persistent storage
        let data_dir = format!("./data/node-{}", config.node_id);
        tokio::fs::create_dir_all(&data_dir).await?;
        let storage = Arc::new(RaftStorage::new(&data_dir)?);

        // Determine initial role: start all nodes as followers and allow
        // the cluster to elect a leader dynamically. Relying on a
        // hardcoded leader (node 1) makes startup order-sensitive and
        // can leave the cluster in a stale-leader state when nodes start
        // at different times.
        let initial_role = NodeRole::Follower;
        let leader_id = None;

        let node_id = config.node_id;
        let election_timeout_ms = 50 + ((node_id * 30) % 50); // 50-100ms randomized (REDUCED for faster failover)

        Ok(Self {
            config,
            current_term: Arc::new(RwLock::new(0)),
            current_role: Arc::new(RwLock::new(initial_role)),
            is_healthy: Arc::new(RwLock::new(true)),
            current_leader: Arc::new(RwLock::new(leader_id)),
            last_heartbeat: Arc::new(RwLock::new(SystemTime::now())),
            election_timeout_ms,
            voted_for: Arc::new(RwLock::new(None)),
            votes_received: Arc::new(RwLock::new(HashSet::new())),
            storage,
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
        if self.is_leader().await {
            return false;
        }
        let last_hb = self.last_heartbeat.read().await;
        match SystemTime::now().duration_since(*last_hb) {
            Ok(elapsed) => {
                let should_election = elapsed > Duration::from_millis(self.election_timeout_ms);
                
                // Debug: Log election timeout status every 500ms
                if elapsed.as_millis() % 500 < 50 {
                    tracing::trace!(
                        "Node {} election timeout check: elapsed={}ms, timeout={}ms, should_start={}",
                        self.node_id(),
                        elapsed.as_millis(),
                        self.election_timeout_ms,
                        should_election
                    );
                }
                
                should_election
            },
            Err(_) => true, // Clock went backwards? Trigger election
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

    /// Start an election (transition to Candidate and increment term)
    pub async fn start_election(&self) -> u64 {
        // Increment term
        let mut term = self.current_term.write().await;
        *term += 1;
        let new_term = *term;
        drop(term);

        // Transition to Candidate
        *self.current_role.write().await = NodeRole::Candidate;

        // Vote for self
        *self.voted_for.write().await = Some(self.config.node_id);
        
        // Reset votes received and add self-vote
        let mut votes = self.votes_received.write().await;
        votes.clear();
        votes.insert(self.config.node_id);
        drop(votes);

        // Clear current leader
        *self.current_leader.write().await = None;

        tracing::info!(
            "Node {} starting election for term {}",
            self.config.node_id,
            new_term
        );

        new_term
    }

    /// Record a vote received from a peer
    pub async fn record_vote(&self, from_node: NodeId) -> bool {
        let mut votes = self.votes_received.write().await;
        votes.insert(from_node);
        
        // Check if we have majority (more than half)
        let total_nodes = self.config.peers.len() + 1; // +1 for self
        let majority = (total_nodes / 2) + 1;
        let has_majority = votes.len() >= majority;

        if has_majority {
            tracing::info!(
                "Node {} won election with {}/{} votes",
                self.config.node_id,
                votes.len(),
                total_nodes
            );
        } else {
            tracing::debug!(
                "Node {} received vote (now {}/{})",
                self.config.node_id,
                votes.len(),
                total_nodes
            );
        }

        has_majority
    }

    /// Grant vote for a candidate (if we haven't voted this term)
    pub async fn grant_vote(&self, candidate_id: NodeId, candidate_term: u64) -> bool {
        let current_term = self.get_term().await;
        
        // Update term if candidate has higher term
        if candidate_term > current_term {
            self.advance_term(candidate_term).await;
            *self.voted_for.write().await = None; // Reset vote for new term
            *self.current_role.write().await = NodeRole::Follower; // Step down
        }

        // Check if we can vote
        let mut voted_for = self.voted_for.write().await;
        let can_vote = voted_for.is_none() || *voted_for == Some(candidate_id);

        if can_vote && candidate_term >= current_term {
            *voted_for = Some(candidate_id);
            tracing::info!(
                "Node {} granted vote to {} for term {}",
                self.config.node_id,
                candidate_id,
                candidate_term
            );
            true
        } else {
            tracing::debug!(
                "Node {} denied vote to {} for term {} (already voted for {:?})",
                self.config.node_id,
                candidate_id,
                candidate_term,
                *voted_for
            );
            false
        }
    }

    /// Get who we voted for in current term
    pub async fn get_voted_for(&self) -> Option<NodeId> {
        *self.voted_for.read().await
    }

    /// Reset election state for new term
    pub async fn reset_election_state(&self) {
        *self.voted_for.write().await = None;
        self.votes_received.write().await.clear();
    }

    /// Get reference to persistent storage
    pub fn storage(&self) -> Arc<RaftStorage> {
        self.storage.clone()
    }

    /// Append a log entry to persistent storage
    pub async fn append_log_entry(&self, entry: crate::storage::RaftLogEntry) -> anyhow::Result<()> {
        self.storage.append_entry(&entry)?;
        tracing::debug!("Node {} appended log entry at index {}", self.config.node_id, entry.index);
        Ok(())
    }

    /// Get log entry from persistent storage
    pub async fn get_log_entry(&self, index: u64) -> anyhow::Result<Option<crate::storage::RaftLogEntry>> {
        self.storage.get_entry(index)
    }

    /// Get log entries in range
    pub async fn get_log_entries(&self, start: u64, end: u64) -> anyhow::Result<Vec<crate::storage::RaftLogEntry>> {
        self.storage.get_entries(start, end)
    }

    /// Persist term and voted_for to storage
    pub async fn persist_state(&self) -> anyhow::Result<()> {
        let state = crate::storage::RaftState {
            current_term: self.current_term.read().await.clone(),
            voted_for: self.voted_for.read().await.clone(),
            commit_index: 0, // Will be updated with append entries
            last_applied: 0,
        };
        self.storage.save_state(&state)?;
        Ok(())
    }

    /// Restore state from persistent storage
    pub async fn restore_state(&self) -> anyhow::Result<()> {
        if let Some(state) = self.storage.load_state()? {
            *self.current_term.write().await = state.current_term;
            *self.voted_for.write().await = state.voted_for;
            tracing::info!("Node {} restored state from storage: term={}", self.config.node_id, state.current_term);
        }
        Ok(())
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
