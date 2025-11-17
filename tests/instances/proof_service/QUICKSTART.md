# Quick Start Guide

Get up and running with the Proof Service in 5 minutes.

## Prerequisites

- Docker with docker-compose
- NVIDIA GPU with docker GPU support
- Your `app.bin` binary
- Witness data file(s)

## Step 1: Setup

```bash
cd /home/aleksandr/zksync-os/tests/instances/proof_service

# Create directories
./setup.sh
```

## Step 2: Add Your Files

```bash
# Copy binary
cp /path/to/your/app.bin data/binary/

# Copy witness data
cp /path/to/your/witness.bin data/input/
```

## Step 3: Run

```bash
# Quick run with defaults
./run-example.sh
```

Or manually:

```bash
# Start server (runs in background)
docker-compose up -d proof-server

# Wait 30 seconds for initialization...

# Run client
docker-compose run --rm proof-client \
  --binary /app/binary/app.bin \
  --input /app/input/witness.bin \
  --output-dir /app/output \
  --block-number 1
```

## Step 4: Check Results

```bash
# List output files
ls -lh data/output/

# View metadata
cat data/output/metadata_block_1.json | jq
```

## Step 5: Stop

```bash
# Stop server
docker-compose down
```

## Common Commands

### Generate multiple proofs

```bash
# Server keeps running, just submit more requests
for i in {1..10}; do
  docker-compose run --rm proof-client \
    --binary /app/binary/app.bin \
    --input /app/input/witness_${i}.bin \
    --output-dir /app/output \
    --block-number $i
done
```

### View server logs

```bash
docker-compose logs -f proof-server
```

### Check GPU usage

```bash
docker exec proof-server nvidia-smi
```

### Rebuild after code changes

```bash
docker-compose build
```

## Troubleshooting

### "nvidia-smi not found"

Install nvidia-docker:
```bash
distribution=$(. /etc/os-release;echo $ID$VERSION_ID)
curl -s -L https://nvidia.github.io/nvidia-docker/gpgkey | sudo apt-key add -
curl -s -L https://nvidia.github.io/nvidia-docker/$distribution/nvidia-docker.list | sudo tee /etc/apt/sources.list.d/nvidia-docker.list
sudo apt-get update && sudo apt-get install -y nvidia-docker2
sudo systemctl restart docker
```

### "connection refused"

Server isn't running or isn't ready:
```bash
# Check status
docker-compose ps

# View logs
docker-compose logs proof-server

# Restart
docker-compose restart proof-server
```

### "file not found"

Check volume mounts:
```bash
# List files in container
docker-compose run --rm proof-client ls -la /app/binary
docker-compose run --rm proof-client ls -la /app/input
```

## Next Steps

- Read [README.md](README.md) for detailed documentation
- See [ARCHITECTURE.md](ARCHITECTURE.md) for system design
- Customize `docker-compose.yml` for your needs
- Add more witness files to `data/input/`
- Process all your blocks!

## Performance Tips

1. **Keep server running** - GPU state persists between requests
2. **Batch processing** - Submit multiple requests in sequence
3. **Tune num_instances** - Adjust based on GPU memory (default: 500)
4. **Monitor GPU** - Use `nvidia-smi` to watch utilization

## Need Help?

- Check logs: `docker-compose logs -f proof-server`
- Verify setup: `./setup.sh`
- Read docs: `README.md` and `ARCHITECTURE.md`
