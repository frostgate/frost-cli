#![allow(unused_imports)]
#![allow(unused_variables)]

use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use frostgate_circuits::sp1_plug::*;
use frostgate_zkip::zkplug::*;

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
        /// Path to guest ELF binary
        #[arg(short, long)]
        guest: PathBuf,
        /// Path to input file (as raw bytes)
        #[arg(short, long)]
        input: PathBuf,
        /// Output path for the proof (serialized)
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Verify a ZK proof using the SP1 backend
    Verify {
        /// Path to guest ELF binary
        #[arg(short, long)]
        guest: PathBuf,
        /// Path to proof file (as serialized)
        #[arg(short, long)]
        proof: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Prove { guest, input, output } => {
            let prover = Arc::new(Sp1Plug::new(None)); // FIX: Use None for config, not guest path

            println!("Reading input from {:?}", input);
            let input_bytes = fs::read(input)?;

            println!("Generating proof...");
            let zk_config = ZkConfig::default();
            let proof = prover
                .prove(&input_bytes, None, Some(&zk_config))
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            // You may need to implement or use a serialization method for your proof type
            let proof_bytes = bincode::serialize(&proof)
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            fs::write(&output, &proof_bytes)?;
            println!(
                "Proof generated and saved to {:?} ({} bytes)",
                output,
                proof_bytes.len()
            );
        }
        Commands::Verify { guest, proof } => {
            let prover = Arc::new(Sp1Plug::new(None)); // FIX: Use None for config, not guest path

            println!("Reading proof from {:?}", proof);
            let proof_bytes = fs::read(&proof)?;
            let zk_proof: ZkProof<<Sp1Plug as ZkPlug>::Proof> = bincode::deserialize(&proof_bytes)
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            println!("Verifying proof...");
            let zk_config = ZkConfig::default();
            let verified = prover
                .verify(&zk_proof, None, Some(&zk_config))
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            if verified {
                println!("✅ Proof verified!");
            } else {
                println!("❌ Proof invalid!");
            }
        }
    }

    Ok(())
}