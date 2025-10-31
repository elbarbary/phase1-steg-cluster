/// Background tasks for Raft consensus
/// Handles election monitoring, heartbeat transmission, and failover
use crate::network::{AppendEntriesRequest, RaftNetworkClient, RequestVoteRequest};
use crate::raft::RaftNode;
use crate::types::NodeRole;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

/// Start election monitoring task (for followers/candidates)
/// Monitors election timeout and triggers elections when leader fails
pub fn start_election_monitor(raft_node: Arc<RaftNode>, network: Arc<RaftNetworkClient>) {
    tokio::spawn(async move {
        let check_interval = Duration::from_millis(50); // Check every 50ms
        let mut interval = time::interval(check_interval);
        let mut last_election_attempt = std::time::Instant::now();
        let min_election_interval = Duration::from_millis(300); // Wait at least 300ms between election attempts
        let startup_time = std::time::Instant::now();
        let single_node_grace_period = Duration::from_secs(5); // After 5s, a single node becomes leader even without quorum
        let mut leader_probe_count = 0; // Track failed probes to current leader

        loop {
            interval.tick().await;

            // Skip if paused or unhealthy
            if !raft_node.is_healthy().await {
                continue;
            }

            // **NEW**: Proactively probe the leader to detect if it's dead
            // If we're a follower and we know who the leader is, try to reach it
            // If we can't reach it several times in a row, force election early
            if !raft_node.is_leader().await {
                if let Some(leader_id) = raft_node.get_current_leader().await {
                    // Try to contact leader
                    let req = RequestVoteRequest {
                        term: raft_node.get_term().await,
                        candidate_id: raft_node.node_id(),
                        last_log_index: 0,
                        last_log_term: 0,
                    };

                    // Try to reach leader to see if it's alive
                    let peers = raft_node.peers();
                    let leader_addr = peers.iter().find(|(id, _)| *id == leader_id).map(|(_, addr)| addr.clone());
                    
                    if let Some(leader_addr) = leader_addr {
                        let client = reqwest::Client::builder()
                            .timeout(std::time::Duration::from_millis(100))
                            .build()
                            .ok();
                        
                        if let Some(client) = client {
                            let url = format!("http://{}/raft/request-vote", leader_addr);
                            let probe_result = tokio::time::timeout(
                                Duration::from_millis(150),
                                client.post(&url).json(&req).send()
                            ).await;
                            
                            if probe_result.is_err() {
                                leader_probe_count += 1;
                                tracing::debug!(
                                    "Node {} failed to reach leader {} (attempt {})",
                                    raft_node.node_id(),
                                    leader_id,
                                    leader_probe_count
                                );
                                
                                // If leader unreachable 3+ times in a row, trigger election immediately
                                if leader_probe_count >= 3 && last_election_attempt.elapsed() > Duration::from_millis(100) {
                                    tracing::warn!(
                                        "Node {} detected leader {} is unreachable (3+ failed probes), triggering immediate election",
                                        raft_node.node_id(),
                                        leader_id
                                    );
                                    leader_probe_count = 0;
                                    last_election_attempt = std::time::Instant::now() - Duration::from_millis(500); // Force next check to proceed
                                }
                            } else {
                                leader_probe_count = 0; // Reset counter if we can reach leader
                            }
                        }
                    }
                }
            }

            // Check if we should start election (timeout-based or early detection)
            if raft_node.should_start_election().await {
                // Add backoff: don't attempt elections too frequently
                // (prevents election storms when isolated single node)
                if last_election_attempt.elapsed() < min_election_interval {
                    continue;
                }

                last_election_attempt = std::time::Instant::now();

                // Start election
                let new_term = raft_node.start_election().await;

                tracing::warn!(
                    "Node {} detected leader timeout or probed dead leader, starting election for term {}",
                    raft_node.node_id(),
                    new_term
                );

                // Broadcast RequestVote to all peers
                let req = RequestVoteRequest {
                    term: new_term,
                    candidate_id: raft_node.node_id(),
                    last_log_index: 0, // Simplified for Phase-2
                    last_log_term: 0,
                };

                let results = network.broadcast_request_vote(req).await;

                // Count votes
                let mut received_vote_count = 1; // Self vote
                for (peer_id, result) in results {
                    match result {
                        Ok(resp) => {
                            if resp.vote_granted {
                                tracing::info!(
                                    "Node {} received vote from {} for term {}",
                                    raft_node.node_id(),
                                    peer_id,
                                    new_term
                                );

                                received_vote_count += 1;

                                // Record vote and check for majority
                                if raft_node.record_vote(peer_id).await {
                                    // Won election!
                                    raft_node.set_leader().await;
                                    tracing::warn!(
                                        "🎉 Node {} won election and became LEADER for term {}",
                                        raft_node.node_id(),
                                        new_term
                                    );

                                    // Start sending heartbeats
                                    start_heartbeat_sender(raft_node.clone(), network.clone());
                                    return; // Exit election monitor
                                }
                            } else {
                                tracing::debug!(
                                    "Node {} vote denied by {} for term {}",
                                    raft_node.node_id(),
                                    peer_id,
                                    new_term
                                );
                            }
                        }
                        Err(e) => {
                            tracing::debug!(
                                "Node {} failed to get vote from {}: {}",
                                raft_node.node_id(),
                                peer_id,
                                e
                            );
                        }
                    }
                }

                // If single node and grace period has passed, become leader anyway
                // This allows a solo node to serve requests while waiting for cluster to form
                if received_vote_count == 1 && startup_time.elapsed() > single_node_grace_period {
                    tracing::warn!(
                        "Node {} is isolated (no peers responding) and grace period passed. Becoming LEADER anyway for term {}",
                        raft_node.node_id(),
                        new_term
                    );
                    raft_node.set_leader().await;
                    start_heartbeat_sender(raft_node.clone(), network.clone());
                    return; // Exit election monitor
                }

                // If we didn't win, log and wait for next timeout
                // (don't immediately retry — that causes election storms)
                tracing::debug!(
                    "Node {} did not win election for term {} ({}/{} votes), waiting for next timeout",
                    raft_node.node_id(),
                    new_term,
                    received_vote_count,
                    1 + raft_node.peers().len()
                );
            }
        }
    });
}

