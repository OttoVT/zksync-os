# Proof Service - Client-Server Architecture

A distributed proof generation system with bidirectional gRPC communication, designed to run in Docker containers with GPU acceleration.

## Architecture

This system consists of two main components:

### Server (`proof_server`)
- Maintains initialized GPU state for optimal performance
- Processes proof generation requests in a queue-based manner
- Implements the proof generation logic from `ethproofs_with_proofs`
- Sends status updates and final proofs back to clients
- Runs continuously as a service

### Client (`proof_client`)
- Sends binary and witness data to the server
- Receives bidirectional status updates during proof generation
- Saves generated proofs and metadata to the output directory
- Runs as needed for each proof request

### Communication Protocol
- Uses gRPC for efficient bidirectional streaming
- Supports binary data transfer for witness and proof data
- Provides real-time status updates during proof generation
- Protocol defined in `proto/proof_service.proto`

## Directory Structure

```
proof_service/
├── proto/
│   └── proof_service.proto    # gRPC service definition
├── server/
│   ├── src/main.rs             # Server implementation
│   ├── Cargo.toml
│   └── build.rs
├── client/
│   ├── src/main.rs             # Client implementation
│   ├── Cargo.toml
│   └── build.rs
├── docker/
│   ├── Dockerfile.server       # Server container
│   └── Dockerfile.client       # Client container
├── docker-compose.yml          # Orchestration
├── data/                       # Runtime data (create this)
│   ├── binary/                 # Place app.bin here
│   ├── input/                  # Place witness files here
│   └── output/                 # Proofs saved here
└── README.md
```

## Prerequisites

- Docker and Docker Compose
- NVIDIA GPU with CUDA support (for server)
- nvidia-docker runtime
- The zkSync binary (`app.bin`)
- Witness data files

## Setup

### 1. Create Data Directories

```bash
cd /home/aleksandr/zksync-os/tests/instances/proof_service
mkdir -p data/{binary,input,output,server_output}
```

### 2. Place Required Files

```bash
# Copy your binary
cp /path/to/app.bin data/binary/

# Copy witness data
cp /path/to/witness.bin data/input/
```

### 3. Verify NVIDIA Docker Runtime

```bash
docker run --rm --gpus all nvidia/cuda:12.2.0-base-ubuntu22.04 nvidia-smi
```

## Usage

### Running with Docker Compose

#### Start the Server

```bash
# Build and start the server
docker-compose up -d proof-server

# View server logs
docker-compose logs -f proof-server
```

#### Run the Client

```bash
# Run client with default settings
docker-compose run --rm proof-client

# Run with custom parameters
docker-compose run --rm proof-client \
  --binary /app/binary/app.bin \
  --input /app/input/witness.bin \
  --output-dir /app/output \
  --server http://proof-server:50051 \
  --block-number 123 \
  --num-instances 500
```

#### Stop the Server

```bash
docker-compose down
```

### Running Locally (Without Docker)

#### Build the Server

```bash
cd server
cargo build --release
```

#### Run the Server

```bash
BINARY_PATH=/path/to/app.bin \
LISTEN_ADDR=0.0.0.0:50051 \
RUST_LOG=info \
./target/release/proof_server
```

#### Build the Client

```bash
cd client
cargo build --release
```

#### Run the Client

```bash
./target/release/proof_client \
  --binary /path/to/app.bin \
  --input /path/to/witness.bin \
  --output-dir ./output \
  --server http://localhost:50051 \
  --block-number 1 \
  --num-instances 500
```

## Client Command-Line Options

```
proof_client [OPTIONS]

Options:
  -b, --binary <BINARY>              Path to the binary file [required]
  -i, --input <INPUT>                Path to the witness/input file [required]
  -o, --output-dir <OUTPUT_DIR>      Output directory to store the proof [required]
  -s, --server <SERVER>              Server address [default: http://localhost:50051]
      --block-number <BLOCK_NUMBER>  Block number or request identifier [default: 0]
  -n, --num-instances <NUM_INSTANCES> Number of parallel instances [default: 500]
  -h, --help                         Print help
  -V, --version                      Print version
```

## Server Environment Variables

- `BINARY_PATH`: Path to the binary file (default: `/app/binary/app.bin`)
- `LISTEN_ADDR`: Server listen address (default: `0.0.0.0:50051`)
- `RUST_LOG`: Log level (default: `info`)

## Output Files

The client saves two files per proof:

1. **Proof File**: `proof_block_<block_number>.txt`
   - Contains the base64-encoded proof

2. **Metadata File**: `metadata_block_<block_number>.json`
   - Contains proof generation metadata:
     ```json
     {
       "block_number": 123,
       "total_time_seconds": 45.67,
       "cycles": 2097152,
       "proof_size_bytes": 12345
     }
     ```

## Example Workflow

```bash
# 1. Start the server
docker-compose up -d proof-server

# 2. Wait for server to initialize (check logs)
docker-compose logs -f proof-server
# Wait for: "Starting gRPC server on 0.0.0.0:50051"

# 3. Run client to generate a proof
docker-compose run --rm proof-client \
  --binary /app/binary/app.bin \
  --input /app/input/witness_block_100.bin \
  --output-dir /app/output \
  --block-number 100

# 4. Check the output
ls -lh data/output/
# proof_block_100.txt
# metadata_block_100.json

# 5. Generate another proof (server state is reused)
docker-compose run --rm proof-client \
  --binary /app/binary/app.bin \
  --input /app/input/witness_block_101.bin \
  --output-dir /app/output \
  --block-number 101
```

## Troubleshooting

### Server fails to start with GPU errors

Ensure nvidia-docker runtime is properly configured:
```bash
# Check GPU availability
docker run --rm --gpus all nvidia/cuda:12.2.0-base-ubuntu22.04 nvidia-smi

# Verify docker daemon has nvidia runtime
cat /etc/docker/daemon.json
```

### Client can't connect to server

- Ensure server is running: `docker-compose ps`
- Check server logs: `docker-compose logs proof-server`
- Verify network connectivity: `docker-compose exec proof-client ping proof-server`

### Build failures

Ensure you have the required dependencies:
```bash
# Install protobuf compiler
sudo apt-get update && sudo apt-get install -y protobuf-compiler
```

### Out of memory errors

Reduce the number of parallel instances:
```bash
docker-compose run --rm proof-client \
  --num-instances 100  # Instead of default 500
```

## Performance Considerations

- **GPU State Persistence**: The server maintains GPU state across requests, eliminating reinitialization overhead
- **Queue Processing**: Multiple clients can submit requests; the server processes them sequentially
- **Parallel Proving**: Adjust `--num-instances` based on available GPU memory
- **Binary Size**: Large binaries increase initialization time; consider caching

## Security Considerations

- The server accepts binary data from clients - ensure proper network isolation
- Use TLS for production deployments (modify the gRPC configuration)
- Implement authentication/authorization as needed
- Limit resource usage to prevent DoS attacks

## Extending the System

### Adding Authentication

Modify the gRPC service to include authentication tokens in metadata.

### Adding Multiple Workers

Scale the server by running multiple instances behind a load balancer.

### Custom Proof Types

Extend the `ProofRequest` message in `proof_service.proto` to support different proof types.

## License

This project follows the workspace license configuration.
