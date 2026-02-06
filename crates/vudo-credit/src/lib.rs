//! VUDO Credit - Escrow-Based Mutual Credit System
//!
//! This crate provides a local-first mutual credit system with:
//! - **Local spend operations**: < 1ms, no network required
//! - **Escrow allocation**: Per-device pre-allocation from global balance
//! - **Double-spend prevention**: Via escrow limits (no coordination needed)
//! - **BFT reconciliation**: Periodic balance confirmation with 3f+1 nodes
//! - **Overdraft detection**: Via CRDT merge comparison
//! - **Reputation tiers**: Credit limits based on trust level (0-5)
//! - **Conflict resolution**: For concurrent overdrafts
//!
//! # Architecture: The Escrow Pattern
//!
//! From RFC-001 Section 5.4:
//!
//! ```text
//! confirmed_balance: BFT-confirmed (strong consistency)
//! local_escrow:      Pre-allocated per device (local operations)
//! pending_credits:   Eventually consistent (CRDT merge)
//!
//! Invariant: local_escrow + pending_debits ≤ confirmed_balance at all times
//! ```
//!
//! # Key Insight
//!
//! Local spend from escrow is **immediate** (offline-first) but **bounded** by pre-allocated limit.
//! When escrow runs low, device requests refresh from BFT committee.
//!
//! # Examples
//!
//! ## Simple Payment (Offline)
//!
//! ```rust
//! use vudo_credit::{MutualCreditScheduler, TransactionMetadata};
//! # use vudo_credit::error::Result;
//!
//! # async fn example() -> Result<()> {
//! # let scheduler = MutualCreditScheduler::new_mock().await?;
//! # let account_id = "alice";
//! # let recipient = "bob";
//! // Local spend (< 1ms, no network)
//! let tx_id = scheduler.spend_local(
//!     account_id,
//!     1000,  // $10.00 in cents
//!     recipient,
//!     TransactionMetadata {
//!         description: "Coffee".to_string(),
//!         category: Some("food".to_string()),
//!         invoice_id: None,
//!     },
//! ).await?;
//!
//! println!("Payment queued: {}", tx_id);
//! # Ok(())
//! # }
//! ```
//!
//! ## Escrow Refresh
//!
//! ```rust
//! use vudo_credit::MutualCreditScheduler;
//! # use vudo_credit::error::Result;
//!
//! # async fn example() -> Result<()> {
//! # let scheduler = MutualCreditScheduler::new_mock().await?;
//! # let account_id = "alice";
//! // Request escrow refresh (queued, processed when online)
//! scheduler.request_escrow_refresh(account_id).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## BFT Reconciliation
//!
//! ```rust
//! use vudo_credit::BftCommittee;
//! # use vudo_credit::error::Result;
//!
//! # async fn example() -> Result<()> {
//! # let committee = BftCommittee::new_mock(4).await?;
//! # let account_id = "alice";
//! // Reconcile account via BFT consensus
//! let result = committee.reconcile_balance(account_id).await?;
//!
//! if result.consensus {
//!     println!("New confirmed balance: {}", result.new_confirmed_balance);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Overdraft Detection
//!
//! ```rust
//! use vudo_credit::MutualCreditScheduler;
//! # use vudo_credit::error::Result;
//!
//! # async fn example() -> Result<()> {
//! # let scheduler = MutualCreditScheduler::new_mock().await?;
//! # let account_id = "alice";
//! // Detect overdrafts after CRDT merge
//! let overdrafts = scheduler.detect_overdrafts(account_id).await?;
//!
//! for overdraft in overdrafts {
//!     println!("Overdraft: tx {} by {}", overdraft.transaction_id, overdraft.deficit);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Performance
//!
//! - **Local spend**: < 1ms (no network required)
//! - **Double-spend prevention**: Via escrow limits
//! - **BFT reconciliation**: 3f+1 nodes verified
//! - **Overdraft detection**: Within 1 reconciliation cycle
//! - **Reputation tiers**: Correctly gate credit limits
//!
//! # Security Considerations
//!
//! - Escrow prevents double-spend WITHOUT requiring online verification
//! - BFT committee size: 3f+1 where f is max Byzantine faults (e.g., 4 members tolerates 1 fault)
//! - Reputation system resists Sybil attacks (new accounts have low limits)
//! - Overdrafts are NOT errors - they're conflict resolution opportunities
//!
//! # Novel Research
//!
//! This is **NOVEL RESEARCH** - no existing system combines CRDTs + mutual credit this way.
//! The escrow pattern enables offline-first operation while preventing double-spend.

pub mod account;
pub mod bft;
pub mod error;
pub mod escrow;
pub mod overdraft;
pub mod reputation;
pub mod scheduler;
pub mod transaction;

// Re-export main types
pub use account::{CreditAccount, CreditAccountHandle};
pub use bft::{BftCommittee, BftVote, ReconciliationResult};
pub use error::{CreditError, Result};
pub use escrow::{DeviceEscrow, EscrowManager};
pub use overdraft::{Overdraft, OverdraftResolution, OverdraftResolver};
pub use reputation::{ReputationManager, ReputationTier};
pub use scheduler::MutualCreditScheduler;
pub use transaction::{Transaction, TransactionId, TransactionMetadata, TransactionStatus};

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
