# 🌐 SETUP INSTRUCTIONS FOR DEVICE 2 AND DEVICE 3

## Overview

This guide explains how to clone the Phase-1 project on Device 2 and Device 3 and configure them for distributed deployment.

---

## 📋 Prerequisites

On **Device 2** and **Device 3**, you need:

### Essential Tools
1. **Rust** (stable toolchain) - Rust compiler and cargo package manager
2. **Git** - Version control system
3. **Build tools** - Compiler and development headers
4. **Network connectivity** - Ethernet/WiFi to other devices
5. **SSH or HTTPS access** - To clone from GitHub

### Optional but Recommended
- **curl** - For testing API endpoints
- **jq** - JSON query tool for pretty-printing API responses
- **tmux or screen** - Terminal multiplexing for running multiple nodes
- **nano or vim** - Text editors
- **Docker & Docker Compose** - If using containerized deployment
- **nginx** - Load balancer (can run on Device 1 or separate)
- **htop** - System monitoring

> **📖 For complete installation details**, see `INSTALL_DEPENDENCIES.md` in the repository root.

---

## 🚀 DEVICE 2 SETUP

### Step 1: Install System Dependencies

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y \
  build-essential \
  clang \
  libclang-dev \
  curl \
  git \
  jq \
  net-tools \
  netcat \
  tmux \
  htop \
  nano \
  pkg-config \
  libssl-dev
```

**macOS:**
```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install \
  rust \
  git \
  curl \
  jq \
  tmux \
  htop \
  nano \
  openssl \
  pkg-config
```

### Step 2: Install Rust (if not already installed)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version  # Verify installation
cargo --version
```

### Step 4: Clone the Repository

Replace `YOUR_GITHUB_USERNAME` with your actual GitHub username:

```bash
cd ~
git clone https://github.com/YOUR_GITHUB_USERNAME/phase1-steg-cluster.git
cd phase1-steg-cluster
```

**Verify clone succeeded:**
```bash
ls -la
```

You should see: `Cargo.toml`, `README.md`, `crates/`, `static/`, etc.

### Step 5: Verify All Tools Are Installed

**Run the verification script:**

```bash
# Check Rust
rustc --version
cargo --version

# Check Git
git --version

# Check build tools
gcc --version
clang --version

# Check utilities
curl --version
jq --version
nc -h

# Check for optional tools
which tmux && tmux -V || echo "tmux not installed"
which htop && htop -v || echo "htop not installed"
```

All should return version numbers. If any are missing, refer back to Step 1 for installation.

### Step 6: Set Static IP Address

Configure Device 2 with a static IP. This will be used in the cluster configuration.

**Example: Set to 172.20.10.3**

**Ubuntu/Debian (Netplan):**
```bash
sudo nano /etc/netplan/00-installer-config.yaml
```

Add or modify:
```yaml
network:
  version: 2
  ethernets:
    eth0:
      dhcp4: no
      addresses:
        - 172.20.10.3/24
      gateway4: 172.20.10.1
      nameservers:
        addresses: [8.8.8.8, 8.8.4.4]
```

Apply:
```bash
sudo netplan apply
ip addr  # Verify IP
```

**macOS:**
```bash
# System Preferences > Network > Advanced > TCP/IP
# Or use command line:
sudo ifconfig en0 inet 172.20.10.3 netmask 255.255.255.0
```

### Step 7: Configure Cluster Config

Edit the cluster configuration to match your network setup:

```bash
nano config/cluster.yaml
```

