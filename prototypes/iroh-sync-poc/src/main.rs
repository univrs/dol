use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod app;
mod metrics;
mod p2p;
mod sync;
mod tests;

use app::TodoApp;
use p2p::IrohNode;

#[derive(Parser)]
#[command(name = "iroh-sync-poc")]
#[command(about = "Iroh P2P + Automerge CRDT Proof of Concept")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a node
    Start {
        /// Node name
        #[arg(short, long)]
        name: String,

        /// Port to bind to (default: random)
        #[arg(short, long)]
        port: Option<u16>,

        /// Peer node ID to connect to
        #[arg(short = 'c', long)]
        connect: Option<String>,

        /// Enable relay server mode
        #[arg(long)]
        relay: bool,
    },
    /// Run connectivity tests
    Test {
        /// Test scenario (S1-S6)
        #[arg(short, long)]
        scenario: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "iroh_sync_poc=info,iroh=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start {
            name,
            port,
            connect,
            relay,
        } => {
            start_node(name, port, connect, relay).await?;
        }
        Commands::Test { scenario } => {
            run_test_scenario(&scenario).await?;
        }
    }

    Ok(())
}

async fn start_node(
    name: String,
    port: Option<u16>,
    peer_id: Option<String>,
    relay: bool,
) -> Result<()> {
    tracing::info!("Starting node: {}", name);

    // Create P2P node
    let node = IrohNode::new(name.clone(), port, relay).await?;
    tracing::info!("Node started with ID: {}", node.node_id());

    // Create TodoApp with Automerge
    let mut app = TodoApp::new(name.clone());

    // If peer ID provided, connect to peer
    if let Some(peer) = peer_id {
        tracing::info!("Connecting to peer: {}", peer);
        node.connect_to_peer(&peer).await?;
    }

    // Start sync loop
    tracing::info!("Node ready. Press Ctrl+C to exit.");
    app.run(node).await?;

    Ok(())
}

async fn run_test_scenario(scenario: &str) -> Result<()> {
    tracing::info!("Running test scenario: {}", scenario);

    match scenario {
        "S1" => tests::scenario_same_lan().await?,
        "S2" => tests::scenario_different_lans().await?,
        "S3" => tests::scenario_cellular_wifi().await?,
        "S4" => tests::scenario_symmetric_nat().await?,
        "S5" => tests::scenario_restrictive_firewall().await?,
        "S6" => tests::scenario_partition_healing().await?,
        _ => anyhow::bail!("Unknown scenario: {}", scenario),
    }

    Ok(())
}
