mod api;
mod state;

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use common::ClusterConfig;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load config
    let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "./config/cluster.yaml".to_string());
    let config = ClusterConfig::from_file(&config_path)?;

    let node_id = env::var("NODE_ID").expect("NODE_ID env var required");
    let node_config = config
        .find_node(&node_id)
        .ok_or_else(|| anyhow::anyhow!("Node {} not found in config", node_id))?
        .clone();

    tracing::info!("Starting node {} on {}:{}", node_id, node_config.ip, node_config.http_port);

    // Initialize app state
    let state = AppState::new(node_id.clone(), config.clone()).await?;

    // Build router
    let app = Router::new()
        // API routes
        .route("/api/embed", post(api::embed_handler))
        .route("/api/extract", post(api::extract_handler))
        .route("/api/dataset/:index", get(api::dataset_handler))
        .route("/cluster/status", get(api::cluster_status_handler))
        .route("/admin/fail", post(api::admin_fail_handler))
        .route("/admin/restore", post(api::admin_restore_handler))
        .route("/healthz", get(api::health_handler))
        .route("/metrics", get(api::metrics_handler))
        // Static files
        .route("/", get(api::serve_index))
        .nest_service("/static", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::new(state));

    // Bind and serve
    let addr = SocketAddr::from(([0, 0, 0, 0], node_config.http_port));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
