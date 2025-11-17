#!/bin/bash
set -e

# Script to build airbender-server and airbender-client Docker images
# Run from the repository root: ./tests/instances/proof_service/docker/build.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"

echo "========================================="
echo "Building Airbender Docker Images"
echo "========================================="
echo "Repository root: $REPO_ROOT"
echo ""

# Change to repo root
cd "$REPO_ROOT"

# Set CUDA 12.9 environment to match Docker runtime
echo "Setting CUDA 12.9 environment..."
export CUDA_HOME=/usr/local/cuda-12.9
export CUDA_PATH=/usr/local/cuda-12.9
export PATH=/usr/local/cuda-12.9/bin:$PATH
export LD_LIBRARY_PATH=/usr/local/cuda-12.9/lib64:${LD_LIBRARY_PATH}

echo "CUDA_HOME: $CUDA_HOME"
echo "Using CUDA version: $(nvcc --version | grep release | awk '{print $5}' | tr -d ',')"
echo ""

# Step 1: Build binaries on host
echo "[1/3] Building binaries on host with CUDA 12.9..."
echo "  - Building proof_server (with RUST_MIN_STACK=267108864)..."
RUST_MIN_STACK=267108864 cargo build --release -p proof_server

echo "  - Building proof_client..."
cargo build --release -p proof_client

echo ""
echo "[2/3] Building airbender-server Docker image..."
docker build \
  -f tests/instances/proof_service/docker/Dockerfile.server.prebuilt \
  -t airbender-server:latest \
  -t airbender-server:$(date +%Y%m%d) \
  .

echo ""
echo "[3/3] Building airbender-client Docker image..."
docker build \
  -f tests/instances/proof_service/docker/Dockerfile.client.prebuilt \
  -t airbender-client:latest \
  -t airbender-client:$(date +%Y%m%d) \
  .

echo ""
echo "========================================="
echo "Build complete!"
echo "========================================="
echo ""
echo "Images created:"
docker images | grep -E "REPOSITORY|airbender"
echo ""
echo "To run the server:"
echo "  docker run --gpus all -p 50051:50051 \\"
echo "    -v \$(pwd)/data/binary:/app/binary \\"
echo "    -v \$(pwd)/data/output:/app/output \\"
echo "    airbender-server"
echo ""
echo "To run the client:"
echo "  docker run \\"
echo "    -v \$(pwd)/data/input:/app/input \\"
echo "    -v \$(pwd)/data/output:/app/output \\"
echo "    -e SERVER_ADDR=<server-host>:50051 \\"
echo "    airbender-client <args>"
