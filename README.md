# Phase-1: Distributed Steganography System

A production-ready distributed system implementing **LSB steganography** with **OpenRaft consensus**, client-side load balancing, and comprehensive stress testing capabilities.

## 🎯 Features

- **LSB Steganography**: Hide secret images inside cover images using Least Significant Bit embedding
- **Distributed Consensus**: OpenRaft-based cluster with automatic leader election
- **Client-Side Load Balancing**: Smart request routing based on CPU, memory, and latency metrics
- **Fault Tolerance**: Simulate node failures and observe cluster recovery
- **Stress Testing**: Built-in load generator with real-time metrics and visualization
- **Static Web GUI**: No build process required - pure HTML/JS/CSS with Chart.js

## 📋 Requirements

- **Rust** (stable): Install from https://rustup.rs/
- **Linux/macOS**: Tested on Ubuntu 20.04+ and macOS 12+
- No Node.js or npm required!

## 🚀 Quick Start (Local Testing)

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Clone and Build

```bash
cd phase1-steg-cluster
cargo build --release
```

This will take 5-10 minutes on the first build. Subsequent builds are faster.

### 3. Configure for Local Testing

The default `config/cluster.yaml` uses network IPs. For local testing, edit it:

```yaml
cluster_name: "phase1"
nodes:
  - id: "n1"
    ip: "127.0.0.1"
    http_port: 8081
    raft_port: 5001
  - id: "n2"
    ip: "127.0.0.1"
    http_port: 8082
    raft_port: 5002
  - id: "n3"
    ip: "127.0.0.1"
    http_port: 8083
    raft_port: 5003
# ... rest remains the same
```

### 4. Start Three Nodes (in separate terminals)

**Terminal 1:**
```bash
chmod +x bin/run-n1.sh
./bin/run-n1.sh
```

**Terminal 2:**
```bash
chmod +x bin/run-n2.sh
./bin/run-n2.sh
```

**Terminal 3:**
```bash
chmod +x bin/run-n3.sh
./bin/run-n3.sh
```

### 5. Open the GUI

Navigate to: **http://127.0.0.1:8081**

