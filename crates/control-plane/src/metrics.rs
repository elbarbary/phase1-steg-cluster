use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::System;

const WINDOW_SIZE: usize = 60; // 60 seconds

#[derive(Debug, Clone)]
pub struct RequestRecord {
    pub timestamp: Instant,
    pub latency_ms: f64,
    pub success: bool,
}

#[derive(Debug, Clone, Default)]
pub struct NodeMetrics {
    pub cpu_pct: f32,
    pub mem_pct: f32,
    pub qps_1m: f32,
    pub p95_ms: f32,
}

pub struct MetricsCollector {
    requests: Arc<DashMap<String, Vec<RequestRecord>>>,
    system: Arc<parking_lot::Mutex<System>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        Self {
            requests: Arc::new(DashMap::new()),
            system: Arc::new(parking_lot::Mutex::new(system)),
        }
    }

    pub fn record_request(&self, node_id: &str, latency_ms: f64, success: bool) {
        let record = RequestRecord {
            timestamp: Instant::now(),
            latency_ms,
            success,
        };

        let mut entry = self.requests.entry(node_id.to_string()).or_insert_with(Vec::new);
        entry.push(record);

        // Keep only recent records
        let cutoff = Instant::now() - Duration::from_secs(WINDOW_SIZE as u64);
        entry.retain(|r| r.timestamp > cutoff);
    }

    pub fn get_metrics(&self, node_id: &str) -> NodeMetrics {
        // Note: sysinfo API changed; using simplified metrics for Phase-1
        // In production: use proper system metric collection
        
        let cpu_pct = 25.0; // Placeholder
        let mem_pct = 40.0; // Placeholder

        let (qps, p95) = if let Some(records) = self.requests.get(node_id) {
            let cutoff = Instant::now() - Duration::from_secs(60);
            let recent: Vec<_> = records.iter().filter(|r| r.timestamp > cutoff).collect();
            
            let qps_1m = recent.len() as f32 / 60.0;
            
            let mut latencies: Vec<f64> = recent.iter().map(|r| r.latency_ms).collect();
            latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            let p95_ms = if !latencies.is_empty() {
                let idx = ((latencies.len() as f64 * 0.95) as usize).min(latencies.len() - 1);
                latencies[idx] as f32
            } else {
                0.0
            };

            (qps_1m, p95_ms)
        } else {
            (0.0, 0.0)
        };

        NodeMetrics {
            cpu_pct,
            mem_pct,
            qps_1m: qps,
            p95_ms: p95,
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
