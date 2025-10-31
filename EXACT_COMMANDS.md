# EXACT COMMANDS TO RUN

## STEP 1: On Your Local Machine (Where You Are Now)

```bash
# Navigate to project
cd "/home/youssef-mansour@auc.egy/dist/phase1-steg-cluster"

# Stage all changes
git add .

# Commit with message
git commit -m "feat: implement Raft-based leader routing with dynamic failover

- Add getClusterLeaderAndNodes() for dynamic leader discovery
- Update selectBestNode() to route all requests through cluster leader
- Scale concurrent workers to match numClients (removes 100 worker bottleneck)
- Add node distribution tracking for load balancing verification
- Browser automatically discovers new leader if current one fails
- All 3 servers now handle distributed load correctly"

# Push to GitHub
git push origin master
```

**Expected Output:**
```
[master abc1234] feat: implement Raft-based leader routing
 4 files changed, 850 insertions(+)
...
To github.com:elbarbary/phase1-steg-cluster.git
   abc1234  master -> master
```

---

## STEP 2: On Server 1 (172.20.10.2)

Copy and paste this EXACT command (one line):

```bash
ssh user@172.20.10.2 "cd /path/to/phase1-steg-cluster && git pull origin master && systemctl restart phase1-steg"
```

**Or do it manually step by step:**

```bash
# Open SSH connection
ssh user@172.20.10.2

# Inside server 1:
cd /path/to/phase1-steg-cluster
git pull origin master
systemctl restart phase1-steg

# Exit server
exit
```

**Expected Output:**
```
Updating abc1234..def5678
Fast-forward
 static/app.js  |  150 ++++++++++++++++++
 static/index.html  |    5 ++++
 3 files changed, 155 insertions(+)
 
● phase1-steg.service - Phase 1 Steg Cluster
   Loaded: loaded (/etc/systemd/system/phase1-steg.service; enabled; vendor preset: enabled)
   Active: active (running) since ...
```

---

## STEP 3: On Server 2 (172.20.10.3)

Copy and paste this EXACT command (one line):

```bash
ssh user@172.20.10.3 "cd /path/to/phase1-steg-cluster && git pull origin master && systemctl restart phase1-steg"
```

**Or manually:**

```bash
ssh user@172.20.10.3

# Inside server 2:
cd /path/to/phase1-steg-cluster
git pull origin master
systemctl restart phase1-steg

exit
```

---

## STEP 4: On Server 3 (172.20.10.6)

Copy and paste this EXACT command (one line):

```bash
ssh user@172.20.10.6 "cd /path/to/phase1-steg-cluster && git pull origin master && systemctl restart phase1-steg"
```

**Or manually:**

```bash
ssh user@172.20.10.6

# Inside server 3:
cd /path/to/phase1-steg-cluster
git pull origin master
systemctl restart phase1-steg

exit
```

---

## STEP 5: Verify All Running

Run this from your local machine:

```bash
curl http://172.20.10.2:8081/healthz
curl http://172.20.10.3:8082/healthz
curl http://172.20.10.6:8083/healthz
```

**Expected Output:**
```
{"status":"healthy"}
{"status":"healthy"}
{"status":"healthy"}
```

Or check cluster status:

```bash
curl http://172.20.10.2:8081/cluster/status | jq '.leader_id, .nodes[] | {id: .id, healthy: .healthy}'
```

---

## ALL COMMANDS IN ONE SECTION (Copy & Paste Ready)

### Local Machine - Commit & Push

```bash
cd "/home/youssef-mansour@auc.egy/dist/phase1-steg-cluster"
git add .
git commit -m "feat: Raft-based leader routing"
git push origin master
```

### Server 1 (One liner)

```bash
ssh user@172.20.10.2 "cd /path/to/phase1-steg-cluster && git pull origin master && systemctl restart phase1-steg"
```

### Server 2 (One liner)

```bash
ssh user@172.20.10.3 "cd /path/to/phase1-steg-cluster && git pull origin master && systemctl restart phase1-steg"
```

### Server 3 (One liner)

```bash
ssh user@172.20.10.6 "cd /path/to/phase1-steg-cluster && git pull origin master && systemctl restart phase1-steg"
```

### Verify

```bash
curl http://172.20.10.2:8081/healthz && \
curl http://172.20.10.3:8082/healthz && \
curl http://172.20.10.6:8083/healthz
```

---

## IMPORTANT NOTES

1. **Replace `/path/to/` with actual path**
   - Example: `/home/user/phase1-steg-cluster` or wherever your project is
   - Ask system admin if you don't know

2. **Replace `user` with your SSH username**
   - Could be: `root`, `ubuntu`, `ec2-user`, `deploy`, etc.
   - Ask if unsure

3. **Replace service name if different**
   - We assumed: `phase1-steg`
   - Check with: `systemctl list-units --type=service | grep steg`

4. **If systemctl doesn't work:**
   - Try: `pkill -f "cargo run" && cargo run --release --bin server &`
   - Or: `docker restart phase1-steg-n1` (if using Docker)

---

## QUICK CHECKLIST

Before running commands:
- [ ] You're in the correct directory: `/home/youssef-mansour@auc.egy/dist/phase1-steg-cluster`
- [ ] You have SSH access to all 3 servers
- [ ] You know the actual `/path/to/phase1-steg-cluster` on servers
- [ ] You know the SSH username for servers

After running commands:
- [ ] `git push` succeeded (no errors)
- [ ] All 3 servers showed "Fast-forward" or "Already up to date"
- [ ] All 3 servers restarted successfully
- [ ] All 3 healthz endpoints return `{"status":"healthy"}`

---

## TROUBLESHOOTING

### "SSH: command not found"
Use your SSH key:
```bash
ssh -i /path/to/key.pem user@172.20.10.2 "..."
```

### "git pull rejected"
```bash
# On server:
git fetch origin
git reset --hard origin/master
```

### "systemctl: command not found"
```bash
# Try this instead on server:
pkill -f "cargo run"
sleep 2
cargo run --release --bin server &
```

### "Permission denied"
```bash
# Your user may not have restart permission, try:
sudo systemctl restart phase1-steg
```

### "curl: connection refused"
```bash
# Server might not have restarted yet, wait 10 seconds:
sleep 10
curl http://172.20.10.2:8081/healthz
```

---

## EXPECTED TOTAL TIME

- Commit & push: 1 minute
- Server 1 (git pull + restart): 1 minute
- Server 2 (git pull + restart): 1 minute
- Server 3 (git pull + restart): 1 minute
- Verify: 1 minute

**Total: ~5-6 minutes**

---

Done! Run the commands and you're all set! 🚀
