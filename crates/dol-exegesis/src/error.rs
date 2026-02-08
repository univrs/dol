//! Error types for DOL exegesis operations.

use thiserror::Error;

/// Result type for exegesis operations.
pub type Result<T> = std::result::Result<T, ExegesisError>;

/// Error types for exegesis operations.
#[derive(Debug, Error)]
pub enum ExegesisError {
    /// Exegesis document not found.
    #[error("Exegesis not found for gene {0} version {1}")]
    NotFound(String, String),

    /// Invalid gene version format.
    #[error("Invalid version format: {0}")]
    InvalidVersion(String),

    /// Invalid DID format.
    #[error("Invalid DID format: {0}")]
    InvalidDid(String),

    /// CRDT merge conflict.
    #[error("CRDT merge conflict: {0}")]
    MergeConflict(String),

    /// State engine error.
    #[error("State engine error: {0}")]
    StateEngine(#[from] vudo_state::StateError),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Automerge error.
    #[error("Automerge error: {0}")]
    Automerge(#[from] automerge::AutomergeError),

    /// P2P sync error.
    #[error("P2P sync error: {0}")]
    P2PSync(String),

    /// Internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}
