use anyhow::Context;
use bincode::config::standard;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tonic::{transport::Server, Request, Response, Status};
use tracing::{error, info, warn};

use cli_lib::prover_utils::{
    create_proofs_internal, create_recursion_proofs, load_binary_from_path, GpuSharedState,
    Machine, MainCircuitType, ProgramProof, RecursionStrategy,
};

// Include the generated protobuf code
pub mod proof_service {
    tonic::include_proto!("proof_service");
}

use proof_service::proof_service_server::{ProofService, ProofServiceServer};
use proof_service::{ProofRequest, ProofResponse};

/// Proof generation task with request and response channel
struct ProofTask {
    request: ProofRequest,
    response_tx: mpsc::UnboundedSender<Result<ProofResponse, Status>>,
}

/// Server state containing GPU state and binary
struct ServerState {
    binary: Vec<u32>,
    gpu_state: Option<GpuSharedState>,
    recursion_circuit_type: MainCircuitType,
}

impl ServerState {
    fn new(binary_path: &str) -> anyhow::Result<Self> {
        info!("Loading binary from: {}", binary_path);
        let binary = load_binary_from_path(&binary_path.to_string());
        let recursion_circuit_type = MainCircuitType::ReducedRiscVLog23Machine;

        info!("Initializing GPU state...");
        let gpu_state = Some(GpuSharedState::new(&binary, recursion_circuit_type));

        Ok(Self {
            binary,
            gpu_state,
            recursion_circuit_type,
        })
    }
}

/// gRPC service implementation
pub struct ProofServiceImpl {
    task_tx: mpsc::UnboundedSender<ProofTask>,
}

impl ProofServiceImpl {
    fn new(task_tx: mpsc::UnboundedSender<ProofTask>) -> Self {
        Self { task_tx }
    }
}

#[tonic::async_trait]
impl ProofService for ProofServiceImpl {
    type GenerateProofStream = tokio_stream::wrappers::UnboundedReceiverStream<Result<ProofResponse, Status>>;
    type GenerateProofUnaryStream = tokio_stream::wrappers::UnboundedReceiverStream<Result<ProofResponse, Status>>;

