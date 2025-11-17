#!/bin/bash

# Quick start script for running an example proof generation
set -e

echo "=== Proof Service Quick Start ==="
echo ""

# Check if data directories exist
if [ ! -d "data/binary" ]; then
    echo "Error: data/binary directory not found. Run ./setup.sh first"
    exit 1
fi

# Check if binary exists
if [ ! -f "data/binary/app.bin" ]; then
    echo "Error: data/binary/app.bin not found"
    echo "Please copy your binary: cp /path/to/app.bin data/binary/"
    exit 1
fi

# Check if witness exists
WITNESS_FILE="${1:-data/input/witness.bin}"
if [ ! -f "$WITNESS_FILE" ]; then
    echo "Error: Witness file not found: $WITNESS_FILE"
    echo "Usage: $0 [witness_file]"
    echo "Please copy witness data: cp /path/to/witness.bin data/input/"
    exit 1
fi

echo "Configuration:"
echo "  Binary: data/binary/app.bin"
echo "  Witness: $WITNESS_FILE"
echo "  Output: data/output/"
echo ""

# Check if server is running
if ! docker-compose ps | grep -q "proof-server.*Up"; then
    echo "Starting proof server..."
    docker-compose up -d proof-server
    echo "Waiting for server to initialize (30 seconds)..."
    sleep 30
else
    echo "Server is already running"
fi

echo ""
echo "Sending proof request to server..."
docker-compose run --rm proof-client \
    --binary /app/binary/app.bin \
    --input "/app/input/$(basename $WITNESS_FILE)" \
    --output-dir /app/output \
    --server http://proof-server:50051

echo ""
echo "=== Proof generation complete! ==="
echo "Output files:"
ls -lht data/output/ | head -5
echo ""
echo "To view latest metadata:"
echo "  cat \$(ls -t data/output/metadata_*.json | head -1) | jq"
echo ""
