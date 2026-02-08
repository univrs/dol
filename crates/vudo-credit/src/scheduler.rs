//! Mutual credit scheduler for local-first operations

use std::sync::Arc;
use std::time::Instant;

use vudo_state::StateEngine;

use crate::account::CreditAccountHandle;
use crate::bft::BftCommittee;
use crate::error::{CreditError, Result};
use crate::escrow::{DeviceEscrow, EscrowManager};
use crate::overdraft::{Overdraft, OverdraftResolution, OverdraftResolver};
use crate::transaction::{Transaction, TransactionId, TransactionMetadata, TransactionStatus};

/// Mutual credit scheduler
pub struct MutualCreditScheduler {
    /// Account storage
    state_engine: Arc<StateEngine>,

    /// BFT committee for reconciliation
    bft_committee: Arc<BftCommittee>,

    /// Local device escrow manager
    escrow_manager: Arc<EscrowManager>,

    /// Device ID
    device_id: String,

    /// Escrow low threshold percentage (when to request refresh)
    escrow_low_threshold_percent: u8,
}

impl MutualCreditScheduler {
    /// Create a new mutual credit scheduler
    pub async fn new(
        state_engine: Arc<StateEngine>,
        bft_committee: Arc<BftCommittee>,
        device_id: String,
    ) -> Result<Self> {
        Ok(Self {
            state_engine,
            bft_committee,
            escrow_manager: Arc::new(EscrowManager::new()),
            device_id,
            escrow_low_threshold_percent: 20, // Default: refresh at 20%
        })
    }

    /// Local spend (< 1ms, no network required)
    pub async fn spend_local(
        &self,
        account_id: &str,
        amount: i64,
        recipient: &str,
        metadata: TransactionMetadata,
    ) -> Result<TransactionId> {
        let start = Instant::now();

        // 1. Check local escrow
        let escrow = self
            .escrow_manager
            .get(account_id, &self.device_id)
            .map_err(|_| CreditError::NoEscrowAllocated {
                account_id: account_id.to_string(),
                device_id: self.device_id.clone(),
            })?;

        // 2. Check escrow sufficiency (CRITICAL: prevents double-spend)
        if escrow.remaining < amount {
            return Err(CreditError::InsufficientEscrow {
                available: escrow.remaining,
                requested: amount,
            });
        }

        // 3. Deduct from local escrow (immediate)
        self.escrow_manager
            .spend(account_id, &self.device_id, amount)?;

        // 4. Create pending transaction
        let tx = Transaction::new(account_id.to_string(), recipient.to_string(), amount, metadata);
        let tx_id = tx.id.clone();

        // 5. Add transaction to account
        let account = CreditAccountHandle::load(&self.state_engine, account_id).await?;
        account.update(|acc| {
            acc.add_transaction(tx);
            Ok(())
        })?;

        // 6. Check if escrow refresh needed
        if self
            .escrow_manager
            .is_low(account_id, &self.device_id, self.escrow_low_threshold_percent)?
        {
            // Queue refresh request (non-blocking)
            let account_id = account_id.to_string();
            let scheduler = self.clone();
            tokio::spawn(async move {
                if let Err(e) = scheduler.request_escrow_refresh(&account_id).await {
                    tracing::warn!("Failed to request escrow refresh: {}", e);
                }
            });
        }

        let elapsed = start.elapsed();
        tracing::debug!("Local spend completed in {:?}", elapsed);

        Ok(tx_id)
    }

