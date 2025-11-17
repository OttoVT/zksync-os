# Proof Service Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         Docker Network                          │
│                                                                 │
│  ┌──────────────────┐              ┌──────────────────┐        │
│  │  Proof Client    │              │  Proof Server    │        │
│  │  Container       │◄────gRPC────►│  Container       │        │
│  │                  │              │                  │        │
│  │ ┌──────────────┐ │              │ ┌──────────────┐ │        │
│  │ │ proof_client │ │              │ │ proof_server │ │        │
│  │ │   binary     │ │              │ │   binary     │ │        │
│  │ └──────────────┘ │              │ └──────────────┘ │        │
│  │                  │              │                  │        │
│  │  Reads:          │              │  Maintains:      │        │
│  │  - Binary data   │              │  - GPU State     │        │
│  │  - Witness data  │              │  - Request Queue │        │
│  │                  │              │  - Binary        │        │
│  │  Writes:         │              │                  │        │
│  │  - Proof file    │              │  Uses:           │        │
│  │  - Metadata      │              │  - GPU (CUDA)    │        │
│  └──────────────────┘              │  - CPU cores     │        │
│           │                        └──────────────────┘        │
│           │                                 │                  │
└───────────┼─────────────────────────────────┼──────────────────┘
            │                                 │
            ▼                                 ▼
    ┌──────────────┐                  ┌─────────────┐
    │ Host Volumes │                  │   NVIDIA    │
    │              │                  │     GPU     │
    │ - binary/    │                  └─────────────┘
    │ - input/     │
    │ - output/    │
    └──────────────┘
```

## Communication Flow

```
Client                          Server
  │                               │
  │  1. Connect (gRPC)            │
  ├──────────────────────────────►│
  │                               │
  │  2. Send ProofRequest         │
  │     - binary_data             │
  │     - witness_data            │
  │     - num_instances           │
  │     - block_number            │
  ├──────────────────────────────►│
  │                               │
  │                               ├─► Add to Queue
  │                               │
  │  3. Initial Status            │
  │◄──────────────────────────────┤
  │                               │
  │                               ├─► Process Request:
  │                               │   - Use cached GPU state
  │                               │   - Generate basic proofs
  │                               │   - Generate recursion proofs
  │                               │   - Serialize & encode
  │                               │
  │  4. Final ProofResponse       │
  │     - encoded_proof           │
  │     - total_time_seconds      │
  │     - cycles                  │
  │     - proof_size_bytes        │
  │◄──────────────────────────────┤
  │                               │
  ├─► Save to files:              │
  │   - proof_block_N.txt         │
  │   - metadata_block_N.json     │
  │                               │
  │  5. Close connection          │
  ├──────────────────────────────►│
  │                               │
```

## Queue Processing

The server processes requests sequentially to optimize GPU usage:

```
Request Queue                    Worker Thread
┌──────────────┐                ┌──────────────────┐
│ Request 1    │───────────────►│                  │
│ (Block 100)  │                │  GPU State       │
├──────────────┤                │  (Initialized    │
│ Request 2    │                │   once, reused)  │
│ (Block 101)  │                │                  │
├──────────────┤                │  1. Lock state   │
│ Request 3    │                │  2. Generate     │
│ (Block 102)  │                │     proof        │
├──────────────┤                │  3. Return       │
│     ...      │                │     result       │
└──────────────┘                │  4. Unlock       │
                                └──────────────────┘
