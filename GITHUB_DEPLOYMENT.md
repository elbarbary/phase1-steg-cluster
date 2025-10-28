# 🚀 GITHUB DEPLOYMENT GUIDE - COMPLETE WORKFLOW

## 📋 Overview

This guide covers:
1. **Pushing code to GitHub** (from Device 1)
2. **Cloning on Device 2 & Device 3**
3. **Configuring each device** for the 3-node cluster
4. **Running the distributed system**

---

## 🎯 PHASE 1: PUSH TO GITHUB (Device 1 - Current Machine)

### 1️⃣ Create GitHub Repository

**On GitHub.com:**

1. Go to https://github.com/new
2. Enter:
   - Repository name: `phase1-steg-cluster`
   - Description: `Distributed Steganography with OpenRaft Consensus`
   - Visibility: Public
3. **❌ DO NOT** check "Initialize with README"
4. Click "Create repository"

**Copy the URL you see** (format: `https://github.com/YOUR_USERNAME/phase1-steg-cluster.git`)

### 2️⃣ Configure Git Locally

```bash
cd /home/youssef-mansour@auc.egy/dist/phase1-steg-cluster

# Configure your GitHub credentials
git config user.email "your.email@example.com"
git config user.name "Your Name"

# Verify git status
git status
# Should show: On branch master, nothing to commit, working tree clean
```

### 3️⃣ Add Remote and Push

**Replace `YOUR_USERNAME` with your actual GitHub username:**

```bash
# Add remote
git remote add origin https://github.com/YOUR_USERNAME/phase1-steg-cluster.git

# Verify remote
git remote -v
# Should show two lines with origin

# Push to GitHub
git push -u origin master
```

**When prompted:**
- Username: Your GitHub username
- Password: Your GitHub Personal Access Token (NOT your password!)

**If you don't have a token:**
1. Go to https://github.com/settings/tokens/new
2. Click "Generate new token"
3. Name: `phase1-push`
4. Select scope: `repo` (full control)
5. Click "Generate token"
6. Copy the token and paste it as password

### 4️⃣ Verify Push

```bash
# Check git log
git log --oneline
# Should show your commits

# Check remote
git remote -v
# Should show origin pointing to GitHub
```

Visit your GitHub repository in a browser:
```
https://github.com/YOUR_USERNAME/phase1-steg-cluster
```

✅ You should see all your files and folders on GitHub!

---

## 🌐 PHASE 2: SETUP DEVICE 2 (Second Machine)

### 📋 Prerequisites on Device 2

- Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Install Git: `sudo apt install git` (Ubuntu) or `brew install git` (macOS)
- Static IP address: 10.0.0.12

### 🔧 Clone Repository

```bash
# Navigate to home
cd ~

# Clone the project
git clone https://github.com/YOUR_USERNAME/phase1-steg-cluster.git

# Enter directory
cd phase1-steg-cluster

# Verify
ls -la
# Should see: Cargo.toml, README.md, config/, crates/, static/, etc.
```

### ⚙️ Configure for Device 2

**Set static IP to 10.0.0.12** (skip if already static):

**Ubuntu/Debian:**
```bash
sudo nano /etc/netplan/00-installer-config.yaml
```

Add:
```yaml
network:
  version: 2
  ethernets:
    eth0:
      dhcp4: no
      addresses:
        - 10.0.0.12/24
      gateway4: 10.0.0.1
      nameservers:
        addresses: [8.8.8.8, 8.8.4.4]
```

Apply:
```bash
sudo netplan apply
ip addr  # Verify
```

**Edit cluster config:**

```bash
nano config/cluster.yaml
```

Ensure it has:
```yaml
nodes:
  - id: "n1"
    ip: "10.0.0.11"         # Device 1
    http_port: 8081
    raft_port: 5001
  - id: "n2"
    ip: "10.0.0.12"         # Device 2 (THIS ONE)
    http_port: 8082
    raft_port: 5002
  - id: "n3"
    ip: "10.0.0.13"         # Device 3
    http_port: 8083
    raft_port: 5003
```

### 🔨 Build on Device 2

```bash
cargo build --release
# Takes 5-10 minutes first time

# Verify
ls -lh target/release/server
```

### 🚀 Run Device 2

```bash
export NODE_ID=n2
./bin/run-n2.sh
```

**Expected output:**
```
Starting node n2 on 10.0.0.12:8082
Loading cover image from assets/cover.png
Raft node initialized
Server listening on 0.0.0.0:8082
```

---

## 🌐 PHASE 3: SETUP DEVICE 3 (Third Machine)

### Same as Device 2, but:

**Static IP:** 10.0.0.13

