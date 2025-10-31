#!/bin/bash
# 🔄 RESTART CLUSTER - Pull latest code, rebuild, and start all 3 nodes
# 
# This script should be run once on EACH device to rebuild with the latest code
# Usage: ./RESTART_CLUSTER.sh <node_id>
# Example: ./RESTART_CLUSTER.sh n1   (on Device 1)
#          ./RESTART_CLUSTER.sh n2   (on Device 2)
#          ./RESTART_CLUSTER.sh n3   (on Device 3)

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if node_id provided
if [ $# -ne 1 ]; then
    echo -e "${RED}❌ Usage: $0 <node_id>${NC}"
    echo "   node_id should be one of: n1, n2, n3"
    echo ""
    echo "Examples:"
    echo "  ./RESTART_CLUSTER.sh n1   (Device 1 - primary)"
    echo "  ./RESTART_CLUSTER.sh n2   (Device 2)"
    echo "  ./RESTART_CLUSTER.sh n3   (Device 3)"
    exit 1
fi

NODE_ID="$1"

# Validate node_id
if [[ ! "$NODE_ID" =~ ^n[1-3]$ ]]; then
    echo -e "${RED}❌ Invalid node_id: $NODE_ID${NC}"
    echo "   Must be one of: n1, n2, n3"
    exit 1
fi

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}🔄 RESTARTING NODE: $NODE_ID${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"

# Step 1: Kill existing processes
echo -e "\n${YELLOW}[1/5] Stopping existing processes...${NC}"
pkill -f "cargo run -p server" || echo "  No existing processes found"
pkill -f "target/release/server" || echo "  No existing processes found"
sleep 2

# Step 2: Clear RocksDB lock files (prevents corruption)
echo -e "\n${YELLOW}[2/5] Clearing RocksDB state...${NC}"
rm -f "data/node-${NODE_ID: -1}/LOCK" || true
rm -f "data/node-1/LOCK" data/node-2/LOCK data/node-3/LOCK || true
echo "  ✓ Cleared lock files"

# Step 3: Pull latest code
echo -e "\n${YELLOW}[3/5] Pulling latest code from GitHub...${NC}"
git pull origin master
echo "  ✓ Code updated"

# Step 4: Build release binary
echo -e "\n${YELLOW}[4/5] Building release binary (this may take 1-2 minutes)...${NC}"
cargo build --release --bin server 2>&1 | tail -5
echo "  ✓ Build complete"

# Verify binary exists
if [ ! -f "target/release/server" ]; then
    echo -e "${RED}❌ Binary not found at target/release/server${NC}"
    exit 1
fi

echo -e "\n${YELLOW}[5/5] Starting node $NODE_ID...${NC}"

# Step 5: Start the node using the prebuilt binary directly
export RUST_LOG=info,openraft=info,axum=info
export NODE_ID="$NODE_ID"
export CONFIG_PATH=./config/cluster.yaml
export WORKER_THREADS=8

echo -e "${GREEN}✓ Node $NODE_ID starting...${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Logs for $NODE_ID:${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"

# Run the prebuilt binary directly (not cargo run)
./target/release/server
