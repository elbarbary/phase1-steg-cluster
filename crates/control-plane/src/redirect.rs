/// Client request redirection and retry logic
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotLeaderResponse {
    pub current_leader: Option<u64>,
    pub leader_address: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryableError {
    pub error: String,
    pub retry_at: Option<String>,
}

impl NotLeaderResponse {
    pub fn new(leader_id: Option<u64>, leader_addr: Option<String>) -> Self {
        Self {
            current_leader: leader_id,
            leader_address: leader_addr,
            message: "This node is not the leader. Please redirect to the current leader.".to_string(),
        }
    }
}
