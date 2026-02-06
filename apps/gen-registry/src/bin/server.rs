//! Gen Registry HTTP Server
//!
//! Serves the web UI and provides REST API for the registry

use gen_registry::{Registry, RegistryConfig};
use std::net::SocketAddr;
use tokio::signal;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("Starting Gen Registry Server");

    // Create registry
    let mut config = RegistryConfig::default();
    config.owner_did = std::env::var("GEN_REGISTRY_DID")
        .unwrap_or_else(|_| "did:key:server".to_string());
    config.data_dir = std::env::var("GEN_REGISTRY_DATA_DIR")
        .unwrap_or_else(|_| "./gen-registry-data".to_string());
    config.enable_p2p = true;
    config.enable_search = true;
    config.auto_sync = true;

    let registry = Registry::with_config(config).await?;

    // Start P2P sync
    info!("Starting P2P sync");
    registry.start_sync().await?;

    // Bind address
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("Server listening on http://{}", addr);

    // In a real implementation, this would start an HTTP server
    // For now, just keep the registry alive
    warn!("HTTP server not yet implemented - only P2P sync active");

    // Wait for shutdown signal
    signal::ctrl_c().await?;
    info!("Shutting down...");

    Ok(())
}