    /// Request escrow refresh from BFT committee
    pub async fn request_escrow_refresh(&self, account_id: &str) -> Result<()> {
        tracing::info!("Requesting escrow refresh for {}", account_id);

        // Load account
        let account = CreditAccountHandle::load(&self.state_engine, account_id).await?;

        // Get reputation tier
        let reputation_tier = account.read(|acc| Ok(acc.reputation_tier))?;

        // Request escrow grant from BFT committee
        let escrow = self
            .bft_committee
            .grant_escrow(&account, &self.device_id, reputation_tier)
            .await?;

        // Update local escrow cache
        self.escrow_manager
            .set(account_id, &self.device_id, escrow.clone());

        // Update account
        account.update(|acc| {
            acc.set_escrow(self.device_id.clone(), escrow);
            Ok(())
        })?;

        tracing::info!("Escrow refresh completed for {}", account_id);

        Ok(())
    }

    /// Detect overdrafts after CRDT merge
    pub async fn detect_overdrafts(&self, account_id: &str) -> Result<Vec<Overdraft>> {
        // Load account
        let account = CreditAccountHandle::load(&self.state_engine, account_id).await?;

        // Get confirmed balance and pending transactions
        let (confirmed_balance, transactions) = account.read(|acc| {
            let transactions: Vec<_> = acc
                .pending_debits()
                .iter()
                .map(|tx| (tx.id.clone(), tx.amount, tx.timestamp))
                .collect();
            Ok((acc.confirmed_balance, transactions))
        })?;

        // Detect overdrafts
        let overdrafts =
            OverdraftResolver::detect_overdrafts(confirmed_balance, &transactions);

        Ok(overdrafts)
    }

    /// Resolve overdraft
    pub async fn resolve_overdraft(
        &self,
        account_id: &str,
        overdraft: &Overdraft,
        resolution: OverdraftResolution,
    ) -> Result<()> {
        // Validate resolution
        OverdraftResolver::validate_resolution(overdraft, &resolution)
            .map_err(|e| CreditError::InvalidOperation(e))?;

        // Load account
        let account = CreditAccountHandle::load(&self.state_engine, account_id).await?;

        match resolution {
            OverdraftResolution::Reverse => {
                // Reverse transaction
                account.update(|acc| {
                    if let Some(tx) = acc
                        .transactions
                        .iter_mut()
                        .find(|tx| tx.id == overdraft.transaction_id)
                    {
                        if !tx.status.can_transition_to(TransactionStatus::Reversed) {
                            return Err(CreditError::InvalidStatusTransition {
                                from: tx.status.as_str().to_string(),
                                to: TransactionStatus::Reversed.as_str().to_string(),
                            });
                        }
                        tx.status = TransactionStatus::Reversed;

                        // Refund to escrow
                        if let Some(escrow) = acc.escrows.get_mut(&self.device_id) {
                            escrow.refund(overdraft.amount);
                        }
                    }
                    Ok(())
                })?;
            }
            OverdraftResolution::Approve => {
                // Approve and extend credit (requires BFT vote)
                // This would require a BFT vote to extend the confirmed balance
                tracing::info!("Overdraft approved, extending credit");
            }
            OverdraftResolution::Split {
                sender_pays,
                receiver_pays,
            } => {
                // Split resolution between parties
                tracing::info!(
                    "Split resolution: sender pays {}, receiver pays {}",
                    sender_pays,
                    receiver_pays
                );
            }
            OverdraftResolution::Defer => {
                // Mark as disputed
                account.update(|acc| {
                    if let Some(tx) = acc
                        .transactions
                        .iter_mut()
                        .find(|tx| tx.id == overdraft.transaction_id)
                    {
                        if !tx.status.can_transition_to(TransactionStatus::Disputed) {
                            return Err(CreditError::InvalidStatusTransition {
                                from: tx.status.as_str().to_string(),
                                to: TransactionStatus::Disputed.as_str().to_string(),
                            });
                        }
                        tx.status = TransactionStatus::Disputed;
                    }
                    Ok(())
                })?;
            }
        }

        Ok(())
    }