/// Start heartbeat sender task (for leaders only)
/// Sends periodic AppendEntries to all followers to maintain leadership
pub fn start_heartbeat_sender(raft_node: Arc<RaftNode>, network: Arc<RaftNetworkClient>) {
    tokio::spawn(async move {
        let heartbeat_interval = Duration::from_millis(raft_node.heartbeat_interval_ms());
        let mut interval = time::interval(heartbeat_interval);

        loop {
            interval.tick().await;

            // Only send heartbeats if we're the leader
            if !raft_node.is_leader().await {
                tracing::debug!(
                    "Node {} is no longer leader, stopping heartbeat sender",
                    raft_node.node_id()
                );
                return; // Exit task
            }

            // Skip if paused
            if !raft_node.is_healthy().await {
                continue;
            }

            let term = raft_node.get_term().await;
            let req = AppendEntriesRequest {
                term,
                leader_id: raft_node.node_id(),
                prev_log_index: 0,
                prev_log_term: 0,
                entries: Vec::new(), // Empty for heartbeat
                leader_commit: 0,
            };

            let results = network.broadcast_append_entries(req).await;

            let mut success_count = 0;
            let mut step_down = false;

            for (peer_id, result) in results {
                match result {
                    Ok(resp) => {
                        if resp.success {
                            success_count += 1;
                        } else if resp.term > term {
                            // Another node has higher term, step down
                            tracing::warn!(
                                "Node {} stepping down: peer {} has higher term {}",
                                raft_node.node_id(),
                                peer_id,
                                resp.term
                            );
                            step_down = true;
                            raft_node.advance_term(resp.term).await;
                        }
                    }
                    Err(e) => {
                        tracing::debug!(
                            "Node {} failed to send heartbeat to {}: {}",
                            raft_node.node_id(),
                            peer_id,
                            e
                        );
                    }
                }
            }

            if step_down {
                raft_node.set_follower().await;
                raft_node.set_current_leader(None).await;
                tracing::warn!(
                    "Node {} stepped down from leader to follower",
                    raft_node.node_id()
                );
                return; // Exit heartbeat sender
            }

            tracing::debug!(
                "Node {} sent heartbeat to {} peers (term {})",
                raft_node.node_id(),
                success_count,
                term
            );
        }
    });
}

/// Start all Raft background tasks
pub fn start_raft_tasks(raft_node: Arc<RaftNode>, network: Arc<RaftNetworkClient>) {
    // Leaders send heartbeats, followers monitor elections
    let node_id = raft_node.node_id();
    
    tokio::spawn(async move {
        // Small delay to ensure initialization completes
        tokio::time::sleep(Duration::from_millis(100)).await;

        let role = raft_node.get_role().await;
        
        match role {
            NodeRole::Leader => {
                tracing::info!("Node {} starting as Leader, enabling heartbeat sender", node_id);
                start_heartbeat_sender(raft_node.clone(), network.clone());
            }
            NodeRole::Follower | NodeRole::Candidate => {
                tracing::info!("Node {} starting as {:?}, enabling election monitor", node_id, role);
                start_election_monitor(raft_node.clone(), network.clone());
            }
            NodeRole::Learner => {
                tracing::info!("Node {} is Learner, no active tasks", node_id);
            }
        }
    });
}
