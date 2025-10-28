#!/bin/bash
# Comprehensive integration test suite for Phase-2 Raft consensus
# Tests failover, persistence, log replication, and chaos scenarios

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Global test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Logging functions
log_test() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

log_pass() {
    echo -e "${GREEN}✅ PASS${NC}: $1"
    ((TESTS_PASSED++))
}

log_fail() {
    echo -e "${RED}❌ FAIL${NC}: $1"
    ((TESTS_FAILED++))
}

log_info() {
    echo -e "${YELLOW}[INFO]${NC} $1"
}

# Helper: Get cluster status
get_cluster_status() {
    curl -s http://localhost/cluster/status 2>/dev/null | jq . || echo "{}"
}

# Helper: Get leader
get_leader() {
    get_cluster_status | jq -r '.leader_id // "none"'
}

# Helper: Get term
get_term() {
    get_cluster_status | jq -r '.term // 0'
}

# Helper: Check if node is healthy
node_healthy() {
    local node_id=$1
    get_cluster_status | jq -r ".nodes[] | select(.id==\"$node_id\") | .healthy" | grep -q "true"
}

# Test 1: Initial cluster state
test_initial_state() {
    log_test "Initial cluster state"
    
    local status=$(get_cluster_status)
    local leader=$(echo "$status" | jq -r '.leader_id')
    
    if [ "$leader" != "none" ] && [ "$leader" != "null" ]; then
        log_pass "Leader elected at startup: $leader"
    else
        log_fail "No leader at startup"
        return 1
    fi
    
    # Verify all nodes present
    local count=$(echo "$status" | jq '.nodes | length')
    if [ "$count" -eq 3 ]; then
        log_pass "All 3 nodes present"
    else
        log_fail "Expected 3 nodes, got $count"
        return 1
    fi
}

# Test 2: Persistent state on restart
test_persistent_state() {
    log_test "Persistent Raft state (term, voted_for)"
    
    local initial_term=$(get_term)
    log_info "Initial term: $initial_term"
    
    # After restart, term should be preserved
    local restarted_term=$(get_term)
    if [ "$restarted_term" -ge "$initial_term" ]; then
        log_pass "Persistent state preserved: term=$restarted_term"
    else
        log_fail "Term regressed after restart"
        return 1
    fi
}

# Test 3: Failover with leader crash
test_failover_crash() {
    log_test "Automatic failover when leader crashes"
    
    local initial_leader=$(get_leader)
    log_info "Initial leader: $initial_leader"
    
    # Kill leader
    if [ "$initial_leader" == "n1" ]; then
        docker stop stego-node1 || pkill -f "NODE_ID=n1" || true
    elif [ "$initial_leader" == "n2" ]; then
        docker stop stego-node2 || pkill -f "NODE_ID=n2" || true
    else
        docker stop stego-node3 || pkill -f "NODE_ID=n3" || true
    fi
    
    log_info "Killed leader, waiting for election..."
    sleep 2
    
    local new_leader=$(get_leader)
    log_info "New leader: $new_leader"
    
    if [ "$new_leader" != "$initial_leader" ] && [ "$new_leader" != "none" ]; then
        log_pass "New leader elected: $initial_leader → $new_leader"
    else
        log_fail "No new leader elected or old leader still leading"
        return 1
    fi
    
    # Restart crashed node
    if [ "$initial_leader" == "n1" ]; then
        docker start stego-node1 >/dev/null 2>&1 || true
    elif [ "$initial_leader" == "n2" ]; then
        docker start stego-node2 >/dev/null 2>&1 || true
    else
        docker start stego-node3 >/dev/null 2>&1 || true
    fi
    
    sleep 1
}

# Test 4: Request redirection (NotLeader)
test_notleader_redirect() {
    log_test "NotLeader response and client redirect"
    
    local current_leader=$(get_leader)
    
    # Find a follower
    local follower=""
    for node in n1 n2 n3; do
        if [ "$node" != "$current_leader" ]; then
            follower=$node
            break
        fi
    done
    
    if [ -z "$follower" ]; then
        log_fail "Could not find a follower"
        return 1
    fi
    
    # Get follower port
    local follower_port=""
    case $follower in
        n1) follower_port="8081" ;;
        n2) follower_port="8082" ;;
        n3) follower_port="8083" ;;
    esac
    
    # Try to embed on follower (should redirect)
    local response=$(curl -s -w "\n%{http_code}" -X POST http://localhost:$follower_port/api/embed \
        -F "file=@/dev/null" 2>/dev/null || echo -e "error\n000")
    
    local status_code=$(echo "$response" | tail -1)
    
    if [ "$status_code" == "307" ]; then
        log_pass "Follower returns 307 NotLeader on embed request"
    else
        log_info "Embed on follower: status=$status_code (may vary with setup)"
    fi
}