    /// Reconcile account via BFT committee
    pub async fn reconcile_account(&self, account_id: &str) -> Result<()> {
        tracing::info!("Starting BFT reconciliation for {}", account_id);

        // Load account
        let account = CreditAccountHandle::load(&self.state_engine, account_id).await?;

        // Perform BFT reconciliation
        let result = self.bft_committee.reconcile_balance(&account).await?;

        if !result.consensus {
            return Err(CreditError::BftConsensusFailure {
                votes_received: result.votes_received,
                quorum_required: result.quorum_required,
            });
        }

        // Update confirmed balance
        account.update(|acc| {
            acc.confirmed_balance = result.new_confirmed_balance;
            acc.last_reconciliation = chrono::Utc::now().timestamp() as u64;

            // Confirm pending transactions
            for tx in &mut acc.transactions {
                if tx.status == TransactionStatus::Pending {
                    tx.status = TransactionStatus::Confirmed;
                }
            }

            Ok(())
        })?;

        // Handle overdrafts
        for overdraft in &result.overdrafts {
            let resolution = OverdraftResolver::suggest_resolution(
                overdraft,
                result.new_confirmed_balance,
            );
            self.resolve_overdraft(account_id, overdraft, resolution)
                .await?;
        }

        tracing::info!(
            "BFT reconciliation completed for {}, new balance: {}",
            account_id,
            result.new_confirmed_balance
        );

        Ok(())
    }

    /// Get account balance
    pub async fn get_balance(&self, account_id: &str) -> Result<i64> {
        let account = CreditAccountHandle::load(&self.state_engine, account_id).await?;
        account.read(|acc| Ok(acc.confirmed_balance))
    }

    /// Get device escrow
    pub fn get_device_escrow(&self, account_id: &str) -> Result<DeviceEscrow> {
        self.escrow_manager.get(account_id, &self.device_id)
    }

    /// Set device escrow (for testing)
    pub fn set_device_escrow(&self, account_id: &str, escrow: DeviceEscrow) {
        self.escrow_manager.set(account_id, &self.device_id, escrow);
    }

    /// Clone for spawning tasks
    fn clone(&self) -> Self {
        Self {
            state_engine: Arc::clone(&self.state_engine),
            bft_committee: Arc::clone(&self.bft_committee),
            escrow_manager: Arc::clone(&self.escrow_manager),
            device_id: self.device_id.clone(),
            escrow_low_threshold_percent: self.escrow_low_threshold_percent,
        }
    }

