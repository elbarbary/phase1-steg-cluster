# 📦 Complete Installation Dependencies Guide

This document lists all required and optional tools needed for the Phase-1/2 Distributed Steganography System.

## 🔴 Required Dependencies

### Rust Ecosystem
- **rustc** - Rust compiler (installed via rustup)
- **cargo** - Package manager and build system
- **clang** - C compiler required by some Rust dependencies
- **libclang-dev** - Development headers for clang

```bash
# Verify installation
rustc --version
cargo --version
```

### Version Control
- **git** - Version control system

```bash
# Verify installation
git --version
```

### Build Tools
- **build-essential** (Ubuntu) or Xcode Command Line Tools (macOS)
- **gcc** - GNU C compiler
- **pkg-config** - Package configuration helper
- **libssl-dev** - OpenSSL development headers
- **cmake** - Build system (required by RocksDB)

```bash
# Verify installation
gcc --version
clang --version
pkg-config --version
```

## 🟡 Recommended Tools

### Network Utilities
- **curl** - HTTP client for testing APIs
- **netcat (nc)** - Network diagnostics
- **net-tools** - Network configuration tools (ifconfig, etc.)
- **dig/nslookup** - DNS query tools

```bash
# Ubuntu installation
sudo apt install curl netcat net-tools dnsutils

# macOS installation
brew install curl netcat net-tools bind
```

### JSON Processing
- **jq** - Command-line JSON query tool (makes API output readable)

```bash
# Ubuntu installation
sudo apt install jq

# macOS installation
brew install jq

# Verify
jq --version
```

### Terminal Multiplexing
- **tmux** - Terminal multiplexer for running multiple nodes simultaneously
- **screen** - Alternative terminal multiplexer

```bash
# Ubuntu installation
sudo apt install tmux screen

# macOS installation
brew install tmux

# Verify
tmux -V
```

### System Monitoring
- **htop** - Interactive process viewer (better than 'top')
- **iotop** - I/O monitoring
- **nethogs** - Network monitoring per process

```bash
# Ubuntu installation
sudo apt install htop iotop nethogs

# macOS installation
brew install htop

# Verify
htop -v
```

### Text Editors
- **nano** - Easy-to-use text editor (for beginners)
- **vim** - Advanced text editor
- **VSCode** - Visual Studio Code (if using GUI)

```bash
# Ubuntu installation
sudo apt install nano vim

# macOS installation (nano usually pre-installed)
brew install vim
```

## 🟢 Optional (For Advanced Features)

### Containerization
- **Docker** - Container runtime
- **Docker Compose** - Multi-container orchestration

```bash
# Ubuntu installation
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# macOS installation
brew install docker

# Verify
docker --version
docker-compose --version
```

### Reverse Proxy / Load Balancer
- **nginx** - Web server and load balancer
- **haproxy** - Load balancer alternative

```bash
# Ubuntu installation
sudo apt install nginx

# macOS installation
brew install nginx

# Verify
nginx -v
```

### Performance Testing
- **apache2-utils** - Contains 'ab' (Apache Bench) for load testing
- **vegeta** - HTTP load testing tool written in Go
- **wrk** - Modern HTTP load testing tool

```bash
# Ubuntu installation
sudo apt install apache2-utils

# macOS installation
brew install apache2-utils

# Install vegeta
go install github.com/tsenart/vegeta@latest

# Install wrk
brew install wrk
```

### Database Tools
- **sqlite3** - SQLite client (useful for testing)
- **postgresql-client** - PostgreSQL client tools
- **redis-tools** - Redis command-line tools

```bash
# Ubuntu installation
sudo apt install sqlite3 postgresql-client redis-tools

# macOS installation
brew install sqlite postgresql redis
```

### Development Tools
- **gdb** - GNU Debugger (for debugging Rust binaries)
- **valgrind** - Memory profiling tool
- **perf** - Linux performance profiler

```bash
# Ubuntu installation
sudo apt install gdb valgrind linux-tools-generic

# macOS installation
brew install gdb
```

## ✅ Complete Installation Scripts

### Ubuntu/Debian All-In-One

```bash
#!/bin/bash
# Update package manager
sudo apt update
sudo apt upgrade -y

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install all essential + recommended tools
sudo apt install -y \
  build-essential \
  clang \
  libclang-dev \
  curl \
  git \
  jq \
  net-tools \
  netcat \
  tmux \
  htop \
  nano \
  pkg-config \
  libssl-dev \
  cmake

# Optional: Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Optional: Install nginx
sudo apt install -y nginx

# Optional: Install performance testing tools
sudo apt install -y apache2-utils

# Verify installations
echo "=== Verification ==="
rustc --version
cargo --version
git --version
gcc --version
clang --version
curl --version
jq --version
tmux -V
htop -v
nginx -v
```

### macOS All-In-One

