//! Credit account structure and management

use std::collections::HashMap;
use std::sync::Arc;

use automerge::{transaction::Transactable, ReadDoc, ROOT};
use serde::{Deserialize, Serialize};
use vudo_state::{DocumentHandle, DocumentId, StateEngine};

use crate::error::Result;
use crate::escrow::DeviceEscrow;
use crate::reputation::ReputationTier;
use crate::transaction::Transaction;

/// Credit account (Automerge-backed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditAccount {
    /// Account owner DID
    pub owner: String,

    /// Confirmed balance (BFT-verified, strong consistency, in cents)
    pub confirmed_balance: i64,

    /// Reputation tier (0-5, affects credit limits)
    pub reputation_tier: ReputationTier,

    /// Transaction log
    pub transactions: Vec<Transaction>,

    /// Escrow allocations per device
    pub escrows: HashMap<String, DeviceEscrow>,

    /// Pending credits (eventually consistent, in cents)
    pub pending_credits: i64,

    /// Last BFT reconciliation timestamp (Unix epoch seconds)
    pub last_reconciliation: u64,
}

impl CreditAccount {
    /// Create a new credit account
    pub fn new(owner: String, initial_balance: i64) -> Self {
        Self {
            owner,
            confirmed_balance: initial_balance,
            reputation_tier: ReputationTier::default(),
            transactions: Vec::new(),
            escrows: HashMap::new(),
            pending_credits: 0,
            last_reconciliation: chrono::Utc::now().timestamp() as u64,
        }
    }

    /// Add a transaction
    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    /// Get transaction by ID
    pub fn get_transaction(&self, tx_id: &str) -> Option<&Transaction> {
        self.transactions.iter().find(|tx| tx.id == tx_id)
    }

    /// Get all pending transactions from this account
    pub fn pending_debits(&self) -> Vec<&Transaction> {
        self.transactions
            .iter()
            .filter(|tx| tx.is_from(&self.owner) && tx.is_pending())
            .collect()
    }

    /// Get all pending credits to this account
    pub fn pending_credits_txs(&self) -> Vec<&Transaction> {
        self.transactions
            .iter()
            .filter(|tx| tx.is_to(&self.owner) && tx.is_pending())
            .collect()
    }

    /// Calculate total pending debits
    pub fn total_pending_debits(&self) -> i64 {
        self.pending_debits().iter().map(|tx| tx.amount).sum()
    }

    /// Calculate total pending credits
    pub fn total_pending_credits_amount(&self) -> i64 {
        self.pending_credits_txs().iter().map(|tx| tx.amount).sum()
    }

    /// Get escrow for device
    pub fn get_escrow(&self, device_id: &str) -> Option<&DeviceEscrow> {
        self.escrows.get(device_id)
    }

    /// Set escrow for device
    pub fn set_escrow(&mut self, device_id: String, escrow: DeviceEscrow) {
        self.escrows.insert(device_id, escrow);
    }

    /// Remove escrow for device
    pub fn remove_escrow(&mut self, device_id: &str) -> Option<DeviceEscrow> {
        self.escrows.remove(device_id)
    }

    /// Calculate total allocated escrow
    pub fn total_escrow_allocated(&self) -> i64 {
        self.escrows.values().map(|e| e.allocated).sum()
    }

    /// Calculate total remaining escrow
    pub fn total_escrow_remaining(&self) -> i64 {
        self.escrows.values().map(|e| e.remaining).sum()
    }

    /// Check if account can allocate new escrow
    pub fn can_allocate_escrow(&self, amount: i64) -> bool {
        let available = self.confirmed_balance - self.total_escrow_allocated();
        available >= amount
    }

    /// Upgrade reputation tier
    pub fn upgrade_reputation(&mut self) -> Result<()> {
        self.reputation_tier.upgrade()
    }

    /// Downgrade reputation tier
    pub fn downgrade_reputation(&mut self) -> Result<()> {
        self.reputation_tier.downgrade()
    }
}

/// Credit account handle with Automerge document
pub struct CreditAccountHandle {
    /// Document ID
    pub id: DocumentId,

    /// Document handle
    doc_handle: DocumentHandle,

    /// Cached account data
    cached_account: Arc<parking_lot::RwLock<Option<CreditAccount>>>,
}

impl CreditAccountHandle {
    /// Create a new account handle
    pub fn new(id: DocumentId, doc_handle: DocumentHandle) -> Self {
        Self {
            id,
            doc_handle,
            cached_account: Arc::new(parking_lot::RwLock::new(None)),
        }
    }

    /// Create account in state engine
    pub async fn create(
        state_engine: &StateEngine,
        owner: String,
        initial_balance: i64,
    ) -> Result<Self> {
        let doc_id = DocumentId::new("credit", &owner);
        let handle = state_engine.create_document(doc_id.clone()).await?;

        // Initialize account data
        let account = CreditAccount::new(owner.clone(), initial_balance);
        let account_json = serde_json::to_string(&account)?;

        handle.update(|tx| {
            tx.put(ROOT, "owner", owner)?;
            tx.put(ROOT, "confirmed_balance", initial_balance)?;
            tx.put(ROOT, "reputation_tier", 0i64)?;
            tx.put(ROOT, "pending_credits", 0i64)?;
            tx.put(ROOT, "last_reconciliation", chrono::Utc::now().timestamp())?;
            tx.put(ROOT, "data_json", account_json)?;
            Ok(())
        })?;

        Ok(Self::new(doc_id, handle))
    }

    /// Load account from state engine
    pub async fn load(state_engine: &StateEngine, owner: &str) -> Result<Self> {
        let doc_id = DocumentId::new("credit", owner);
        let handle = state_engine.get_document(&doc_id).await?;

        Ok(Self::new(doc_id, handle))
    }

