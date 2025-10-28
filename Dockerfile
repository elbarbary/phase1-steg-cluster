# Multi-stage build for optimized Rust binary
FROM rust:1.90-slim as builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build release binary
RUN cargo build --release --bin server

# Runtime stage - minimal image
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /build/target/release/server /app/phase1-server

# Copy configuration (will be overridden by volume mount)
COPY config /app/config

# Create directories
RUN mkdir -p /app/assets /app/data

# Expose ports (HTTP API and Raft RPC)
EXPOSE 8081 8082 8083 5001 5002 5003

# Health check
HEALTHCHECK --interval=5s --timeout=2s --start-period=10s --retries=3 \
    CMD wget --quiet --tries=1 --spider http://localhost:8081/healthz || \
        wget --quiet --tries=1 --spider http://localhost:8082/healthz || \
        wget --quiet --tries=1 --spider http://localhost:8083/healthz || exit 1

# Run the server
ENTRYPOINT ["/app/phase1-server"]
