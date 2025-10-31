# ⚡ QUICK START - Deploy & Test

## 🚀 ONE-TIME SETUP (Do once per device)

On each device (n1, n2, n3 - sequential, not parallel):

```bash
cd ~/phase1-steg-cluster
./RESTART_CLUSTER.sh n1   # On Device 1
./RESTART_CLUSTER.sh n2   # On Device 2  (wait for n1 to finish)
./RESTART_CLUSTER.sh n3   # On Device 3  (wait for n2 to finish)
```

That's it! Cluster will auto-elect leader and start.

---

## ✅ VERIFY IT WORKS

```bash
# Check status
curl -s http://172.20.10.2:8081/cluster/status | jq '.nodes[] | {id, role, healthy}'

# Web UI: http://172.20.10.2:8081
```

Expected: One leader, two followers, all healthy.

---

## 🧪 QUICK TESTS

### Test 1: Single node becomes leader
```bash
# Stop n2 and n3 (or just run n1 alone)
./RESTART_CLUSTER.sh n1

# After ~5 seconds, should show as Leader
curl -s http://172.20.10.2:8081/cluster/status | jq '.nodes[0]'
```

### Test 2: Leader failure triggers new election
```bash
# All 3 running, then kill leader
pkill -f "target/release/server"

# Within 100ms, new leader elected
# Check logs: should show new leader
curl -s http://172.20.10.2:8081/cluster/status | jq '.nodes[] | {id, role}'
```

### Test 3: Stress test with failover
```bash
# Terminal 1: Start stress test
cargo run -p loadgen --release -- \
  --mode embed \
  --num-clients 20 \
  --reqs-per-client 200 \
  --server-list "http://172.20.10.2:8081,http://172.20.10.3:8082,http://172.20.10.4:8083"

# Terminal 2: While running, kill leader
pkill -f "target/release/server"

# Stress test should continue with <1% errors
```

---

## 🔍 WHAT TO CHECK IN LOGS

Good signs:
- ✅ "Node X won election and became LEADER"
- ✅ "Node X received vote from Y"
- ✅ "AppendEntries from leader"
- ✅ No ERROR or PANIC messages

Bad signs:
- ❌ "Election timeout" repeated infinitely
- ❌ "vote denied" from all peers
- ❌ "connection refused"

---

## ⏱️ KEY TIMEOUTS

| Event | Timeout | Notes |
|-------|---------|-------|
| Follower detects dead leader | ~100ms | Via proactive health checks |
| Election completes | 50-100ms | Election timeout randomized per node |
| New leader elected | ~200ms | Usually completes within 2 heartbeat intervals |
| Single node grace period | 5s | Isolated node becomes leader after 5s |
| Min election backoff | 300ms | Prevents election storms |

---

## 🆘 EMERGENCY COMMANDS

```bash
# Kill everything and restart fresh
pkill -9 cargo; pkill -9 server; sleep 2; rm -f data/node-*/LOCK

# Check which nodes are alive
ps aux | grep target/release/server

# Check ports are open
ss -tlnp | grep 808

# Clear stuck RocksDB
rm -rf data/node-1 data/node-2 data/node-3

# Full restart from scratch
git pull origin master
cargo build --release --bin server
./RESTART_CLUSTER.sh n1
```

---

## 💡 TIPS

1. **Always pull latest code first**: `git pull origin master`
2. **Start nodes sequentially**: Wait for each to finish before starting next
3. **Check logs for errors**: Look for WARN/ERROR/PANIC messages
4. **Test single node first**: Verify n1 becomes leader within 5s
5. **Then test 3-node cluster**: Should elect leader within 1-2s
6. **Finally test failover**: Kill leader while stress test runs

---

**Status**: ✅ Ready to deploy!