    /// Read account data
    pub fn read<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&CreditAccount) -> Result<R>,
    {
        // Try to use cached account first
        if let Some(account) = self.cached_account.read().as_ref() {
            return f(account);
        }

        // Otherwise, load from document
        let account = self.doc_handle.read(|doc| {
            // Try to read the full JSON first
            if let Some((automerge::Value::Scalar(s), _)) = doc.get(ROOT, "data_json")? {
                if let automerge::ScalarValue::Str(json_str) = s.as_ref() {
                    let account: CreditAccount = serde_json::from_str(&json_str.to_string())
                        .map_err(|e| {
                            vudo_state::StateError::Internal(format!(
                                "Failed to deserialize account: {}",
                                e
                            ))
                        })?;
                    return Ok(account);
                }
            }

            // Fallback: reconstruct from individual fields
            Err(vudo_state::StateError::Internal(
                "Account data not found".to_string(),
            ))
        })?;

        // Cache the account
        *self.cached_account.write() = Some(account.clone());

        f(&account)
    }

    /// Update account data
    pub fn update<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&mut CreditAccount) -> Result<()>,
    {
        // Load current account
        let mut account = self.read(|acc| Ok(acc.clone()))?;

        // Apply update
        f(&mut account)?;

        // Serialize and save
        let account_json = serde_json::to_string(&account)?;

        self.doc_handle.update(|tx| {
            tx.put(ROOT, "owner", account.owner.clone())?;
            tx.put(ROOT, "confirmed_balance", account.confirmed_balance)?;
            tx.put(ROOT, "reputation_tier", account.reputation_tier.value() as i64)?;
            tx.put(ROOT, "pending_credits", account.pending_credits)?;
            tx.put(ROOT, "last_reconciliation", account.last_reconciliation as i64)?;
            tx.put(ROOT, "data_json", account_json)?;
            Ok(())
        })?;

        // Update cache
        *self.cached_account.write() = Some(account);

        Ok(())
    }

    /// Invalidate cache (call after external changes)
    pub fn invalidate_cache(&self) {
        *self.cached_account.write() = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credit_account_new() {
        let account = CreditAccount::new("alice".to_string(), 10000);
        assert_eq!(account.owner, "alice");
        assert_eq!(account.confirmed_balance, 10000);
        assert_eq!(account.reputation_tier.value(), 0);
    }

    #[test]
    fn test_credit_account_add_transaction() {
        let mut account = CreditAccount::new("alice".to_string(), 10000);
        let tx = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            1000,
            crate::transaction::TransactionMetadata::default(),
        );
        account.add_transaction(tx.clone());
        assert_eq!(account.transactions.len(), 1);
        assert_eq!(account.get_transaction(&tx.id), Some(&tx));
    }

    #[test]
    fn test_credit_account_pending_debits() {
        let mut account = CreditAccount::new("alice".to_string(), 10000);
        let tx1 = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            1000,
            crate::transaction::TransactionMetadata::default(),
        );
        let tx2 = Transaction::new(
            "bob".to_string(),
            "alice".to_string(),
            500,
            crate::transaction::TransactionMetadata::default(),
        );
        account.add_transaction(tx1);
        account.add_transaction(tx2);

        let pending_debits = account.pending_debits();
        assert_eq!(pending_debits.len(), 1);
        assert_eq!(pending_debits[0].from, "alice");
    }

    #[test]
    fn test_credit_account_escrow() {
        let mut account = CreditAccount::new("alice".to_string(), 10000);
        let escrow = DeviceEscrow::new("device1".to_string(), 5000, 7);

        account.set_escrow("device1".to_string(), escrow.clone());
        assert_eq!(account.get_escrow("device1"), Some(&escrow));
        assert_eq!(account.total_escrow_allocated(), 5000);
        assert_eq!(account.total_escrow_remaining(), 5000);
    }

    #[test]
    fn test_credit_account_can_allocate_escrow() {
        let mut account = CreditAccount::new("alice".to_string(), 10000);
        assert!(account.can_allocate_escrow(5000));

        let escrow = DeviceEscrow::new("device1".to_string(), 8000, 7);
        account.set_escrow("device1".to_string(), escrow);

        assert!(!account.can_allocate_escrow(5000)); // Only 2000 available
        assert!(account.can_allocate_escrow(2000));
    }

    #[test]
    fn test_credit_account_reputation() {
        let mut account = CreditAccount::new("alice".to_string(), 10000);
        assert_eq!(account.reputation_tier.value(), 0);

        account.upgrade_reputation().unwrap();
        assert_eq!(account.reputation_tier.value(), 1);

        account.downgrade_reputation().unwrap();
        assert_eq!(account.reputation_tier.value(), 0);
    }

    #[tokio::test]
    async fn test_credit_account_handle_create() {
        let state_engine = StateEngine::new().await.unwrap();
        let handle = CreditAccountHandle::create(&state_engine, "alice".to_string(), 10000)
            .await
            .unwrap();

        let owner = handle.read(|acc| Ok(acc.owner.clone())).unwrap();
        assert_eq!(owner, "alice");
    }

    #[tokio::test]
    async fn test_credit_account_handle_update() {
        let state_engine = StateEngine::new().await.unwrap();
        let handle = CreditAccountHandle::create(&state_engine, "alice".to_string(), 10000)
            .await
            .unwrap();

        handle
            .update(|acc| {
                acc.confirmed_balance = 15000;
                Ok(())
            })
            .unwrap();

        let balance = handle
            .read(|acc| Ok(acc.confirmed_balance))
            .unwrap();
        assert_eq!(balance, 15000);
    }
}
