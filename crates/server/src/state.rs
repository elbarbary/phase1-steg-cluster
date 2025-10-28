use common::ClusterConfig;
use control_plane::{MetricsCollector, RaftNetworkClient, RaftNode, RaftNodeConfig, start_raft_tasks};
use image::DynamicImage;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppState {
    pub node_id: String,
    pub config: ClusterConfig,
    pub cover_image: Arc<RwLock<DynamicImage>>,
    pub metrics: Arc<MetricsCollector>,
    pub raft_node: Arc<RaftNode>,
    pub is_paused: Arc<AtomicBool>,
}

impl AppState {
    pub async fn new(node_id: String, config: ClusterConfig) -> anyhow::Result<Self> {
        // Load or generate cover image
        let cover_image = Self::load_or_generate_cover().await?;

        // Initialize metrics collector
        let metrics = Arc::new(MetricsCollector::new());

        // Initialize Raft node (simplified - just track state, not full consensus)
        let node_config_data = config.find_node(&node_id)
            .ok_or_else(|| anyhow::anyhow!("Node {} not found in cluster config", node_id))?;
        let node_numeric_id = match node_id.as_str() {
            "n1" => 1u64,
            "n2" => 2u64,
            "n3" => 3u64,
            _ => 1u64,
        };

        let peers: Vec<(u64, String)> = config
            .nodes
            .iter()
            .filter(|n| n.id != node_id)
            .map(|n| {
                let peer_id = match n.id.as_str() {
                    "n1" => 1u64,
                    "n2" => 2u64,
                    "n3" => 3u64,
                    _ => 1u64,
                };
                (peer_id, format!("{}:{}", n.ip, n.raft_port))
            })
            .collect();

        let raft_config = RaftNodeConfig {
            node_id: node_numeric_id,
            raft_addr: format!("{}:{}", node_config_data.ip, node_config_data.raft_port),
            peers: peers.clone(),
        };

        let raft_node = Arc::new(RaftNode::new(raft_config).await?);

        // Initialize network client for Raft communication
        let network = Arc::new(RaftNetworkClient::new(peers));

        // Start Raft background tasks (election monitoring, heartbeat sending)
        start_raft_tasks(raft_node.clone(), network);

        Ok(Self {
            node_id,
            config,
            cover_image: Arc::new(RwLock::new(cover_image)),
            metrics,
            raft_node,
            is_paused: Arc::new(AtomicBool::new(false)),
        })
    }

    async fn load_or_generate_cover() -> anyhow::Result<DynamicImage> {
        let cover_path = PathBuf::from("assets/cover.png");

        if cover_path.exists() {
            tracing::info!("Loading cover image from {:?}", cover_path);
            let img = image::open(&cover_path)?;
            Ok(img)
        } else {
            tracing::info!("Generating default cover image");
            let img = stego::generate_cover_image(1920, 1080);

            // Create directory and save
            if let Some(parent) = cover_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            img.save(&cover_path)?;
            tracing::info!("Saved cover image to {:?}", cover_path);

            Ok(img)
        }
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused.load(Ordering::SeqCst)
    }

    pub fn pause(&self) {
        self.is_paused.store(true, Ordering::SeqCst);
    }

    pub fn restore(&self) {
        self.is_paused.store(false, Ordering::SeqCst);
    }
}
