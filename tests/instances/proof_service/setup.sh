#!/bin/bash

# Setup script for proof service
set -e

echo "Setting up Proof Service..."

# Create data directories
echo "Creating data directories..."
mkdir -p data/{binary,input,output,server_output}

echo ""
echo "Setup complete!"
echo ""
echo "Next steps:"
echo "1. Copy your binary to: data/binary/app.bin"
echo "   cp /path/to/app.bin data/binary/"
echo ""
echo "2. Copy witness data to: data/input/"
echo "   cp /path/to/witness.bin data/input/"
echo ""
echo "3. Start the server:"
echo "   docker-compose up -d proof-server"
echo ""
echo "4. Run the client:"
echo "   docker-compose run --rm proof-client \\"
echo "     --binary /app/binary/app.bin \\"
echo "     --input /app/input/witness.bin \\"
echo "     --output-dir /app/output \\"
echo "     --block-number 1"
echo ""
