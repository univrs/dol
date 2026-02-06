//! BFT committee for account reconciliation

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::account::CreditAccountHandle;
use crate::error::{CreditError, Result};
use crate::escrow::DeviceEscrow;
use crate::overdraft::Overdraft;
use crate::reputation::ReputationManager;

/// BFT vote from a committee member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BftVote {
    /// Voter DID
    pub voter: String,

    /// Proposed balance
    pub proposed_balance: i64,

    /// Vote timestamp
    pub timestamp: u64,

    /// Signature (ED25519)
    pub signature: Vec<u8>,
}

/// BFT reconciliation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationResult {
    /// New confirmed balance
    pub new_confirmed_balance: i64,

    /// Detected overdrafts
    pub overdrafts: Vec<Overdraft>,

    /// Whether consensus was reached
    pub consensus: bool,

    /// Number of votes received
    pub votes_received: usize,

    /// Quorum required
    pub quorum_required: usize,
}

/// BFT Committee for account reconciliation
pub struct BftCommittee {
    /// Committee members (DIDs)
    members: Vec<String>,

    /// Quorum size (2f+1 for safety, where f is max Byzantine faults)
    quorum_size: usize,

    /// Vote storage (account_id -> votes)
    votes: Arc<RwLock<HashMap<String, Vec<BftVote>>>>,
}

