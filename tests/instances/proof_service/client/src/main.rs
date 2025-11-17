use anyhow::Context;
use clap::Parser;
use serde::Serialize;
use std::path::PathBuf;
use tokio::fs;
use tonic::Request;
use tracing::{error, info};

// Include the generated protobuf code
pub mod proof_service {
    tonic::include_proto!("proof_service");
}

use proof_service::proof_service_client::ProofServiceClient;
use proof_service::ProofRequest;

/// Metadata structure for proof generation
#[derive(Serialize)]
struct ProofMetadata {
    /// Unix timestamp when the proof was generated
    timestamp: u64,
    /// Total time taken for proof generation in milliseconds
    total_time_ms: u64,
    /// Number of cycles in the proof
    cycles: u64,
    /// Size of the proof in bytes
    proof_size_bytes: u64,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the binary file
    #[arg(short, long)]
    binary: PathBuf,

    /// Path to the witness/input file
    #[arg(short, long)]
    input: PathBuf,

    /// Output directory to store the proof
    #[arg(short, long)]
    output_dir: PathBuf,

    /// Server address (default: http://localhost:50051)
    #[arg(short, long, default_value = "http://localhost:50051")]
    server: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args = Args::parse();

    info!("Proof Client Starting");
    info!("Binary: {:?}", args.binary);
    info!("Input: {:?}", args.input);
    info!("Output directory: {:?}", args.output_dir);
    info!("Server: {}", args.server);

    // Create output directory if it doesn't exist
    fs::create_dir_all(&args.output_dir)
        .await
        .context("Failed to create output directory")?;

    // Read binary data
    info!("Reading binary file...");
    let binary_data = fs::read(&args.binary)
        .await
        .context("Failed to read binary file")?;
    info!("Binary size: {} bytes", binary_data.len());

    // Read witness/input data
    info!("Reading witness/input file...");
    let witness_data = fs::read(&args.input)
        .await
        .context("Failed to read witness file")?;
    info!("Witness size: {} bytes", witness_data.len());

    // Connect to the server
    info!("Connecting to server at {}...", args.server);

    // Set max message size to 100MB for local usage
    let max_message_size = 100 * 1024 * 1024; // 100MB

    let mut client = ProofServiceClient::connect(args.server.clone())
        .await
        .context("Failed to connect to server")?
        .max_decoding_message_size(max_message_size)
        .max_encoding_message_size(max_message_size);
    info!("Connected to server");

    // Create proof request
    let request = ProofRequest {
        binary_data,
        witness_data,
    };

    info!("Sending proof request...");

    // Send request and receive streaming responses
    let mut response_stream = client
        .generate_proof_unary(Request::new(request))
        .await
        .context("Failed to send proof request")?
        .into_inner();

    let mut final_response = None;

    // Process responses
    while let Some(response) = response_stream
        .message()
        .await
        .context("Error receiving response")?
    {
        if !response.error.is_empty() {
            error!("Server error: {}", response.error);
            anyhow::bail!("Proof generation failed: {}", response.error);
        }

        if response.proof_data.is_empty() {
            // Status update
            info!("Received status update from server");
        } else {
            // Final response with proof
            info!("Received proof from server");
            info!("  Time: {}ms ({:.2}s)", response.total_time_ms, response.total_time_ms as f64 / 1000.0);
            info!("  Cycles: {}", response.cycles);
            info!("  Proof size: {} bytes", response.proof_size_bytes);
            final_response = Some(response);
        }
    }

    // Save the proof to output directory
    if let Some(response) = final_response {
        // Use timestamp for unique filenames
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let output_file = args
            .output_dir
            .join("proof.bin");

        info!("Saving proof to {:?}", output_file);
        fs::write(&output_file, &response.proof_data)
            .await
            .context("Failed to write proof to file")?;

        // Also save metadata
        let metadata_file = args
            .output_dir
            .join("metadata.json");

        let metadata = ProofMetadata {
            timestamp,
            total_time_ms: response.total_time_ms,
            cycles: response.cycles,
            proof_size_bytes: response.proof_size_bytes,
        };

        let metadata_json = serde_json::to_string_pretty(&metadata)
            .context("Failed to serialize metadata")?;

        fs::write(&metadata_file, metadata_json)
            .await
            .context("Failed to write metadata to file")?;

        info!("Proof saved successfully!");
        info!("  Proof: {:?}", output_file);
        info!("  Metadata: {:?}", metadata_file);
    } else {
        anyhow::bail!("No proof received from server");
    }

    Ok(())
}