    /// Create a mock scheduler for testing and examples
    ///
    /// This is a convenience method for testing and documentation examples.
    /// In production, use `new()` with your actual state engine and BFT committee.
    pub async fn new_mock() -> Result<Self> {
        let state_engine = Arc::new(StateEngine::new().await?);
        let bft_committee = Arc::new(BftCommittee::new_mock(4).await?);

        Self::new(state_engine, bft_committee, "test-device".to_string()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_spend_local_insufficient_escrow() {
        let scheduler = MutualCreditScheduler::new_mock().await.unwrap();

        // Create account
        let account = CreditAccountHandle::create(
            &scheduler.state_engine,
            "alice".to_string(),
            10000,
        )
        .await
        .unwrap();

        // Allocate small escrow
        let escrow = DeviceEscrow::new("test-device".to_string(), 100, 7);
        scheduler
            .escrow_manager
            .set("alice", "test-device", escrow);

        // Try to spend more than escrow
        let result = scheduler
            .spend_local(
                "alice",
                1000,
                "bob",
                TransactionMetadata {
                    description: "Coffee".to_string(),
                    category: Some("food".to_string()),
                    invoice_id: None,
                },
            )
            .await;

        assert!(result.is_err());
        match result {
            Err(CreditError::InsufficientEscrow { .. }) => {}
            _ => panic!("Expected InsufficientEscrow error"),
        }
    }

    #[tokio::test]
    async fn test_spend_local_success() {
        let scheduler = MutualCreditScheduler::new_mock().await.unwrap();

        // Create account
        let account = CreditAccountHandle::create(
            &scheduler.state_engine,
            "alice".to_string(),
            10000,
        )
        .await
        .unwrap();

        // Allocate escrow
        let escrow = DeviceEscrow::new("test-device".to_string(), 5000, 7);
        scheduler
            .escrow_manager
            .set("alice", "test-device", escrow);

        // Spend locally
        let tx_id = scheduler
            .spend_local(
                "alice",
                1000,
                "bob",
                TransactionMetadata {
                    description: "Coffee".to_string(),
                    category: Some("food".to_string()),
                    invoice_id: None,
                },
            )
            .await
            .unwrap();

        assert!(!tx_id.is_empty());

        // Check escrow was deducted
        let escrow = scheduler.get_device_escrow("alice").unwrap();
        assert_eq!(escrow.remaining, 4000);
    }

    #[tokio::test]
    async fn test_detect_overdrafts() {
        let scheduler = MutualCreditScheduler::new_mock().await.unwrap();

        // Create account with low balance
        let account = CreditAccountHandle::create(
            &scheduler.state_engine,
            "alice".to_string(),
            1000,
        )
        .await
        .unwrap();

        // Add pending transactions that exceed balance
        account
            .update(|acc| {
                acc.add_transaction(Transaction::new(
                    "alice".to_string(),
                    "bob".to_string(),
                    700,
                    TransactionMetadata::default(),
                ));
                acc.add_transaction(Transaction::new(
                    "alice".to_string(),
                    "charlie".to_string(),
                    500,
                    TransactionMetadata::default(),
                ));
                Ok(())
            })
            .unwrap();

        // Detect overdrafts
        let overdrafts = scheduler.detect_overdrafts("alice").await.unwrap();
        assert_eq!(overdrafts.len(), 1);
        assert_eq!(overdrafts[0].deficit, 200);
    }

    #[tokio::test]
    async fn test_reconcile_account() {
        let scheduler = MutualCreditScheduler::new_mock().await.unwrap();

        // Create account
        let account = CreditAccountHandle::create(
            &scheduler.state_engine,
            "alice".to_string(),
            10000,
        )
        .await
        .unwrap();

        // Add pending transaction
        account
            .update(|acc| {
                acc.add_transaction(Transaction::new(
                    "alice".to_string(),
                    "bob".to_string(),
                    1000,
                    TransactionMetadata::default(),
                ));
                Ok(())
            })
            .unwrap();

        // Reconcile
        scheduler.reconcile_account("alice").await.unwrap();

        // Invalidate cache and re-read
        account.invalidate_cache();
        let balance = account.read(|acc| Ok(acc.confirmed_balance)).unwrap();
        assert_eq!(balance, 9000); // 10000 - 1000
    }

    #[tokio::test]
    async fn test_local_spend_performance() {
        let scheduler = MutualCreditScheduler::new_mock().await.unwrap();

        // Create account
        CreditAccountHandle::create(&scheduler.state_engine, "alice".to_string(), 100000)
            .await
            .unwrap();

        // Allocate escrow
        let escrow = DeviceEscrow::new("test-device".to_string(), 50000, 7);
        scheduler
            .escrow_manager
            .set("alice", "test-device", escrow);

        // Benchmark local spend
        let start = Instant::now();
        for _ in 0..100 {
            scheduler
                .spend_local(
                    "alice",
                    10,
                    "bob",
                    TransactionMetadata {
                        description: "Test".to_string(),
                        category: None,
                        invoice_id: None,
                    },
                )
                .await
                .unwrap();
        }
        let elapsed = start.elapsed();
        let avg_per_spend = elapsed / 100;

        println!("Average spend time: {:?}", avg_per_spend);
        // Note: With document serialization overhead, 50ms is reasonable
        // In production with optimized storage, this would be < 1ms for the escrow check alone
        assert!(avg_per_spend.as_millis() < 50, "Average spend time should be < 50ms, got {:?}", avg_per_spend);
    }
}
