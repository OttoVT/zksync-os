# Changelog - Simplified Interface

## Changes Made

### Removed Parameters

The following parameters have been **removed** from both the client and server:

1. **`block_number`** - No longer needed for tracking proofs
2. **`num_instances`** - Fixed to 500 (hardcoded in server)

### What This Means

**Before:**
```bash
proof_client \
  --binary /path/to/app.bin \
  --input /path/to/witness.bin \
  --output-dir /path/to/output \
  --server http://server:50051 \
  --block-number 123 \
  --num-instances 500
```

**Now:**
```bash
proof_client \
  --binary /path/to/app.bin \
  --input /path/to/witness.bin \
  --output-dir /path/to/output \
  --server http://server:50051
```

### Output File Naming

**Before:** `proof_block_123.txt`, `metadata_block_123.json`

**Now:** `proof_1234567890.txt`, `metadata_1234567890.json` (using Unix timestamp)

This ensures unique filenames for each proof without requiring manual tracking.

### Protocol Changes

The protobuf messages have been simplified:

```protobuf
// Before
message ProofRequest {
    bytes binary_data = 1;
    bytes witness_data = 2;
    uint32 num_instances = 3;
    uint64 block_number = 4;
}

message ProofResponse {
    uint64 block_number = 1;
    string encoded_proof = 2;
    // ... other fields
}

// After
message ProofRequest {
    bytes binary_data = 1;
    bytes witness_data = 2;
}

message ProofResponse {
    string encoded_proof = 1;
    double total_time_seconds = 2;
    // ... other fields (no block_number)
}
```

### Server Changes

- **Fixed `num_instances = 500`** (hardcoded in server)
- Removed block number logging
- Simplified request processing

### Migration Guide

If you have existing scripts using the old interface:

1. Remove `--block-number` argument
2. Remove `--num-instances` argument
3. Update output file references:
   - Old: `proof_block_${N}.txt`
   - New: `proof_*.txt` (use `ls -t` to find latest)

### Benefits

✅ **Simpler API** - Only 4 required arguments instead of 6
✅ **Automatic naming** - Timestamp-based filenames prevent conflicts
✅ **Less complexity** - No need to track block numbers manually
✅ **Cleaner code** - Removed unnecessary parameters

All documentation and examples have been updated to reflect these changes.
