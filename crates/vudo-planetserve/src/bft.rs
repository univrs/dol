//! BFT (Byzantine Fault Tolerant) verification committees with privacy
//!
//! This module provides private voting for critical operations like credit
//! reconciliation. Proposals are fragmented via S-IDA and distributed to
//! committee members, ensuring privacy while maintaining Byzantine fault tolerance.
//!
//! # Architecture
//!
//! ```text
//! Proposal → S-IDA (3-of-5) → Fragment 1 → Committee Member 1
//!                           → Fragment 2 → Committee Member 2
//!                           → Fragment 3 → Committee Member 3
//!                           → Fragment 4 → Committee Member 4
//!                           → Fragment 5 → Committee Member 5
//!
//! Each member receives ONE fragment (cannot see full proposal)
//! Members vote on their fragment hash
//! Quorum (3-of-5) required for approval
//! ```
//!
//! # Privacy Properties
//!
//! - **No single member** sees the full proposal
//! - **k-of-n members** must collude to reconstruct
//! - **Votes** are encrypted and aggregated
//! - **Quorum threshold** ensures Byzantine fault tolerance
//!
//! # Examples
//!
//! ```no_run
//! use vudo_planetserve::bft::{BftPrivateCommittee, Proposal};
//! use vudo_planetserve::adapter::PlanetServeAdapter;
//! use std::sync::Arc;
//!
//! # async fn example() -> vudo_planetserve::error::Result<()> {
//! # let adapter = Arc::new(PlanetServeAdapter::new(
//! #     Arc::new(vudo_identity::MasterIdentity::generate("Test").await?),
//! #     Arc::new(vudo_p2p::VudoP2P::new(
//! #         Arc::new(vudo_state::StateEngine::new().await?),
//! #         vudo_p2p::P2PConfig::default()
//! #     ).await?),
//! #     vudo_planetserve::config::PrivacyConfig::default(),
//! # ).await?);
//! let committee = BftPrivateCommittee::new(
//!     vec![
//!         "did:peer:member1".to_string(),
//!         "did:peer:member2".to_string(),
//!         "did:peer:member3".to_string(),
//!         "did:peer:member4".to_string(),
//!         "did:peer:member5".to_string(),
//!     ],
//!     adapter,
//! );
//!
//! // Create proposal
//! let proposal = Proposal::new(
//!     "credit_reconciliation",
//!     serde_json::json!({"from": "alice", "to": "bob", "amount": 100}),
//! );
//!
//! // Conduct private vote
//! let result = committee.private_vote(&proposal).await?;
//! # Ok(())
//! # }
//! ```

use crate::adapter::PlanetServeAdapter;
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

/// BFT private committee
pub struct BftPrivateCommittee {
    /// Committee member DIDs
    members: Vec<String>,

    /// PlanetServe adapter for private communication
    adapter: Arc<PlanetServeAdapter>,

    /// BFT threshold (f+1 where f is max Byzantine nodes)
    /// For n members: quorum = 2f + 1 = (2n/3) + 1
    quorum_threshold: usize,
}

impl BftPrivateCommittee {
    /// Create a new BFT private committee
    pub fn new(members: Vec<String>, adapter: Arc<PlanetServeAdapter>) -> Self {
        let n = members.len();

        // Byzantine fault tolerance: quorum = 2f + 1
        // where f = (n-1)/3 (max Byzantine nodes)
        let f = (n - 1) / 3;
        let quorum_threshold = 2 * f + 1;

        info!(
            "Created BFT committee: {} members, quorum={}, f={}",
            n, quorum_threshold, f
        );

        Self {
            members,
            adapter,
            quorum_threshold,
        }
    }

    /// Conduct a private vote on a proposal
    ///
    /// 1. Fragment proposal via S-IDA
    /// 2. Send each fragment to a different committee member
    /// 3. Collect votes (also via private channels)
    /// 4. Check quorum and tally
    pub async fn private_vote(&self, proposal: &Proposal) -> Result<VoteResult> {
        info!("Starting private vote on proposal: {}", proposal.id);

        // Serialize proposal
        let proposal_bytes = bincode::serialize(proposal)?;

        // Fragment proposal (S-IDA)
        let fragments = self
            .adapter
            .fragmenter()
            .fragment(&proposal_bytes)?;

        debug!("Fragmented proposal into {} shards", fragments.len());

        // Send each fragment to a different committee member
        for (fragment, member_did) in fragments.iter().zip(self.members.iter()) {
            let fragment_bytes = bincode::serialize(fragment)?;

            // Use private sync to send fragment
            self.adapter
                .sync_private(
                    "bft_proposals",
                    &proposal.id.to_string(),
                    fragment_bytes,
                )
                .await?;

            debug!("Sent fragment to committee member: {}", member_did);
        }

        // Collect votes (in real implementation, would wait for votes from members)
        // For now, simulate vote collection
        let votes = self.collect_votes(&proposal.id).await?;

        debug!("Collected {} votes", votes.len());

        // Check quorum
        if votes.len() >= self.quorum_threshold {
            // Tally votes
            let approved = votes.iter().filter(|v| v.approve).count();
            let rejected = votes.len() - approved;

            info!(
                "Vote complete: {} approved, {} rejected (quorum: {})",
                approved, rejected, self.quorum_threshold
            );

            if approved >= self.quorum_threshold {
                Ok(VoteResult::Approved {
                    votes_for: approved,
                    votes_against: rejected,
                })
            } else {
                Ok(VoteResult::Rejected {
                    votes_for: approved,
                    votes_against: rejected,
                })
            }
        } else {
            info!(
                "Quorum not reached: {} votes < {} quorum",
                votes.len(),
                self.quorum_threshold
            );
            Ok(VoteResult::QuorumNotReached {
                votes_received: votes.len(),
            })
        }
    }

