# Airbender Proof Service - Docker Guide

This directory contains Docker configurations for running the Airbender proof service (client and server).

## Prerequisites

- Docker with GPU support (nvidia-docker2)
- NVIDIA GPU with compute capability 8.9 (e.g., L4)
- Built binaries: `proof_server` and `proof_client`

## Quick Start

### 1. Build the Docker Images

```bash
# From the repository root
./tests/instances/proof_service/docker/build.sh
```

This will:
- Build `proof_server` and `proof_client` binaries with proper settings
- Create Docker images `airbender-server` and `airbender-client`

### 2. Prepare Data Directories

```bash
# Create data directories
cd tests/instances/proof_service/docker
mkdir -p data/binary data/input data/output

# Copy your binary to prove into data/binary/
cp /path/to/your/app.bin data/binary/app.bin
```

### 3. Start the Server

#### Option A: Using docker-compose (Recommended)

```bash
# Start the server in the background
docker compose up -d proof-server

# View logs
docker compose logs -f proof-server

# Stop the server
docker compose down
```

#### Option B: Using docker run

```bash
docker run -d \
  --name airbender-server \
  --gpus all \
  -p 50051:50051 \
  -v $(pwd)/data/binary:/app/binary \
  -v $(pwd)/data/output:/app/output \
  -e BINARY_PATH=/app/binary/app.bin \
  -e RUST_LOG=info \
  airbender-server
```

### 4. Run the Client

#### Option A: Using docker-compose

```bash
# Run a proof generation request
docker compose run --rm proof-client prove \
  --binary /app/binary/app.bin \
  --output /app/output/proof.bin

# Check server status
docker compose run --rm proof-client status
```

#### Option B: Using docker run

```bash
# Connect to server running via docker-compose
docker run --rm \
  --network proof_service_proof-network \
  -v $(pwd)/data/binary:/app/binary \
  -v $(pwd)/data/output:/app/output \
  -e SERVER_ADDR=proof-server:50051 \
  airbender-client prove \
  --binary /app/binary/app.bin \
  --output /app/output/proof.bin

# Or connect to standalone server (host network)
docker run --rm \
  --network host \
  -v $(pwd)/data/binary:/app/binary \
  -v $(pwd)/data/output:/app/output \
  -e SERVER_ADDR=localhost:50051 \
  airbender-client prove \
  --binary /app/binary/app.bin \
  --output /app/output/proof.bin
```

## Configuration

### Environment Variables

#### Server

| Variable | Default | Description |
|----------|---------|-------------|
| `BINARY_PATH` | `/app/binary/app.bin` | Path to the binary to prove |
| `LISTEN_ADDR` | `0.0.0.0:50051` | gRPC server listen address |
| `RUST_LOG` | `info` | Log level (trace, debug, info, warn, error) |
| `RUST_MIN_STACK` | `267108864` | Minimum stack size (256MB) |

#### Client

| Variable | Default | Description |
|----------|---------|-------------|
| `SERVER_ADDR` | `proof-server:50051` | gRPC server address |
| `RUST_LOG` | `info` | Log level |

### Volume Mounts

- `/app/binary` - Directory containing binaries to prove
- `/app/input` - Input data for client (optional)
- `/app/output` - Output directory for generated proofs

## Common Operations

### Check Server Status

```bash
# View server logs
docker compose logs -f proof-server

# Check if server is running
docker compose ps

# Check GPU usage
docker exec airbender-server nvidia-smi
```

### Generate a Proof

```bash
# Using docker-compose
docker compose run --rm proof-client prove \
  --binary /app/binary/app.bin \
  --output /app/output/proof_$(date +%s).bin

# The proof will be saved in data/output/
```

### Stop Everything

```bash
# Stop and remove containers
docker compose down

# Stop and remove containers + volumes
docker compose down -v
```

### Rebuild Images

```bash
# Rebuild after code changes
./build.sh

# Restart services with new images
docker compose down
docker compose up -d proof-server
```

## Troubleshooting

### GPU Not Available

```bash
# Check if NVIDIA runtime is available
docker run --rm --gpus all nvidia/cuda:12.9.1-runtime-ubuntu24.04 nvidia-smi

# If this fails, install nvidia-docker2:
# sudo apt-get install nvidia-docker2
# sudo systemctl restart docker
```

### Connection Refused

```bash
# Check if server is running
docker compose ps

# Check server logs for errors
docker compose logs proof-server

# Verify network connectivity
docker compose run --rm proof-client ping proof-server
```

### Out of Memory

If you see OOM errors, increase Docker's memory limit or adjust CUDA settings:

```bash
# Check memory usage
docker stats airbender-server
```

### Permission Denied on Output Files

```bash
# Fix permissions on data directories
sudo chown -R $USER:$USER data/
chmod -R 755 data/
```

## Advanced Usage

### Running Multiple Servers

Edit `docker-compose.yml` to add more server instances:

```yaml
proof-server-2:
  extends: proof-server
  container_name: airbender-server-2
  ports:
    - "50052:50051"
```

### Using Different GPUs

```yaml
deploy:
  resources:
    reservations:
      devices:
        - driver: nvidia
          device_ids: ['0']  # Use specific GPU
          capabilities: [gpu]
```

### Custom Binary Path

```bash
# Override BINARY_PATH at runtime
docker compose run --rm \
  -e BINARY_PATH=/app/binary/custom.bin \
  proof-client prove --binary /app/binary/custom.bin --output /app/output/proof.bin
```

## Files

- `Dockerfile.server.prebuilt` - Server runtime image
- `Dockerfile.client.prebuilt` - Client runtime image
- `docker-compose.yml` - Compose configuration
- `build.sh` - Build script for images
- `README.md` - This file

## Support

For issues or questions, check the main repository documentation or server logs:

```bash
docker compose logs --tail=100 proof-server
```
