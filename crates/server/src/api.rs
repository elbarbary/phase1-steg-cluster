use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Json,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use image::ImageFormat;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

use crate::state::AppState;
use control_plane::{ClusterStatus, NodeRole, NodeStatus, NotLeaderResponse};

// ============================================================================
// Helper: Check if this node is leader
// ============================================================================

async fn check_is_leader(state: &Arc<AppState>) -> Result<(), AppError> {
    if !state.raft_node.is_leader().await {
        let current_leader = state.raft_node.get_current_leader().await;
        let leader_addr = current_leader.and_then(|lid| {
            state.config.find_node_by_id(lid as u64).map(|n| format!("{}:{}", n.ip, n.http_port))
        });

        let response = NotLeaderResponse::new(current_leader, leader_addr);
        return Err(AppError::NotLeader(response));
    }
    Ok(())
}

// ============================================================================
// Embed Handler
// ============================================================================

#[derive(Serialize)]
pub struct EmbedResponse {
    request_id: String,
    cover_info: CoverInfoResponse,
    secret_size_bytes: u64,
    payload_size_bytes: u64,
    stego_image_b64: String,
    notes: String,
}

#[derive(Serialize)]
pub struct CoverInfoResponse {
    width: u32,
    height: u32,
    channels: u8,
    lsb_per_channel: u8,
    capacity_bytes: u64,
}

pub async fn embed_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<EmbedResponse>, AppError> {
    let start = Instant::now();

    if state.is_paused() {
        return Err(AppError::ServiceUnavailable);
    }

    // Extract uploaded file
    let mut secret_bytes = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::BadRequest(format!("Failed to read multipart: {}", e))
    })? {
        if field.name() == Some("file") {
            let data = field.bytes().await.map_err(|e| {
                AppError::BadRequest(format!("Failed to read file bytes: {}", e))
            })?;
            secret_bytes = Some(data.to_vec());
            break;
        }
    }

    let secret_bytes = secret_bytes.ok_or_else(|| {
        AppError::BadRequest("No file field found in multipart data".to_string())
    })?;

    let secret_size = secret_bytes.len() as u64;

    // Get cover image
    let cover = state.cover_image.read().await;

    // Perform embedding
    let lsb_per_channel = state.config.stego.lsb_per_channel;
    let compress = state.config.stego.compress;

    let (stego_img, cover_info) = stego::embed(&cover, &secret_bytes, lsb_per_channel, compress)
        .map_err(|e| match e {
            stego::StegoError::CapacityExceeded { needed, available } => {
                AppError::PayloadTooLarge { needed, available }
            }
            _ => AppError::Internal(format!("Embedding failed: {}", e)),
        })?;

    // Encode stego image to PNG bytes
    let mut png_bytes = Vec::new();
    stego_img
        .write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)
        .map_err(|e| AppError::Internal(format!("PNG encoding failed: {}", e)))?;

    let stego_b64 = BASE64.encode(&png_bytes);
    let payload_size = png_bytes.len() as u64;

    let request_id = Uuid::new_v4().to_string();

    // Record metrics
    let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
    state.metrics.record_request(&state.node_id, latency_ms, true);

    Ok(Json(EmbedResponse {
        request_id,
        cover_info: CoverInfoResponse {
            width: cover_info.width,
            height: cover_info.height,
            channels: cover_info.channels,
            lsb_per_channel: cover_info.lsb_per_channel,
            capacity_bytes: cover_info.capacity_bytes,
        },
        secret_size_bytes: secret_size,
        payload_size_bytes: payload_size,
        stego_image_b64: stego_b64,
        notes: "steganography (no normal encryption)".to_string(),
    }))
}

// ============================================================================
// Extract Handler
// ============================================================================

#[derive(Serialize)]
pub struct ExtractResponse {
    request_id: String,
    recovered_size_bytes: u64,
    recovered_mime: String,
    recovered_b64: String,
}

