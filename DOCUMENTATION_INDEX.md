# 📚 Complete Documentation Index

This is a comprehensive guide to all documentation files in the phase1-steg-cluster repository.

---

## 🎯 Quick Navigation

### I Need to...

| Goal | Document |
|------|----------|
| Get started immediately (single machine) | [QUICKSTART.md](#quickstart) |
| Setup distributed 3-device system | [DEVICE_SETUP.md](#device-setup) |
| Push code to GitHub and deploy | [PUSH_AND_DEPLOY.md](#push-and-deploy) |
| Install all dependencies | [INSTALL_DEPENDENCIES.md](#install-dependencies) |
| Understand system architecture | [README.md](#readme) |
| Learn about Phase-1 features | [README.md](#readme) |
| Learn about Phase-2 features | [PHASE2_COMPLETE.md](#phase2-complete) or [README.md](#readme) |
| Write a technical report | [REPORT.md](#report) |
| See deployment examples | [GITHUB_DEPLOYMENT.md](#github-deployment) |
| Run stress tests | [README.md](#readme) or [QUICKSTART.md](#quickstart) |
| Demo the system to professor | [README.md](#readme) - "Professor Demo" section |

---

## 📄 All Documentation Files

### 🔴 Essential Files (Read First)

#### [README.md](./README.md) {#readme}
**The main project documentation**

- **Features:** Complete list of Phase-1 and Phase-2 features
- **Quick Start:** Multiple setup options (Docker, Manual, Distributed)
- **Architecture:** System design and Raft consensus details
- **API Documentation:** All endpoints with examples
- **Stress Testing:** How to run load tests
- **Failure Testing:** Fault injection procedures
- **Professor Demo:** Step-by-step demo walkthrough
- **Monitoring:** Commands to check system status

**When to read:** First thing after cloning the repo

---

#### [QUICKSTART.md](./QUICKSTART.md) {#quickstart}
**Fast local setup (no network configuration needed)**

- **Fastest path to running:** 3 commands to start all 3 nodes locally
- **Local testing:** Run embed/extract operations
- **Stress testing:** Run load tests on local machine
- **Fault injection:** Pause/crash nodes and observe recovery
- **Typical time:** 10-15 minutes from clone to running

**When to use:** Development, testing on single machine

**Command:**
```bash
./bin/run-n1.sh  # Terminal 1
./bin/run-n2.sh  # Terminal 2
./bin/run-n3.sh  # Terminal 3
```

---

### 🟡 Setup & Installation Files

#### [DEVICE_SETUP.md](./DEVICE_SETUP.md) {#device-setup}
**Configure and run the system on 3 separate physical devices**

- **Step-by-step setup** for Device 2 and Device 3
- **Static IP configuration** using Netplan (Ubuntu) or ifconfig (macOS)
- **Network verification** commands
- **Coordinated startup** procedures for all 3 devices
- **Cluster verification** checks
- **Stress testing** across distributed nodes
- **Troubleshooting** guide

**Prerequisites:** Already cloned repo on all devices

**When to use:** Distributed system across multiple machines

**Time required:** 30-45 minutes for each device

---

#### [INSTALL_DEPENDENCIES.md](./INSTALL_DEPENDENCIES.md) {#install-dependencies}
**Complete dependency installation guide**

- **Required tools:** Rust, Git, build tools, OpenSSL
- **Recommended tools:** curl, jq, tmux, htop, netcat
- **Optional tools:** Docker, nginx, performance tools
- **Platform-specific:** Ubuntu/Debian and macOS instructions
- **All-in-one scripts:** Bash scripts to install everything
- **Troubleshooting:** Common installation issues and solutions
- **Verification:** Checklist to verify all tools are installed

**When to read:** Before starting any setup (on each device)

**Commands:**
```bash
# Ubuntu all-in-one
curl -fsSL https://example.com/ubuntu-install.sh | bash

# macOS all-in-one
curl -fsSL https://example.com/macos-install.sh | bash
```

---

#### [PUSH_AND_DEPLOY.md](./PUSH_AND_DEPLOY.md) {#push-and-deploy}
**Complete GitHub push and deployment workflow**

- **GitHub setup:** Create repository, generate tokens
- **Git configuration:** Set up credentials and SSH keys
- **Push workflow:** Step-by-step code publishing to GitHub
- **Clone on remote devices:** Pull code on Device 2 & 3
- **Distributed deployment:** Run system across 3 devices
- **Pull updates:** Keep all devices in sync
- **Security best practices:** Credential management, SSH keys
- **Troubleshooting:** Common push/pull errors and solutions

**When to use:** Publishing code, deploying to multiple devices

**Time required:** 5-10 minutes for initial setup, 30-45 minutes for device deployment

---

### 🟢 Deployment & GitHub Files

#### [GITHUB_DEPLOYMENT.md](./GITHUB_DEPLOYMENT.md) {#github-deployment}
**Complete workflow for GitHub integration**

- **Phase 1:** Push code to GitHub from Device 1
- **Phase 2:** Clone and setup on Device 2
- **Phase 3:** Clone and setup on Device 3
- **Phase 4:** Run coordinated distributed system
- **Testing:** Embed/extract across nodes
- **Stress testing:** Distributed load testing
- **Fault tolerance:** Test pause/restore on nodes
- **Documentation structure:** How docs are organized

**When to use:** First-time GitHub deployment

**Similar to:** PUSH_AND_DEPLOY.md (older version)

---

#### [GITHUB_PUSH.md](./GITHUB_PUSH.md) {#github-push}
**Quick reference for pushing to GitHub**

- **Repository creation**
- **Git configuration**
- **Push commands**
- **Verification**

**When to use:** Quick reminder of push steps

**Similar to:** Condensed version of PUSH_AND_DEPLOY.md

---

#### [GITHUB_QUICK_START.md](./GITHUB_QUICK_START.md) {#github-quick-start}
**Quick start with GitHub repository**

- **Prerequisites**
- **Setup steps**
- **Verification**

**When to use:** Very quick reference

---

#### [GITHUB_SUMMARY.md](./GITHUB_SUMMARY.md) {#github-summary}
**Summary of GitHub integration**

- **Overview of deployment**
- **Quick commands**
- **Key concepts**

**When to use:** High-level overview

---

### 🔵 Features & Documentation

#### [PHASE2_COMPLETE.md](./PHASE2_COMPLETE.md) {#phase2-complete}
**Detailed Phase-2 implementation documentation**

- **Automatic Leader Election:** Sub-300ms failover
- **Vote Counting:** Majority quorum logic
- **Heartbeat Transmission:** Active leader monitoring
- **Docker Deployment:** docker-compose setup
- **Nginx Load Balancer:** Automatic failover and health checks
- **Testing:** Automated failover tests
- **Metrics:** Election times, failover performance

**When to read:** Understanding Phase-2 Raft consensus

---

#### [COMPLETE.md](./COMPLETE.md) {#complete}
**Project completion status**

- **What's implemented**
- **What's tested**
- **Performance metrics**

**When to read:** Project status overview

---

#### [PROJECT_SUMMARY.md](./PROJECT_SUMMARY.md) {#project-summary}
**High-level project overview**

- **Goals and objectives**
- **Features implemented**
- **System architecture**
- **Deployment model**

**When to read:** Understanding project scope

---

#### [IMPLEMENTATION_SUMMARY.txt](./IMPLEMENTATION_SUMMARY.txt) {#implementation-summary}
**Technical implementation details**

- **Code structure**
- **Key components**
- **Database schema**
- **API design**

**When to read:** Deep dive into technical details

---

### 📝 Academic & Reporting

#### [REPORT.md](./REPORT.md) {#report}
**Academic report template**

- **Introduction:** Problem statement
- **Related Work:** Background and literature
- **Proposed Solution:** Architecture and design
- **Implementation:** Code and algorithms
- **Experiments:** Test results and metrics
- **Results:** Performance evaluation
- **Conclusion:** Summary and future work
- **References:** Bibliographic citations

**When to use:** Writing academic paper or formal report

**Sections to fill:** All marked with TODO or [FILL THIS IN]

---

### 🗂️ System Architecture Files

#### [docker-compose.yml](./docker-compose.yml)
**Docker Compose configuration**

- **3-node cluster:** stego-node1, stego-node2, stego-node3
- **nginx load balancer:** Reverse proxy with health checks
- **Networks:** Internal cluster network
- **Volumes:** Persistent RocksDB storage

**When to use:** Docker deployment

**Command:**
```bash
docker-compose up -d
```

---

#### [Dockerfile](./Dockerfile)
**Single node Docker image**

- **Base image:** Rust build environment
- **Build:** Cargo build process
- **Runtime:** Single server instance
- **Ports:** Exposed HTTP and Raft ports

---

#### [nginx.conf](./nginx.conf)
**Load balancer configuration**

- **Upstream:** 3 backend servers
- **Health checks:** Passive health monitoring
- **Load balancing:** Round-robin distribution
- **SSL:** Optional HTTPS configuration

---

#### [config/cluster.yaml](./config/cluster.yaml)
**Cluster configuration file**

- **Nodes:** n1, n2, n3 definitions
- **IPs and ports:** Per-node network settings
- **Steganography settings:** LSB parameters
- **GUI settings:** Web interface configuration
- **Load generator settings:** Test parameters

---

#### [.gitignore](./.gitignore)
**Git ignore patterns**

- **Compiled binaries:** Rust target/ directory
- **Dependencies:** cargo cache
- **Credentials:** Private keys, tokens
- **System files:** macOS .DS_Store, IDE files

---

### 🧪 Testing & Automation

#### [integration_tests.sh](./integration_tests.sh)
**Integration test script**

- **Runs:** All unit tests
- **Verifies:** Build succeeds
- **Checks:** API endpoints
- **Reports:** Test results

---

#### [test_failover.sh](./test_failover.sh)
**Failover testing script**

- **Pauses:** Leader node
- **Verifies:** New leader elected
- **Checks:** Election time
- **Restores:** Failed node

---

### 📦 Project Structure

#### [crates/](./crates/)
**Rust workspace crates**

- **common/:** Shared types and utilities
- **control-plane/:** Raft consensus and state machine
- **stego/:** Steganography algorithms (LSB)
- **server/:** HTTP server and API
- **loadgen/:** Load generator and stress testing

---

#### [bin/](./bin/)
**Startup scripts**

- **run-n1.sh:** Start node 1 (leader)
- **run-n2.sh:** Start node 2 (follower)
- **run-n3.sh:** Start node 3 (follower)

---

#### [static/](./static/)
**Web GUI files**

- **index.html:** Main dashboard
- **styles.css:** UI styling
- **app.js:** Frontend logic
- **Chart.js:** Real-time graphing

---

#### [assets/](./assets/)
**Static assets**

- **cover.png:** Default steganography cover image
- **README:** Asset descriptions

---

---

## 🗺️ Reading Paths

### Path 1: Quick Start (15 minutes)
1. Clone repo
2. Read [QUICKSTART.md](#quickstart)
3. Run 3 nodes locally
4. Test embed/extract API
5. Run stress test

### Path 2: Local Development (30 minutes)
1. Read [INSTALL_DEPENDENCIES.md](#install-dependencies)
2. Install all tools
3. Read [QUICKSTART.md](#quickstart)
4. Run distributed local testing
5. Study [README.md](#readme) architecture

### Path 3: Multi-Device Deployment (2-3 hours)
1. Device 1: Read [INSTALL_DEPENDENCIES.md](#install-dependencies)
2. Device 1: Read [PUSH_AND_DEPLOY.md](#push-and-deploy)
3. Device 1: Push code to GitHub
4. Device 2: Read [INSTALL_DEPENDENCIES.md](#install-dependencies)
5. Device 2: Read [DEVICE_SETUP.md](#device-setup)
6. Device 2: Clone and build
7. Device 3: Repeat steps 4-6
8. All devices: Run coordinated startup
9. Verify cluster status

### Path 4: Academic Report (1-2 hours)
1. Read [README.md](#readme) - Full system overview
2. Read [IMPLEMENTATION_SUMMARY.txt](#implementation-summary) - Technical details
3. Read [PHASE2_COMPLETE.md](#phase2-complete) - Advanced features
4. Use [REPORT.md](#report) template
5. Fill in sections with findings

### Path 5: Professor Demo (30 minutes)
1. Read [README.md](#readme) - "Professor Demo" section
2. Start 3-node system (follow [QUICKSTART.md](#quickstart))
3. Show cluster status and metrics
4. Demonstrate embed/extract
5. Run stress test
6. Show fault injection and recovery

---

## 🔍 Search by Topic

### Steganography
- [README.md](#readme) - Feature overview
- [IMPLEMENTATION_SUMMARY.txt](#implementation-summary) - Algorithm details
- `crates/stego/` - Source code

### Distributed Systems & Raft
- [README.md](#readme) - Architecture section
- [PHASE2_COMPLETE.md](#phase2-complete) - Leader election, voting
- [IMPLEMENTATION_SUMMARY.txt](#implementation-summary) - State transitions
- `crates/control-plane/` - Raft implementation

### Deployment
- [PUSH_AND_DEPLOY.md](#push-and-deploy) - Complete workflow
- [DEVICE_SETUP.md](#device-setup) - Multi-device setup
- [GITHUB_DEPLOYMENT.md](#github-deployment) - GitHub integration
- [docker-compose.yml](#docker-composeyml) - Docker setup

### API & HTTP
- [README.md](#readme) - API documentation
- `crates/server/` - Server implementation
- [QUICKSTART.md](#quickstart) - Example API calls

### Load Testing & Performance
- [README.md](#readme) - Stress testing section
- [QUICKSTART.md](#quickstart) - How to run tests
- `crates/loadgen/` - Load generator code

### Troubleshooting
- [INSTALL_DEPENDENCIES.md](#install-dependencies) - Installation issues
- [DEVICE_SETUP.md](#device-setup) - Network and config issues
- [README.md](#readme) - Troubleshooting section
- [PUSH_AND_DEPLOY.md](#push-and-deploy) - Git/GitHub issues

---

## 📊 Document Relationships

```
README.md (START HERE)
  ├─ QUICKSTART.md (quick local setup)
  ├─ INSTALL_DEPENDENCIES.md (before any setup)
  ├─ DEVICE_SETUP.md (for distributed)
  ├─ PUSH_AND_DEPLOY.md (for GitHub)
  └─ PHASE2_COMPLETE.md (advanced features)

GITHUB_DEPLOYMENT.md (alternative to PUSH_AND_DEPLOY.md)
  ├─ GITHUB_PUSH.md (quick reference)
  ├─ GITHUB_QUICK_START.md (very quick reference)
  └─ GITHUB_SUMMARY.md (high-level overview)

REPORT.md (academic writing)
  ├─ IMPLEMENTATION_SUMMARY.txt (fill in technical details)
  ├─ PROJECT_SUMMARY.md (fill in overview)
  └─ README.md (reference for facts)

PROJECT_SUMMARY.md, COMPLETE.md, PHASE2_COMPLETE.md (status)

Docker: docker-compose.yml, Dockerfile, nginx.conf
Config: config/cluster.yaml
Code: crates/ directory
Tests: integration_tests.sh, test_failover.sh
```

---

## ✅ Checklist: Which Docs to Read

- [ ] First visit? Start with [README.md](#readme)
- [ ] Setting up locally? Read [QUICKSTART.md](#quickstart)
- [ ] Setting up on multiple devices? Read [DEVICE_SETUP.md](#device-setup)
- [ ] Pushing to GitHub? Read [PUSH_AND_DEPLOY.md](#push-and-deploy)
- [ ] Installing dependencies? Read [INSTALL_DEPENDENCIES.md](#install-dependencies)
- [ ] Writing academic report? Read [REPORT.md](#report)
- [ ] Understanding Raft? Read [PHASE2_COMPLETE.md](#phase2-complete)
- [ ] Demo to professor? Read [README.md](#readme) - Demo section
- [ ] Deep technical dive? Read [IMPLEMENTATION_SUMMARY.txt](#implementation-summary)

---

## 🚀 Next Steps

1. **Read appropriate docs** based on your goal (see Checklist above)
2. **Install dependencies** using [INSTALL_DEPENDENCIES.md](#install-dependencies)
3. **Run the system** using [QUICKSTART.md](#quickstart) or [DEVICE_SETUP.md](#device-setup)
4. **Test functionality** with API calls from [README.md](#readme)
5. **Run stress tests** to verify performance
6. **Document your findings** using [REPORT.md](#report) template

---

## 📞 Document Update Log

| Date | File | Change |
|------|------|--------|
| 2024-10-28 | All | Initial comprehensive documentation |
| 2024-10-28 | INSTALL_DEPENDENCIES.md | Created with platform-specific instructions |
| 2024-10-28 | PUSH_AND_DEPLOY.md | Created with complete workflow |
| 2024-10-28 | DOCUMENTATION_INDEX.md | Created this file |

---

## 📧 Support

For questions about:
- **Setup issues:** See [INSTALL_DEPENDENCIES.md](#install-dependencies)
- **Configuration:** See [DEVICE_SETUP.md](#device-setup)
- **Git/GitHub:** See [PUSH_AND_DEPLOY.md](#push-and-deploy)
- **APIs:** See [README.md](#readme)
- **Features:** See [PHASE2_COMPLETE.md](#phase2-complete)
- **Architecture:** See [IMPLEMENTATION_SUMMARY.txt](#implementation-summary)

---

**Happy deploying! 🚀**
