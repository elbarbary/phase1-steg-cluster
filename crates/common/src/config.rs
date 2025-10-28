use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    pub cluster_name: String,
    pub nodes: Vec<NodeConfig>,
    pub stego: StegoConfig,
    pub gui: GuiConfig,
    pub loadgen: LoadgenConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub id: String,
    pub ip: String,
    pub http_port: u16,
    pub raft_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StegoConfig {
    pub lsb_per_channel: u8,
    pub compress: bool,
    pub max_pixels: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiConfig {
    pub status_poll_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadgenConfig {
    pub request_timeout_ms: u64,
    pub max_retries: u32,
}

impl ClusterConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: ClusterConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn find_node(&self, node_id: &str) -> Option<&NodeConfig> {
        self.nodes.iter().find(|n| n.id == node_id)
    }

    pub fn get_all_node_urls(&self) -> Vec<String> {
        self.nodes
            .iter()
            .map(|n| format!("http://{}:{}", n.ip, n.http_port))
            .collect()
    }
}
