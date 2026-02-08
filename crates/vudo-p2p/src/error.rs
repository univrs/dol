//! Error types for the P2P layer.

use thiserror::Error;

/// Result type for P2P operations.
pub type Result<T> = std::result::Result<T, P2PError>;

/// P2P error types.
#[derive(Debug, Error)]
pub enum P2PError {
    /// Iroh network error.
    #[error("Iroh network error: {0}")]
    IrohError(#[from] anyhow::Error),

    /// State engine error.
    #[error("State engine error: {0}")]
    StateError(#[from] vudo_state::StateError),

    /// Peer not found.
    #[error("Peer not found: {0}")]
    PeerNotFound(String),

    /// Connection failed.
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// Sync protocol error.
    #[error("Sync protocol error: {0}")]
    SyncProtocolError(String),

    /// Document not found.
    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error.
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// Gossip error.
    #[error("Gossip error: {0}")]
    GossipError(String),

    /// Bandwidth limit exceeded.
    #[error("Bandwidth limit exceeded")]
    BandwidthLimitExceeded,

    /// Invalid message.
    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    /// Timeout.
    #[error("Operation timed out")]
    Timeout,

    /// Internal error.
    #[error("Internal error: {0}")]
    Internal(String),

    /// Willow protocol error.
    #[error("Willow protocol error: {0}")]
    WillowError(String),

    /// Permission denied (capability check failed).
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Invalid namespace.
    #[error("Invalid namespace: {0}")]
    InvalidNamespace(String),

    /// Invalid path.
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// Entry not found.
    #[error("Entry not found")]
    EntryNotFound,

    /// Resource limit exceeded.
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    /// Capability delegation error.
    #[error("Capability delegation error: {0}")]
    CapabilityDelegationError(String),
}

impl From<serde_json::Error> for P2PError {
    fn from(err: serde_json::Error) -> Self {
        P2PError::SerializationError(err.to_string())
    }
}

impl From<bincode::Error> for P2PError {
    fn from(err: bincode::Error) -> Self {
        P2PError::SerializationError(err.to_string())
    }
}
