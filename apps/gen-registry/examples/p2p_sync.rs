//! Example: P2P synchronization

use gen_registry::{Registry, RegistryConfig};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    println!("Gen Registry - P2P Sync Example\n");

    // Create registry with P2P enabled
    let mut config = RegistryConfig::default();
    config.owner_did = "did:key:example".to_string();
    config.data_dir = "./example-registry-data".to_string();
    config.enable_p2p = true;
    config.auto_sync = true;

    let registry = Registry::with_config(config).await?;

    // Start P2P sync
    println!("Starting P2P sync...");
    registry.start_sync().await?;

    // Discover peers
    println!("Discovering peers...");
    let peers = registry.discover_peers().await?;
    println!("Connected to {} peers", peers.len());

    // Sync specific module
    println!("\nSyncing io.univrs.user...");
    match registry.sync_module("io.univrs.user").await {
        Ok(()) => println!("✓ Synced io.univrs.user"),
        Err(e) => println!("✗ Sync failed: {}", e),
    }

    // Keep running for a bit
    println!("\nKeeping sync active for 30 seconds...");
    sleep(Duration::from_secs(30)).await;

    // Show final status
    let final_peers = registry.discover_peers().await?;
    println!("\nFinal status:");
    println!("  Peers: {}", final_peers.len());
    println!("  Installed modules: {}", registry.list_installed().len());

    Ok(())
}
