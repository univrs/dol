//! Example: Searching for modules

use gen_registry::{Registry, RegistryConfig, SearchQuery};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    println!("Gen Registry - Search Example\n");

    // Create registry
    let mut config = RegistryConfig::default();
    config.owner_did = "did:key:example".to_string();
    config.data_dir = "./example-registry-data".to_string();

    let registry = Registry::with_config(config).await?;

    // Search by keyword
    println!("Searching for 'authentication'...");
    let results = registry.search("authentication").await;

    match results {
        Ok(modules) => {
            println!("Found {} modules:\n", modules.len());
            for result in modules {
                println!("  {} v{}", result.name, result.version);
                println!("    {}", result.description);
                println!("    Score: {:.2}\n", result.score);
            }
        }
        Err(e) => {
            println!("Search failed: {}", e);
        }
    }

    // Search by tags
    println!("\nSearching by tags: ['database', 'sql']");
    let tag_results = registry.search_by_tags(&["database", "sql"]).await?;

    println!("Found {} modules:", tag_results.len());
    for result in tag_results {
        println!("  {} v{}", result.name, result.version);
    }

    Ok(())
}