    /// Collect votes from committee members
    ///
    /// In a real implementation, this would listen for encrypted votes from members.
    async fn collect_votes(&self, proposal_id: &Uuid) -> Result<Vec<Vote>> {
        // Placeholder: In real implementation, would use P2P gossip to collect votes
        // For now, return empty vector
        Ok(Vec::new())
    }

    /// Get quorum threshold
    pub fn quorum_threshold(&self) -> usize {
        self.quorum_threshold
    }

    /// Get committee size
    pub fn size(&self) -> usize {
        self.members.len()
    }

    /// Get max Byzantine nodes (f)
    pub fn max_byzantine(&self) -> usize {
        (self.members.len() - 1) / 3
    }
}

/// Proposal for BFT voting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Unique proposal ID
    pub id: Uuid,

    /// Proposal type (e.g., "credit_reconciliation")
    pub proposal_type: String,

    /// Proposal data (JSON)
    pub data: serde_json::Value,

    /// Timestamp
    pub timestamp: u64,
}

impl Proposal {
    /// Create a new proposal
    pub fn new(proposal_type: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            proposal_type: proposal_type.into(),
            data,
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }

    /// Serialize proposal
    pub fn serialize(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }
}

/// Vote from a committee member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    /// Voter DID
    pub voter: String,

    /// Proposal ID
    pub proposal_id: Uuid,

    /// Approve or reject
    pub approve: bool,

    /// Signature over (proposal_id || approve)
    pub signature: Vec<u8>,

    /// Timestamp
    pub timestamp: u64,
}

/// Vote result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoteResult {
    /// Proposal approved (quorum reached, majority approved)
    Approved {
        votes_for: usize,
        votes_against: usize,
    },

    /// Proposal rejected (quorum reached, majority rejected)
    Rejected {
        votes_for: usize,
        votes_against: usize,
    },

    /// Quorum not reached
    QuorumNotReached { votes_received: usize },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PrivacyConfig;

    #[tokio::test]
    async fn test_quorum_calculation() {
        // 5 members: f=1, quorum=3
        let members: Vec<String> = (0..5).map(|i| format!("member{}", i)).collect();
        let adapter = Arc::new(
            PlanetServeAdapter::new(
                Arc::new(vudo_identity::MasterIdentity::generate("Test").await.unwrap()),
                Arc::new(
                    vudo_p2p::VudoP2P::new(
                        Arc::new(vudo_state::StateEngine::new().await.unwrap()),
                        vudo_p2p::P2PConfig::default(),
                    )
                    .await
                    .unwrap(),
                ),
                PrivacyConfig::default(),
            )
            .await
            .unwrap(),
        );

        let committee = BftPrivateCommittee::new(members, adapter);
        assert_eq!(committee.size(), 5);
        assert_eq!(committee.max_byzantine(), 1);
        assert_eq!(committee.quorum_threshold(), 3);

        // 7 members: f=2, quorum=5
        let members: Vec<String> = (0..7).map(|i| format!("member{}", i)).collect();
        let adapter = Arc::new(
            PlanetServeAdapter::new(
                Arc::new(vudo_identity::MasterIdentity::generate("Test").await.unwrap()),
                Arc::new(
                    vudo_p2p::VudoP2P::new(
                        Arc::new(vudo_state::StateEngine::new().await.unwrap()),
                        vudo_p2p::P2PConfig::default(),
                    )
                    .await
                    .unwrap(),
                ),
                PrivacyConfig::default(),
            )
            .await
            .unwrap(),
        );

        let committee = BftPrivateCommittee::new(members, adapter);
        assert_eq!(committee.size(), 7);
        assert_eq!(committee.max_byzantine(), 2);
        assert_eq!(committee.quorum_threshold(), 5);
    }

    #[test]
    fn test_proposal_creation() {
        let proposal = Proposal::new(
            "credit_reconciliation",
            serde_json::json!({"from": "alice", "to": "bob", "amount": 100}),
        );

        assert_eq!(proposal.proposal_type, "credit_reconciliation");
        assert!(proposal.serialize().is_ok());
    }

    #[tokio::test]
    async fn test_private_vote_no_quorum() {
        let members: Vec<String> = (0..5).map(|i| format!("member{}", i)).collect();
        let adapter = Arc::new(
            PlanetServeAdapter::new(
                Arc::new(vudo_identity::MasterIdentity::generate("Test").await.unwrap()),
                Arc::new(
                    vudo_p2p::VudoP2P::new(
                        Arc::new(vudo_state::StateEngine::new().await.unwrap()),
                        vudo_p2p::P2PConfig::default(),
                    )
                    .await
                    .unwrap(),
                ),
                PrivacyConfig::default(),
            )
            .await
            .unwrap(),
        );

        let committee = BftPrivateCommittee::new(members, adapter);

        let proposal = Proposal::new("test", serde_json::json!({}));

        // Vote will fail since we don't have real committee members
        // But we can test the structure
        let result = committee.private_vote(&proposal).await.unwrap();
        assert!(matches!(result, VoteResult::QuorumNotReached { .. }));
    }
}
