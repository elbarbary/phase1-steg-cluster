#!/usr/bin/env bash
# Quick local cluster startup script

set -euo pipefail

echo "🚀 Starting Phase-1 Steganography Cluster (Local Mode)"
echo ""

# Check if already running
if lsof -Pi :8081 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo "⚠️  Port 8081 already in use. Cluster may already be running."
    echo "   Kill existing processes with: pkill -f 'cargo run -p server'"
    exit 1
fi

# Build first
echo "📦 Building release binaries..."
cargo build --release

echo ""
echo "✅ Build complete!"
echo ""
echo "Starting 3 nodes in background..."
echo ""

# Start nodes in background
export RUST_LOG=info

# Node 1
export NODE_ID=n1
cargo run -p server --release > /tmp/phase1-n1.log 2>&1 &
N1_PID=$!
echo "  n1 started (PID: $N1_PID, Port: 8081)"

# Node 2
export NODE_ID=n2
cargo run -p server --release > /tmp/phase1-n2.log 2>&1 &
N2_PID=$!
echo "  n2 started (PID: $N2_PID, Port: 8082)"

# Node 3
export NODE_ID=n3
cargo run -p server --release > /tmp/phase1-n3.log 2>&1 &
N3_PID=$!
echo "  n3 started (PID: $N3_PID, Port: 8083)"

echo ""
echo "🎉 Cluster started!"
echo ""
echo "📊 Access GUI at:"
echo "   http://127.0.0.1:8081"
echo "   http://127.0.0.1:8082"
echo "   http://127.0.0.1:8083"
echo ""
echo "📝 Logs:"
echo "   tail -f /tmp/phase1-n1.log"
echo "   tail -f /tmp/phase1-n2.log"
echo "   tail -f /tmp/phase1-n3.log"
echo ""
echo "🛑 Stop cluster with:"
echo "   pkill -f 'cargo run -p server'"
echo ""
