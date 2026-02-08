//! Error types for vudo-identity

use thiserror::Error;

/// Result type alias for vudo-identity operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for identity operations
#[derive(Debug, Error)]
pub enum Error {
    /// DID parsing or creation error
    #[error("DID error: {0}")]
    Did(String),

    /// UCAN creation or verification error
    #[error("UCAN error: {0}")]
    Ucan(String),

    /// UCAN has expired
    #[error("UCAN has expired")]
    UcanExpired,

    /// UCAN is not yet valid
    #[error("UCAN is not yet valid")]
    UcanNotYetValid,

    /// Insufficient delegation in UCAN chain
    #[error("Insufficient delegation: {0}")]
    InsufficientDelegation(String),

    /// Signature verification failed
    #[error("Signature verification failed: {0}")]
    SignatureVerification(String),

    /// Key error
    #[error("Key error: {0}")]
    Key(String),

    /// Encoding/decoding error
    #[error("Encoding error: {0}")]
    Encoding(String),

    /// Identity not found
    #[error("Identity not found: {0}")]
    IdentityNotFound(String),

    /// Device not found
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    /// Device already linked
    #[error("Device already linked: {0}")]
    DeviceAlreadyLinked(String),

    /// Device revoked
    #[error("Device has been revoked: {0}")]
    DeviceRevoked(String),

    /// Key rotation error
    #[error("Key rotation error: {0}")]
    KeyRotation(String),

    /// Revocation error
    #[error("Revocation error: {0}")]
    Revocation(String),

    /// Resolution error
    #[error("DID resolution error: {0}")]
    Resolution(String),

    /// JWT error
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid capability
    #[error("Invalid capability: {0}")]
    InvalidCapability(String),

    /// Invalid multibase encoding
    #[error("Invalid multibase encoding: {0}")]
    InvalidMultibase(String),

    /// Invalid multicodec
    #[error("Invalid multicodec: {0}")]
    InvalidMulticodec(String),
}

impl From<ed25519_dalek::SignatureError> for Error {
    fn from(e: ed25519_dalek::SignatureError) -> Self {
        Error::SignatureVerification(e.to_string())
    }
}
