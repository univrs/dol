//! Error types for Gen Registry

use std::io;
use thiserror::Error;

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// Registry errors
#[derive(Error, Debug)]
pub enum Error {
    #[error("Module not found: {0}")]
    ModuleNotFound(String),

    #[error("Version not found: {module}@{version}")]
    VersionNotFound { module: String, version: String },

    #[error("Invalid module ID: {0}")]
    InvalidModuleId(String),

    #[error("Invalid semver: {0}")]
    InvalidSemver(#[from] semver::Error),

    #[error("Version conflict: {0}")]
    VersionConflict(String),

    #[error("Dependency cycle detected: {0}")]
    DependencyCycle(String),

    #[error("Missing dependency: {0}")]
    MissingDependency(String),

    #[error("WASM validation failed: {0}")]
    WasmValidationFailed(String),

    #[error("Hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Module already published: {0}")]
    ModuleAlreadyPublished(String),

    #[error("Search index error: {0}")]
    SearchIndexError(String),

    #[error("Sync error: {0}")]
    SyncError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("VUDO state error: {0}")]
    VudoStateError(String),

    #[error("VUDO P2P error: {0}")]
    VudoP2PError(String),

    #[error("Automerge error: {0}")]
    AutomergeError(String),

    #[error("Other error: {0}")]
    Other(String),
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::SerializationError(e.to_string())
    }
}

impl From<bincode::Error> for Error {
    fn from(e: bincode::Error) -> Self {
        Error::SerializationError(e.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::NetworkError(e.to_string())
    }
}

impl From<automerge::AutomergeError> for Error {
    fn from(e: automerge::AutomergeError) -> Self {
        Error::AutomergeError(e.to_string())
    }
}
