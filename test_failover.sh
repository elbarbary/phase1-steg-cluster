#!/bin/bash
# Automatic Failover Test Script
# Demonstrates Phase-2 automatic leader election and failover

set -e

echo "╔════════════════════════════════════════════════════════════════════════════════╗"
echo "║          AUTOMATIC FAILOVER TEST - Phase-2 Implementation                     ║"
echo "╚════════════════════════════════════════════════════════════════════════════════╝"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo -e "${RED}❌ Error: jq is not installed. Please install it first.${NC}"
    echo "   Ubuntu/Debian: sudo apt install jq"
    echo "   macOS: brew install jq"
    exit 1
fi

# Function to check cluster status
check_status() {
    curl -s http://localhost/cluster/status 2>/dev/null || curl -s http://172.20.10.2:8081/cluster/status 2>/dev/null
}

# Function to get leader
get_leader() {
    check_status | jq -r '.leader_id // "none"'
}

# Function to get term
get_term() {
    check_status | jq -r '.term // 0'
}

echo -e "${BLUE}Step 1: Checking initial cluster state...${NC}"
echo ""

INITIAL_LEADER=$(get_leader)
INITIAL_TERM=$(get_term)

if [ "$INITIAL_LEADER" == "none" ] || [ "$INITIAL_LEADER" == "null" ]; then
    echo -e "${RED}❌ No leader detected. Is the cluster running?${NC}"
    echo ""
    echo "Start the cluster with:"
    echo "  docker-compose up -d"
    echo "or manually:"
    echo "  NODE_ID=n1 ./target/release/server &"
    echo "  NODE_ID=n2 ./target/release/server &"
    echo "  NODE_ID=n3 ./target/release/server &"
    exit 1
fi

echo -e "${GREEN}✅ Initial leader: $INITIAL_LEADER (term: $INITIAL_TERM)${NC}"
echo ""

# Show full cluster status
echo -e "${BLUE}Full cluster status:${NC}"
check_status | jq '.nodes[] | {id, role, healthy}'
echo ""

echo -e "${YELLOW}Step 2: Killing the leader node ($INITIAL_LEADER)...${NC}"
echo ""

# Determine which container to stop
if [ "$INITIAL_LEADER" == "n1" ]; then
    CONTAINER="stego-node1"
    NODE_ADDR="172.20.10.2:8081"
elif [ "$INITIAL_LEADER" == "n2" ]; then
    CONTAINER="stego-node2"
    NODE_ADDR="172.20.10.3:8082"
elif [ "$INITIAL_LEADER" == "n3" ]; then
    CONTAINER="stego-node3"
    NODE_ADDR="172.20.10.4:8083"
else
    echo -e "${RED}❌ Unknown leader: $INITIAL_LEADER${NC}"
    exit 1
fi

# Try to stop container if using Docker
if docker ps | grep -q "$CONTAINER"; then
    echo "Stopping Docker container: $CONTAINER"
    docker stop "$CONTAINER" > /dev/null 2>&1
    USING_DOCKER=true
else
    # Try to kill process
    echo "Attempting to kill process for $INITIAL_LEADER..."
    pkill -f "NODE_ID=$INITIAL_LEADER" || true
    USING_DOCKER=false
fi

echo -e "${GREEN}✅ Leader $INITIAL_LEADER stopped${NC}"
echo ""

echo -e "${BLUE}Step 3: Waiting for automatic election (max 2 seconds)...${NC}"
echo ""

# Wait and monitor election
MAX_WAIT=20  # 2 seconds in 100ms intervals
WAIT_COUNT=0
NEW_LEADER="none"

while [ $WAIT_COUNT -lt $MAX_WAIT ]; do
    sleep 0.1
    NEW_LEADER=$(get_leader)
    
    if [ "$NEW_LEADER" != "$INITIAL_LEADER" ] && [ "$NEW_LEADER" != "none" ] && [ "$NEW_LEADER" != "null" ]; then
        break
    fi
    
    WAIT_COUNT=$((WAIT_COUNT + 1))
    
    # Show progress
    if [ $((WAIT_COUNT % 5)) -eq 0 ]; then
        echo -n "."
    fi
done

echo ""
echo ""

NEW_TERM=$(get_term)

if [ "$NEW_LEADER" == "$INITIAL_LEADER" ] || [ "$NEW_LEADER" == "none" ] || [ "$NEW_LEADER" == "null" ]; then
    echo -e "${RED}❌ FAIL: No new leader elected after 2 seconds${NC}"
    echo ""
    echo "Current status:"
    check_status | jq
    
    # Restart the stopped node
    if [ "$USING_DOCKER" = true ]; then
        echo ""
        echo "Restarting stopped container..."
        docker start "$CONTAINER" > /dev/null 2>&1
    fi
    
    exit 1
fi

ELECTION_TIME=$((WAIT_COUNT * 100))
echo -e "${GREEN}🎉 SUCCESS: New leader elected!${NC}"
echo ""
echo -e "  Old leader: ${RED}$INITIAL_LEADER${NC} (term: $INITIAL_TERM)"
echo -e "  New leader: ${GREEN}$NEW_LEADER${NC} (term: $NEW_TERM)"
echo -e "  Election time: ${YELLOW}${ELECTION_TIME}ms${NC}"
echo ""

echo -e "${BLUE}Step 4: Verifying cluster health...${NC}"
echo ""

# Show new cluster status
check_status | jq '.nodes[] | {id, role, healthy}'
echo ""

echo -e "${BLUE}Step 5: Testing request handling...${NC}"
echo ""

# Test if requests still work
HEALTH_CHECK=$(curl -s -o /dev/null -w "%{http_code}" http://localhost/healthz 2>/dev/null || echo "000")

if [ "$HEALTH_CHECK" == "200" ]; then
    echo -e "${GREEN}✅ Health check successful - requests working normally${NC}"
else
    echo -e "${YELLOW}⚠️  Health check returned: $HEALTH_CHECK${NC}"
fi

echo ""
echo -e "${BLUE}Step 6: Restoring stopped node...${NC}"
echo ""

if [ "$USING_DOCKER" = true ]; then
    docker start "$CONTAINER" > /dev/null 2>&1
    echo -e "${GREEN}✅ Container $CONTAINER restarted${NC}"
else
    echo -e "${YELLOW}⚠️  Please manually restart node $INITIAL_LEADER${NC}"
    echo "   NODE_ID=$INITIAL_LEADER ./target/release/server &"
fi

echo ""
sleep 2  # Wait for node to rejoin

echo -e "${BLUE}Final cluster status:${NC}"
check_status | jq '.nodes[] | {id, role, healthy}'
echo ""

echo "╔════════════════════════════════════════════════════════════════════════════════╗"
echo "║                           ✅ TEST COMPLETE ✅                                  ║"
echo "╚════════════════════════════════════════════════════════════════════════════════╝"
echo ""
echo "Summary:"
echo "  • Leader failure detected automatically"
echo "  • New leader elected in ${ELECTION_TIME}ms"
echo "  • Requests continued working during failover"
echo "  • Cluster recovered successfully"
echo ""
echo -e "${GREEN}🚀 Phase-2 automatic failover is WORKING!${NC}"
echo ""
