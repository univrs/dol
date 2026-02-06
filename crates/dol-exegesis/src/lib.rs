//! DOL Exegesis Preservation for Local-First Mode
//!
//! This crate provides CRDT-backed exegesis (documentation) storage and synchronization
//! for DOL declarations in local-first mode. Key features include:
//!
//! - **Peritext CRDT**: Rich text collaborative editing for exegesis content
//! - **Version Linking**: Exegesis versions tied to Gene evolution versions
//! - **Offline Authoring**: Full support for offline editing with merge on sync
//! - **Concurrent Editing**: Multiple developers can edit exegesis simultaneously
//!
//! # Architecture
//!
//! - `ExegesisDocument`: CRDT-backed document model with immutable gene_id/version,
//!   Peritext content, LWW last_modified, and RGA contributors list
//! - `ExegesisManager`: High-level API for creating, editing, and versioning exegesis
//! - `CollaborativeEditor`: Real-time collaborative editing with change subscriptions
//!
//! # Example
//!
//! ```no_run
//! use dol_exegesis::{ExegesisManager, ExegesisDocument};
//! use vudo_state::StateEngine;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize state engine
//!     let state_engine = StateEngine::new().await?;
//!     let manager = ExegesisManager::new(state_engine).await?;
//!
//!     // Create exegesis for a Gene
//!     let doc = manager.create_exegesis(
//!         "user.profile",
//!         "1.0.0",
//!         "A user profile contains identity and preferences."
//!     ).await?;
//!
//!     // Edit concurrently (offline-safe)
//!     manager.edit_exegesis(
//!         "user.profile",
//!         "1.0.0",
//!         "did:peer:alice",
//!         |content| {
//!             content.push_str("\nUpdated by Alice.");
//!         }
//!     ).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod collaborative;
pub mod error;
pub mod manager;
pub mod model;

pub use collaborative::{CollaborativeEditor, Subscription};
pub use error::{ExegesisError, Result};
pub use manager::ExegesisManager;
pub use model::ExegesisDocument;

/// Re-export common types for convenience
pub use chrono::{DateTime, Utc};
pub use vudo_state::StateEngine;
