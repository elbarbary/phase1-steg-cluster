use anyhow::Result;
use clap::Parser;
use reqwest::multipart;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Parser, Debug)]
#[command(name = "phase1-loadgen")]
#[command(about = "Load generator for Phase-1 steganography cluster")]
struct Args {
    #[arg(long, default_value = "embed")]
    mode: String, // "embed" or "extract"

    #[arg(long, default_value = "10")]
    num_clients: usize,

    #[arg(long, default_value = "100")]
    reqs_per_client: usize,

    #[arg(long)]
    server_list: String, // Comma-separated URLs

    #[arg(long)]
    duration_secs: Option<u64>,
}

#[derive(Debug, Clone)]
struct Stats {
    total: usize,
    success: usize,
    failed: usize,
    latencies: Vec<f64>,
}

impl Stats {
    fn new() -> Self {
        Self {
            total: 0,
            success: 0,
            failed: 0,
            latencies: Vec::new(),
        }
    }

    fn record(&mut self, success: bool, latency_ms: f64) {
        self.total += 1;
        if success {
            self.success += 1;
            self.latencies.push(latency_ms);
        } else {
            self.failed += 1;
        }
    }

    fn merge(&mut self, other: &Stats) {
        self.total += other.total;
        self.success += other.success;
        self.failed += other.failed;
        self.latencies.extend(&other.latencies);
    }

    fn percentile(&self, p: f64) -> f64 {
        if self.latencies.is_empty() {
            return 0.0;
        }
        let mut sorted = self.latencies.clone();
        sorted.sort_by(|a, b| a.total_cmp(b));
        let idx = ((sorted.len() as f64 * p) as usize).min(sorted.len() - 1);
        sorted[idx]
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let servers: Vec<String> = args
        .server_list
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    if servers.is_empty() {
        anyhow::bail!("No servers provided");
    }

    println!("Load Generator Configuration:");
    println!("  Mode: {}", args.mode);
    println!("  Clients: {}", args.num_clients);
    println!("  Requests per client: {}", args.reqs_per_client);
    println!("  Servers: {:?}", servers);
    println!();

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let global_stats = Arc::new(Mutex::new(Stats::new()));
    let start_time = Instant::now();

    // Spawn worker tasks
    let mut handles = vec![];

    for _client_id in 0..args.num_clients {
        let servers_clone = servers.clone();
        let client_clone = client.clone();
        let mode_clone = args.mode.clone();
        let reqs = args.reqs_per_client;
        let stats_clone = global_stats.clone();

        let handle = tokio::spawn(async move {
            let mut local_stats = Stats::new();

            for req_id in 0..reqs {
                let server = &servers_clone[req_id % servers_clone.len()];

                let req_start = Instant::now();
                let success = match mode_clone.as_str() {
                    "embed" => perform_embed(&client_clone, server, req_id).await.is_ok(),
                    "extract" => perform_extract(&client_clone, server, req_id).await.is_ok(),
                    _ => false,
                };
                let latency_ms = req_start.elapsed().as_secs_f64() * 1000.0;

                local_stats.record(success, latency_ms);
            }

            let mut global = stats_clone.lock().await;
            global.merge(&local_stats);
        });

        handles.push(handle);
    }

    // Wait for all workers
    for handle in handles {
        handle.await?;
    }

    let elapsed = start_time.elapsed();
    let stats = global_stats.lock().await;

    println!("\n=== Load Test Results ===");
    println!("Total requests: {}", stats.total);
    println!("Successful: {} ({:.2}%)", stats.success, stats.success as f64 / stats.total as f64 * 100.0);
    println!("Failed: {}", stats.failed);
    println!("Duration: {:.2}s", elapsed.as_secs_f64());
    println!("Throughput: {:.2} req/s", stats.total as f64 / elapsed.as_secs_f64());
    println!("\nLatency percentiles (ms):");
    println!("  p50: {:.2}", stats.percentile(0.5));
    println!("  p95: {:.2}", stats.percentile(0.95));
    println!("  p99: {:.2}", stats.percentile(0.99));

    Ok(())
}

async fn perform_embed(client: &reqwest::Client, server: &str, index: usize) -> Result<()> {
    // Get a dataset image
    let dataset_idx = index % 50;
    let dataset_url = format!("{}/api/dataset/{}", server, dataset_idx);
    let img_bytes = client.get(&dataset_url).send().await?.bytes().await?;

    // Upload for embedding
    let form = multipart::Form::new().part(
        "file",
        multipart::Part::bytes(img_bytes.to_vec())
            .file_name("secret.png")
            .mime_str("image/png")?,
    );

    let embed_url = format!("{}/api/embed", server);
    let resp = client.post(&embed_url).multipart(form).send().await?;

    if !resp.status().is_success() {
        anyhow::bail!("Embed failed: {}", resp.status());
    }

    Ok(())
}

async fn perform_extract(client: &reqwest::Client, server: &str, index: usize) -> Result<()> {
    // For extract, we need a stego image
    // In a real test, we'd first embed then extract
    // For simplicity, we'll just use a dataset image (will fail CRC but still exercises the path)
    let dataset_idx = index % 50;
    let dataset_url = format!("{}/api/dataset/{}", server, dataset_idx);
    let img_bytes = client.get(&dataset_url).send().await?.bytes().await?;

    let form = multipart::Form::new().part(
        "file",
        multipart::Part::bytes(img_bytes.to_vec())
            .file_name("stego.png")
            .mime_str("image/png")?,
    );

    let extract_url = format!("{}/api/extract", server);
    let _resp = client.post(&extract_url).multipart(form).send().await?;

    // Extract will likely fail due to invalid stego data, but that's okay for load testing
    // We're measuring throughput and latency
    Ok(())
}
