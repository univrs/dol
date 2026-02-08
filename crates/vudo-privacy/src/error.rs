//! Error types for VUDO Privacy.

use thiserror::Error;

/// Result type for privacy operations.
pub type Result<T> = std::result::Result<T, PrivacyError>;

/// Error types for privacy operations.
#[derive(Debug, Error)]
pub enum PrivacyError {
    /// Data encryption key not found.
    #[error("Data encryption key not found for owner: {0}")]
    DekNotFound(String),

    /// Data encryption key has been deleted (GDPR erasure).
    #[error("Data encryption key has been deleted - data permanently erased")]
    KeyDeleted,

    /// Encrypted data cannot be decrypted because key was deleted.
    #[error("Data permanently erased - decryption key was deleted")]
    DataPermanentlyErased,

    /// Encryption operation failed.
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    /// Decryption operation failed.
    #[error("Decryption failed - invalid ciphertext or corrupted data")]
    DecryptionFailed,

    /// Invalid DID format.
    #[error("Invalid DID format: {0}")]
    InvalidDid(String),

    /// Invalid actor ID.
    #[error("Invalid actor ID: {0}")]
    InvalidActorId(String),

    /// Audit log operation failed.
    #[error("Audit log error: {0}")]
    AuditLogError(String),

    /// GDPR deletion request failed.
    #[error("GDPR deletion request failed: {0}")]
    GdprDeletionFailed(String),

    /// Willow adapter error.
    #[error("Willow adapter error: {0}")]
    WillowError(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// UTF-8 conversion error.
    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    /// JSON error.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// I/O error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Generic error.
    #[error("{0}")]
    Other(String),
}

impl From<String> for PrivacyError {
    fn from(s: String) -> Self {
        PrivacyError::Other(s)
    }
}

impl From<&str> for PrivacyError {
    fn from(s: &str) -> Self {
        PrivacyError::Other(s.to_string())
    }
}