```

## Data Flow

```
┌─────────────┐
│ User Places │
│   Files     │
└──────┬──────┘
       │
       ├─► data/binary/app.bin
       └─► data/input/witness.bin
              │
              │
    ┌─────────▼────────┐
    │  Docker Volume   │
    │    Mounts        │
    └─────────┬────────┘
              │
    ┌─────────▼────────────┐
    │  Client Container    │
    │                      │
    │  Reads files from    │
    │  /app/binary/ and    │
    │  /app/input/         │
    └─────────┬────────────┘
              │
    ┌─────────▼────────────┐
    │  Send via gRPC       │
    │  (ProofRequest)      │
    └─────────┬────────────┘
              │
    ┌─────────▼────────────┐
    │  Server Container    │
    │                      │
    │  - Deserialize data  │
    │  - Generate proof    │
    │  - Return response   │
    └─────────┬────────────┘
              │
    ┌─────────▼────────────┐
    │  Client Container    │
    │                      │
    │  Writes to           │
    │  /app/output/        │
    └─────────┬────────────┘
              │
    ┌─────────▼────────────┐
    │  Docker Volume       │
    │  Mount               │
    └─────────┬────────────┘
              │
              ▼
    data/output/proof_block_N.txt
    data/output/metadata_block_N.json
```

## Component Responsibilities

### Protobuf Definition (`proto/proof_service.proto`)
- Defines the gRPC service interface
- Specifies message formats for requests and responses
- Ensures type safety across client and server

### Server (`server/src/main.rs`)
- **Main Components:**
  - `ServerState`: Manages GPU state and binary
  - `ProofServiceImpl`: Implements gRPC service
  - `proof_worker`: Processes queue asynchronously
  - `generate_proof`: Core proof generation logic

- **Key Features:**
  - One-time GPU initialization
  - Queue-based request processing
  - Thread-safe state management
  - Streaming response support

### Client (`client/src/main.rs`)
- **Main Components:**
  - CLI argument parsing
  - File I/O for binary and witness data
  - gRPC client connection
  - Response handling and file writing

- **Key Features:**
  - Flexible command-line interface
  - Automatic output directory creation
  - Metadata extraction and saving
  - Error handling and reporting

### Docker Configuration

#### Server Dockerfile
- Multi-stage build for smaller image
- CUDA runtime support
- Exposes port 50051 for gRPC

#### Client Dockerfile
- Multi-stage build
- Lightweight runtime
- Configurable via command-line args

#### Docker Compose
- Orchestrates both containers
- Configures GPU passthrough
- Sets up shared network
- Manages volume mounts

## Performance Characteristics

### Initialization
- **First Request**: GPU state initialization (~10-30 seconds)
- **Subsequent Requests**: Immediate processing (state is cached)

### Processing
- **Queue-based**: One proof at a time for optimal GPU usage
- **Parallel Instances**: Configurable (default: 500)
- **Memory Usage**: Depends on num_instances and binary size

### Scalability
- **Vertical**: Increase GPU memory and num_instances
- **Horizontal**: Run multiple server instances (requires load balancer)

## Security Model

### Current Implementation
- **No Authentication**: Direct gRPC communication
- **No Encryption**: Plaintext protocol
- **Network Isolation**: Docker network (bridge mode)

### Production Recommendations
- Implement TLS for gRPC
- Add authentication tokens
- Use secrets management for credentials
- Implement rate limiting
- Add request validation
- Monitor for abuse

## Extension Points

### Custom Proof Types
Extend `ProofRequest` in proto file:
```protobuf
message ProofRequest {
    // ... existing fields ...
    ProofType proof_type = 5;
    bytes custom_params = 6;
}
```

### Multiple Workers
Modify server to spawn multiple worker threads:
```rust
for _ in 0..num_workers {
    let worker_state = state.clone();
    tokio::spawn(proof_worker(task_rx.clone(), worker_state));
}
```

### Status Updates
Enhance with detailed progress:
```rust
// In generate_proof function
response_tx.send(Ok(ProofStatus {
    stage: "generating_basic_proofs",
    progress: 50,
    ...
}));
```

## Monitoring and Debugging

### Server Logs
```bash
docker-compose logs -f proof-server
```

### Client Logs
```bash
# Logs are shown directly when running
docker-compose run --rm proof-client ...
```

### GPU Monitoring
```bash
docker exec proof-server nvidia-smi
```

### Network Debugging
```bash
docker-compose exec proof-client nc -zv proof-server 50051
```
