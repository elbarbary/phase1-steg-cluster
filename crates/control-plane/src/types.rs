use serde::{Deserialize, Serialize};

pub type NodeId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeRole {
    Leader,
    Follower,
    Learner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub id: String,
    pub ip: String,
    pub http_port: u16,
    pub raft_port: u16,
    pub role: NodeRole,
    pub healthy: bool,
    pub cpu_pct: f32,
    pub mem_pct: f32,
    pub qps_1m: f32,
    pub p95_ms: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatus {
    pub term: u64,
    pub leader_id: Option<String>,
    pub nodes: Vec<NodeStatus>,
}