    async fn generate_proof(
        &self,
        request: Request<tonic::Streaming<ProofRequest>>,
    ) -> Result<Response<Self::GenerateProofStream>, Status> {
        let mut stream = request.into_inner();
        let (response_tx, response_rx) = mpsc::unbounded_channel();

        let task_tx = self.task_tx.clone();

        tokio::spawn(async move {
            while let Some(result) = stream.message().await.transpose() {
                match result {
                    Ok(req) => {
                        let (proof_response_tx, mut proof_response_rx) = mpsc::unbounded_channel();

                        if let Err(e) = task_tx.send(ProofTask {
                            request: req,
                            response_tx: proof_response_tx,
                        }) {
                            error!("Failed to queue task: {}", e);
                            let _ = response_tx.send(Err(Status::internal("Failed to queue task")));
                            return;
                        }

                        // Forward responses from the worker to the client
                        while let Some(resp) = proof_response_rx.recv().await {
                            if response_tx.send(resp).is_err() {
                                warn!("Client disconnected");
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error receiving request: {}", e);
                        let _ = response_tx.send(Err(Status::internal(format!("Stream error: {}", e))));
                        return;
                    }
                }
            }
        });

        Ok(Response::new(tokio_stream::wrappers::UnboundedReceiverStream::new(response_rx)))
    }

    async fn generate_proof_unary(
        &self,
        request: Request<ProofRequest>,
    ) -> Result<Response<Self::GenerateProofUnaryStream>, Status> {
        let req = request.into_inner();
        let (response_tx, response_rx) = mpsc::unbounded_channel();

        if let Err(e) = self.task_tx.send(ProofTask {
            request: req,
            response_tx,
        }) {
            error!("Failed to queue task: {}", e);
            return Err(Status::internal("Failed to queue task"));
        }

        Ok(Response::new(tokio_stream::wrappers::UnboundedReceiverStream::new(response_rx)))
    }
}

/// Worker that processes proof generation tasks from the queue
async fn proof_worker(
    mut task_rx: mpsc::UnboundedReceiver<ProofTask>,
    state: Arc<Mutex<ServerState>>,
) {
    info!("Proof worker started");

    while let Some(task) = task_rx.recv().await {
        info!("Processing proof request");

        // Send initial status
        let _ = task.response_tx.send(Ok(ProofResponse {
            proof_data: Vec::new(),
            total_time_ms: 0,
            cycles: 0,
            proof_size_bytes: 0,
            error: String::new(),
        }));

        // Generate proof
        match generate_proof(task.request, state.clone()).await {
            Ok(response) => {
                info!("Proof generated successfully");
                let _ = task.response_tx.send(Ok(response));
            }
            Err(e) => {
                error!("Failed to generate proof: {}", e);
                let _ = task.response_tx.send(Ok(ProofResponse {
                    proof_data: Vec::new(),
                    total_time_ms: 0,
                    cycles: 0,
                    proof_size_bytes: 0,
                    error: e.to_string(),
                }));
            }
        }
    }
}

/// Generate a proof from the request
async fn generate_proof(
    request: ProofRequest,
    state: Arc<Mutex<ServerState>>,
) -> anyhow::Result<ProofResponse> {
    let witness_bytes = request.witness_data;
    let num_instances = 500; // Fixed default value

    // Check if witness is hex-encoded (ASCII text) or raw binary
    let witness: Vec<u32> = if witness_bytes.iter().all(|&b| b.is_ascii_hexdigit()) {
        // Hex-encoded witness - decode from hex string
        let hex_string = String::from_utf8(witness_bytes)
            .context("Witness data is not valid UTF-8 hex string")?;

        // Parse hex string into u32 words (8 hex chars = 1 u32)
        anyhow::ensure!(
            hex_string.len() % 8 == 0,
            "Hex witness data length must be a multiple of 8 hex characters"
        );

        hex_string
            .as_bytes()
            .chunks_exact(8)
            .map(|chunk| {
                let hex_str = std::str::from_utf8(chunk).unwrap();
                // Parse as big-endian (matches to_be_bytes() used when writing)
                u32::from_str_radix(hex_str, 16)
                    .with_context(|| format!("Failed to parse hex: {}", hex_str))
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        // Raw binary witness - convert from bytes to u32 words
        anyhow::ensure!(
            witness_bytes.len() % 4 == 0,
            "Binary witness data length must be a multiple of 4 bytes"
        );
        witness_bytes
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect()
    };

    // Lock the state for proof generation
    let state_guard = state.lock().await;

    let start_time = std::time::Instant::now();
    let mut total_proof_time = Some(0.0);

    // Get binary reference (we need to use as_ptr/as_slice pattern to work around borrow checker)
    let binary = &state_guard.binary;

    // Generate basic proofs
    // Use unsafe to get around borrow checker - we know gpu_state doesn't alias with binary
    let (proof_list, proof_metadata) = unsafe {
        let state_ptr = &*state_guard as *const ServerState as *mut ServerState;
        let mut gpu_state_opt = (*state_ptr).gpu_state.as_mut();

        create_proofs_internal(
            binary,
            witness,
            &Machine::Standard,
            num_instances,
            None,
            &mut gpu_state_opt,
            &mut total_proof_time,
        )
    };

    // Calculate cycles
    let cycles = proof_list.basic_proofs.len() * (1 << 22);

    // Generate recursion proofs
    let recursion_mode = RecursionStrategy::UseReducedLog23MachineInBothLayers;
    let (recursion_proof_list, recursion_proof_metadata) = unsafe {
        let state_ptr = &*state_guard as *const ServerState as *mut ServerState;
        let mut gpu_state_opt = (*state_ptr).gpu_state.as_mut();

        create_recursion_proofs(
            proof_list,
            proof_metadata,
            recursion_mode,
            &None,
            &mut gpu_state_opt,
            &mut total_proof_time,
        )
    };

    // Create program proof
    let program_proof = ProgramProof::from_proof_list_and_metadata(
        &recursion_proof_list,
        &recursion_proof_metadata,
    );

    // Serialize the proof
    let serialized_proof = bincode::serde::encode_to_vec(&program_proof, standard())
        .context("Failed to serialize the program proof")?;

    // Calculate total time in milliseconds
    // Note: total_proof_time is in seconds (f64), need to convert to milliseconds
    let total_time_ms = total_proof_time
        .map(|t| (t * 1000.0) as u64)
        .unwrap_or_else(|| start_time.elapsed().as_millis() as u64);

    Ok(ProofResponse {
        proof_data: serialized_proof.clone(),
        total_time_ms,
        cycles: cycles as u64,
        proof_size_bytes: serialized_proof.len() as u64,
        error: String::new(),
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Get configuration from environment variables
    let binary_path = std::env::var("BINARY_PATH")
        .unwrap_or_else(|_| "/app/binary/app.bin".to_string());
    let listen_addr = std::env::var("LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50051".to_string());

    info!("Initializing proof server...");
    info!("Binary path: {}", binary_path);
    info!("Listen address: {}", listen_addr);

    // Initialize server state
    let state = Arc::new(Mutex::new(ServerState::new(&binary_path)?));

    // Create task queue
    let (task_tx, task_rx) = mpsc::unbounded_channel();

    // Spawn proof worker
    let worker_state = state.clone();
    tokio::spawn(async move {
        proof_worker(task_rx, worker_state).await;
    });

    // Create gRPC service
    let service = ProofServiceImpl::new(task_tx);
    let addr = listen_addr.parse()?;

    info!("Starting gRPC server on {}", addr);

    // Set max message size to 100MB for local usage
    let max_message_size = 100 * 1024 * 1024; // 100MB

    Server::builder()
        // HTTP/2 max frame size is limited to 16MB - 1 byte (16777215 bytes)
        .max_frame_size(Some((16 * 1024 * 1024) - 1))
        .add_service(
            ProofServiceServer::new(service)
                .max_decoding_message_size(max_message_size)
                .max_encoding_message_size(max_message_size)
        )
        .serve(addr)
        .await?;

    Ok(())
}
