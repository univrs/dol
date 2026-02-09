//! S-IDA Fragmentation Demo
//!
//! This example demonstrates how S-IDA (Secure Information Dispersal Algorithm)
//! fragments a message across multiple peers, ensuring that no single peer can
//! observe the full message.
//!
//! Run with:
//! ```
//! cargo run --example sida_fragmentation
//! ```

use vudo_planetserve::config::SidaConfig;
use vudo_planetserve::sida::SidaFragmenter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== S-IDA Fragmentation Demo ===\n");

    // Configuration: 3-of-5 threshold
    let config = SidaConfig { k: 3, n: 5 };
    let fragmenter = SidaFragmenter::new(config)?;

    println!("Configuration: k={}, n={}", config.k, config.n);
    println!("  - Any {} fragments can reconstruct the message", config.k);
    println!("  - Having < {} fragments reveals NO information\n", config.k);

    // Original message
    let message = b"Alice is transferring 100 credits to Bob for medical supplies";
    println!("Original message ({} bytes):", message.len());
    println!("  \"{}\"", String::from_utf8_lossy(message));
    println!();

    // Fragment the message
    println!("Fragmenting message into {} shards...", config.n);
    let fragments = fragmenter.fragment(message)?;

    println!("\nFragment distribution:");
    for (i, fragment) in fragments.iter().enumerate() {
        println!(
            "  Peer {} receives fragment {} ({} bytes)",
            i + 1,
            fragment.index,
            fragment.size()
        );
        println!("    Data preview: {:?}...", &fragment.data[..std::cmp::min(20, fragment.data.len())]);
    }

    // Demonstrate that single fragment reveals nothing
    println!("\n=== Privacy Test ===");
    println!("Can Peer 1 (with only fragment 0) read the message?");
    let single_fragment_data = &fragments[0].data;
    println!("  Fragment 0 data: {:?}...", &single_fragment_data[..std::cmp::min(40, single_fragment_data.len())]);
    println!("  Can reconstruct? NO - need at least {} fragments", config.k);
    println!();

    // Demonstrate reconstruction with k fragments
    println!("=== Reconstruction Test 1: Using fragments [0, 1, 2] ===");
    let subset1: Vec<_> = fragments.iter().take(3).cloned().collect();
    let reconstructed1 = fragmenter.reconstruct(subset1)?;
    println!("Reconstructed message:");
    println!("  \"{}\"", String::from_utf8_lossy(&reconstructed1));
    println!("  Match: {}", reconstructed1 == message);
    println!();

    // Demonstrate reconstruction with different k fragments
    println!("=== Reconstruction Test 2: Using fragments [1, 3, 4] ===");
    let subset2 = vec![
        fragments[1].clone(),
        fragments[3].clone(),
        fragments[4].clone(),
    ];
    let reconstructed2 = fragmenter.reconstruct(subset2)?;
    println!("Reconstructed message:");
    println!("  \"{}\"", String::from_utf8_lossy(&reconstructed2));
    println!("  Match: {}", reconstructed2 == message);
    println!();

    // Demonstrate failure with insufficient fragments
    println!("=== Reconstruction Test 3: Using only 2 fragments (SHOULD FAIL) ===");
    let insufficient: Vec<_> = fragments.iter().take(2).cloned().collect();
    match fragmenter.reconstruct(insufficient) {
        Ok(_) => println!("  ERROR: Should not have succeeded!"),
        Err(e) => println!("  Failed as expected: {}", e),
    }
    println!();

    println!("=== Summary ===");
    println!("✓ Message fragmented into {} shards", config.n);
    println!("✓ No single peer can read the message");
    println!("✓ Any {} peers can collaborate to reconstruct", config.k);
    println!("✓ Privacy preserved while maintaining availability");

    Ok(())
}
