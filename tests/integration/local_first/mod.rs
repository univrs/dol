//! Local-first integration tests for VUDO Runtime.
//!
//! This test suite validates the complete local-first stack including:
//! - State engine (vudo-state)
//! - Storage adapters (vudo-storage)
//! - P2P networking (vudo-p2p)
//! - Offline-first workflows
//! - Network partitions and healing
//! - CRDT convergence
//!
//! # Test Categories
//!
//! - **airplane_mode**: Offline → online → sync workflows
//! - **network_partition**: Network splits and convergence
//! - **concurrent_edits**: Multi-node concurrent operations
//! - **large_documents**: Performance with large Automerge documents
//! - **schema_evolution**: Mixed schema version compatibility
//!
//! # Running Tests
//!
//! ```bash
//! # Run all integration tests
//! cargo test --test integration
//!
//! # Run specific category
//! cargo test --test integration airplane_mode
//! cargo test --test integration network_partition
//!
//! # Run with output
//! cargo test --test integration -- --nocapture
//! ```

pub mod test_harness;

pub mod airplane_mode;
pub mod concurrent_edits;
pub mod large_documents;
pub mod network_partition;
pub mod schema_evolution;
