//! Error types for VUDO Credit system

use thiserror::Error;

/// Result type for credit operations
pub type Result<T> = std::result::Result<T, CreditError>;

/// Errors that can occur in the credit system
#[derive(Error, Debug, Clone)]
pub enum CreditError {
    /// No escrow allocated for this device
    #[error("No escrow allocated for account {account_id}, device {device_id}")]
    NoEscrowAllocated {
        account_id: String,
        device_id: String,
    },

    /// Insufficient escrow for operation
    #[error("Insufficient escrow: available {available}, requested {requested}")]
    InsufficientEscrow { available: i64, requested: i64 },

    /// BFT consensus failure
    #[error("BFT consensus failed: {votes_received}/{quorum_required} votes")]
    BftConsensusFailure {
        votes_received: usize,
        quorum_required: usize,
    },

    /// BFT escrow grant failed
    #[error("BFT escrow grant failed to reach consensus")]
    BftEscrowGrantFailed,

    /// Insufficient balance for escrow allocation
    #[error("Insufficient balance for escrow allocation")]
    InsufficientBalanceForEscrow,

    /// Account not found
    #[error("Account not found: {0}")]
    AccountNotFound(String),

    /// Transaction not found
    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),

    /// Invalid reputation tier
    #[error("Invalid reputation tier: {0} (must be 0-5)")]
    InvalidReputationTier(u8),

    /// Escrow expired
    #[error("Escrow expired at {expired_at}")]
    EscrowExpired { expired_at: u64 },

    /// Invalid transaction status transition
    #[error("Invalid transaction status transition: {from} -> {to}")]
    InvalidStatusTransition { from: String, to: String },

    /// State engine error
    #[error("State engine error: {0}")]
    StateEngine(String),

    /// Identity error
    #[error("Identity error: {0}")]
    Identity(String),

    /// P2P error
    #[error("P2P error: {0}")]
    P2p(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<vudo_state::StateError> for CreditError {
    fn from(err: vudo_state::StateError) -> Self {
        CreditError::StateEngine(err.to_string())
    }
}

impl From<vudo_identity::Error> for CreditError {
    fn from(err: vudo_identity::Error) -> Self {
        CreditError::Identity(err.to_string())
    }
}

impl From<serde_json::Error> for CreditError {
    fn from(err: serde_json::Error) -> Self {
        CreditError::Serialization(err.to_string())
    }
}

impl From<automerge::AutomergeError> for CreditError {
    fn from(err: automerge::AutomergeError) -> Self {
        CreditError::Internal(err.to_string())
    }
}
