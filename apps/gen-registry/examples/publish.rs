//! Example: Publishing a module to the registry

use gen_registry::{Capability, Dependency, GenModule, ModuleVersion, Registry, RegistryConfig};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup logging
    tracing_subscriber::fmt::init();

    println!("Gen Registry - Publishing Example\n");

    // Create registry
    let mut config = RegistryConfig::default();
    config.owner_did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string();
    config.data_dir = "./example-registry-data".to_string();

    let registry = Registry::with_config(config).await?;

    // Create module
    let mut module = GenModule::new(
        "io.example.auth",
        "Authentication Module",
        "Provides user authentication with JWT tokens, OAuth, and session management",
        &registry.config.owner_did,
        "MIT",
    );

    // Add tags
    module.add_tag("authentication");
    module.add_tag("security");
    module.add_tag("jwt");
    module.add_tag("oauth");

    // Add dependencies
    module.add_dependency(Dependency::new("io.univrs.crypto", "^1.0.0"));
    module.add_dependency(Dependency::new("io.univrs.time", "^0.5.0"));

    println!("Module: {}", module.name);
    println!("  ID: {}", module.id);
    println!("  Author: {}", module.author_did);
    println!("  License: {}", module.license);
    println!("  Tags: {:?}", module.tags);
    println!("  Dependencies: {}", module.dependencies.len());

    // Create WASM module (in real scenario, this would be compiled)
    let wasm_bytes = create_example_wasm();
    let wasm_path = Path::new("./example-auth.wasm");
    std::fs::write(wasm_path, wasm_bytes)?;

    // Publish
    println!("\nPublishing version 1.0.0...");
    registry
        .publish(
            module,
            "1.0.0",
            wasm_path,
            "Initial release with JWT and OAuth support",
        )
        .await?;

    println!("✓ Successfully published io.example.auth v1.0.0");

    // Clean up
    std::fs::remove_file(wasm_path)?;

    Ok(())
}

fn create_example_wasm() -> Vec<u8> {
    // Minimal valid WASM module
    vec![
        0x00, 0x61, 0x73, 0x6d, // Magic: \0asm
        0x01, 0x00, 0x00, 0x00, // Version: 1
        // Empty module
    ]
}