pub async fn extract_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<ExtractResponse>, AppError> {
    let start = Instant::now();

    if state.is_paused() {
        return Err(AppError::ServiceUnavailable);
    }

    // Extract uploaded stego file
    let mut stego_bytes = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::BadRequest(format!("Failed to read multipart: {}", e))
    })? {
        if field.name() == Some("file") {
            let data = field.bytes().await.map_err(|e| {
                AppError::BadRequest(format!("Failed to read file bytes: {}", e))
            })?;
            stego_bytes = Some(data.to_vec());
            break;
        }
    }

    let stego_bytes = stego_bytes.ok_or_else(|| {
        AppError::BadRequest("No file field found in multipart data".to_string())
    })?;

    // Load stego image
    let stego_img = image::load_from_memory(&stego_bytes)
        .map_err(|e| AppError::BadRequest(format!("Invalid image file: {}", e)))?;

    // Extract secret
    let lsb_per_channel = state.config.stego.lsb_per_channel;
    let compress = state.config.stego.compress;

    let recovered = stego::extract(&stego_img, lsb_per_channel, compress)
        .map_err(|e| AppError::UnprocessableEntity(format!("Extraction failed: {}", e)))?;

    let recovered_size = recovered.len() as u64;
    let recovered_mime = stego::get_mime_type(&recovered).to_string();
    let recovered_b64 = BASE64.encode(&recovered);

    let request_id = Uuid::new_v4().to_string();

    // Record metrics
    let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
    state.metrics.record_request(&state.node_id, latency_ms, true);

    Ok(Json(ExtractResponse {
        request_id,
        recovered_size_bytes: recovered_size,
        recovered_mime,
        recovered_b64,
    }))
}

// ============================================================================
// Dataset Handler (for stress testing)
// ============================================================================

pub async fn dataset_handler(
    Path(index): Path<usize>,
) -> Result<impl IntoResponse, AppError> {
    if index >= 50 {
        return Err(AppError::NotFound);
    }

    // Generate synthetic image
    let img = stego::utils::generate_dataset_image(index, 800, 600);

    // Encode to PNG
    let mut png_bytes = Vec::new();
    img.write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)
        .map_err(|e| AppError::Internal(format!("PNG encoding failed: {}", e)))?;

    Ok((
        StatusCode::OK,
        [("Content-Type", "image/png")],
        png_bytes,
    ))
}

// ============================================================================
// Cluster Status Handler
// ============================================================================

pub async fn cluster_status_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ClusterStatus>, AppError> {
    let term = state.raft_node.get_term().await;
    
    // Get leader from Raft state - convert numeric ID to string
    let leader_id = state.raft_node.get_current_leader().await.map(|id| {
        match id {
            1 => "n1".to_string(),
            2 => "n2".to_string(),
            3 => "n3".to_string(),
            _ => format!("n{}", id),
        }
    });

    // Build node statuses
    let mut nodes = Vec::new();

    for node_config in &state.config.nodes {
        let is_current = node_config.id == state.node_id;
        let metrics = if is_current {
            state.metrics.get_metrics(&state.node_id)
        } else {
            // In production, query other nodes
            control_plane::NodeMetrics::default()
        };

        let role = if Some(&node_config.id) == leader_id.as_ref() {
            NodeRole::Leader
        } else {
            NodeRole::Follower
        };

        let healthy = if is_current {
            !state.is_paused()
        } else {
            // Check remote node health via HTTP
            let node_addr = format!("{}:{}", node_config.ip, node_config.http_port);
            control_plane::health_check(&node_addr).await
        };

        nodes.push(NodeStatus {
            id: node_config.id.clone(),
            ip: node_config.ip.clone(),
            http_port: node_config.http_port,
            raft_port: node_config.raft_port,
            role,
            healthy,
            cpu_pct: metrics.cpu_pct,
            mem_pct: metrics.mem_pct,
            qps_1m: metrics.qps_1m,
            p95_ms: metrics.p95_ms,
        });
    }

    Ok(Json(ClusterStatus {
        term,
        leader_id,
        nodes,
    }))
}

// ============================================================================
// Admin Handlers
// ============================================================================

#[derive(Deserialize)]
pub struct FailRequest {
    action: String, // "crash" or "pause"
}

pub async fn admin_fail_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FailRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    match req.action.as_str() {
        "crash" => {
            tracing::warn!("Node {} crashing by admin request", state.node_id);
            std::process::exit(1);
        }
        "pause" => {
            tracing::warn!("Node {} pausing by admin request", state.node_id);
            state.pause();
            Ok(Json(serde_json::json!({ "status": "paused" })))
        }
        _ => Err(AppError::BadRequest(format!(
            "Unknown action: {}",
            req.action
        ))),
    }
}

