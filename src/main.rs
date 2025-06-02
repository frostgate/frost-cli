#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_must_use)]
#![allow(dead_code)]

use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

use frostgate_circuits::sp1::{Sp1Plug, Sp1PlugConfig};
use frostgate_zkip::zkplug::ZkPlug;
use anyhow::Context;

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
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Prove { program, input, output, build_dir } => {
            // Create SP1 config with optional build directory
            let mut config = Sp1PlugConfig::default();
            if let Some(dir) = build_dir {
                config.build_dir = Some(dir);
            }

            // Initialize the SP1 plug
            let mut plug = Sp1Plug::new(config);

            // Read program and input
            println!("Reading program from {:?}", program);
            let program_bytes = fs::read(&program)
                .with_context(|| format!("Failed to read program from {:?}", program))?;

            println!("Reading input from {:?}", input);
            let input_bytes = fs::read(&input)
                .with_context(|| format!("Failed to read input from {:?}", input))?;

            println!("Generating proof...");
            let result = plug.execute(&program_bytes, &input_bytes, None, None)
                .await
                .with_context(|| "Failed to generate proof")?;

            // Serialize and save the proof
            let proof_bytes = bincode::serialize(&result.proof)
                .with_context(|| "Failed to serialize proof")?;

            fs::write(&output, &proof_bytes)
                .with_context(|| format!("Failed to write proof to {:?}", output))?;

            println!(
                "✅ Proof generated and saved to {:?} ({} bytes)",
                output,
                proof_bytes.len()
            );
        }
        Commands::Verify { program, proof, input } => {
            // Initialize SP1 plug with default config for verification
            let mut plug = Sp1Plug::new(Sp1PlugConfig::default());

            // Read necessary files
            println!("Reading program from {:?}", program);
            let program_bytes = fs::read(&program)
                .with_context(|| format!("Failed to read program from {:?}", program))?;

            println!("Reading proof from {:?}", proof);
            let proof_bytes = fs::read(&proof)
                .with_context(|| format!("Failed to read proof from {:?}", proof))?;

            println!("Reading input from {:?}", input);
            let input_bytes = fs::read(&input)
                .with_context(|| format!("Failed to read input from {:?}", input))?;

            // Deserialize the proof
            let proof = bincode::deserialize(&proof_bytes)
                .with_context(|| "Failed to deserialize proof")?;

            println!("Verifying proof...");
            let is_valid = plug.verify(&proof, Some(&input_bytes), None)
                .await
                .with_context(|| "Failed to verify proof")?;

            if is_valid {
                println!("✅ Proof verified successfully!");
            } else {
                println!("❌ Proof verification failed!");
                anyhow::bail!("Proof verification failed");
            }
        }
    }

    Ok(())
}