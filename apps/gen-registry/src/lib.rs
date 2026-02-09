//! Gen Registry - P2P Community Registry for DOL Gen Modules
//!
//! A local-first, CRDT-based registry for discovering, publishing, and
//! sharing DOL Gen module definitions with the community.
//!
//! # Architecture
//!
//! ```text
//! ┌────────────────────────────────────────────────────────┐
//! │                   Gen Registry                         │
//! ├────────────────────────────────────────────────────────┤
//! │  Search Layer   │  Tantivy full-text search            │
//! ├────────────────────────────────────────────────────────┤
//! │  Registry Layer │  Module metadata + CRDT state        │
//! ├────────────────────────────────────────────────────────┤
//! │  CRDT Layer     │  Automerge (OR-Set, RGA, PN-Counter) │
//! ├────────────────────────────────────────────────────────┤
//! │  Storage Layer  │  VUDO Storage (IndexedDB/SQLite)     │
//! ├────────────────────────────────────────────────────────┤
//! │  P2P Layer      │  Iroh + Willow Protocol              │
//! └────────────────────────────────────────────────────────┘
//! ```
//!
//! # Features
//!
//! - **Publish**: Upload Gen definition + compiled WASM module
//! - **Discover**: Full-text search by name, tags, capabilities
//! - **Version**: Semantic versioning with changelog
//! - **Dependencies**: DAG-based dependency resolution
//! - **Ratings**: Community feedback (CRDT-backed)
//! - **Sync**: P2P distribution via Iroh
//! - **Offline**: Full offline browsing of cached modules
//!
//! # Examples
//!
//! ## Publishing a Module
//!
//! ```no_run
//! use gen_registry::{Registry, GenModule, ModuleVersion};
//! use std::path::Path;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let registry = Registry::new("did:key:alice").await?;
//!
//! // Create module metadata
//! let module = GenModule::new(
//!     "io.univrs.user",
//!     "User Management Gen",
//!     "Authentication and user profile management",
//!     "did:key:alice",
//!     "MIT",
//! );
//!
//! // Publish version
//! let wasm_path = Path::new("target/wasm32-unknown-unknown/release/user.wasm");
//! registry.publish(module, "1.0.0", wasm_path, "Initial release").await?;
//!
//! println!("Published io.univrs.user v1.0.0");
//! # Ok(())
//! # }
//! ```
//!
//! ## Searching for Modules
//!
//! ```no_run
//! use gen_registry::Registry;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let registry = Registry::new("did:key:alice").await?;
//!
//! // Search by keyword
//! let results = registry.search("authentication").await?;
//! for module in results {
//!     println!("{} v{} - {}", module.name, module.latest_version, module.description);
//! }
//!
//! // Search by tag
//! let auth_modules = registry.search_by_tags(&["authentication", "security"]).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Installing a Module
//!
//! ```no_run
//! use gen_registry::Registry;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let registry = Registry::new("did:key:alice").await?;
//!
//! // Install latest version
//! registry.install("io.univrs.user", None).await?;
//!
//! // Install specific version
//! registry.install("io.univrs.database", Some("2.1.0")).await?;
//!
//! // Install with auto-update
//! registry.install_with_auto_update("io.univrs.http", "^3.0.0").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## P2P Synchronization
//!
//! ```no_run
//! use gen_registry::Registry;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let registry = Registry::new("did:key:alice").await?;
//!
//! // Start P2P sync
//! registry.start_sync().await?;
//!
//! // Discover peers
//! let peers = registry.discover_peers().await?;
//! println!("Connected to {} peers", peers.len());
//!
//! // Sync specific module
//! registry.sync_module("io.univrs.user").await?;
//! # Ok(())
//! # }
//! ```

mod error;
mod models;
mod registry;
mod search;
mod sync;
mod version;
mod wasm;

pub use error::{Error, Result};
pub use models::{
    Capability, Dependency, GenModule, InstalledModule, ModuleVersion, PublishCapability, Rating,
    SearchIndex, SyncState,
};
pub use registry::{Registry, RegistryConfig};
pub use search::{SearchQuery, SearchResult};
pub use sync::{P2PSync, SyncProgress};
pub use version::{VersionResolver, VersionRequirement};
pub use wasm::{WasmModule, WasmValidator};

/// Re-export VUDO types
pub use vudo_identity::DID;
pub use vudo_p2p::{Capability as WillowCapability, WillowAdapter};
pub use vudo_state::StateEngine;
