# Metadata Format

## Overview

The proof service generates a JSON metadata file for each proof, containing timing and size information.

## File Naming

- **Proof file:** `proof_<timestamp>.bin` (raw binary data)
- **Metadata file:** `metadata_<timestamp>.json` (JSON format)

Where `<timestamp>` is the Unix timestamp in seconds when the client saved the files.

## JSON Structure

The metadata is serialized using a proper Rust struct (`ProofMetadata`) with the following format:

```json
{
  "timestamp": 1730217890,
  "total_time_ms": 45678,
  "cycles": 2097152,
  "proof_size_bytes": 123456
}
```

## Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `timestamp` | `u64` | Unix timestamp (seconds since epoch) when the proof was saved |
| `total_time_ms` | `u64` | Total time taken for proof generation in **milliseconds** |
| `cycles` | `u64` | Number of cycles in the proof computation |
| `proof_size_bytes` | `u64` | Size of the proof file in bytes |

## Time Format

**Important:** Time is now in **milliseconds** (not seconds) for better precision.

- Value: `45678` ms = 45.678 seconds
- Format: Always as integer milliseconds
- Type: `u64` (unsigned 64-bit integer)

## Example Usage

### Reading Metadata in Bash

```bash
# Get latest metadata file
latest_metadata=$(ls -t data/output/metadata_*.json | head -1)

# Pretty print
cat "$latest_metadata" | jq

# Extract specific field
cat "$latest_metadata" | jq '.total_time_ms'

# Convert to seconds
cat "$latest_metadata" | jq '.total_time_ms / 1000'
```

### Reading Metadata in Rust

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct ProofMetadata {
    timestamp: u64,
    total_time_ms: u64,
    cycles: u64,
    proof_size_bytes: u64,
}

// Read and parse
let metadata_json = std::fs::read_to_string("metadata_1730217890.json")?;
let metadata: ProofMetadata = serde_json::from_str(&metadata_json)?;

println!("Proof took {} ms ({:.2} seconds)",
    metadata.total_time_ms,
    metadata.total_time_ms as f64 / 1000.0
);
```

### Reading Metadata in Python

```python
import json

# Read metadata
with open('metadata_1730217890.json', 'r') as f:
    metadata = json.load(f)

# Access fields
print(f"Timestamp: {metadata['timestamp']}")
print(f"Time: {metadata['total_time_ms']}ms ({metadata['total_time_ms']/1000:.2f}s)")
print(f"Cycles: {metadata['cycles']}")
print(f"Size: {metadata['proof_size_bytes']} bytes")
```

## Changes from Previous Version

### Before
```json
{
  "block_number": 123,
  "total_time_seconds": 45.678,
  "cycles": 2097152,
  "proof_size_bytes": 123456
}
```

### Now
```json
{
  "timestamp": 1730217890,
  "total_time_ms": 45678,
  "cycles": 2097152,
  "proof_size_bytes": 123456
}
```

### Key Differences

1. **`block_number` → `timestamp`**
   - More flexible identification
   - Automatically unique
   - Matches filename convention

2. **`total_time_seconds` (f64) → `total_time_ms` (u64)**
   - Changed from floating-point seconds to integer milliseconds
   - Better precision (no floating-point rounding)
   - More standard format for timing data

3. **Type-safe struct**
   - Now uses proper Rust `#[derive(Serialize)]` struct
   - Type-checked at compile time
   - Self-documenting code

## Console Output

When the client receives a proof, it displays:

```
INFO  Received proof from server
INFO    Time: 45678ms (45.68s)
INFO    Cycles: 2097152
INFO    Proof size: 123456 bytes
INFO  Proof saved successfully!
INFO    Proof: "data/output/proof_1730217890.bin"
INFO    Metadata: "data/output/metadata_1730217890.json"
```

## Protocol Definition

In the protobuf definition ([proto/proof_service.proto](proto/proof_service.proto)):

```protobuf
message ProofResponse {
    bytes proof_data = 1;
    uint64 total_time_ms = 2;  // milliseconds
    uint64 cycles = 3;
    uint64 proof_size_bytes = 4;
    string error = 5;
}
```

## Best Practices

1. **Store both files together** - Keep proof and metadata files in the same directory
2. **Match timestamps** - The timestamp in the metadata matches the filename
3. **Use milliseconds** - When processing, remember time is in ms, not seconds
4. **Type safety** - When deserializing, use strongly-typed structs
5. **Precision** - Integer milliseconds avoid floating-point precision issues
