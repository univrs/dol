//! VUDO PlanetServe - Privacy-Preserving Sync Infrastructure
//!
//! This crate provides privacy-preserving synchronization for VUDO Runtime using:
//! - **S-IDA (Secure Information Dispersal Algorithm)**: Fragment messages across peers
//! - **Onion Routing**: Hide WHO is syncing with WHOM
//! - **Metadata Obfuscation**: Hide WHEN syncing occurs and WHAT is being synced
//! - **BFT Verification**: Private Byzantine fault-tolerant voting
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                   PlanetServe Adapter                       │
//! ├─────────────────┬──────────────┬───────────────────────────┤
//! │   S-IDA         │    Onion     │      Metadata             │
//! │  Fragmenter     │   Router     │     Obfuscator            │
//! │                 │              │                           │
//! │  k-of-n         │  Multi-hop   │  Padding + Timing Jitter  │
//! │  Erasure Coding │  Encryption  │  + Cover Traffic          │
//! └─────────────────┴──────────────┴───────────────────────────┘
//!           │                │                   │
//!           └────────────────┴───────────────────┘
//!                            │
//!                      Iroh P2P Network
//! ```
//!
//! # Privacy Guarantees
//!
//! ## S-IDA Fragmentation
//! - Message split into n fragments
//! - Any k fragments can reconstruct
//! - Having < k fragments reveals NO information
//! - No single peer observes full message
//!
//! ## Onion Routing
//! - Entry relay knows sender, not receiver
//! - Exit relay knows receiver, not sender
//! - Middle relays know neither
//! - No single relay can correlate sender-receiver
//!
//! ## Metadata Obfuscation
//! - Message padding hides content size
//! - Timing jitter hides sync timing
//! - Cover traffic hides sync frequency
//!
//! # Performance
//!
//! ## Privacy vs. Speed Trade-offs
//!
//! | Level    | S-IDA | Onion | Padding | Jitter | Cover | Latency Overhead |
//! |----------|-------|-------|---------|--------|-------|------------------|
//! | None     | No    | No    | No      | No     | No    | 0ms              |
//! | Basic    | No    | No    | Yes     | No     | No    | <5ms             |
//! | Standard | No    | No    | Yes     | Yes    | No    | ~100ms           |
//! | Maximum  | Yes   | Yes   | Yes     | Yes    | Yes   | ~500ms           |
//!
//! # Examples
//!
//! ## Fast-Open Mode (No Privacy)
//!
//! ```no_run
//! use vudo_planetserve::{PlanetServeAdapter, config::PrivacyConfig};
//! use vudo_identity::MasterIdentity;
//! use vudo_p2p::VudoP2P;
//! use std::sync::Arc;
//!
//! # async fn example() -> vudo_planetserve::error::Result<()> {
//! let identity = Arc::new(MasterIdentity::generate("Alice").await?);
//! let p2p = Arc::new(VudoP2P::new(
//!     Arc::new(vudo_state::StateEngine::new().await?),
//!     vudo_p2p::P2PConfig::default()
//! ).await?);
//!
//! let adapter = PlanetServeAdapter::new(
//!     identity,
//!     p2p,
//!     PrivacyConfig::fast_open(), // No privacy, maximum speed
//! ).await?;
//!
//! // Sync document (direct, no privacy overhead)
//! adapter.sync_private("namespace", "doc_id", vec![1, 2, 3]).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Privacy-Max Mode (Full Privacy)
//!
//! ```no_run
//! use vudo_planetserve::{PlanetServeAdapter, config::PrivacyConfig};
//! use vudo_identity::MasterIdentity;
//! use vudo_p2p::VudoP2P;
//! use std::sync::Arc;
//!
//! # async fn example() -> vudo_planetserve::error::Result<()> {
//! let identity = Arc::new(MasterIdentity::generate("Alice").await?);
//! let p2p = Arc::new(VudoP2P::new(
//!     Arc::new(vudo_state::StateEngine::new().await?),
//!     vudo_p2p::P2PConfig::default()
//! ).await?);
//!
//! let adapter = PlanetServeAdapter::new(
//!     identity,
//!     p2p,
//!     PrivacyConfig::privacy_max(), // Maximum privacy
//! ).await?;
//!
//! // Start cover traffic
//! adapter.start().await?;
//!
//! // Sync document (fragmented + onion-routed + cover traffic)
//! adapter.sync_private("namespace", "doc_id", vec![1, 2, 3]).await?;
//!
//! // Stop cover traffic
//! adapter.stop().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## BFT Private Voting
//!
//! ```no_run
//! use vudo_planetserve::{PlanetServeAdapter, bft::{BftPrivateCommittee, Proposal}};
//! use vudo_planetserve::config::PrivacyConfig;
//! use std::sync::Arc;
//!
//! # async fn example() -> vudo_planetserve::error::Result<()> {
//! # let identity = Arc::new(vudo_identity::MasterIdentity::generate("Test").await?);
//! # let p2p = Arc::new(vudo_p2p::VudoP2P::new(
//! #     Arc::new(vudo_state::StateEngine::new().await?),
//! #     vudo_p2p::P2PConfig::default()
//! # ).await?);
//! let adapter = Arc::new(PlanetServeAdapter::new(
//!     identity,
//!     p2p,
//!     PrivacyConfig::privacy_max(),
//! ).await?);
//!
//! // Create BFT committee
//! let committee = BftPrivateCommittee::new(
//!     vec![
//!         "did:peer:member1".to_string(),
//!         "did:peer:member2".to_string(),
//!         "did:peer:member3".to_string(),
//!         "did:peer:member4".to_string(),
//!         "did:peer:member5".to_string(),
//!     ],
//!     adapter,
//! );
//!
//! // Create proposal
//! let proposal = Proposal::new(
//!     "credit_reconciliation",
//!     serde_json::json!({"from": "alice", "to": "bob", "amount": 100}),
//! );
//!
//! // Conduct private vote
//! let result = committee.private_vote(&proposal).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Security Considerations
//!
//! ## Threat Model
//!
//! PlanetServe provides defense against:
//! - **Passive observers**: Cannot see message contents or correlation
//! - **Compromised relays**: Single relay compromise reveals nothing
//! - **Traffic analysis**: Padding, jitter, and cover traffic resist timing attacks
//! - **Partial collusion**: Need k-of-n fragments to reconstruct
//!
//! PlanetServe does NOT protect against:
//! - **Global passive adversary**: Can correlate all network traffic
//! - **k-of-n relay collusion**: Can reconstruct if controlling k+ relays
//! - **Endpoint compromise**: Plaintext visible at sender/receiver
//!
//! ## Best Practices
//!
//! - Use **Maximum** privacy for sensitive operations
//! - Use **Standard** privacy for normal operations
//! - Use **Basic** or **None** for public data
//! - Rotate relay pools regularly
//! - Monitor relay reliability and latency
//! - Set appropriate k and n values for S-IDA
//!
//! # Performance Tuning
//!
//! ## S-IDA Parameters
//!
//! - `k=2, n=3`: Fast, basic redundancy (1 failure tolerated)
//! - `k=3, n=5`: Balanced (2 failures tolerated, DEFAULT)
//! - `k=5, n=7`: High redundancy (2 failures tolerated, slower)
//!
//! ## Onion Routing
//!
//! - `hops=1`: No anonymity, fast
//! - `hops=2`: Basic anonymity (DEFAULT)
//! - `hops=3`: Strong anonymity (Tor-level)
//!
//! ## Padding
//!
//! - `1024 bytes`: Fast, moderate privacy
//! - `4096 bytes`: Balanced (DEFAULT)
//! - `16384 bytes`: Strong privacy, slower
//!
//! # References
//!
//! - [S-IDA Paper](https://dl.acm.org/doi/10.1145/1315245.1315318)
//! - [Tor Onion Routing](https://www.torproject.org/)
//! - [Vuvuzela Metadata Privacy](https://vuvuzela.io/)
//! - [PlanetLab Research](https://www.planet-lab.org/)

pub mod adapter;
pub mod bft;
pub mod config;
pub mod error;
pub mod obfuscator;
pub mod onion;
pub mod sida;

// Re-export main types
pub use adapter::PlanetServeAdapter;
pub use bft::{BftPrivateCommittee, Proposal, Vote, VoteResult};
pub use config::{
    OnionConfig, PrivacyConfig, PrivacyLevel, RelaySelectionStrategy, SidaConfig,
};
pub use error::{Error, Result};
pub use obfuscator::{CoverTrafficHandle, MetadataObfuscator};
pub use onion::{OnionCircuit, OnionRouter, RelayNode};
pub use sida::{Fragment, SidaFragmenter};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
