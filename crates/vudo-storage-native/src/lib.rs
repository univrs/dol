//! Native SQLite storage adapter for VUDO Runtime.
//!
//! This crate provides high-performance persistent storage for Desktop, Mobile,
//! and Server platforms using native SQLite with WAL mode.
//!
//! # Features
//!
//! - Native SQLite with Write-Ahead Logging (WAL) for concurrency
//! - Connection pooling (multiple readers, single writer)
//! - Optimized bulk inserts
//! - 100K+ writes/sec performance target
//!
//! # Example
//!
//! ```no_run
//! use vudo_storage_native::SqliteAdapter;
//! use vudo_storage::StorageAdapter;
//! use bytes::Bytes;
//!
//! #[tokio::main]
//! async fn main() -> vudo_storage::Result<()> {
//!     let storage = SqliteAdapter::new("./vudo.db").await?;
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

pub mod sqlite_adapter;

pub use sqlite_adapter::SqliteAdapter;
