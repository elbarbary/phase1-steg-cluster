#!/usr/bin/env bash
# GitHub Setup and Push Script for Phase-1 Steganography Cluster

set -euo pipefail

echo "╔════════════════════════════════════════════════════════════╗"
echo "║  Phase-1: GitHub Setup & Push Instructions                ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Check git is installed
if ! command -v git &> /dev/null; then
    echo "❌ Git not found. Please install git:"
    echo "   Ubuntu: sudo apt install git"
    echo "   macOS:  brew install git"
    exit 1
fi

echo "✅ Git found: $(git --version)"
echo ""

# Check if already initialized
if [ -d ".git" ]; then
    echo "✅ Git repository already initialized"
else
    echo "⚠️  Git not initialized. Initialize with: git init"
    exit 1
fi

echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║  STEP 1: Create GitHub Repository                         ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "1. Go to https://github.com/new"
echo "2. Fill in:"
echo "   - Repository name: phase1-steg-cluster"
echo "   - Description: Distributed Steganography with OpenRaft Consensus"
echo "   - Visibility: Public"
echo "   - ❌ Do NOT initialize with README (we have one)"
echo ""
echo "3. Click 'Create repository'"
echo ""
read -p "Press Enter once repository is created..."

echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║  STEP 2: Configure Remote                                 ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "Enter your GitHub username (e.g., john-doe):"
read -p "> " GITHUB_USER

REPO_URL="https://github.com/${GITHUB_USER}/phase1-steg-cluster.git"

echo ""
echo "Adding remote: $REPO_URL"
git remote add origin "$REPO_URL" || git remote set-url origin "$REPO_URL"

echo "✅ Remote configured"
echo ""

echo "╔════════════════════════════════════════════════════════════╗"
echo "║  STEP 3: Push to GitHub                                   ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Try to push
echo "Pushing master branch..."
if git push -u origin master; then
    echo "✅ Push successful!"
else
    echo "❌ Push failed. Please authenticate:"
    echo ""
    echo "Option A: Use Personal Access Token (recommended)"
    echo "1. Create token at: https://github.com/settings/tokens"
    echo "2. When prompted for password, paste the token"
    echo ""
    echo "Option B: Setup SSH keys"
    echo "1. Generate keys: ssh-keygen -t ed25519"
    echo "2. Add to GitHub: https://github.com/settings/keys"
    echo "3. Update remote: git remote set-url origin git@github.com:${GITHUB_USER}/phase1-steg-cluster.git"
    echo ""
    exit 1
fi

echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║  SUCCESS! Repository Created                              ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "📍 Repository URL: $REPO_URL"
echo ""
echo "✅ You can now:"
echo "   1. Share this URL with your team"
echo "   2. Clone on Device 2 and Device 3"
echo "   3. All devices will have identical code"
echo ""
echo "📋 Next step: Copy DEVICE_SETUP.md to other devices"
echo ""