pub async fn admin_restore_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    tracing::info!("Node {} restoring by admin request", state.node_id);
    state.restore();
    Ok(Json(serde_json::json!({ "status": "restored" })))
}

// ============================================================================
// Health & Metrics Handlers
// ============================================================================

pub async fn health_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    if state.is_paused() {
        return Err(AppError::ServiceUnavailable);
    }
    Ok(Json(serde_json::json!({ "status": "healthy" })))
}

pub async fn metrics_handler() -> impl IntoResponse {
    // Return simple Prometheus-compatible metrics
    let body = "# HELP requests_total Total number of requests\n\
                # TYPE requests_total counter\n\
                requests_total 0\n";
    (StatusCode::OK, [("Content-Type", "text/plain")], body)
}

// ============================================================================
// Static File Handler
// ============================================================================

pub async fn serve_index() -> Html<&'static str> {
    Html(include_str!("../../../static/index.html"))
}

// ============================================================================
// Error Handling
// ============================================================================

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    NotFound,
    PayloadTooLarge { needed: u64, available: u64 },
    UnprocessableEntity(String),
    Internal(String),
    ServiceUnavailable,
    NotLeader(NotLeaderResponse),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotLeader(resp) => {
                let status = StatusCode::TEMPORARY_REDIRECT;
                let body = Json(resp);
                (status, body).into_response()
            }
            _ => {
                let (status, message) = match self {
                    AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
                    AppError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
                    AppError::PayloadTooLarge { needed, available } => (
                        StatusCode::PAYLOAD_TOO_LARGE,
                        format!(
                            "Payload too large: need {} bytes, available {} bytes",
                            needed, available
                        ),
                    ),
                    AppError::UnprocessableEntity(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
                    AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
                    AppError::ServiceUnavailable => {
                        (StatusCode::SERVICE_UNAVAILABLE, "Service paused".to_string())
                    }
                    AppError::NotLeader(_) => unreachable!(),
                };

                let body = Json(serde_json::json!({
                    "error": message,
                }));

                (status, body).into_response()
            }
        }
    }
}

// ============================================================================
// Raft RPC Handlers
// ============================================================================

pub async fn raft_append_entries_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<control_plane::AppendEntriesRequest>,
) -> Result<Json<control_plane::AppendEntriesResponse>, AppError> {
    // Update term if needed
    if req.term > state.raft_node.get_term().await {
        state.raft_node.advance_term(req.term).await;
    }

    // If request is from leader, update leader and role
    if req.term >= state.raft_node.get_term().await {
        state.raft_node.set_current_leader(Some(req.leader_id)).await;
        
        // If we're currently leader but received AppendEntries from another leader
        // with same or higher term, we must step down (only one leader per term)
        let our_node_id = state.raft_node.node_id();
        if state.raft_node.is_leader().await && req.leader_id != our_node_id {
            tracing::warn!(
                "Node {} stepping down: received AppendEntries from {} in term {}",
                state.node_id,
                req.leader_id,
                req.term
            );
            state.raft_node.set_follower().await;
        } else if !state.raft_node.is_leader().await {
            state.raft_node.set_follower().await;
        }
        
        // Record heartbeat to reset election timeout
        state.raft_node.record_heartbeat().await;
    }

    // Respond with current term and success
    let response = control_plane::AppendEntriesResponse {
        term: state.raft_node.get_term().await,
        success: true,
        conflict_opt: None,
    };

    tracing::debug!(
        "Node {} received AppendEntries from leader {} (term: {})",
        state.node_id,
        req.leader_id,
        req.term
    );

    Ok(Json(response))
}

pub async fn raft_request_vote_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<control_plane::RequestVoteRequest>,
) -> Result<Json<control_plane::RequestVoteResponse>, AppError> {
    // Use the new grant_vote logic which handles term updates and vote tracking
    let vote_granted = state.raft_node.grant_vote(req.candidate_id, req.term).await;

    let response = control_plane::RequestVoteResponse {
        term: state.raft_node.get_term().await,
        vote_granted,
    };

    Ok(Json(response))
}
