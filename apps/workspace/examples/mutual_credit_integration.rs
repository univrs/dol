/// Mutual Credit System Integration Example
///
/// Demonstrates how the User Profile integrates with the mutual credit
/// system for value exchange within the workspace, using PN-Counter CRDT
/// for monotonic balance tracking and escrow-based transactions.

use automerge::{Automerge, ObjType, ROOT};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

/// User Profile with Credit Account (from workspace.user_profile DOL schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub did: String,
    pub display_name: String,
    pub credit_account: CreditAccount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditAccount {
    pub account_id: String,
    pub balance: i64,  // Derived: total_earned - total_spent
    pub total_earned: i64,  // PN-Counter (monotonic increase)
    pub total_spent: i64,   // PN-Counter (monotonic increase)
    pub credit_limit: i64,
    pub transactions: Vec<CreditTransaction>,
    pub trust_relationships: Vec<TrustRelationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditTransaction {
    pub transaction_id: String,
    pub from: String,
    pub to: String,
    pub amount: i64,
    pub timestamp: i64,
    pub status: TransactionStatus,
    pub description: String,
    pub escrow_proof: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    InEscrow,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustRelationship {
    pub peer_did: String,
    pub trust_limit: i64,
    pub established_at: i64,
    pub successful_transactions: i32,
}

/// PN-Counter based Credit Account implementation
pub struct MutualCreditAccount {
    doc: Automerge,
}

impl MutualCreditAccount {
    /// Create a new credit account
    pub fn new(account_id: String, owner_did: String, credit_limit: i64) -> Self {
        let mut doc = Automerge::new();

        // Initialize account metadata
        doc.put(ROOT, "account_id", account_id).unwrap();
        doc.put(ROOT, "owner_did", owner_did).unwrap();
        doc.put(ROOT, "credit_limit", credit_limit).unwrap();
        doc.put(ROOT, "created_at", chrono::Utc::now().timestamp()).unwrap();

        // Initialize PN-Counter components
        // PN-Counter = (P, N) where P = increments, N = decrements
        // Balance = sum(P) - sum(N)
        doc.put_object(ROOT, "earned_counter", ObjType::Map).unwrap();
        doc.put_object(ROOT, "spent_counter", ObjType::Map).unwrap();

        // Initialize transaction log (RGA)
        doc.put_object(ROOT, "transactions", ObjType::List).unwrap();

        // Initialize trust relationships (RGA)
        doc.put_object(ROOT, "trust_relationships", ObjType::List).unwrap();

        Self { doc }
    }

    /// Increment earned credits (PN-Counter increment)
    pub fn earn_credits(&mut self, amount: i64, peer_id: &str) -> Result<(), String> {
        if amount <= 0 {
            return Err("Amount must be positive".to_string());
        }

        let earned_obj = self.doc.get(ROOT, "earned_counter")
            .ok_or("Earned counter not found")?
            .1;

        // Get current value for this peer (or 0 if first time)
        let current = self.get_counter_value(earned_obj, peer_id);

        // Increment counter (monotonic increase)
        self.doc.put(earned_obj, peer_id, current + amount)
            .map_err(|e| format!("Failed to increment earned: {:?}", e))?;

        Ok(())
    }

    /// Increment spent credits (PN-Counter increment)
    pub fn spend_credits(&mut self, amount: i64, peer_id: &str) -> Result<(), String> {
        if amount <= 0 {
            return Err("Amount must be positive".to_string());
        }

        // Check balance before spending
        let balance = self.get_balance()?;
        if balance < amount {
            return Err(format!("Insufficient balance: {} < {}", balance, amount));
        }

        let spent_obj = self.doc.get(ROOT, "spent_counter")
            .ok_or("Spent counter not found")?
            .1;

        let current = self.get_counter_value(spent_obj, peer_id);

        // Increment counter (monotonic increase)
        self.doc.put(spent_obj, peer_id, current + amount)
            .map_err(|e| format!("Failed to increment spent: {:?}", e))?;

        Ok(())
    }

    /// Get current balance (PN-Counter value)
    pub fn get_balance(&self) -> Result<i64, String> {
        let earned_obj = self.doc.get(ROOT, "earned_counter")
            .ok_or("Earned counter not found")?
            .1;
        let spent_obj = self.doc.get(ROOT, "spent_counter")
            .ok_or("Spent counter not found")?
            .1;

        let total_earned = self.sum_counter(earned_obj);
        let total_spent = self.sum_counter(spent_obj);

        Ok(total_earned - total_spent)
    }

    /// Transfer credits to another user (escrow-based)
    pub fn transfer_credits(
        &mut self,
        to_did: &str,
        amount: i64,
        description: String,
    ) -> Result<String, String> {
        // Step 1: Check balance and trust limit
        let balance = self.get_balance()?;
        if balance < amount {
            return Err("Insufficient balance".to_string());
        }

        let trust_limit = self.get_trust_limit(to_did)?;
        if amount > trust_limit {
            return Err(format!("Transfer exceeds trust limit: {} > {}", amount, trust_limit));
        }

        // Step 2: Create transaction with escrow proof
        let transaction_id = format!("txn-{}", uuid::Uuid::new_v4());
        let escrow_proof = self.create_escrow_proof(&transaction_id, amount)?;

        // Step 3: Record transaction
        let transactions_obj = self.doc.get(ROOT, "transactions")
            .ok_or("Transactions list not found")?
            .1;

        let txn_idx = self.doc.length(transactions_obj);
        let txn_obj = self.doc.insert_object(transactions_obj, txn_idx, ObjType::Map)
            .map_err(|e| format!("Failed to create transaction: {:?}", e))?;

        self.doc.put(txn_obj, "transaction_id", transaction_id.clone()).unwrap();
        self.doc.put(txn_obj, "to", to_did).unwrap();
        self.doc.put(txn_obj, "amount", amount).unwrap();
        self.doc.put(txn_obj, "description", description).unwrap();
        self.doc.put(txn_obj, "status", "InEscrow").unwrap();
        self.doc.put(txn_obj, "escrow_proof", escrow_proof).unwrap();
        self.doc.put(txn_obj, "timestamp", chrono::Utc::now().timestamp()).unwrap();

        // Step 4: Move funds to escrow (increment spent counter)
        self.spend_credits(amount, to_did)?;

        Ok(transaction_id)
    }

    /// Complete a transaction (recipient confirms)
    pub fn complete_transaction(&mut self, transaction_id: &str, from_did: &str) -> Result<(), String> {
        // Find transaction
        let (txn_obj, amount) = self.find_transaction(transaction_id)?;

        // Update status
        self.doc.put(txn_obj, "status", "Completed").unwrap();
        self.doc.put(txn_obj, "completed_at", chrono::Utc::now().timestamp()).unwrap();

        // Credit recipient (increment earned counter)
        self.earn_credits(amount, from_did)?;

        // Update trust relationship
        self.increment_successful_transactions(from_did)?;

        Ok(())
    }

    /// Establish trust relationship with peer
    pub fn establish_trust(&mut self, peer_did: &str, trust_limit: i64) -> Result<(), String> {
        let trust_obj = self.doc.get(ROOT, "trust_relationships")
            .ok_or("Trust relationships list not found")?
            .1;

        let trust_idx = self.doc.length(trust_obj);
        let trust_rel = self.doc.insert_object(trust_obj, trust_idx, ObjType::Map)
            .map_err(|e| format!("Failed to create trust relationship: {:?}", e))?;

        self.doc.put(trust_rel, "peer_did", peer_did).unwrap();
        self.doc.put(trust_rel, "trust_limit", trust_limit).unwrap();
        self.doc.put(trust_rel, "established_at", chrono::Utc::now().timestamp()).unwrap();
        self.doc.put(trust_rel, "successful_transactions", 0i32).unwrap();

        Ok(())
    }

    /// Merge changes from another peer
    pub fn merge(&mut self, other_changes: &[u8]) -> Result<(), String> {
        self.doc.apply_changes(other_changes.to_vec())
            .map_err(|e| format!("Failed to merge changes: {:?}", e))?;
        Ok(())
    }

    /// Get changes to send to peers
    pub fn get_changes(&self) -> Vec<u8> {
        self.doc.save()
    }

    // Helper methods

    fn get_counter_value(&self, counter_obj: automerge::ObjId, peer_id: &str) -> i64 {
        if let Some((automerge::Value::Scalar(s), _)) = self.doc.get(counter_obj, peer_id) {
            if let automerge::ScalarValue::Int(i) = s.as_ref() {
                return *i;
            }
        }
        0
    }

    fn sum_counter(&self, counter_obj: automerge::ObjId) -> i64 {
        // Sum all peer values in the counter
        let keys = self.doc.keys(counter_obj);
        let mut sum = 0i64;
        for key in keys {
            sum += self.get_counter_value(counter_obj, &key);
        }
        sum
    }

    fn get_trust_limit(&self, peer_did: &str) -> Result<i64, String> {
        let trust_obj = self.doc.get(ROOT, "trust_relationships")
            .ok_or("Trust relationships not found")?
            .1;

        let len = self.doc.length(trust_obj);
        for i in 0..len {
            if let Some((_, rel_obj)) = self.doc.get(trust_obj, i) {
                if let Some((automerge::Value::Scalar(s), _)) = self.doc.get(rel_obj, "peer_did") {
                    if s.to_string().trim_matches('"') == peer_did {
                        if let Some((automerge::Value::Scalar(limit), _)) = self.doc.get(rel_obj, "trust_limit") {
                            if let automerge::ScalarValue::Int(l) = limit.as_ref() {
                                return Ok(*l);
                            }
                        }
                    }
                }
            }
        }
        Err(format!("No trust relationship with {}", peer_did))
    }

    fn create_escrow_proof(&self, transaction_id: &str, amount: i64) -> Result<String, String> {
        // Create cryptographic proof of escrow
        // In production, use proper cryptographic signatures
        let mut hasher = Sha256::new();
        hasher.update(transaction_id.as_bytes());
        hasher.update(amount.to_le_bytes());
        hasher.update(chrono::Utc::now().timestamp().to_le_bytes());
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    fn find_transaction(&self, transaction_id: &str) -> Result<(automerge::ObjId, i64), String> {
        let transactions_obj = self.doc.get(ROOT, "transactions")
            .ok_or("Transactions list not found")?
            .1;

        let len = self.doc.length(transactions_obj);
        for i in 0..len {
            if let Some((_, txn_obj)) = self.doc.get(transactions_obj, i) {
                if let Some((automerge::Value::Scalar(s), _)) = self.doc.get(txn_obj, "transaction_id") {
                    if s.to_string().trim_matches('"') == transaction_id {
                        if let Some((automerge::Value::Scalar(amt), _)) = self.doc.get(txn_obj, "amount") {
                            if let automerge::ScalarValue::Int(a) = amt.as_ref() {
                                return Ok((txn_obj, *a));
                            }
                        }
                    }
                }
            }
        }
        Err(format!("Transaction {} not found", transaction_id))
    }

    fn increment_successful_transactions(&mut self, peer_did: &str) -> Result<(), String> {
        let trust_obj = self.doc.get(ROOT, "trust_relationships")
            .ok_or("Trust relationships not found")?
            .1;

        let len = self.doc.length(trust_obj);
        for i in 0..len {
            if let Some((_, rel_obj)) = self.doc.get(trust_obj, i) {
                if let Some((automerge::Value::Scalar(s), _)) = self.doc.get(rel_obj, "peer_did") {
                    if s.to_string().trim_matches('"') == peer_did {
                        if let Some((automerge::Value::Scalar(count), _)) = self.doc.get(rel_obj, "successful_transactions") {
                            if let automerge::ScalarValue::Int(c) = count.as_ref() {
                                self.doc.put(rel_obj, "successful_transactions", c + 1).unwrap();
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// Example: Simulating mutual credit exchange
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credit_transfer() {
        // Alice and Bob create accounts
        let mut alice = MutualCreditAccount::new(
            "acc-alice".to_string(),
            "did:key:alice".to_string(),
            5000,
        );

        let mut bob = MutualCreditAccount::new(
            "acc-bob".to_string(),
            "did:key:bob".to_string(),
            5000,
        );

        // Alice earns initial credits (from system)
        alice.earn_credits(1000, "system").unwrap();
        assert_eq!(alice.get_balance().unwrap(), 1000);

        // Alice establishes trust with Bob
        alice.establish_trust("did:key:bob", 2000).unwrap();

        // Alice transfers 500 credits to Bob
        let txn_id = alice.transfer_credits(
            "did:key:bob",
            500,
            "Task completion payment".to_string(),
        ).unwrap();

        // Alice's balance is reduced
        assert_eq!(alice.get_balance().unwrap(), 500);

        // Bob confirms transaction
        bob.complete_transaction(&txn_id, "did:key:alice").unwrap();

        // Sync changes
        let bob_changes = bob.get_changes();
        bob.merge(&alice.get_changes()).unwrap();
        alice.merge(&bob_changes).unwrap();

        // Bob's balance increases
        assert_eq!(bob.get_balance().unwrap(), 500);

        println!("✅ Credit transfer completed successfully");
    }

    #[test]
    fn test_pn_counter_convergence() {
        // Two peers earning credits concurrently
        let mut peer1 = MutualCreditAccount::new(
            "acc-shared".to_string(),
            "did:key:shared".to_string(),
            10000,
        );

        let mut peer2 = MutualCreditAccount::new(
            "acc-shared".to_string(),
            "did:key:shared".to_string(),
            10000,
        );

        // Concurrent operations
        peer1.earn_credits(500, "client-a").unwrap();
        peer2.earn_credits(300, "client-b").unwrap();

        // Sync
        let peer1_changes = peer1.get_changes();
        let peer2_changes = peer2.get_changes();

        peer1.merge(&peer2_changes).unwrap();
        peer2.merge(&peer1_changes).unwrap();

        // Both converge to same balance
        assert_eq!(peer1.get_balance().unwrap(), peer2.get_balance().unwrap());
        assert_eq!(peer1.get_balance().unwrap(), 800);

        println!("✅ PN-Counter converged: {}", peer1.get_balance().unwrap());
    }

    #[test]
    fn test_trust_limit_enforcement() {
        let mut alice = MutualCreditAccount::new(
            "acc-alice".to_string(),
            "did:key:alice".to_string(),
            5000,
        );

        alice.earn_credits(2000, "system").unwrap();
        alice.establish_trust("did:key:bob", 500).unwrap();

        // Try to transfer more than trust limit
        let result = alice.transfer_credits(
            "did:key:bob",
            1000,
            "Large payment".to_string(),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("trust limit"));

        println!("✅ Trust limit enforcement works");
    }
}

/// Usage example in a real application
///
/// ```rust
/// // Create user's credit account
/// let mut account = MutualCreditAccount::new(
///     "acc-001".to_string(),
///     "did:key:alice".to_string(),
///     5000,
/// );
///
/// // Earn credits for completed work
/// account.earn_credits(500, "did:key:bob")?;
///
/// // Check balance
/// let balance = account.get_balance()?;
/// println!("Balance: {}", balance);
///
/// // Establish trust with peer
/// account.establish_trust("did:key:carol", 1000)?;
///
/// // Transfer credits
/// let txn_id = account.transfer_credits(
///     "did:key:carol",
///     200,
///     "Design work payment".to_string(),
/// )?;
///
/// // Broadcast changes via P2P
/// let changes = account.get_changes();
/// sync_engine.broadcast_changes(&changes).await?;
/// ```
