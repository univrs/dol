//! Simple P2P sync example.
//!
//! This example demonstrates basic P2P connectivity and document synchronization
//! between two nodes.
//!
//! Usage:
//! ```bash
//! # Terminal 1: Start first node
//! cargo run --example simple_sync -- node1
//!
//! # Terminal 2: Start second node (copy node1's address from output)
//! cargo run --example simple_sync -- node2 <node1_address>
//! ```

use std::sync::Arc;
use vudo_p2p::{P2PConfig, VudoP2P};
use vudo_state::{DocumentId, StateEngine};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Get command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <node_name> [peer_address]", args[0]);
        std::process::exit(1);
    }

    let node_name = &args[1];
    let peer_addr = args.get(2);

    println!("Starting node: {}", node_name);

    // Create state engine
    let state_engine = Arc::new(StateEngine::new().await?);

    // Create P2P config
    let config = P2PConfig {
        node_name: node_name.clone(),
        enable_relay: true,
        enable_mdns: true,
        enable_dht: false, // Disable DHT for local testing
        ..Default::default()
    };

    // Create P2P instance
    let p2p = VudoP2P::new(Arc::clone(&state_engine), config).await?;

    // Start P2P services
    p2p.start().await?;

    // Print node address
    let node_addr = p2p.node_addr().await?;
    println!("\nNode ID: {}", p2p.node_id());
    println!("Node Address: {:?}", node_addr);
    println!("\nWaiting for connections...");

    // Connect to peer if address provided
    if let Some(addr_str) = peer_addr {
        println!("\nConnecting to peer: {}", addr_str);
        // In a real implementation, you'd parse the address string
        // For now, this is a placeholder
        println!("Note: Peer connection requires proper address parsing");
    }

    // Create a test document
    let doc_id = DocumentId::new("examples", "test-doc");
    let handle = state_engine.create_document(doc_id.clone()).await?;

    // Add some data
    handle.update(|doc| {
        use automerge::transaction::Transactable;
        use automerge::ROOT;
        doc.put(ROOT, "message", format!("Hello from {}", node_name))?;
        doc.put(ROOT, "timestamp", chrono::Utc::now().to_rfc3339())?;
        Ok(())
    })?;

    println!("\nCreated test document: {}/{}", doc_id.namespace, doc_id.key);

    // Print stats periodically
    for i in 1..=10 {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        let peers = p2p.connected_peers();
        let bandwidth_stats = p2p.bandwidth_stats();
        let sync_stats = p2p.sync_stats();

        println!("\n[{}] Stats:", i * 5);
        println!("  Connected peers: {}", peers.len());
        println!("  Bandwidth - Sent: {} bytes, Received: {} bytes",
                 bandwidth_stats.bytes_sent,
                 bandwidth_stats.bytes_received);
        println!("  Sync - Tracked documents: {}",
                 sync_stats.tracked_documents);
    }

    // Stop P2P services
    p2p.stop().await?;

    println!("\nNode stopped");

    Ok(())
}
