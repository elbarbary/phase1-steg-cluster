# 🚀 Complete Push & Deployment Guide

This document provides complete instructions for pushing code to GitHub and deploying to remote devices.

---

## 📋 PART 1: SETUP GITHUB REPOSITORY (First Time Only)

### Step 1: Create Repository on GitHub

1. Go to https://github.com/new
2. Fill in:
   - **Repository name:** `phase1-steg-cluster`
   - **Description:** `Distributed Steganography with OpenRaft Consensus and Automatic Failover`
   - **Visibility:** Public (so others can clone)
3. **Important:** Do NOT check "Initialize this repository with:"
4. Click "Create repository"

You should see a page with:
```
…or push an existing repository from the command line
git remote add origin https://github.com/YOUR_USERNAME/phase1-steg-cluster.git
git branch -M main
git push -u origin main
```

Copy the URL for the next step.

### Step 2: Get GitHub Personal Access Token

1. Go to https://github.com/settings/tokens
2. Click "Generate new token" → "Generate new token (classic)"
3. Set:
   - **Note:** `phase1-steg-cluster-push`
   - **Expiration:** 30 days (or as needed)
   - **Scopes:** Check `repo` (full control of private repositories)
4. Click "Generate token"
5. **Copy and save the token** (you won't see it again!)

---

## 🔐 PART 2: CONFIGURE GIT CREDENTIALS (First Time Only)

### On Device 1:

```bash
cd /home/youssef-mansour@auc.egy/dist/phase1-steg-cluster

# Configure Git user info
git config user.email "your.email@example.com"
git config user.name "Your Full Name"

# Verify configuration
git config --list | grep user
```

### Setup GitHub Authentication

**Option A: Store Token (Recommended for personal devices)**

```bash
# This stores token in a plain text file (use only on trusted devices)
git config --global credential.helper store

# Git will prompt for username/token on first push
# After that, credentials are cached
```

**Option B: SSH Key (More secure)**

```bash
# Generate SSH key (if you don't have one)
ssh-keygen -t ed25519 -C "your.email@example.com"
# Press Enter for default location
# Press Enter for no passphrase (or set one for security)

# Copy SSH key to GitHub
# Go to https://github.com/settings/keys
# Click "New SSH key"
# Paste the contents of ~/.ssh/id_ed25519.pub
# Click "Add SSH key"

# Test SSH connection
ssh -T git@github.com
# Should output: "Hi YOUR_USERNAME! You've successfully authenticated..."
```

---

## 🔄 PART 3: PUSH TO GITHUB

### Step 1: Add Remote Repository

```bash
cd /home/youssef-mansour@auc.egy/dist/phase1-steg-cluster

# Add the remote (use HTTPS if using token, SSH if using key)
git remote add origin https://github.com/YOUR_USERNAME/phase1-steg-cluster.git

# Verify remote was added
git remote -v
# Should show:
# origin  https://github.com/YOUR_USERNAME/phase1-steg-cluster.git (fetch)
# origin  https://github.com/YOUR_USERNAME/phase1-steg-cluster.git (push)
```

### Step 2: Check Status

```bash
git status
# Should show: "On branch master, nothing to commit, working tree clean"
# If not, commit any uncommitted changes first:
git add .
git commit -m "Your commit message"
```

### Step 3: Push to GitHub

```bash
# Push all commits to GitHub
git push -u origin master

# First time, you may be prompted for credentials:
# Username: YOUR_GITHUB_USERNAME
# Password: YOUR_PERSONAL_ACCESS_TOKEN (or leave blank if using SSH)
```

### Step 4: Verify Push

```bash
# Check git log
git log --oneline -5
# Compare local and remote
git status
# Should show: "On branch master, Your branch is up to date with 'origin/master'."

# Visit your GitHub repo in browser
# https://github.com/YOUR_USERNAME/phase1-steg-cluster
```

---

## 📦 PART 4: CLONE ON DEVICE 2

### Step 1: Install Dependencies

```bash
# See INSTALL_DEPENDENCIES.md for complete instructions
# Quick install (Ubuntu):
sudo apt update
sudo apt install -y \
  build-essential clang libclang-dev curl git jq \
  net-tools netcat tmux htop nano pkg-config libssl-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Step 2: Clone Repository

```bash
cd ~
git clone https://github.com/YOUR_USERNAME/phase1-steg-cluster.git
cd phase1-steg-cluster

# Verify clone
ls -la
# Should show: Cargo.toml, README.md, crates/, config/, etc.

git log --oneline -3
# Should show your commits
```

### Step 3: Build Project

```bash
# First build takes 5-10 minutes
cargo build --release

# Verify build
ls -lh target/release/server
```

### Step 4: Verify Network

```bash
# Test connection to Device 1 (172.20.10.2)
ping -c 1 172.20.10.2
nc -zv 172.20.10.2 5001  # Test Raft port
```

---

## 📦 PART 5: CLONE ON DEVICE 3

### Same as Device 2

Follow steps 1-4 from Part 4 on Device 3 as well.

---

## 🎬 PART 6: RUN DISTRIBUTED SYSTEM

### Terminal 1 (Device 1):

```bash
cd ~/phase1-steg-cluster
export NODE_ID=n1
./bin/run-n1.sh
# Wait for: "Server listening on 0.0.0.0:8081"
```

### Terminal 2 (Device 2) - Wait 2-3 seconds:

```bash
cd ~/phase1-steg-cluster
export NODE_ID=n2
./bin/run-n2.sh
# Wait for: "Server listening on 0.0.0.0:8082"
```

### Terminal 3 (Device 3) - Wait 2-3 seconds:

```bash
cd ~/phase1-steg-cluster
export NODE_ID=n3
./bin/run-n3.sh
# Wait for: "Server listening on 0.0.0.0:8083"
```

### Verify Cluster

```bash
# From any device
curl http://172.20.10.2:8081/cluster/status | jq

# Should show all 3 nodes healthy, one as leader
```

---

## 🔄 PART 7: PULL UPDATES FROM GITHUB

### On Any Device:

```bash
cd ~/phase1-steg-cluster

# Check if updates available
git fetch origin
git status
# If shows "Your branch is behind...", run:
git pull origin master

# If changes to Rust code, rebuild
cargo build --release
```

### Update All Devices

If Device 2 and 3 need updates:

```bash
# On Device 2
cd ~/phase1-steg-cluster
git pull origin master
cargo build --release

# On Device 3
cd ~/phase1-steg-cluster
git pull origin master
cargo build --release
```

---

## 💾 PART 8: PUSH NEW CHANGES

### After Making Changes on Device 1:

```bash
cd ~/phase1-steg-cluster

# See what changed
git status
git diff

# Stage changes
git add <file1> <file2> ...
# Or add all:
git add .

# Commit with descriptive message
git commit -m "Clear description of changes

Optionally include more details here.
- Feature 1
- Bug fix 1
- Performance improvement 1"

# Push to GitHub
git push origin master

# Verify
git log --oneline -1
```

### If Others Have Made Changes:

```bash
# Before pushing, pull latest
git pull origin master

# If there are conflicts, resolve them
# Then:
git add <resolved-files>
git commit -m "Merge latest changes"
git push origin master
```

---

## 📊 COMPLETE PUSH WORKFLOW (All Steps)

### 1️⃣ On Device 1 (First Time Setup)

```bash
cd /home/youssef-mansour@auc.egy/dist/phase1-steg-cluster

# Configure git
git config user.email "your.email@example.com"
git config user.name "Your Name"
git config --global credential.helper store

# Add remote
git remote add origin https://github.com/YOUR_USERNAME/phase1-steg-cluster.git

# Verify remote
git remote -v

# Push to GitHub
git push -u origin master
# Enter credentials when prompted

# Verify push
git log --oneline -1
```

### 2️⃣ Create Remote Devices

On Device 2:
```bash
# Install dependencies (see INSTALL_DEPENDENCIES.md)
# Clone from GitHub
git clone https://github.com/YOUR_USERNAME/phase1-steg-cluster.git
cd phase1-steg-cluster

# Configure static IP
# Edit config/cluster.yaml for Device 2

# Build
cargo build --release
```

On Device 3: (Same as Device 2)

### 3️⃣ Run Distributed System

Terminal 1: `./bin/run-n1.sh`
Terminal 2: `./bin/run-n2.sh`
Terminal 3: `./bin/run-n3.sh`

### 4️⃣ Make Changes and Push

```bash
# On Device 1, after making changes
git add <files>
git commit -m "Your message"
git push origin master

# On Device 2 & 3
git pull origin master
cargo build --release
# Restart nodes if needed
```

---

## 🔐 SECURITY BEST PRACTICES

### Credential Management

```bash
# Never commit credentials!
echo ".env" >> .gitignore
echo "secrets.toml" >> .gitignore
echo "*.pem" >> .gitignore
git add .gitignore
git commit -m "Add secrets to gitignore"
```

### SSH Key Protection

```bash
# Generate key with passphrase
ssh-keygen -t ed25519 -C "your.email@example.com"
# When prompted, enter a passphrase (remember it!)

# Add key to ssh-agent so you don't need to type passphrase each time
ssh-add ~/.ssh/id_ed25519
```

### Token Management

```bash
# Use environment variables instead of storing in files
export GITHUB_TOKEN="your_token_here"

# Or use Git credential manager
git config --global credential.helper manager
```

---

## 🧪 TESTING THE WORKFLOW

### Test 1: Push Changes

```bash
# On Device 1
echo "# Test Update" >> README.md
git add README.md
git commit -m "Test push workflow"
git push origin master

# Verify on GitHub
# https://github.com/YOUR_USERNAME/phase1-steg-cluster
# Should see the change in README.md
```

### Test 2: Pull on Other Devices

```bash
# On Device 2
cd ~/phase1-steg-cluster
git pull origin master
# Should show the README.md change

# On Device 3
cd ~/phase1-steg-cluster
git pull origin master
# Should show the README.md change
```

### Test 3: Rebuild After Pull

```bash
# On Device 2
cargo build --release

# On Device 3
cargo build --release
```

---

## 🚨 TROUBLESHOOTING

### Issue: "fatal: 'origin' does not appear to be a 'git' repository"

**Solution:**
```bash
# Check remote is configured
git remote -v

# If empty, add it:
git remote add origin https://github.com/YOUR_USERNAME/phase1-steg-cluster.git
```

### Issue: "Permission denied (publickey)"

**Solution (using HTTPS instead):**
```bash
# Change remote to HTTPS
git remote set-url origin https://github.com/YOUR_USERNAME/phase1-steg-cluster.git

# Try push again
git push origin master
```

**Solution (if using SSH):**
```bash
# Generate SSH key
ssh-keygen -t ed25519

# Add to SSH agent
ssh-add ~/.ssh/id_ed25519

# Add public key to GitHub (https://github.com/settings/keys)
cat ~/.ssh/id_ed25519.pub
```

### Issue: "error: src refspec master does not match any"

**Solution:**
```bash
# No commits yet, make a commit first
git add .
git commit -m "Initial commit"
git push origin master
```

### Issue: "Your branch is ahead of origin by X commits"

**Solution:**
```bash
# Push your commits
git push origin master

# Verify
git status
```

### Issue: "Merge conflict" after pull

**Solution:**
```bash
# View conflicts
git status
# Open files with <<<< and >>>> markers
# Fix conflicts manually

# Stage resolved files
git add <resolved-files>

# Commit merge
git commit -m "Resolve merge conflicts"

# Push
git push origin master
```

---

## ✅ COMPLETE CHECKLIST

### First Time Setup
- [ ] GitHub repository created
- [ ] Personal access token generated
- [ ] Git credentials configured on Device 1
- [ ] Remote added to local repo
- [ ] Code pushed to GitHub
- [ ] GitHub repo is public and visible

### Device 2 Setup
- [ ] Dependencies installed
- [ ] Repository cloned from GitHub
- [ ] Project built successfully
- [ ] Static IP configured (172.20.10.3)
- [ ] config/cluster.yaml updated for Device 2
- [ ] Network connectivity to Device 1 verified

### Device 3 Setup
- [ ] Dependencies installed
- [ ] Repository cloned from GitHub
- [ ] Project built successfully
- [ ] Static IP configured (172.20.10.4)
- [ ] config/cluster.yaml updated for Device 3
- [ ] Network connectivity verified

### System Running
- [ ] All 3 nodes start successfully
- [ ] Cluster status shows 3 healthy nodes
- [ ] One node elected as leader
- [ ] API endpoints respond correctly
- [ ] Web GUI accessible from any device
- [ ] Load balancing working across nodes

### Updates & Maintenance
- [ ] Changes can be pushed from Device 1
- [ ] Changes can be pulled on Device 2 & 3
- [ ] System can be rebuilt after updates
- [ ] Nodes can be restarted without manual reconfiguration

---

## 🎉 NEXT STEPS

After successful deployment:

1. **Test the system:**
   ```bash
   cargo run -p loadgen --release -- \
     --mode embed \
     --num-clients 20 \
     --reqs-per-client 200 \
     --server-list "http://172.20.10.2:8081,http://172.20.10.3:8082,http://172.20.10.4:8083"
   ```

2. **Monitor cluster:**
   ```bash
   watch -n 1 'curl -s http://172.20.10.2:8081/cluster/status | jq .'
   ```

3. **Test failover:**
   ```bash
   # Pause Device 1
   curl -X POST http://172.20.10.2:8081/admin/fail

   # Check cluster status
   curl http://172.20.10.2:8081/cluster/status | jq .
   ```

---

## 📞 SUPPORT

For issues, see:
- `INSTALL_DEPENDENCIES.md` - Dependency troubleshooting
- `DEVICE_SETUP.md` - Network and configuration issues
- `README.md` - Architecture and feature overview
- `QUICKSTART.md` - Quick local setup
