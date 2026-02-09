//! Browser storage adapter for VUDO Runtime.
//!
//! This crate provides persistent storage for browser environments with support for:
//! - In-memory storage (current implementation)
//! - IndexedDB storage (planned)
//! - OPFS + SQLite WASM (planned for production)
//! - Multi-tab coordination (planned)
//!
//! # Architecture
//!
//! The browser adapter is designed to support multiple storage backends:
//!
//! 1. **In-Memory** (current): Fast, for testing and development
//! 2. **IndexedDB**: Browser-native key-value storage with good support
//! 3. **OPFS + SQLite WASM**: Target production backend for 10K+ writes/sec
//!
//! # Multi-Tab Coordination
//!
//! When multiple tabs access the same storage, coordination is needed to prevent
//! conflicts. This will be implemented using:
//! - BroadcastChannel API for change notifications
//! - Shared Worker for write coordination (where supported)
//! - IndexedDB locks as fallback
//!
//! # Example
//!
//! ```
//! use vudo_storage_browser::MemoryAdapter;
//! use vudo_storage::StorageAdapter;
//! use bytes::Bytes;
//!
//! #[tokio::main]
//! async fn main() -> vudo_storage::Result<()> {
//!     let storage = MemoryAdapter::new();
//!     storage.init().await?;
//!
//!     // Save a document
//!     let data = Bytes::from("document content");
//!     storage.save("users", "alice", data.clone()).await?;
//!
//!     // Load it back
//!     let loaded = storage.load("users", "alice").await?;
//!     assert_eq!(loaded, Some(data));
//!
//!     Ok(())
//! }
//! ```

pub mod memory_adapter;

pub use memory_adapter::MemoryAdapter;
