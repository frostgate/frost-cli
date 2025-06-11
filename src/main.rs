#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_must_use)]
#![allow(dead_code)]

use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use frostgate_circuits::sp1::{Sp1Backend, Sp1Config};
use frostgate_zkip::{ZkBackend, ZkBackendExt, types::ZkConfig};
use anyhow::{Context, Result};

#[derive(Parser)]
#[command(name = "frost-cli")]
#[command(about = "Frostgate ZK-VM Prover CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a ZK proof using the SP1 backend
    Prove {
        /// Path to guest program
        #[arg(short, long)]
        program: PathBuf,
        /// Path to input file (as raw bytes)
        #[arg(short, long)]
        input: PathBuf,
        /// Output path for the proof
        #[arg(short, long)]
        output: PathBuf,
        /// Optional build directory for caching
        #[arg(short, long)]
        build_dir: Option<PathBuf>,
        /// Use GPU acceleration if available
        #[arg(long)]
        gpu: bool,
    },
    /// Verify a ZK proof using the SP1 backend
    Verify {
        /// Path to guest program
        #[arg(short, long)]
        program: PathBuf,
        /// Path to proof file
        #[arg(short, long)]
        proof: PathBuf,
        /// Path to input file (as raw bytes)
        #[arg(short, long)]
        input: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Prove { program, input, output, build_dir, gpu } => {
            // Create SP1 backend with config
            let config = Sp1Config {
                max_concurrent: Some(num_cpus::get()),
                cache_size: 100,
                use_gpu: gpu,
            };
            let backend = Arc::new(Sp1Backend::with_config(config));

            // Read program and input
            let program_bytes = fs::read(&program)
                .with_context(|| format!("Failed to read program file: {}", program.display()))?;
            let input_bytes = fs::read(&input)
                .with_context(|| format!("Failed to read input file: {}", input.display()))?;

            // Generate proof
            let (proof_bytes, metadata) = backend.prove(&program_bytes, &input_bytes, None)
                .await
                .with_context(|| "Failed to generate proof")?;

            // Write proof to output file
            fs::write(&output, &proof_bytes)
                .with_context(|| format!("Failed to write proof to: {}", output.display()))?;

            println!("Successfully generated proof:");
            println!("  Size: {} bytes", metadata.proof_size);
            println!("  Program hash: {}", metadata.program_hash);
            println!("  Generation time: {:?}", metadata.generation_time);
            Ok(())
        },
        Commands::Verify { program, proof, input } => {
            // Create SP1 backend with default config
            let backend = Arc::new(Sp1Backend::new());

            // Read files
            let program_bytes = fs::read(&program)
                .with_context(|| format!("Failed to read program file: {}", program.display()))?;
            let proof_bytes = fs::read(&proof)
                .with_context(|| format!("Failed to read proof file: {}", proof.display()))?;
            let input_bytes = fs::read(&input)
                .with_context(|| format!("Failed to read input file: {}", input.display()))?;

            // Verify proof
            let result = backend.verify(&program_bytes, &proof_bytes, None)
                .await
                .with_context(|| "Failed to verify proof")?;

            if result {
                println!("Proof verification successful!");
            } else {
                println!("Proof verification failed!");
            }
            Ok(())
        }
    }
}