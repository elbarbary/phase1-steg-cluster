#!/usr/bin/env bash
set -euo pipefail

export RUST_LOG=info,openraft=info,axum=info
export NODE_ID=n2
export CONFIG_PATH=./config/cluster.yaml
export WORKER_THREADS=8

cargo run -p server --release
