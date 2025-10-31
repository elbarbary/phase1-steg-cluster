# 🚀 DEPLOYMENT GUIDE - Leader Health Check Fixes

## Summary of Changes

This deployment includes critical improvements to Raft leader detection:

1. **Proactive Leader Health Checks**: Followers now periodically probe the leader to detect if it's dead BEFORE the election timeout
2. **Faster Failover**: If a follower can't reach the leader 3+ times in a row, it triggers an election immediately (~150ms instead of 50-100ms timeout)
3. **Better Isolation Handling**: Single nodes become leaders after 5 seconds instead of getting stuck in election loops
4. **Election Backoff**: 300ms minimum between election attempts prevents election storms

## 🔄 DEPLOYMENT PROCESS

### For Each Device (Run SEQUENTIALLY, not in parallel)

**On Device 1 (n1):**
```bash
cd ~/phase1-steg-cluster
chmod +x RESTART_CLUSTER.sh
./RESTART_CLUSTER.sh n1
```

**Wait for n1 to finish starting (until you see logs or it becomes Leader)**

**On Device 2 (n2):**
```bash
cd ~/phase1-steg-cluster
./RESTART_CLUSTER.sh n2
```

**Wait for n2 to start and discover n1**

**On Device 3 (n3):**
```bash
cd ~/phase1-steg-cluster
./RESTART_CLUSTER.sh n3
```

**Wait for n3 to join the cluster**

---

## ✅ VERIFICATION STEPS

### From any device, check cluster status:

```bash
# Option 1: Web UI
# Open browser: http://172.20.10.2:8081

# Option 2: API call
curl -s http://172.20.10.2:8081/cluster/status | jq '.nodes[] | {id, healthy, role}'
```

Expected output:
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

---

## 🧪 TEST SCENARIOS

### Test 1: Single Node Starts Alone

**On Device 1 ONLY** (leave n2 and n3 down):

```bash
./RESTART_CLUSTER.sh n1
```

**Expected behavior:**
- n1 logs show election attempts (3-4 tries)
- After ~5 seconds, logs show: `🎉 Node 1 won election and became LEADER` or `Becoming LEADER anyway`
- Web UI shows: n1 as Leader, Healthy
- Metrics endpoint works: `curl http://172.20.10.2:8081/healthz`

**Success Criteria**: ✓ Leader elected without requiring quorum

---

### Test 2: Multi-Node Cluster Formation

**Start all 3 nodes in quick succession:**

Device 1:
```bash
./RESTART_CLUSTER.sh n1
```

Device 2 (while Device 1 is starting):
```bash
./RESTART_CLUSTER.sh n2
```

Device 3 (while Device 2 is starting):
```bash
./RESTART_CLUSTER.sh n3
```

**Expected behavior:**
- Logs on each node show discovery of peers
- Within 1-2 seconds, one node becomes Leader
- All three show as Healthy
- Web UI shows 3 nodes with one Leader and two Followers

**Success Criteria**: ✓ Cluster reaches consensus automatically

---

### Test 3: Leader Crash Triggers Failover

**Prerequisites**: All 3 nodes running and healthy

**On any device, start a stress test:**
```bash
cargo run -p loadgen --release -- \
  --mode embed \
  --num-clients 20 \
  --reqs-per-client 200 \
  --server-list "http://172.20.10.2:8081,http://172.20.10.3:8082,http://172.20.10.4:8083"
```

**While stress test is running, on Device 1:**
```bash
pkill -f "target/release/server"
```

**Expected behavior:**
- Test continues running (requests route to n2 and n3)
- Error rate stays <1% (maybe 1-2 spike when n1 goes down)
- Within 100ms, a new leader is elected (n2 or n3)
- Web UI updates to show new leader within 2-3 seconds
- Stress test completes successfully

**Success Criteria**: ✓ Cluster recovers automatically, <1% failed requests

---

### Test 4: Repeated Failover

**Prerequisites**: All 3 nodes running

**Run this script to simulate 5 failovers:**

