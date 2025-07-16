use anyhow::Result;
use clap::Parser;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

mod api;
mod blockchain;
mod p2p;
mod storage;

#[derive(Parser, Debug)]
#[command(
    name = "bits",
    version,
    about = "Decentralized E2EE content marketplace node"
)]
struct Args {
    /// Enable development mode (relaxed security, verbose logging)
    #[arg(short, long)]
    dev: bool,

    /// P2P listening port
    #[arg(short, long, default_value = "9000")]
    port: u16,

    /// Data directory for node storage
    #[arg(long, default_value = "./data")]
    data_dir: String,

    /// Blockchain RPC endpoint
    #[arg(short = 'r', long, default_value = "http://localhost:8545")]
    rpc_url: String,

    /// Bootstrap nodes (multiaddr format)
    #[arg(short = 'b', long)]
    bootstrap: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logging
    let filter = if args.dev {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"))
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    };

    tracing_subscriber::fmt().with_env_filter(filter).init();

    info!("Starting Bits node v{}", env!("CARGO_PKG_VERSION"));

    if args.dev {
        warn!("Running in development mode - security features relaxed!");
    }

    // Initialize storage
    let storage = storage::Storage::new(&args.data_dir).await?;
    info!("Storage initialized at: {}", args.data_dir);

    // Initialize P2P network
    let p2p = p2p::Network::new(args.port, args.bootstrap).await?;
    info!("P2P network initialized on port {}", args.port);

    // Initialize blockchain connection
    let blockchain = blockchain::Client::new(&args.rpc_url).await?;
    info!("Connected to blockchain at: {}", args.rpc_url);

    // Start API server
    let api = api::Server::new(8080, p2p.clone(), storage.clone(), blockchain.clone());

    tokio::select! {
        res = p2p.run() => {
            if let Err(e) = res {
                tracing::error!("P2P network error: {}", e);
            }
        }
        res = api.run() => {
            if let Err(e) = res {
                tracing::error!("API server error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Shutting down...");
        }
    }

    Ok(())
}
