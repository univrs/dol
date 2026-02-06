//! Privacy Levels Comparison
//!
//! This example compares the different privacy levels available in PlanetServe:
//! - None: Direct sync, no privacy
//! - Basic: Message padding only
//! - Standard: Padding + timing jitter
//! - Maximum: S-IDA + Onion routing + Cover traffic
//!
//! Run with:
//! ```
//! cargo run --example privacy_levels
//! ```

use std::sync::Arc;
use std::time::Instant;
use vudo_identity::MasterIdentity;
use vudo_p2p::{P2PConfig, VudoP2P};
use vudo_planetserve::config::{PrivacyConfig, PrivacyLevel};
use vudo_planetserve::PlanetServeAdapter;
use vudo_state::StateEngine;

async fn benchmark_privacy_level(name: &str, config: PrivacyConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== {} ===", name);

    let identity = Arc::new(MasterIdentity::generate("Test").await?);
    let state_engine = Arc::new(StateEngine::new().await?);
    let p2p = Arc::new(VudoP2P::new(state_engine, P2PConfig::default()).await?);

    let adapter = PlanetServeAdapter::new(identity, p2p, config).await?;

    // Start if needed
    if adapter.config().level == PrivacyLevel::Maximum {
        adapter.start().await?;
    }

    println!("Configuration:");
    println!("  Privacy Level: {:?}", adapter.config().level);
    println!("  S-IDA: k={}, n={}", adapter.config().sida.k, adapter.config().sida.n);
    println!("  Onion Hops: {}", adapter.config().onion.hops);
    println!("  Padding: {} bytes", adapter.config().padding_size);
    println!("  Timing Jitter: {} ms", adapter.config().timing_jitter);
    println!("  Cover Traffic: {} msg/min", adapter.config().cover_traffic_rate);
    println!();

    // Benchmark sync
    let message = b"Test sync message with some data";
    println!("Syncing message ({} bytes)...", message.len());

    let start = Instant::now();
    adapter.sync_private("test_ns", "doc1", message.to_vec()).await?;
    let elapsed = start.elapsed();

    println!("  ✓ Sync completed in {:?}", elapsed);
    println!();

    // Stop if needed
    if adapter.config().level == PrivacyLevel::Maximum {
        adapter.stop().await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PlanetServe Privacy Levels Comparison ===\n");

    // Test all privacy levels
    benchmark_privacy_level("None (Fast-Open)", PrivacyConfig::fast_open()).await?;
    benchmark_privacy_level("Basic", PrivacyConfig::basic()).await?;
    benchmark_privacy_level("Standard", PrivacyConfig::default()).await?;
    benchmark_privacy_level("Maximum (Privacy-Max)", PrivacyConfig::privacy_max()).await?;

    println!("=== Privacy vs. Performance Trade-offs ===\n");

    println!("| Level    | S-IDA | Onion | Padding | Jitter | Cover | Est. Latency |");
    println!("|----------|-------|-------|---------|--------|-------|--------------|");
    println!("| None     | No    | No    | No      | No     | No    | 0ms          |");
    println!("| Basic    | No    | No    | Yes     | No     | No    | <5ms         |");
    println!("| Standard | No    | No    | Yes     | Yes    | No    | ~100ms       |");
    println!("| Maximum  | Yes   | Yes   | Yes     | Yes    | Yes   | ~500ms       |");
    println!();

    println!("=== Use Cases ===\n");

    println!("None (Fast-Open):");
    println!("  - Public data");
    println!("  - Performance-critical operations");
    println!("  - No privacy requirements");
    println!();

    println!("Basic:");
    println!("  - Internal operations");
    println!("  - Hide message size");
    println!("  - Minimal overhead");
    println!();

    println!("Standard (Default):");
    println!("  - Normal operations");
    println!("  - Balance privacy and performance");
    println!("  - Resist basic traffic analysis");
    println!();

    println!("Maximum (Privacy-Max):");
    println!("  - Sensitive operations (credit transfers)");
    println!("  - Maximum privacy requirements");
    println!("  - Resist sophisticated attacks");
    println!();

    println!("=== Summary ===");
    println!("✓ Four privacy levels for different use cases");
    println!("✓ Configurable trade-offs between privacy and performance");
    println!("✓ User can choose appropriate level for each operation");

    Ok(())
}
