//! Error types for PlanetServe privacy-preserving sync

use thiserror::Error;

/// Result type for PlanetServe operations
pub type Result<T> = std::result::Result<T, Error>;

/// PlanetServe errors
#[derive(Debug, Error)]
pub enum Error {
    /// S-IDA configuration error
    #[error("Invalid S-IDA configuration: {0}")]
    InvalidSidaConfig(String),

    /// Insufficient fragments for reconstruction
    #[error("Insufficient fragments: have {have}, need {need}")]
    InsufficientFragments { have: usize, need: usize },

    /// S-IDA fragmentation failed
    #[error("S-IDA fragmentation failed: {0}")]
    FragmentationFailed(String),

    /// S-IDA reconstruction failed
    #[error("S-IDA reconstruction failed: {0}")]
    ReconstructionFailed(String),

    /// Encryption error
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    /// Decryption error
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    /// Onion routing error
    #[error("Onion routing failed: {0}")]
    OnionRoutingFailed(String),

    /// Circuit build error
    #[error("Failed to build onion circuit: {0}")]
    CircuitBuildFailed(String),

    /// Relay selection error
    #[error("Failed to select relays: {0}")]
    RelaySelectionFailed(String),

    /// Key agreement error
    #[error("Key agreement failed: {0}")]
    KeyAgreementFailed(String),

    /// Invalid fragment
    #[error("Invalid fragment: {0}")]
    InvalidFragment(String),

    /// P2P error
    #[error("P2P error: {0}")]
    P2P(#[from] vudo_p2p::error::P2PError),

    /// Identity error
    #[error("Identity error: {0}")]
    Identity(#[from] vudo_identity::error::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Anyhow error wrapper
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
