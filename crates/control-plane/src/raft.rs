use openraft::Config;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::types::NodeRole;

pub type NodeId = u64;
pub type Node = openraft::BasicNode;

openraft::declare_raft_types!(
    pub TypeConfig:
        D = Request,
        R = Response,
        NodeId = NodeId,
        Node = Node,
        Entry = openraft::Entry<TypeConfig>,
        SnapshotData = (),
);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub operation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub success: bool,
}

pub type Raft = openraft::Raft<TypeConfig>;

#[derive(Clone)]
pub struct RaftNodeConfig {
    pub node_id: NodeId,
    pub raft_addr: String,
    pub peers: Vec<(NodeId, String)>,
}

pub struct RaftNode {
    pub raft: Arc<Raft>,
    config: RaftNodeConfig,
    current_term: Arc<RwLock<u64>>,
    current_role: Arc<RwLock<NodeRole>>,
}

impl RaftNode {
    pub async fn new(config: RaftNodeConfig) -> anyhow::Result<Self> {
        // For this phase, we use a minimal in-memory store
        // In production, you'd use persistent storage
        let raft_config = Config {
            heartbeat_interval: 500,
            election_timeout_min: 1500,
            election_timeout_max: 3000,
            ..Default::default()
        };

        let store = Arc::new(openraft::MemStore::new(config.node_id));
        let network = Arc::new(openraft::impls::OneshotNetwork::default());

        let raft = Raft::new(config.node_id, raft_config.clone(), network, store)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create Raft node: {:?}", e))?;

        Ok(Self {
            raft: Arc::new(raft),
            config,
            current_term: Arc::new(RwLock::new(0)),
            current_role: Arc::new(RwLock::new(NodeRole::Follower)),
        })
    }

    pub async fn get_term(&self) -> u64 {
        // Try to get current term from Raft metrics
        if let Ok(metrics) = self.raft.metrics().borrow().clone() {
            *self.current_term.write().await = metrics.current_term;
            return metrics.current_term;
        }
        *self.current_term.read().await
    }

    pub async fn get_role(&self) -> NodeRole {
        // Try to determine role from Raft state
        if let Ok(metrics) = self.raft.metrics().borrow().clone() {
            let role = if metrics.current_leader == Some(self.config.node_id) {
                NodeRole::Leader
            } else if metrics.current_leader.is_some() {
                NodeRole::Follower
            } else {
                NodeRole::Learner
            };
            *self.current_role.write().await = role.clone();
            return role;
        }
        self.current_role.read().await.clone()
    }

    pub async fn is_leader(&self) -> bool {
        matches!(self.get_role().await, NodeRole::Leader)
    }

    pub fn node_id(&self) -> NodeId {
        self.config.node_id
    }
}

// Simplified implementations for this phase
// In production, you'd implement proper Raft RPC handlers
impl openraft::RaftNetworkFactory<TypeConfig> for Arc<openraft::impls::OneshotNetwork<TypeConfig>> {
    type Network = openraft::impls::OneshotNetwork<TypeConfig>;

    async fn new_client(&mut self, _target: NodeId, _node: &Node) -> Self::Network {
        self.as_ref().clone()
    }
}
