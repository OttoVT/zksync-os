# Running Airbender Proof Service with Docker

Simple instructions for running the server and client using `docker run`.

## Prerequisites

- Docker with GPU support (nvidia-docker2)
- Built Docker images: `airbender-server` and `airbender-client`

## Build Images

```bash
cd /home/aleksandr/zksync-os
./tests/instances/proof_service/docker/build.sh
```

## Running the Server

```bash
docker run -d \
  --name airbender-server \
  --gpus all \
  -p 50051:50051 \
  -v /home/aleksandr/zksync-os/tests/instances/proof_service/data/binary:/app/binary \
  -v /home/aleksandr/zksync-os/tests/instances/proof_service/data/output:/app/output \
  -e BINARY_PATH=/app/binary/app.bin \
  -e LISTEN_ADDR=0.0.0.0:50051 \
  -e RUST_LOG=info \
  -e RUST_MIN_STACK=267108864 \
  airbender-server
```

**What this does:**
- `-d` - Run in background (detached)
- `--name airbender-server` - Container name
- `--gpus all` - Give container access to all GPUs
- `-p 50051:50051` - Expose gRPC port
- `-v ...:/app/binary` - Mount binary directory
- `-v ...:/app/output` - Mount output directory for proofs
- Environment variables configure the server

## Running the Client

The client connects to the server to request proofs. Run it each time you want to generate a proof:

```bash
docker run --rm \
  --network host \
  -v /home/aleksandr/zksync-os/tests/instances/proof_service/data:/app/data \
  -e SERVER_ADDR=localhost:50051 \
  -e RUST_LOG=info \
  airbender-client prove \
  --binary /app/data/binary/app.bin \
  --output /app/data/output/proof_$(date +%s).bin
```

**What this does:**
- `--rm` - Remove container after it exits
- `--network host` - Use host network to access server on localhost:50051
- `-v .../data:/app/data` - Mount entire data directory
- Environment variables configure the client
- `prove` - Client command (replace with your actual command)
- Output proof gets timestamped filename

## Managing the Server

### Check Server Status
```bash
# View logs
docker logs -f airbender-server

# Check if running
docker ps | grep airbender

# Check GPU usage inside container
docker exec airbender-server nvidia-smi
```

### Stop Server
```bash
docker stop airbender-server
```

### Start Stopped Server
```bash
docker start airbender-server
```

### Remove Server Container
```bash
docker stop airbender-server
docker rm airbender-server
```

### Restart Server
```bash
docker restart airbender-server
```

## Quick Commands

**Start Everything:**
```bash
# Start server
docker run -d --name airbender-server --gpus all -p 50051:50051 \
  -v /home/aleksandr/zksync-os/tests/instances/proof_service/data/binary:/app/binary \
  -v /home/aleksandr/zksync-os/tests/instances/proof_service/data/output:/app/output \
  airbender-server

# Wait a moment for server to start, then run client
sleep 2

# Run client
docker run --rm --network host \
  -v /home/aleksandr/zksync-os/tests/instances/proof_service/data:/app/data \
  -e SERVER_ADDR=localhost:50051 \
  airbender-client prove \
  --binary /app/data/binary/app.bin \
  --output /app/data/output/proof.bin
```

**Stop Everything:**
```bash
docker stop airbender-server
docker rm airbender-server
```

## Troubleshooting

### Server won't start
```bash
# Check logs
docker logs airbender-server

# Check if binary exists
ls -lh /home/aleksandr/zksync-os/tests/instances/proof_service/data/binary/app.bin
```

### Client can't connect
```bash
# Verify server is running
docker ps | grep airbender-server

# Check server logs
docker logs airbender-server

# Test connectivity
curl localhost:50051
```

### GPU not available
```bash
# Test GPU access
docker run --rm --gpus all nvidia/cuda:12.9.1-runtime-ubuntu24.04 nvidia-smi

# If this fails, restart Docker
sudo systemctl restart docker
```

### Permission errors on output files
```bash
# Fix permissions
sudo chown -R $USER:$USER /home/aleksandr/zksync-os/tests/instances/proof_service/data/
chmod -R 755 /home/aleksandr/zksync-os/tests/instances/proof_service/data/
```

## Environment Variables

### Server
- `BINARY_PATH` - Path to binary file (default: `/app/binary/app.bin`)
- `LISTEN_ADDR` - Server address (default: `0.0.0.0:50051`)
- `RUST_LOG` - Log level: `trace`, `debug`, `info`, `warn`, `error`
- `RUST_MIN_STACK` - Stack size in bytes (default: `267108864` = 256MB)

### Client
- `SERVER_ADDR` - Server address (default: `localhost:50051`)
- `RUST_LOG` - Log level

## Directory Structure

```
/home/aleksandr/zksync-os/tests/instances/proof_service/data/
├── binary/          # Put your app.bin here
│   └── app.bin
├── input/           # Optional input files
└── output/          # Generated proofs appear here
    └── proof_*.bin
```

## Example: Generate Multiple Proofs

```bash
# Server should already be running

# Generate proof 1
docker run --rm --network host \
  -v /home/aleksandr/zksync-os/tests/instances/proof_service/data:/app/data \
  -e SERVER_ADDR=localhost:50051 \
  airbender-client prove --binary /app/data/binary/app.bin --output /app/data/output/proof1.bin

# Generate proof 2
docker run --rm --network host \
  -v /home/aleksandr/zksync-os/tests/instances/proof_service/data:/app/data \
  -e SERVER_ADDR=localhost:50051 \
  airbender-client prove --binary /app/data/binary/app.bin --output /app/data/output/proof2.bin

# Check outputs
ls -lh /home/aleksandr/zksync-os/tests/instances/proof_service/data/output/
```