Make sure:
- Device 1 (this machine): n2 with IP 172.20.10.3
- Device 2: n1 with IP 172.20.10.2 (Device 1's IP)
- Device 3: n3 with IP 172.20.10.4

```yaml
cluster_name: "phase1"
nodes:
  - id: "n1"
    ip: "172.20.10.2"         # Device 1
    http_port: 8081
    raft_port: 5001
  - id: "n2"
    ip: "172.20.10.3"         # Device 2 (THIS MACHINE)
    http_port: 8082
    raft_port: 5002
  - id: "n3"
    ip: "172.20.10.4"         # Device 3
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

Save: `Ctrl+X`, then `Y`, then `Enter`

### Step 8: Test Build

```bash
cargo build --release
```

First build takes 5-10 minutes. Subsequent builds are faster.

**Verify binary exists:**
```bash
ls -lh target/release/server
```

### Step 9: Verify Connection to Other Nodes

Test network connectivity:

```bash
# Test Device 1 (172.20.10.2)
ping -c 1 172.20.10.2
nc -zv 172.20.10.2 5001  # Raft port

# Test Device 3 (172.20.10.4) - skip if not yet running
ping -c 1 172.20.10.4
```

### Step 10: Run Device 2 (Node n2)

```bash
export NODE_ID=n2
export CONFIG_PATH=./config/cluster.yaml
./bin/run-n2.sh
```

**Expected output:**
```
🚀 Starting node n2 on 172.20.10.3:8082
[INFO] Loading cover image from assets/cover.png
[INFO] Raft node initialized
[INFO] Server listening on 0.0.0.0:8082
```

---

## 🚀 DEVICE 3 SETUP

### Step 1-4: Same as Device 2

Install system dependencies, Rust, Git, and clone repository as shown above.

### Step 4: Set Static IP Address

Configure Device 3 with **172.20.10.4**

**Ubuntu/Debian (Netplan):**
```bash
sudo nano /etc/netplan/00-installer-config.yaml
```

```yaml
network:
  version: 2
  ethernets:
    eth0:
      dhcp4: no
      addresses:
        - 172.20.10.4/24
      gateway4: 172.20.10.1
      nameservers:
        addresses: [8.8.8.8, 8.8.4.4]
```

Apply:
```bash
sudo netplan apply
ip addr
```

### Step 5: Configure Cluster Config

Edit `config/cluster.yaml` (same as Device 2, but make sure n3 IP is 172.20.10.4):

```yaml
cluster_name: "phase1"
nodes:
  - id: "n1"
    ip: "172.20.10.2"         # Device 1
    http_port: 8081
    raft_port: 5001
  - id: "n2"
    ip: "172.20.10.3"         # Device 2
    http_port: 8082
    raft_port: 5002
  - id: "n3"
    ip: "172.20.10.4"         # Device 3 (THIS MACHINE)
    http_port: 8083
    raft_port: 5003
# ... rest same as Device 2
```

### Step 6: Build and Verify

```bash
cargo build --release
ls -lh target/release/server
```

### Step 7: Test Network Connectivity

```bash
ping -c 1 172.20.10.2  # Device 1
ping -c 1 172.20.10.3  # Device 2
```

### Step 8: Run Device 3 (Node n3)

```bash
export NODE_ID=n3
export CONFIG_PATH=./config/cluster.yaml
./bin/run-n3.sh
```

**Expected output:**
```
🚀 Starting node n3 on 172.20.10.4:8083
[INFO] Loading cover image from assets/cover.png
[INFO] Raft node initialized
[INFO] Server listening on 0.0.0.0:8083
```

---

## 🎬 COORDINATED STARTUP (All 3 Devices)

Once all three devices are ready, start them in order:

### Device 1 (172.20.10.2)
```bash
cd ~/phase1-steg-cluster
export NODE_ID=n1
./bin/run-n1.sh
```

**Wait 2-3 seconds, then:**

### Device 2 (172.20.10.3)
```bash
cd ~/phase1-steg-cluster
export NODE_ID=n2
./bin/run-n2.sh
```

**Wait 2-3 seconds, then:**

### Device 3 (172.20.10.4)
```bash
cd ~/phase1-steg-cluster
export NODE_ID=n3
./bin/run-n3.sh
```

**All three nodes should now be running and discovering each other.**

---

## ✅ VERIFY CLUSTER IS OPERATIONAL

### From Any Device:

```bash
# Test Device 1's HTTP port
curl -s http://172.20.10.2:8081/healthz | jq .

# Get cluster status
curl -s http://172.20.10.2:8081/cluster/status | jq .

# Check all three nodes are healthy
curl -s http://172.20.10.2:8081/cluster/status | jq '.nodes[] | {id, healthy, role}'
```

**Expected output:**
```json
{
  "id": "n1",
  "healthy": true,
  "role": "Leader"
}
{
  "id": "n2",
  "healthy": true,
  "role": "Follower"
}
{
  "id": "n3",
  "healthy": true,
  "role": "Follower"
}
```

### Access Web GUI:

From any computer on the network:
- http://172.20.10.2:8081
- http://172.20.10.3:8082
- http://172.20.10.4:8083

All three should show the same cluster status with 3 healthy nodes.

---

## 🧪 TEST EMBEDDING (Distributed)

### From Any Node:

```bash
# Test embed on Device 1
curl -X POST -F "file=@/path/to/image.png" http://172.20.10.2:8081/api/embed | jq .

# Should return:
# {
#   "request_id": "uuid",
#   "cover_info": {...},
#   "stego_image_b64": "...",
#   "notes": "steganography (no normal encryption)"
# }
```

---

## 📊 RUN STRESS TEST (Distributed)

### From Device 1 (or any device):

```bash
cargo run -p loadgen --release -- \
  --mode embed \
  --num-clients 30 \
  --reqs-per-client 200 \
  --server-list "http://172.20.10.2:8081,http://172.20.10.3:8082,http://172.20.10.4:8083"
```

This will:
- Create 30 concurrent clients
- Send 200 requests from each client (6000 total)
- Distribute across all 3 nodes via client-side LB
- Print summary statistics

**Expected output:**
```
=== Load Test Results ===
Total requests: 6000
Successful: 5994 (99.90%)
Failed: 6
Duration: 45.23s
Throughput: 132.65 req/s

Latency percentiles (ms):
  p50: 23.45
  p95: 67.89
  p99: 94.23
```

---

## 🔧 TROUBLESHOOTING

### Issue: "Connection refused" between devices

**Solution:**
1. Check firewall allows traffic on ports 8081-8083 (HTTP) and 5001-5003 (Raft)
2. Verify IPs are correct: `ip addr` on each device
3. Test connectivity: `nc -zv 172.20.10.2 5001`

### Issue: Node won't start ("Address already in use")

**Solution:**
```bash
# Kill existing process
pkill -f "cargo run -p server"

# Or find specific port
lsof -ti:8082 | xargs kill

# Then restart
./bin/run-n2.sh
```

### Issue: Build fails with "openssl not found"

**Solution (Ubuntu):**
```bash
sudo apt install libssl-dev pkg-config
```

**Solution (macOS):**
```bash
brew install openssl
export LDFLAGS="-L/usr/local/opt/openssl/lib"
export CPPFLAGS="-I/usr/local/opt/openssl/include"
cargo build --release
```

### Issue: Different nodes show different cluster status

**Solution:**
1. Wait 2-3 seconds for cluster to stabilize
2. Verify all nodes are running
3. Check firewall allows inter-node communication

---

## 🎓 PROFESSOR DEMO (Distributed)

### Part 1: Show All Three Nodes

From a laptop on the network:
```bash
curl http://172.20.10.2:8081/cluster/status | jq .
```

Show:
- All 3 nodes present
- One is "Leader"
- All are "Healthy"
- Metrics updating in real-time

### Part 2: Embed/Extract from Any Node

```bash
# Embed on Device 1
curl -X POST -F "file=@secret.png" http://172.20.10.2:8081/api/embed

# Extract on Device 2
curl -X POST -F "file=@stego.png" http://172.20.10.3:8082/api/extract

# Both work because load is balanced across nodes
```

### Part 3: Distributed Stress Test

From Device 1:
```bash
cargo run -p loadgen --release -- \
  --mode embed \
  --num-clients 20 \
  --reqs-per-client 200 \
  --server-list "http://172.20.10.2:8081,http://172.20.10.3:8082,http://172.20.10.4:8083"
```

Watch requests distributed across all 3 nodes.

### Part 4: Fail Node (Distributed)

```bash
# Pause Node 1
curl -X POST -H "Content-Type: application/json" \
  -d '{"action":"pause"}' \
  http://172.20.10.2:8081/admin/fail

# Observe:
# - Load balancer routes to remaining nodes
# - Node 1 still appears in cluster (marked "Down")
# - Stress test continues successfully
# - Throughput barely affected
```

### Part 5: Crash Node (Distributed)

```bash
# Crash Node 1 (kill process)
curl -X POST -H "Content-Type: application/json" \
  -d '{"action":"crash"}' \
  http://172.20.10.2:8081/admin/fail

# Observe:
# - Process exits
# - Other nodes elect new leader (if n1 was leader)
# - Web GUI shows node as "Down"
# - Services continue normally
# - Restore by restarting on Device 1: ./bin/run-n1.sh
```

---

## 📈 MONITORING COMMANDS

### Check Node Health

```bash
# Device 1
curl http://172.20.10.2:8081/healthz

# Device 2
curl http://172.20.10.3:8082/healthz

# Device 3
curl http://172.20.10.4:8083/healthz
```

### Get Metrics

```bash
curl http://172.20.10.2:8081/metrics
```

### Monitor Logs

On each device (in separate terminals):

```bash
# Device 1
export NODE_ID=n1
tail -f /tmp/phase1-n1.log

# Device 2
export NODE_ID=n2
tail -f /tmp/phase1-n2.log

# Device 3
export NODE_ID=n3
tail -f /tmp/phase1-n3.log
```

---

## 🛑 STOPPING NODES

### Gracefully Stop One Node

On Device 1:
```bash
pkill -f "cargo run -p server"
```

### Stop All Nodes

On each device:
```bash
pkill -f "cargo run -p server"
```

Or from one device (if SSH configured):
```bash
ssh user@172.20.10.2 "pkill -f 'cargo run -p server'"
ssh user@172.20.10.3 "pkill -f 'cargo run -p server'"
ssh user@172.20.10.4 "pkill -f 'cargo run -p server'"
```

---

## 📋 SUMMARY CHECKLIST

- [ ] Device 1: Clone, build, running on 172.20.10.2:8081
- [ ] Device 2: Clone, build, running on 172.20.10.3:8082
- [ ] Device 3: Clone, build, running on 172.20.10.4:8083
- [ ] All devices can ping each other
- [ ] Cluster status shows 3 healthy nodes
- [ ] Web GUI accessible from any device
- [ ] Embed/Extract works from any node
- [ ] Stress test distributes across all nodes
- [ ] Fault injection working (fail/restore)
- [ ] Leader re-election observable

---

## 🎉 You're All Set!

Your distributed Phase-1 Steganography System is now operational across 3 physical devices with:

✅ Automatic service discovery  
✅ Load balancing across nodes  
✅ Fault tolerance & fast recovery  
✅ Real-time monitoring  
✅ Distributed stress testing  

**Good luck with your demo! 🚀**
