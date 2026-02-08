//! Transaction types for mutual credit system

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Transaction ID (UUID v4)
pub type TransactionId = String;

/// Transaction in the mutual credit system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Transaction {
    /// Unique transaction ID
    pub id: TransactionId,

    /// Sender DID
    pub from: String,

    /// Recipient DID
    pub to: String,

    /// Amount in cents
    pub amount: i64,

    /// Transaction timestamp (Unix epoch seconds)
    pub timestamp: u64,

    /// Transaction status
    pub status: TransactionStatus,

    /// Transaction metadata
    pub metadata: TransactionMetadata,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(
        from: String,
        to: String,
        amount: i64,
        metadata: TransactionMetadata,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from,
            to,
            amount,
            timestamp: Utc::now().timestamp() as u64,
            status: TransactionStatus::Pending,
            metadata,
        }
    }

    /// Check if transaction is from a given account
    pub fn is_from(&self, account_id: &str) -> bool {
        self.from == account_id
    }

    /// Check if transaction is to a given account
    pub fn is_to(&self, account_id: &str) -> bool {
        self.to == account_id
    }

    /// Check if transaction is pending
    pub fn is_pending(&self) -> bool {
        matches!(self.status, TransactionStatus::Pending)
    }

    /// Check if transaction is confirmed
    pub fn is_confirmed(&self) -> bool {
        matches!(self.status, TransactionStatus::Confirmed)
    }

    /// Check if transaction is reversed
    pub fn is_reversed(&self) -> bool {
        matches!(self.status, TransactionStatus::Reversed)
    }

    /// Check if transaction is disputed
    pub fn is_disputed(&self) -> bool {
        matches!(self.status, TransactionStatus::Disputed)
    }
}

/// Transaction status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionStatus {
    /// Pending (queued, not yet reconciled)
    Pending,

    /// Confirmed (BFT-confirmed)
    Confirmed,

    /// Reversed (reversed due to overdraft)
    Reversed,

    /// Disputed (under review)
    Disputed,
}

impl TransactionStatus {
    /// Get status as string
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionStatus::Pending => "pending",
            TransactionStatus::Confirmed => "confirmed",
            TransactionStatus::Reversed => "reversed",
            TransactionStatus::Disputed => "disputed",
        }
    }

    /// Check if transition to new status is valid
    pub fn can_transition_to(&self, new_status: TransactionStatus) -> bool {
        match (self, new_status) {
            // Pending can transition to any other status
            (TransactionStatus::Pending, _) => true,
            // Confirmed can be reversed or disputed
            (TransactionStatus::Confirmed, TransactionStatus::Reversed) => true,
            (TransactionStatus::Confirmed, TransactionStatus::Disputed) => true,
            // Reversed cannot transition
            (TransactionStatus::Reversed, _) => false,
            // Disputed can be confirmed or reversed
            (TransactionStatus::Disputed, TransactionStatus::Confirmed) => true,
            (TransactionStatus::Disputed, TransactionStatus::Reversed) => true,
            // Cannot transition to self (no-op)
            _ => false,
        }
    }
}

/// Transaction metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransactionMetadata {
    /// Human-readable description
    pub description: String,

    /// Optional category (e.g., "food", "transport", "entertainment")
    pub category: Option<String>,

    /// Optional invoice ID for linking
    pub invoice_id: Option<String>,
}

impl Default for TransactionMetadata {
    fn default() -> Self {
        Self {
            description: "".to_string(),
            category: None,
            invoice_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_new() {
        let tx = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            1000,
            TransactionMetadata {
                description: "Coffee".to_string(),
                category: Some("food".to_string()),
                invoice_id: None,
            },
        );

        assert!(tx.is_from("alice"));
        assert!(tx.is_to("bob"));
        assert_eq!(tx.amount, 1000);
        assert!(tx.is_pending());
    }

    #[test]
    fn test_transaction_status_transitions() {
        // Pending can transition to any other status
        assert!(TransactionStatus::Pending.can_transition_to(TransactionStatus::Confirmed));
        assert!(TransactionStatus::Pending.can_transition_to(TransactionStatus::Reversed));
        assert!(TransactionStatus::Pending.can_transition_to(TransactionStatus::Disputed));

        // Confirmed can be reversed or disputed
        assert!(TransactionStatus::Confirmed.can_transition_to(TransactionStatus::Reversed));
        assert!(TransactionStatus::Confirmed.can_transition_to(TransactionStatus::Disputed));
        assert!(!TransactionStatus::Confirmed.can_transition_to(TransactionStatus::Pending));

        // Reversed cannot transition
        assert!(!TransactionStatus::Reversed.can_transition_to(TransactionStatus::Confirmed));
        assert!(!TransactionStatus::Reversed.can_transition_to(TransactionStatus::Pending));
        assert!(!TransactionStatus::Reversed.can_transition_to(TransactionStatus::Disputed));

        // Disputed can be confirmed or reversed
        assert!(TransactionStatus::Disputed.can_transition_to(TransactionStatus::Confirmed));
        assert!(TransactionStatus::Disputed.can_transition_to(TransactionStatus::Reversed));
        assert!(!TransactionStatus::Disputed.can_transition_to(TransactionStatus::Pending));
    }

    #[test]
    fn test_transaction_status_as_str() {
        assert_eq!(TransactionStatus::Pending.as_str(), "pending");
        assert_eq!(TransactionStatus::Confirmed.as_str(), "confirmed");
        assert_eq!(TransactionStatus::Reversed.as_str(), "reversed");
        assert_eq!(TransactionStatus::Disputed.as_str(), "disputed");
    }
}