```bash
#!/bin/bash
# Install Homebrew if not already installed
if ! command -v brew &> /dev/null; then
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
fi

# Update Homebrew
brew update

# Install Rust (if not already installed)
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source $HOME/.cargo/env
fi

# Install all essential + recommended tools
brew install \
  rust \
  git \
  curl \
  jq \
  tmux \
  htop \
  nano \
  vim \
  openssl \
  pkg-config \
  cmake

# Optional: Install Docker Desktop
# Download from https://www.docker.com/products/docker-desktop or:
# brew install docker

# Optional: Install nginx
brew install nginx

# Optional: Install performance testing tools
brew install apache2-utils wrk

# Verify installations
echo "=== Verification ==="
rustc --version
cargo --version
git --version
clang --version
curl --version
jq --version
tmux -V
htop -v
nginx -v
```

## 🔧 Troubleshooting Installation

### Issue: "command not found: rustc"

**Solution:**
```bash
# Add Rust to PATH
source $HOME/.cargo/env

# Or add to .bashrc / .zshrc
echo 'source $HOME/.cargo/env' >> ~/.bashrc
source ~/.bashrc
```

### Issue: "openssl not found" during build

**Ubuntu:**
```bash
sudo apt install libssl-dev pkg-config
export PKG_CONFIG_PATH="/usr/lib/x86_64-linux-gnu/pkgconfig"
cargo build --release
```

**macOS:**
```bash
brew install openssl
export LDFLAGS="-L/usr/local/opt/openssl/lib"
export CPPFLAGS="-I/usr/local/opt/openssl/include"
export PKG_CONFIG_PATH="/usr/local/opt/openssl/lib/pkgconfig"
cargo build --release
```

### Issue: "clang not found"

**Ubuntu:**
```bash
sudo apt install clang libclang-dev
```

**macOS:**
```bash
brew install clang
# Or install Xcode Command Line Tools:
xcode-select --install
```

### Issue: "git not found"

**Ubuntu:**
```bash
sudo apt install git
```

**macOS:**
```bash
brew install git
# Or install Xcode Command Line Tools:
xcode-select --install
```

## 📋 Verification Checklist

Run this to verify all tools are installed:

```bash
#!/bin/bash

echo "=== Required Tools ==="
echo "✓ Rust: $(rustc --version 2>/dev/null || echo 'NOT INSTALLED')"
echo "✓ Cargo: $(cargo --version 2>/dev/null || echo 'NOT INSTALLED')"
echo "✓ Git: $(git --version 2>/dev/null || echo 'NOT INSTALLED')"
echo "✓ GCC: $(gcc --version 2>/dev/null | head -1 || echo 'NOT INSTALLED')"
echo "✓ Clang: $(clang --version 2>/dev/null | head -1 || echo 'NOT INSTALLED')"

echo ""
echo "=== Recommended Tools ==="
echo "✓ curl: $(curl --version 2>/dev/null | head -1 || echo 'NOT INSTALLED')"
echo "✓ jq: $(jq --version 2>/dev/null || echo 'NOT INSTALLED')"
echo "✓ tmux: $(tmux -V 2>/dev/null || echo 'NOT INSTALLED')"
echo "✓ htop: $(htop -v 2>/dev/null | head -1 || echo 'NOT INSTALLED')"
echo "✓ netcat: $(nc -h 2>&1 | head -1 | grep -q netcat && echo 'INSTALLED' || echo 'NOT INSTALLED')"

echo ""
echo "=== Optional Tools ==="
echo "✓ Docker: $(docker --version 2>/dev/null || echo 'NOT INSTALLED')"
echo "✓ Docker Compose: $(docker-compose --version 2>/dev/null || echo 'NOT INSTALLED')"
echo "✓ nginx: $(nginx -v 2>&1 || echo 'NOT INSTALLED')"
echo "✓ PostgreSQL Client: $(psql --version 2>/dev/null || echo 'NOT INSTALLED')"
```

## 🎯 Minimum vs. Recommended Setup

### Minimum (Local Testing Only)
- Rust
- Git
- Build tools (gcc, clang, libssl-dev)
- curl

### Recommended (For Device Setup)
- All minimum tools
- jq (for API testing)
- tmux (for running 3 nodes)
- htop (for monitoring)
- netcat (for network diagnostics)

### Complete (For Production/Docker)
- All recommended tools
- Docker & Docker Compose
- nginx
- Performance testing tools (apache2-utils, wrk)

## 🚀 Next Steps

After installing dependencies:

1. **Clone the repository:**
   ```bash
   git clone https://github.com/YOUR_USERNAME/phase1-steg-cluster.git
   cd phase1-steg-cluster
   ```

2. **Build the project:**
   ```bash
   cargo build --release
   ```

3. **Run tests:**
   ```bash
   cargo test --all
   ```

4. **Start the system:**
   - Local: See `QUICKSTART.md`
   - Docker: `docker-compose up`
   - Distributed: See `DEVICE_SETUP.md`

---

For more detailed setup instructions, see:
- `QUICKSTART.md` - Local 3-node setup
- `DEVICE_SETUP.md` - Multi-device distributed setup
- `GITHUB_DEPLOYMENT.md` - GitHub and deployment guide
