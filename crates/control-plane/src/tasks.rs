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

        loop {
            interval.tick().await;

            // Skip if paused or unhealthy
            if !raft_node.is_healthy().await {
                continue;
            }

            // Check if we should start election
            if raft_node.should_start_election().await {
                // Start election
                let new_term = raft_node.start_election().await;

                tracing::warn!(
                    "Node {} detected leader timeout, starting election for term {}",
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

                // If we didn't win, continue monitoring (may retry in next term)
                tracing::info!(
                    "Node {} did not win election for term {}, continuing as candidate",
                    raft_node.node_id(),
                    new_term
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