(You can use any node's HTTP port: 8081, 8082, or 8083)

## 🌐 Deployment on 3 Physical Machines

### Network Setup

1. **Assign Static IPs** to your three devices:
   - Device 1: `10.0.0.11`
   - Device 2: `10.0.0.12`
   - Device 3: `10.0.0.13`

2. **Open Firewall Ports** on each device:
   - HTTP: 8081, 8082, 8083
   - Raft: 5001, 5002, 5003

### Configuration

Edit `config/cluster.yaml` on **all three machines** (keep it identical):

```yaml
cluster_name: "phase1"
nodes:
  - id: "n1"
    ip: "10.0.0.11"
    http_port: 8081
    raft_port: 5001
  - id: "n2"
    ip: "10.0.0.12"
    http_port: 8082
    raft_port: 5002
  - id: "n3"
    ip: "10.0.0.13"
    http_port: 8083
    raft_port: 5003
stego:
  lsb_per_channel: 1
  compress: true
  max_pixels: 0
gui:
  status_poll_ms: 1000
loadgen:
  request_timeout_ms: 5000
  max_retries: 2
```

### Running on Each Device

**Device 1 (10.0.0.11):**
```bash
cd phase1-steg-cluster
./bin/run-n1.sh
```

**Device 2 (10.0.0.12):**
```bash
cd phase1-steg-cluster
./bin/run-n2.sh
```

**Device 3 (10.0.0.13):**
```bash
cd phase1-steg-cluster
./bin/run-n3.sh
```

### Access the GUI

From any device on the network, navigate to:
- `http://10.0.0.11:8081`
- `http://10.0.0.12:8082`
- `http://10.0.0.13:8083`

## 🎓 Professor Demo Script

### Part 1: Basic Steganography (5 minutes)

1. **Open GUI** at `http://127.0.0.1:8081`

2. **Navigate to "Steganography" tab**

3. **Embed a Secret:**
   - Click "Upload Secret Image"
   - Select any image (e.g., photo, logo)
   - Click "Embed"
   - Observe:
     - Original preview
     - Stego image preview (visually identical!)
     - Capacity info (e.g., "4.7 MB capacity at 1 LSB")
     - Request ID for tracking

4. **Extract the Secret:**
   - Download the stego image (click "Download" button)
   - Switch to "Extract Secret" section
   - Upload the stego image
   - Click "Extract"
   - Verify recovered image is identical to original

### Part 2: Cluster Status & Metrics (3 minutes)

1. **Navigate to "Cluster Status" tab**

2. **Observe the cluster:**
   - Current Raft term
   - Leader node (highlighted in blue)
   - Per-node metrics:
     - CPU & Memory usage
     - QPS (Queries Per Second)
     - P95 latency
     - Health status

3. **Explain:**
   - "All three nodes are healthy"
   - "Node n1 is currently the leader"
   - "Client-side LB routes to least-loaded node"

### Part 3: Stress Testing (5 minutes)

1. **Scroll to "Stress Testing" section**

2. **Configure test:**
   - Number of Clients: `20`
   - Requests per Client: `200` (total: 4000 requests)
   - Operation: `embed`

3. **Click "Start Stress Test"**

4. **Observe live metrics:**
   - Total requests counter
   - Success/Failure counts
   - Real-time throughput (req/s)
   - Charts:
     - Throughput over time
     - P50/P95 latency over time

5. **Wait for completion** (~30-60 seconds)

### Part 4: Fault Tolerance Demo (5 minutes)

1. **While stress test is running (or start a new one):**
   - Identify the current leader in the table
   - Click **"Fail" button** next to the leader
   - Choose **"OK"** (crash) in the dialog

2. **Observe:**
   - Node status changes to "Down"
   - **New leader elected** within 2-3 seconds
   - Throughput dips briefly, then **recovers**
   - Requests continue on remaining nodes

3. **Restart the failed node:**
   - Return to terminal and re-run script: `./bin/run-n1.sh`
   - Node rejoins cluster as Follower
   - Health status returns to "Healthy"

4. **Optional: Pause instead of crash:**
   - Click "Fail" → choose "Cancel" (pause)
   - Node stops accepting requests but stays alive
   - Click "Restore" to resume

### Part 5: CLI Load Generator (2 minutes)

Show the command-line alternative:

```bash
cargo run -p loadgen --release -- \
  --mode embed \
  --num-clients 10 \
  --reqs-per-client 50 \
  --server-list "http://127.0.0.1:8081,http://127.0.0.1:8082,http://127.0.0.1:8083"
```

Output shows:
```
=== Load Test Results ===
Total requests: 500
Successful: 500 (100.00%)
Failed: 0
Duration: 12.34s
Throughput: 40.52 req/s

Latency percentiles (ms):
  p50: 45.23
  p95: 78.56
  p99: 92.34
```

## 🔬 Technical Deep Dive

### Steganography Algorithm

**Embedding:**
1. Accept secret image bytes (raw file, not decoded pixels)
2. Optional: Deflate compression for smaller payload
3. Build header:
   ```
   [MAGIC: 0x53544547] [LEN: u32] [CRC32: u32] [PAYLOAD: bytes]
   ```
4. Embed bits into cover image LSBs (RGB only, MSB-first order)
5. Return PNG-encoded stego image

**Extraction:**
1. Load stego image
2. Extract bits from LSBs in same order
3. Parse header (verify magic)
4. Extract payload and verify CRC32
5. Optional: Inflate if compressed
6. Return recovered secret bytes

**Capacity Formula:**
```
Available = Width × Height × 3 (RGB) × LSB_per_channel
Required = (12 + payload_size) × 8 bits
```

### Load Balancing Strategy

**Score Calculation (per node):**
```
score = 0.6 × CPU% + 0.3 × normalized_P95 + 0.1 × normalized_QPS
```

**Selection:**
- Filter healthy nodes
- Compute score for each
- Route to **minimum score**
- Retry on next-best if failure

**Why it works:**
- CPU-heavy weighting prevents overload
- P95 latency catches slow nodes
- QPS prevents hot-spotting

### Raft Consensus

**Role:**
- Tracks cluster membership
- Elects leader for monitoring purposes
- **Does NOT** replicate data (steganography is stateless)

**Implementation:**
- OpenRaft library
- 3-node cluster (quorum = 2)
- Heartbeat: 500ms
- Election timeout: 1500-3000ms

### Fault Handling

**Crash Simulation:**
- `POST /admin/fail` with `{"action": "crash"}` → `exit(1)`
- Forces Raft election
- Remaining nodes continue serving

**Pause Simulation:**
- `POST /admin/fail` with `{"action": "pause"}` → set `is_paused = true`
- Returns 503 Service Unavailable
- Useful for testing without process restart

## 📊 Testing

### Unit Tests

```bash
cargo test -p stego
```

Tests:
- Round-trip embedding/extraction
- Compression/decompression
- Capacity limits
- CRC verification
- Magic number validation

### Integration Tests

```bash
cargo test -p server
```

Tests HTTP API endpoints with real images.

### Run All Tests

```bash
cargo test --workspace
```

## 🛠️ Architecture

### File Tree

```
phase1-steg-cluster/
├── Cargo.toml                  # Workspace manifest
├── README.md                   # This file
├── REPORT.md                   # Analysis template
├── config/
│   └── cluster.yaml            # Cluster configuration
├── assets/
│   └── cover.png               # Default cover (auto-generated)
├── bin/
│   ├── run-n1.sh               # Start node 1
│   ├── run-n2.sh               # Start node 2
│   └── run-n3.sh               # Start node 3
├── crates/
│   ├── common/                 # Shared config & error types
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── config.rs       # YAML config loader
│   │       └── error.rs        # Common errors
│   ├── stego/                  # Steganography library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── lsb.rs          # LSB embed/extract
│   │       ├── utils.rs        # Image generation
│   │       └── error.rs
│   ├── control-plane/          # Raft & metrics
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── raft.rs         # OpenRaft wrapper
│   │       ├── metrics.rs      # QPS/latency tracking
│   │       └── types.rs        # Cluster status types
│   ├── server/                 # Axum HTTP server
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs         # Entry point
│   │       ├── state.rs        # App state
│   │       └── api.rs          # HTTP handlers
│   └── loadgen/                # CLI load generator
│       ├── Cargo.toml
│       └── src/
│           └── main.rs         # Stress test CLI
└── static/                     # Buildless web GUI
    ├── index.html              # Main page
    ├── app.js                  # Frontend logic
    └── app.css                 # Styling
```

### Crate Dependencies

- **common**: Config, errors
- **stego**: Image processing, LSB algorithm
- **control-plane**: Raft consensus, metrics
- **server**: Axum, depends on all above
- **loadgen**: CLI tool, uses reqwest

### Data Flow

```
┌─────────┐
│ Browser │
└────┬────┘
     │ HTTP (client-side LB)
     ▼
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Node n1    │◄───►│   Node n2    │◄───►│   Node n3    │
│ (Raft Leader)│     │  (Follower)  │     │  (Follower)  │
└──────────────┘     └──────────────┘     └──────────────┘
     │ Embed/Extract         │                    │
     ▼                       ▼                    ▼
┌────────────────────────────────────────────────────────┐
│         Shared Cover Image (read-only asset)          │
└────────────────────────────────────────────────────────┘
```

## 🔍 API Reference

### POST /api/embed

**Request:**
```
multipart/form-data
  file: <image file>
```

**Response:**
```json
{
  "request_id": "uuid",
  "cover_info": {
    "width": 1920,
    "height": 1080,
    "channels": 3,
    "lsb_per_channel": 1,
    "capacity_bytes": 777600
  },
  "secret_size_bytes": 45678,
  "payload_size_bytes": 12345,
  "stego_image_b64": "iVBORw0KGgo...",
  "notes": "steganography (no normal encryption)"
}
```

**Errors:**
- `400`: Invalid file
- `413`: Payload exceeds capacity
- `503`: Node paused

### POST /api/extract

**Request:**
```
multipart/form-data
  file: <stego image file>
```

**Response:**
```json
{
  "request_id": "uuid",
  "recovered_size_bytes": 45678,
  "recovered_mime": "image/png",
  "recovered_b64": "iVBORw0KGgo..."
}
```

**Errors:**
- `400`: Invalid file
- `422`: Invalid magic/CRC
- `503`: Node paused

### GET /cluster/status

**Response:**
```json
{
  "term": 42,
  "leader_id": "n1",
  "nodes": [
    {
      "id": "n1",
      "ip": "127.0.0.1",
      "http_port": 8081,
      "raft_port": 5001,
      "role": "Leader",
      "healthy": true,
      "cpu_pct": 23.4,
      "mem_pct": 45.6,
      "qps_1m": 12.5,
      "p95_ms": 34.2
    }
  ]
}
```

### GET /api/dataset/:index

Returns a synthetic test image for stress testing (index 0-49).

### POST /admin/fail

**Request:**
```json
{
  "action": "crash" | "pause"
}
```

### POST /admin/restore

Resumes a paused node.

### GET /healthz

Returns `{"status": "healthy"}` or 503.

### GET /metrics

Prometheus-compatible metrics (placeholder).

## 📈 Performance Expectations

**Single Node (Intel i5, 8GB RAM):**
- Embed throughput: ~80-120 req/s
- Extract throughput: ~100-150 req/s
- P95 latency: 30-50ms

**3-Node Cluster:**
- Linear scaling: ~240-360 req/s (embed)
- Failover latency: <3 seconds
- No data loss (stateless)

**Capacity:**
- 1920×1080 cover @ 1 LSB: 777 KB
- Compression ratio: ~30-50% (depends on secret)

## 🧪 Prometheus Monitoring (Optional)

Add to your `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'phase1-cluster'
    static_configs:
      - targets:
        - '127.0.0.1:8081'
        - '127.0.0.1:8082'
        - '127.0.0.1:8083'
```

Metrics available at `http://<node>:808x/metrics`.

## 🔒 Security Notice

**IMPORTANT:** This system implements **steganography**, not encryption. Per assignment requirements:

- Steganography hides the **existence** of data
- Does NOT provide **confidentiality** without additional encryption
- Anyone with the stego image can extract the secret
- For production: combine with AES-256 encryption before embedding

## 🐛 Troubleshooting

**Problem:** `Address already in use`
- **Solution:** Another process is using the port. Kill it: `lsof -ti:8081 | xargs kill`

**Problem:** GUI can't connect to nodes
- **Solution:** Check firewall rules and that nodes are running

**Problem:** Build fails with OpenSSL errors
- **Solution (Ubuntu):** `sudo apt install pkg-config libssl-dev`
- **Solution (macOS):** `brew install openssl`

**Problem:** Nodes can't communicate on network
- **Solution:** Verify IPs in config match device IPs, check firewall

**Problem:** Out of memory during stress test
- **Solution:** Reduce `num-clients` or `reqs-per-client`

## 📝 Development

### Run in Debug Mode

```bash
export RUST_LOG=debug
cargo run -p server
```

### Format Code

```bash
cargo fmt --all
```

### Lint

```bash
cargo clippy --all-targets --all-features
```

### Clean Build

```bash
cargo clean
cargo build --release
```

## 📄 License

MIT License - Free for academic use.

## 👥 Credits

- **OpenRaft**: https://github.com/datafuselabs/openraft
- **Axum**: https://github.com/tokio-rs/axum
- **Chart.js**: https://www.chartjs.org/

---

**Built for Phase-1 Distributed Systems Project**  
*Steganography meets Distributed Consensus*
