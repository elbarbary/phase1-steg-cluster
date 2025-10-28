#!/usr/bin/env bash
# Build verification script - ensures everything compiles correctly

set -euo pipefail

echo "╔══════════════════════════════════════════════════════════╗"
echo "║  Phase-1 Steganography Cluster - Build Verification     ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

# Check Rust installation
echo "🔍 Checking Rust installation..."
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust from https://rustup.rs/"
    exit 1
fi

RUST_VERSION=$(rustc --version)
echo "✅ Found: $RUST_VERSION"
echo ""

# Check we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Cargo.toml not found. Please run this script from the project root."
    exit 1
fi

# Clean previous build
echo "🧹 Cleaning previous build artifacts..."
cargo clean
echo "✅ Clean complete"
echo ""

# Build workspace
echo "📦 Building workspace (this may take 5-10 minutes)..."
echo ""
if cargo build --release; then
    echo ""
    echo "✅ Build successful!"
else
    echo ""
    echo "❌ Build failed. Please check the error messages above."
    exit 1
fi
echo ""

# Run tests
echo "🧪 Running tests..."
echo ""
if cargo test --workspace --release; then
    echo ""
    echo "✅ All tests passed!"
else
    echo ""
    echo "⚠️  Some tests failed. Check output above."
fi
echo ""

# Verify binaries
echo "🔎 Verifying binaries..."
if [ -f "target/release/server" ]; then
    echo "✅ server binary: $(du -h target/release/server | cut -f1)"
else
    echo "❌ server binary not found"
    exit 1
fi

if [ -f "target/release/phase1-loadgen" ]; then
    echo "✅ phase1-loadgen binary: $(du -h target/release/phase1-loadgen | cut -f1)"
else
    echo "❌ phase1-loadgen binary not found"
    exit 1
fi
echo ""

# Check static files
echo "📂 Checking static files..."
STATIC_FILES=("static/index.html" "static/app.js" "static/app.css")
for file in "${STATIC_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "✅ $file"
    else
        echo "❌ $file missing"
        exit 1
    fi
done
echo ""

# Check config
echo "⚙️  Checking configuration..."
if [ -f "config/cluster.yaml" ]; then
    echo "✅ config/cluster.yaml"
else
    echo "❌ config/cluster.yaml missing"
    exit 1
fi
echo ""

# Summary
echo "╔══════════════════════════════════════════════════════════╗"
echo "║                   ✅ BUILD VERIFIED                      ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""
echo "📋 Next steps:"
echo ""
echo "   1. For LOCAL testing:"
echo "      • Edit config/cluster.yaml - set all IPs to 127.0.0.1"
echo "      • Run: ./bin/start-local-cluster.sh"
echo "      • Open: http://127.0.0.1:8081"
echo ""
echo "   2. For DISTRIBUTED deployment (3 machines):"
echo "      • Edit config/cluster.yaml with real IPs"
echo "      • Copy project to all 3 machines"
echo "      • Run on each:"
echo "        - Machine 1: ./bin/run-n1.sh"
echo "        - Machine 2: ./bin/run-n2.sh"
echo "        - Machine 3: ./bin/run-n3.sh"
echo ""
echo "   3. Run CLI stress test:"
echo "      cargo run -p loadgen --release -- \\"
echo "        --mode embed \\"
echo "        --num-clients 10 \\"
echo "        --reqs-per-client 100 \\"
echo "        --server-list \"http://127.0.0.1:8081,http://127.0.0.1:8082,http://127.0.0.1:8083\""
echo ""
echo "📖 See README.md for detailed instructions"
echo ""
