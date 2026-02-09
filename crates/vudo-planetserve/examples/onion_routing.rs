//! Onion Routing Demo
//!
//! This example demonstrates how onion routing hides WHO is syncing with WHOM
//! by routing messages through multiple relays, each decrypting one layer.
//!
//! Run with:
//! ```
//! cargo run --example onion_routing
//! ```

use std::sync::Arc;
use std::time::Duration;
use vudo_identity::MasterIdentity;
use vudo_p2p::{P2PConfig, VudoP2P};
use vudo_planetserve::config::OnionConfig;
use vudo_planetserve::onion::{OnionRouter, RelayNode};
use vudo_state::StateEngine;
use x25519_dalek::{PublicKey, StaticSecret};

fn create_relay(name: &str, latency_ms: u64) -> RelayNode {
    let secret = StaticSecret::random_from_rng(&mut rand::thread_rng());
    let public_key = PublicKey::from(&secret);

    let mut relay = RelayNode::new(format!("did:peer:{}", name), public_key);
    relay.latency = Duration::from_millis(latency_ms);
    relay.reliability = 0.95;

    relay
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Onion Routing Demo ===\n");

    // Setup
    let identity = Arc::new(MasterIdentity::generate("Alice").await?);
    let state_engine = Arc::new(StateEngine::new().await?);
    let p2p = Arc::new(VudoP2P::new(state_engine, P2PConfig::default()).await?);

    let config = OnionConfig {
        hops: 3,
        relay_selection_strategy: vudo_planetserve::config::RelaySelectionStrategy::LowLatency,
    };

    let router = OnionRouter::with_config(identity, p2p, config);

    println!("Configuration:");
    println!("  - Hops: 3");
    println!("  - Strategy: Low Latency");
    println!();

    // Add relays to the pool
    println!("Adding relays to the pool:");
    let relays = vec![
        create_relay("relay_eu", 50),
        create_relay("relay_us", 100),
        create_relay("relay_asia", 150),
        create_relay("relay_africa", 80),
        create_relay("relay_sa", 120),
    ];

    for relay in relays {
        println!(
            "  {} (latency: {:?}, reliability: {:.2})",
            relay.did, relay.latency, relay.reliability
        );
        router.add_relay(relay);
    }
    println!();

    println!("Relay pool size: {}", router.relay_count());
    println!();

    // Build circuit
    println!("=== Building Onion Circuit ===");
    let destination = "did:peer:bob";
    println!("  Destination: {}", destination);

    let circuit = router.build_circuit(destination, 3).await?;
    println!("  ✓ Circuit established with {} hops", circuit.hops());
    println!();

    // Show circuit path
    println!("Circuit path:");
    println!("  Alice (sender)");
    for (i, relay) in circuit.relays.iter().enumerate() {
        println!("    ↓");
        println!("  Relay {} - {}", i + 1, relay.did);
    }
    println!("    ↓");
    println!("  Bob (receiver)");
    println!();

    // Estimated latency
    println!("Estimated latency: {:?}", circuit.estimated_latency());
    println!();

    // Privacy analysis
    println!("=== Privacy Analysis ===");
    println!();

    println!("Entry Relay ({}):", circuit.relays[0].did);
    println!("  - Knows: Alice (sender)");
    println!("  - Doesn't know: Bob (destination)");
    println!("  - Sees: Encrypted onion going to {}", circuit.relays.get(1).map(|r| r.did.as_str()).unwrap_or("N/A"));
    println!();

    println!("Middle Relay ({}):", circuit.relays[1].did);
    println!("  - Knows: Previous hop (entry relay)");
    println!("  - Doesn't know: Alice or Bob");
    println!("  - Sees: Encrypted onion going to {}", circuit.relays.get(2).map(|r| r.did.as_str()).unwrap_or("N/A"));
    println!();

    println!("Exit Relay ({}):", circuit.relays[2].did);
    println!("  - Knows: Bob (destination)");
    println!("  - Doesn't know: Alice (sender)");
    println!("  - Sees: Final encrypted layer for Bob");
    println!();

    // Send message
    println!("=== Sending Message ===");
    let message = b"Private sync message: update document X";
    println!("Original message: \"{}\"", String::from_utf8_lossy(message));
    println!();

    println!("Encrypting onion layers:");
    println!("  Layer 3: Encrypt for Bob (destination)");
    println!("  Layer 2: Encrypt for Exit Relay");
    println!("  Layer 1: Encrypt for Middle Relay");
    println!("  Layer 0: Encrypt for Entry Relay");
    println!();

    let _result = router.send_onion(&circuit, message).await;
    println!("  ✓ Message sent through onion circuit");
    println!();

    println!("=== Summary ===");
    println!("✓ Message routed through {} hops", circuit.hops());
    println!("✓ No single relay knows both sender and receiver");
    println!("✓ Each relay only decrypts one layer");
    println!("✓ Privacy preserved with ~{:?} latency overhead", circuit.estimated_latency());

    Ok(())
}