**config/cluster.yaml:** n3 IP should be 10.0.0.13

**Run command:**
```bash
export NODE_ID=n3
./bin/run-n3.sh
```

---

## 🎬 COORDINATED STARTUP (All 3 Devices)

**Terminal on Device 1:**
```bash
cd ~/phase1-steg-cluster
export NODE_ID=n1
./bin/run-n1.sh
```

**Wait 2 seconds, then Terminal on Device 2:**
```bash
cd ~/phase1-steg-cluster
export NODE_ID=n2
./bin/run-n2.sh
```

**Wait 2 seconds, then Terminal on Device 3:**
```bash
cd ~/phase1-steg-cluster
export NODE_ID=n3
./bin/run-n3.sh
```

### ✅ Verify Cluster

From any device:
```bash
# Check Device 1
curl http://10.0.0.11:8081/cluster/status | jq .

# Should show:
# {
#   "term": X,
#   "leader_id": "n1" or "n2" or "n3",
#   "nodes": [
#     { "id": "n1", "healthy": true, "role": "Leader" },
#     { "id": "n2", "healthy": true, "role": "Follower" },
#     { "id": "n3", "healthy": true, "role": "Follower" }
#   ]
# }
```

### 🌐 Access Web GUI

From any computer on the network:
- http://10.0.0.11:8081
- http://10.0.0.12:8082
- http://10.0.0.13:8083

All three should show the same cluster status with 3 healthy nodes!

---

## 📊 TEST THE SYSTEM

### Test Embed/Extract

```bash
# Test embed on any node
curl -X POST -F "file=@test.png" http://10.0.0.11:8081/api/embed

# Test extract on any node
curl -X POST -F "file=@stego.png" http://10.0.0.12:8082/api/extract
```

### Run Distributed Stress Test

From any device:
```bash
cargo run -p loadgen --release -- \
  --mode embed \
  --num-clients 20 \
  --reqs-per-client 200 \
  --server-list "http://10.0.0.11:8081,http://10.0.0.12:8082,http://10.0.0.13:8083"
```

This will send 4000 requests distributed across all 3 nodes!

### Test Fault Tolerance

Pause Device 1:
```bash
curl -X POST -H "Content-Type: application/json" \
  -d '{"action":"pause"}' \
  http://10.0.0.11:8081/admin/fail
```

Check cluster status - n1 should be marked "Down" but service continues!

Restore Device 1:
```bash
curl -X POST http://10.0.0.11:8081/admin/restore
```

---

## 📚 DOCUMENTATION STRUCTURE

| Document | Purpose | For |
|----------|---------|-----|
| **README.md** | Complete system guide | All setup scenarios |
| **GITHUB_PUSH.md** | GitHub push instructions | Device 1 |
| **DEVICE_SETUP.md** | Configure Device 2 & 3 | Device 2 & 3 |
| **QUICKSTART.md** | Fast 3-command startup | Local testing |
| **REPORT.md** | Academic template | Presentation/submission |

---

## 🚨 QUICK TROUBLESHOOTING

### Git: "Permission denied"
→ Use HTTPS token or add SSH key to GitHub

### Build: "openssl not found"
→ Ubuntu: `sudo apt install libssl-dev pkg-config`
→ macOS: `brew install openssl`

### Network: "Connection refused"
→ Check firewall allows ports 8081-8083, 5001-5003
→ Verify IPs with `ip addr` on each device

### Node won't start: "Address already in use"
→ `pkill -f "cargo run -p server"` then retry

---

## ✅ COMPLETE CHECKLIST

- [ ] GitHub repository created
- [ ] Code pushed from Device 1 to GitHub
- [ ] Device 2: Cloned, built, running on 10.0.0.12:8082
- [ ] Device 3: Cloned, built, running on 10.0.0.13:8083
- [ ] All 3 devices running simultaneously
- [ ] Cluster status shows 3 healthy nodes
- [ ] Web GUI accessible from any device
- [ ] Embed/Extract works on all nodes
- [ ] Stress test distributes across all nodes
- [ ] Fault injection working (pause/restore)
- [ ] Leader re-election working (<3 seconds)

---

## 🎉 READY FOR DEMO!

Your distributed Phase-1 Steganography System is now:

✅ Version controlled with Git  
✅ Published on GitHub  
✅ Deployed on 3 physical devices  
✅ Running with load balancing  
✅ Fault tolerant and observable  

**Next Steps:**
1. Run the stress test to show throughput
2. Demonstrate fault tolerance (pause a node)
3. Show the web GUI with live metrics
4. Present the code architecture

**Good luck! 🚀**
