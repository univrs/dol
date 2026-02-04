//! Error types for the DOL ABI

use serde::{Deserialize, Serialize};
use std::fmt;

/// Result type for DOL ABI operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for DOL ABI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Error {
    /// Invalid configuration
    InvalidConfig(String),
    /// Invalid message format
    InvalidMessage(String),
    /// Host operation failed
    HostError(String),
    /// Type mismatch
    TypeMismatch(String),
    /// Generic error
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
            Error::InvalidMessage(msg) => write!(f, "Invalid message: {}", msg),
            Error::HostError(msg) => write!(f, "Host error: {}", msg),
            Error::TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}