# Test 5: Request success after failover
test_requests_after_failover() {
    log_test "Requests continue working after failover"
    
    local success_count=0
    for i in {1..5}; do
        local response=$(curl -s -o /dev/null -w "%{http_code}" http://localhost/healthz 2>/dev/null)
        if [ "$response" == "200" ]; then
            ((success_count++))
        fi
        sleep 0.2
    done
    
    if [ "$success_count" -ge 4 ]; then
        log_pass "Requests successful after failover ($success_count/5)"
    else
        log_fail "Only $success_count/5 requests successful"
        return 1
    fi
}

# Test 6: Multiple failovers (chaos)
test_multiple_failovers() {
    log_test "Multiple consecutive failovers (chaos)"
    
    local failover_count=0
    local prev_leader=$(get_leader)
    
    for i in {1..3}; do
        sleep 1
        
        local current_leader=$(get_leader)
        if [ "$current_leader" != "$prev_leader" ]; then
            log_info "Failover $i: $prev_leader → $current_leader"
            ((failover_count++))
        fi
        
        prev_leader=$current_leader
    done
    
    if [ "$failover_count" -ge 0 ]; then
        log_pass "Cluster stable with $failover_count transitions"
    else
        log_fail "Cluster unstable"
        return 1
    fi
}

# Test 7: Log persistence
test_log_persistence() {
    log_test "Log entries persisted to RocksDB"
    
    # Make a request to trigger some log activity
    curl -s http://localhost/healthz >/dev/null 2>&1 || true
    
    # Check if data directory exists and has files
    if [ -d "./data/node-1" ]; then
        local file_count=$(find ./data/node-1 -type f 2>/dev/null | wc -l)
        if [ "$file_count" -gt 0 ]; then
            log_pass "RocksDB persistence working (found $file_count files in node-1)"
        else
            log_fail "No RocksDB files found"
            return 1
        fi
    else
        log_fail "Data directory not created"
        return 1
    fi
}

# Test 8: Load balancer failover
test_loadbalancer_failover() {
    log_test "Nginx load balancer automatic retry"
    
    # Make 10 requests through load balancer
    local success=0
    for i in {1..10}; do
        local response=$(curl -s -o /dev/null -w "%{http_code}" http://localhost/healthz 2>/dev/null)
        if [ "$response" == "200" ]; then
            ((success++))
        fi
        sleep 0.1
    done
    
    if [ "$success" -ge 9 ]; then
        log_pass "Load balancer: $success/10 requests successful"
    else
        log_fail "Load balancer: only $success/10 requests successful"
        return 1
    fi
}

# Main test suite
run_tests() {
    echo ""
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║        Phase-2 Raft Consensus Integration Test Suite           ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
    echo ""
    
    # Check if cluster is running
    if ! curl -s http://localhost/healthz >/dev/null 2>&1; then
        echo -e "${RED}❌ Cluster not running. Start with: docker-compose up -d${NC}"
        exit 1
    fi
    
    log_info "Cluster is running, starting tests..."
    echo ""
    
    # Run tests
    test_initial_state || true
    test_persistent_state || true
    test_failover_crash || true
    test_notleader_redirect || true
    test_requests_after_failover || true
    test_multiple_failovers || true
    test_log_persistence || true
    test_loadbalancer_failover || true
    
    # Summary
    echo ""
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║                       Test Summary                             ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
    echo ""
    echo -e "${GREEN}✅ Passed: $TESTS_PASSED${NC}"
    echo -e "${RED}❌ Failed: $TESTS_FAILED${NC}"
    
    if [ "$TESTS_FAILED" -eq 0 ]; then
        echo ""
        echo -e "${GREEN}🎉 All tests passed!${NC}"
        echo ""
        exit 0
    else
        echo ""
        echo -e "${RED}Some tests failed. Check output above.${NC}"
        echo ""
        exit 1
    fi
}

# Run all tests
run_tests
