# 🚀 GITHUB & MULTI-DEVICE DEPLOYMENT - QUICK START

## 📍 YOU ARE HERE (Device 1)

Everything is ready! Here's what to do next:

---

## ⏱️ STEP 1: PUSH TO GITHUB (5 minutes)

### 1. Create GitHub Repository

Go to: https://github.com/new

Fill in:
- Repository name: `phase1-steg-cluster`
- Description: `Distributed Steganography with OpenRaft Consensus`
- Visibility: **Public**
- ❌ DO NOT initialize with README

Click "Create repository"

### 2. Push Your Code

```bash
cd /home/youssef-mansour@auc.egy/dist/phase1-steg-cluster

# Add GitHub remote (replace YOUR_USERNAME)
git remote add origin https://github.com/YOUR_USERNAME/phase1-steg-cluster.git

# Push code
git push -u origin master
```

**When asked for password:** Use your GitHub Personal Access Token (NOT your password)

**Need a token?**
1. Go: https://github.com/settings/tokens/new
2. Name: `phase1-push`
3. Scopes: Check `repo`
4. Generate and copy token
5. Paste as password

### ✅ Verify on GitHub

Visit: `https://github.com/YOUR_USERNAME/phase1-steg-cluster`

You should see all your files! ✅

---

## 💻 STEP 2: GIVE LINK TO DEVICE 2 & 3

Share this URL with Device 2 and Device 3:

```
https://github.com/YOUR_USERNAME/phase1-steg-cluster.git
```

---

## 📥 FOR DEVICE 2 (SECOND MACHINE)

Run these commands ON THE SECOND MACHINE:

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Git (if needed)
sudo apt install git  # Ubuntu
# or: brew install git  # macOS

# Set static IP to 10.0.0.12 (if needed)
# See DEVICE_SETUP.md for detailed instructions

# Clone the repository
cd ~
git clone https://github.com/YOUR_USERNAME/phase1-steg-cluster.git
cd phase1-steg-cluster

# Edit config for Device 2
nano config/cluster.yaml
# Make sure n2 has IP: 10.0.0.12 and http_port: 8082

# Build
cargo build --release

# Run Device 2
export NODE_ID=n2
./bin/run-n2.sh
```

**Expected output:**
```
Starting node n2 on 10.0.0.12:8082
Raft node initialized (term=0, role=Follower)
Server listening on 0.0.0.0:8082
```

---

## 📥 FOR DEVICE 3 (THIRD MACHINE)

Same as Device 2, but:

```bash
# Clone
cd ~
git clone https://github.com/YOUR_USERNAME/phase1-steg-cluster.git
cd phase1-steg-cluster

# Set static IP to 10.0.0.13 in config
nano config/cluster.yaml

# Build
cargo build --release

# Run Device 3
export NODE_ID=n3
./bin/run-n3.sh
```

---

## 🎬 START ALL THREE NODES TOGETHER

### Device 1 (THIS MACHINE):
```bash
cd ~/phase1-steg-cluster
export NODE_ID=n1
./bin/run-n1.sh
```

### Device 2 (wait 2 seconds, then):
```bash
cd ~/phase1-steg-cluster
export NODE_ID=n2
./bin/run-n2.sh
```

### Device 3 (wait 2 seconds, then):
```bash
cd ~/phase1-steg-cluster
export NODE_ID=n3
./bin/run-n3.sh
```

### Verify Cluster is Running

From any machine:
```bash
curl http://10.0.0.11:8081/cluster/status | jq .
```

Should show 3 nodes: 1 Leader, 2 Followers, all Healthy ✅

---

## 🌐 ACCESS WEB GUI

From any computer on the network:
- http://10.0.0.11:8081
- http://10.0.0.12:8082
- http://10.0.0.13:8083

You should see the same cluster status with 3 healthy nodes!

---

## 🧪 TEST EVERYTHING

### Test Embed (from any node):
```bash
curl -X POST -F "file=@test.png" http://10.0.0.11:8081/api/embed | jq .
```

### Test Distributed Stress:
```bash
cargo run -p loadgen --release -- \
  --mode embed \
  --num-clients 20 \
  --reqs-per-client 200 \
  --server-list "http://10.0.0.11:8081,http://10.0.0.12:8082,http://10.0.0.13:8083"
```

### Test Fault Tolerance:
```bash
# Pause Device 1
curl -X POST -H "Content-Type: application/json" \
  -d '{"action":"pause"}' \
  http://10.0.0.11:8081/admin/fail

# Check status - should show Device 1 as "Down"
curl http://10.0.0.12:8082/cluster/status | jq .

# Restore Device 1
curl -X POST http://10.0.0.11:8081/admin/restore
```

---

## 📚 DOCUMENTATION FOR REFERENCE

| File | For What |
|------|----------|
| **README.md** | Complete guide |
| **GITHUB_DEPLOYMENT.md** | Full GitHub workflow |
| **DEVICE_SETUP.md** | Device 2 & 3 setup details |
| **GITHUB_PUSH.md** | Troubleshooting GitHub |
| **QUICKSTART.md** | Quick reference |

---

## ⚠️ IMPORTANT NETWORK NOTES

**Firewall:** Make sure these ports are open on all machines:
- HTTP: 8081, 8082, 8083
- Raft: 5001, 5002, 5003

**Static IPs:** Must be set:
- Device 1: 10.0.0.11
- Device 2: 10.0.0.12
- Device 3: 10.0.0.13

**Same config.yaml:** All three machines MUST have identical cluster.yaml config

---

## 🚨 IF SOMETHING GOES WRONG

### Node won't start (Address already in use):
```bash
pkill -f "cargo run -p server"
```

### Can't connect between devices:
```bash
ping 10.0.0.11  # from Device 2 or 3
nc -zv 10.0.0.11 5001  # test port
```

### GitHub push fails:
```bash
# Check remote
git remote -v

# Update if needed
git remote set-url origin https://github.com/YOUR_USERNAME/phase1-steg-cluster.git

# Try again
git push -u origin master
```

---

## ✅ FINAL CHECKLIST

- [ ] Repository created on GitHub
- [ ] Code pushed from Device 1
- [ ] Device 2 cloned and building
- [ ] Device 3 cloned and building
- [ ] All static IPs set (10.0.0.11, 12, 13)
- [ ] All three nodes running
- [ ] Cluster status shows 3 healthy nodes
- [ ] Web GUI accessible from all devices
- [ ] Embed/Extract works
- [ ] Stress test runs successfully

---

## 🎉 YOU'RE DONE!

Your system is now:
✅ Version controlled on GitHub  
✅ Deployed on 3 physical devices  
✅ Running distributed system  
✅ Ready for demo!

**Enjoy your presentation! 🚀**