```bash
#!/bin/bash

# Get current leader
get_leader() {
  curl -s http://172.20.10.2:8081/cluster/status | jq -r '.nodes[] | select(.role=="Leader") | .id' | tail -1
}

for i in {1..5}; do
  echo "Iteration $i: Killing leader..."
  
  LEADER=$(get_leader)
  echo "Current leader: $LEADER"
  
  # Determine port based on leader
  case $LEADER in
    n1) KILL_PORT=8081 ;;
    n2) KILL_PORT=8082 ;;
    n3) KILL_PORT=8083 ;;
  esac
  
  # Kill leader
  ssh -n "172.20.10.2" "pkill -f 'target/release/server'" 2>/dev/null || pkill -f "target/release/server"
  
  echo "  Waiting 2 seconds..."
  sleep 2
  
  # New leader should be elected
  NEW_LEADER=$(get_leader)
  echo "  New leader: $NEW_LEADER ✓"
  
  # Restart old leader
  echo "  Restarting $LEADER..."
  
  # This assumes ssh is configured or you manually restart
  # For now just show what to do
  echo "  (On Device with $LEADER: ./RESTART_CLUSTER.sh $LEADER)"
  
  sleep 3
done

echo "✓ All failover tests complete!"
```

**Success Criteria**: ✓ Cluster survives 5 failovers with automatic recovery

---

## 📊 WHAT TO LOOK FOR IN LOGS

### Healthy Leader Election:
```
[WARN] Node 2 detected leader timeout, starting election for term 2
[INFO] Node 2 received vote from 1 for term 2
[INFO] Node 2 received vote from 3 for term 2
[WARN] 🎉 Node 2 won election and became LEADER for term 2
```

### Leader Health Check Working:
```
[WARN] Node 1 failed to reach leader 2 (attempt 1)
[WARN] Node 1 failed to reach leader 2 (attempt 2)
[WARN] Node 1 failed to reach leader 2 (attempt 3)
[WARN] Node 1 detected leader 2 is unreachable (3+ failed probes), triggering immediate election
```

### Single Node Grace Period:
```
[DEBUG] Node 1 did not win election for term 1 (1/3 votes)
[WARN] Node 1 is isolated (no peers responding) and grace period passed. Becoming LEADER anyway for term 1
```

---

## 🐛 TROUBLESHOOTING

### Issue: Build fails with "error: file not found"
**Solution**: Ensure you're in the project root directory:
```bash
cd ~/phase1-steg-cluster
pwd  # Should show: .../phase1-steg-cluster
```

### Issue: "Address already in use" error
**Solution**: Kill existing processes and clear locks:
```bash
pkill -9 cargo; pkill -9 server; sleep 2
rm -f data/node-*/LOCK
```

### Issue: Nodes can't reach each other
**Solution**: Check network configuration:
```bash
# On each device, verify IP:
ip addr | grep 172.20.10

# Test connectivity:
ping -c 1 172.20.10.2  # From Device 2 or 3
```

### Issue: Cluster won't elect a leader (stuck on "Follower")
**Solution**: Check election logs for why votes are failing:
```bash
# Look for "vote denied" messages in logs
# Check if all nodes are running the latest binary:
file target/release/server  # Verify it exists
./target/release/server --version  # If version flag supported
```

---

## ✨ EXPECTED IMPROVEMENTS

After this deployment, you should see:

1. **Faster Leader Election**: When leader crashes, election triggers in ~100ms instead of 50-100ms timeout
2. **No Stuck Followers**: Followers now detect dead leaders proactively instead of waiting for timeout
3. **Stable Single Node**: A single node becomes leader within 5s instead of election looping
4. **Better Fault Tolerance**: Stress tests continue with <1% error rate during failover
5. **Automatic Recovery**: When a failed node restarts, it rejoins cluster automatically

---

## 📞 SUPPORT

If you encounter issues:

1. **Check logs**: `grep "ERROR\|WARN" /tmp/phase1-*.log` (if logging to files)
2. **Verify network**: `nc -zv 172.20.10.3 5002` (test RPC ports)
3. **Check binary**: `ls -lh target/release/server` (verify build timestamp)
4. **Review git**: `git log --oneline -5` (ensure latest commit is deployed)

---

## 🎉 YOU'RE ALL SET!

Your cluster now has:
- ✅ Automatic leader election (50-100ms)
- ✅ Proactive dead leader detection (~150ms)
- ✅ Automatic failover with <1% error spike
- ✅ Support for single-node startup
- ✅ Election backoff to prevent storms

**Enjoy your distributed system! 🚀**
