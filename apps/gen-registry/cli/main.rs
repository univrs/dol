//! Gen Registry CLI
//!
//! Command-line interface for publishing, searching, and installing Gen modules

use clap::{Parser, Subcommand};
use gen_registry::{GenModule, Registry, RegistryConfig, SearchQuery};
use std::path::PathBuf;
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Parser)]
#[command(name = "gen-registry")]
#[command(about = "Community registry for DOL Gen modules", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// DID for authentication
    #[arg(long, env = "GEN_REGISTRY_DID")]
    did: Option<String>,

    /// Data directory
    #[arg(long, default_value = "./gen-registry-data")]
    data_dir: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Publish a new module
    Publish {
        /// Module ID (e.g., io.univrs.user)
        #[arg(long)]
        id: String,

        /// Module name
        #[arg(long)]
        name: String,

        /// Description
        #[arg(long)]
        description: String,

        /// License (MIT, Apache-2.0, etc.)
        #[arg(long, default_value = "MIT")]
        license: String,

        /// Version (semver)
        #[arg(long)]
        version: String,

        /// Path to WASM file
        #[arg(long)]
        wasm: PathBuf,

        /// Changelog
        #[arg(long)]
        changelog: String,

        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
    },

    /// Search for modules
    Search {
        /// Search query
        query: String,

        /// Maximum results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Install a module
    Install {
        /// Module ID
        module_id: String,

        /// Specific version (default: latest)
        #[arg(short, long)]
        version: Option<String>,

        /// Enable auto-update
        #[arg(long)]
        auto_update: bool,
    },

    /// List installed modules
    List,

    /// Show module details
    Info {
        /// Module ID
        module_id: String,
    },

    /// Rate a module
    Rate {
        /// Module ID
        module_id: String,

        /// Stars (1-5)
        #[arg(short, long)]
        stars: u8,

        /// Review text
        #[arg(short, long)]
        review: Option<String>,
    },

    /// Start P2P sync daemon
    Daemon {
        /// Port for HTTP API
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },

    /// Sync modules from P2P network
    Sync {
        /// Specific module ID (optional)
        module_id: Option<String>,
    },

    /// Show sync status
    Status,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Setup logging
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

    fmt().with_env_filter(filter).init();

    // Get DID
    let did = cli
        .did
        .unwrap_or_else(|| "did:key:default".to_string());

    // Create registry config
    let mut config = RegistryConfig::default();
    config.owner_did = did.clone();
    config.data_dir = cli.data_dir;

    // Execute command
    if let Err(e) = execute_command(cli.command, config).await {
        error!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn execute_command(command: Commands, config: RegistryConfig) -> anyhow::Result<()> {
    match command {
        Commands::Publish {
            id,
            name,
            description,
            license,
            version,
            wasm,
            changelog,
            tags,
        } => {
            info!("Publishing {}@{}", id, version);

            let registry = Registry::with_config(config).await?;

            let mut module = GenModule::new(&id, &name, &description, &config.owner_did, &license);

            if let Some(tags_str) = tags {
                for tag in tags_str.split(',') {
                    module.add_tag(tag.trim());
                }
            }

            registry.publish(module, &version, &wasm, &changelog).await?;

            println!("✓ Published {}@{}", id, version);
        }

        Commands::Search { query, limit } => {
            let registry = Registry::with_config(config).await?;

            let search_query = SearchQuery::new(query).with_limit(limit);
            let results = registry.search(&search_query.text).await?;

            println!("Found {} modules:\n", results.len());
            for result in results {
                println!("  {} v{}", result.name, result.version);
                println!("    {}", result.description);
                println!("    Score: {:.2}\n", result.score);
            }
        }

        Commands::Install {
            module_id,
            version,
            auto_update,
        } => {
            let registry = Registry::with_config(config).await?;

            if auto_update {
                registry
                    .install_with_auto_update(&module_id, version.as_deref().unwrap_or("*"))
                    .await?;
                println!("✓ Installed {} with auto-update", module_id);
            } else {
                registry.install(&module_id, version.as_deref()).await?;
                println!("✓ Installed {}", module_id);
            }
        }

        Commands::List => {
            let registry = Registry::with_config(config).await?;
            let installed = registry.list_installed();

            println!("Installed modules ({}):\n", installed.len());
            for module in installed {
                println!("  {} v{}", module.module_id, module.version);
                if module.auto_update {
                    println!("    Auto-update: enabled");
                }
            }
        }

        Commands::Info { module_id } => {
            let registry = Registry::with_config(config).await?;
            let module = registry.get_module(&module_id).await?;

            println!("{}", module.name);
            println!("  ID: {}", module.id);
            println!("  Description: {}", module.description);
            println!("  Author: {}", module.author_did);
            println!("  License: {}", module.license);
            println!("  Latest: v{}", module.latest_version);
            println!("  Downloads: {}", module.download_count);

            if !module.tags.is_empty() {
                println!("  Tags: {}", module.tags.iter().cloned().collect::<Vec<_>>().join(", "));
            }

            if let Some(avg) = registry.get_average_rating(&module_id) {
                println!("  Rating: {:.1}/5.0", avg);
            }
        }

        Commands::Rate {
            module_id,
            stars,
            review,
        } => {
            let registry = Registry::with_config(config).await?;
            registry
                .rate(&module_id, stars, review.as_deref())
                .await?;
            println!("✓ Rated {} with {} stars", module_id, stars);
        }

        Commands::Daemon { port } => {
            info!("Starting P2P sync daemon on port {}", port);
            let registry = Registry::with_config(config).await?;
            registry.start_sync().await?;

            println!("Registry daemon running on http://localhost:{}", port);
            println!("Press Ctrl+C to stop");

            // Keep running
            tokio::signal::ctrl_c().await?;
            println!("\nShutting down...");
        }

        Commands::Sync { module_id } => {
            let registry = Registry::with_config(config).await?;

            if let Some(id) = module_id {
                println!("Syncing {}...", id);
                registry.sync_module(&id).await?;
                println!("✓ Synced {}", id);
            } else {
                println!("Starting full sync...");
                registry.start_sync().await?;
                println!("✓ Sync complete");
            }
        }

        Commands::Status => {
            let registry = Registry::with_config(config).await?;
            let peers = registry.discover_peers().await?;

            println!("Registry Status:");
            println!("  Peers: {}", peers.len());
            println!("  Modules: {}", registry.list_installed().len());
        }
    }

    Ok(())
}
