/// Raft network communication layer
/// Implements HTTP-based RPC for distributed consensus
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEntriesRequest {
    pub term: u64,
    pub leader_id: u64,
    pub prev_log_index: u64,
    pub prev_log_term: u64,
    pub entries: Vec<LogEntry>,
    pub leader_commit: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEntriesResponse {
    pub term: u64,
    pub success: bool,
    pub conflict_opt: Option<ConflictOpt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictOpt {
    pub last_log_index: u64,
    pub last_log_term: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestVoteRequest {
    pub term: u64,
    pub candidate_id: u64,
    pub last_log_index: u64,
    pub last_log_term: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestVoteResponse {
    pub term: u64,
    pub vote_granted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub term: u64,
    pub index: u64,
    pub data: Vec<u8>,
}

/// Network client for sending Raft RPCs to remote nodes
pub struct RaftNetworkClient {
    client: reqwest::Client,
    peers: Arc<RwLock<Vec<(u64, String)>>>, // (node_id, address)
}

impl RaftNetworkClient {
    pub fn new(peers: Vec<(u64, String)>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .expect("Failed to build HTTP client"),
            peers: Arc::new(RwLock::new(peers)),
        }
    }

    /// Send AppendEntries RPC to a peer (heartbeat or log replication)
    pub async fn send_append_entries(
        &self,
        peer_id: u64,
        req: AppendEntriesRequest,
    ) -> anyhow::Result<AppendEntriesResponse> {
        let peers = self.peers.read().await;
        let peer_addr = peers
            .iter()
            .find(|(id, _)| *id == peer_id)
            .map(|(_, addr)| addr.clone())
            .ok_or_else(|| anyhow::anyhow!("Peer {} not found", peer_id))?;

        let url = format!("http://{}/raft/append-entries", peer_addr);
        let resp = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await?
            .json::<AppendEntriesResponse>()
            .await?;

        Ok(resp)
    }

    /// Send RequestVote RPC to a peer (leader election)
    pub async fn send_request_vote(
        &self,
        peer_id: u64,
        req: RequestVoteRequest,
    ) -> anyhow::Result<RequestVoteResponse> {
        let peers = self.peers.read().await;
        let peer_addr = peers
            .iter()
            .find(|(id, _)| *id == peer_id)
            .map(|(_, addr)| addr.clone())
            .ok_or_else(|| anyhow::anyhow!("Peer {} not found", peer_id))?;

        let url = format!("http://{}/raft/request-vote", peer_addr);
        let resp = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await?
            .json::<RequestVoteResponse>()
            .await?;

        Ok(resp)
    }

    /// Broadcast AppendEntries to all peers (heartbeat or log replication)
    pub async fn broadcast_append_entries(
        &self,
        req: AppendEntriesRequest,
    ) -> Vec<(u64, Result<AppendEntriesResponse, String>)> {
        let peers = self.peers.read().await.clone();
        let mut results = Vec::new();

        for (peer_id, _) in peers {
            let result = self.send_append_entries(peer_id, req.clone()).await;
            results.push((
                peer_id,
                result.map_err(|e| e.to_string()),
            ));
        }

        results
    }

    /// Broadcast RequestVote to all peers (leader election)
    pub async fn broadcast_request_vote(
        &self,
        req: RequestVoteRequest,
    ) -> Vec<(u64, Result<RequestVoteResponse, String>)> {
        let peers = self.peers.read().await.clone();
        let mut results = Vec::new();

        for (peer_id, _) in peers {
            let result = self.send_request_vote(peer_id, req.clone()).await;
            results.push((
                peer_id,
                result.map_err(|e| e.to_string()),
            ));
        }

        results
    }
}

/// Health check for remote nodes
pub async fn health_check(node_addr: &str) -> bool {
    let url = format!("http://{}/healthz", node_addr);
    match tokio::time::timeout(
        std::time::Duration::from_secs(2),
        reqwest::get(&url),
    )
    .await
    {
        Ok(Ok(resp)) => resp.status().is_success(),
        _ => false,
    }
}

/// Check health of all peers
pub async fn check_peer_health(peers: &[(u64, String)]) -> Vec<(u64, bool)> {
    let mut results = Vec::new();
    for (peer_id, addr) in peers {
        let healthy = health_check(addr).await;
        results.push((*peer_id, healthy));
    }
    results
}