impl BftCommittee {
    /// Create a new BFT committee
    ///
    /// # Arguments
    /// * `members` - List of committee member DIDs
    ///
    /// # Panics
    /// Panics if less than 4 members (need 3f+1 with f >= 1)
    pub fn new(members: Vec<String>) -> Result<Self> {
        if members.len() < 4 {
            return Err(CreditError::InvalidOperation(
                "BFT committee requires at least 4 members (3f+1 with f=1)".to_string(),
            ));
        }

        // Calculate quorum: 2f+1 where 3f+1 = total
        // So f = (total - 1) / 3
        // And quorum = 2 * ((total - 1) / 3) + 1
        let f = (members.len() - 1) / 3;
        let quorum_size = 2 * f + 1;

        Ok(Self {
            members,
            quorum_size,
            votes: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Get committee size
    pub fn size(&self) -> usize {
        self.members.len()
    }

    /// Get quorum size
    pub fn quorum(&self) -> usize {
        self.quorum_size
    }

    /// Get maximum tolerated Byzantine faults
    pub fn max_byzantine_faults(&self) -> usize {
        (self.members.len() - 1) / 3
    }

    /// Reconcile account balance (BFT consensus)
    pub async fn reconcile_balance(
        &self,
        account: &CreditAccountHandle,
    ) -> Result<ReconciliationResult> {
        // Read current account state
        let (confirmed_balance, pending_credits, total_debits, transactions) = account.read(|acc| {
            let total_debits = acc.total_pending_debits();
            let transactions: Vec<_> = acc
                .pending_debits()
                .iter()
                .map(|tx| (tx.id.clone(), tx.amount, tx.timestamp))
                .collect();

            Ok((
                acc.confirmed_balance,
                acc.pending_credits,
                total_debits,
                transactions,
            ))
        })?;

        // Calculate proposed new balance
        let proposed_balance = confirmed_balance + pending_credits - total_debits;

        // Simulate collecting votes from committee members
        // In a real implementation, this would be P2P communication
        let votes = self.simulate_vote_collection(&account.id.key, proposed_balance).await?;

        // Check if we reached quorum
        let consensus = votes.len() >= self.quorum_size;

        // Detect overdrafts
        let overdrafts = if consensus {
            crate::overdraft::OverdraftResolver::detect_overdrafts(confirmed_balance, &transactions)
        } else {
            Vec::new()
        };

        Ok(ReconciliationResult {
            new_confirmed_balance: if consensus {
                proposed_balance
            } else {
                confirmed_balance
            },
            overdrafts,
            consensus,
            votes_received: votes.len(),
            quorum_required: self.quorum_size,
        })
    }

    /// Grant escrow to device
    pub async fn grant_escrow(
        &self,
        account: &CreditAccountHandle,
        device_id: &str,
        reputation_tier: crate::reputation::ReputationTier,
    ) -> Result<DeviceEscrow> {
        // Get current confirmed balance
        let (confirmed_balance, total_escrow_allocated) = account.read(|acc| {
            Ok((acc.confirmed_balance, acc.total_escrow_allocated()))
        })?;

        // Calculate escrow allocation based on reputation tier
        let escrow_limit = ReputationManager::escrow_limit(reputation_tier);
        let available = confirmed_balance - total_escrow_allocated;
        let grant_amount = available.min(escrow_limit);

        if grant_amount <= 0 {
            return Err(CreditError::InsufficientBalanceForEscrow);
        }

        // Create escrow
        let duration_days = ReputationManager::escrow_duration_days(reputation_tier);
        let escrow = DeviceEscrow::new(device_id.to_string(), grant_amount, duration_days);

        // Vote on escrow grant
        let votes = self
            .vote_escrow_grant(&account.id.key, &escrow)
            .await?;

        if votes.len() >= self.quorum_size {
            Ok(escrow)
        } else {
            Err(CreditError::BftEscrowGrantFailed)
        }
    }

    /// Simulate vote collection from committee members
    ///
    /// In a real implementation, this would:
    /// 1. Broadcast proposal to all committee members via P2P
    /// 2. Wait for votes with timeout
    /// 3. Verify signatures
    /// 4. Return valid votes
    async fn simulate_vote_collection(
        &self,
        account_id: &str,
        proposed_balance: i64,
    ) -> Result<Vec<BftVote>> {
        // For testing, simulate honest majority voting yes
        let mut votes = Vec::new();
        let honest_members = (self.members.len() + 1) * 2 / 3; // At least 2/3 honest

        for i in 0..honest_members {
            votes.push(BftVote {
                voter: self.members[i].clone(),
                proposed_balance,
                timestamp: chrono::Utc::now().timestamp() as u64,
                signature: vec![0; 64], // Simulated signature
            });
        }

        // Store votes
        self.votes
            .write()
            .await
            .insert(account_id.to_string(), votes.clone());

        Ok(votes)
    }

    /// Vote on escrow grant
    async fn vote_escrow_grant(
        &self,
        _account_id: &str,
        escrow: &DeviceEscrow,
    ) -> Result<Vec<BftVote>> {
        // For testing, simulate honest majority voting yes
        let mut votes = Vec::new();
        let honest_members = (self.members.len() + 1) * 2 / 3;

        for i in 0..honest_members {
            votes.push(BftVote {
                voter: self.members[i].clone(),
                proposed_balance: escrow.allocated,
                timestamp: chrono::Utc::now().timestamp() as u64,
                signature: vec![0; 64],
            });
        }

        Ok(votes)
    }

    /// Get votes for an account
    pub async fn get_votes(&self, account_id: &str) -> Vec<BftVote> {
        self.votes
            .read()
            .await
            .get(account_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Clear votes for an account
    pub async fn clear_votes(&self, account_id: &str) {
        self.votes.write().await.remove(account_id);
    }

    /// Create a mock committee for testing
    #[cfg(test)]
    pub async fn new_mock(size: usize) -> Result<Self> {
        let members: Vec<String> = (0..size).map(|i| format!("member{}", i)).collect();
        Self::new(members)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vudo_state::StateEngine;

    #[tokio::test]
    async fn test_bft_committee_new() {
        let members = vec![
            "member1".to_string(),
            "member2".to_string(),
            "member3".to_string(),
            "member4".to_string(),
        ];
        let committee = BftCommittee::new(members).unwrap();
        assert_eq!(committee.size(), 4);
        assert_eq!(committee.quorum(), 3); // 2f+1 where f=1
        assert_eq!(committee.max_byzantine_faults(), 1);
    }

    #[tokio::test]
    async fn test_bft_committee_insufficient_members() {
        let members = vec!["member1".to_string(), "member2".to_string()];
        let result = BftCommittee::new(members);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_bft_committee_quorum_calculation() {
        // 4 members: f=1, quorum=3
        let committee = BftCommittee::new_mock(4).await.unwrap();
        assert_eq!(committee.quorum(), 3);

        // 7 members: f=2, quorum=5
        let committee = BftCommittee::new_mock(7).await.unwrap();
        assert_eq!(committee.quorum(), 5);

        // 10 members: f=3, quorum=7
        let committee = BftCommittee::new_mock(10).await.unwrap();
        assert_eq!(committee.quorum(), 7);
    }

    #[tokio::test]
    async fn test_reconcile_balance() {
        let state_engine = StateEngine::new().await.unwrap();
        let account = CreditAccountHandle::create(&state_engine, "alice".to_string(), 10000)
            .await
            .unwrap();

        let committee = BftCommittee::new_mock(4).await.unwrap();
        let result = committee.reconcile_balance(&account).await.unwrap();

        assert!(result.consensus);
        assert_eq!(result.new_confirmed_balance, 10000);
        assert!(result.overdrafts.is_empty());
    }

    #[tokio::test]
    async fn test_grant_escrow() {
        let state_engine = StateEngine::new().await.unwrap();
        let account = CreditAccountHandle::create(&state_engine, "alice".to_string(), 10000)
            .await
            .unwrap();

        let committee = BftCommittee::new_mock(4).await.unwrap();
        let tier = crate::reputation::ReputationTier::new(1).unwrap();
        let escrow = committee
            .grant_escrow(&account, "device1", tier)
            .await
            .unwrap();

        assert_eq!(escrow.device_id, "device1");
        assert!(escrow.allocated > 0);
        assert!(escrow.allocated <= ReputationManager::escrow_limit(tier));
    }

    #[tokio::test]
    async fn test_grant_escrow_insufficient_balance() {
        let state_engine = StateEngine::new().await.unwrap();
        let account = CreditAccountHandle::create(&state_engine, "alice".to_string(), 10)
            .await
            .unwrap();

        let committee = BftCommittee::new_mock(4).await.unwrap();
        let tier = crate::reputation::ReputationTier::new(5).unwrap(); // High tier, large limit
        let result = committee.grant_escrow(&account, "device1", tier).await;

        // Should succeed with small allocation (balance = $0.10)
        assert!(result.is_ok());
        let escrow = result.unwrap();
        assert_eq!(escrow.allocated, 10); // Allocates all available balance
    }

    #[tokio::test]
    async fn test_grant_escrow_zero_balance() {
        let state_engine = StateEngine::new().await.unwrap();
        let account = CreditAccountHandle::create(&state_engine, "alice".to_string(), 0)
            .await
            .unwrap();

        let committee = BftCommittee::new_mock(4).await.unwrap();
        let tier = crate::reputation::ReputationTier::new(1).unwrap();
        let result = committee.grant_escrow(&account, "device1", tier).await;

        // Should fail with zero balance
        assert!(result.is_err());
    }
}
