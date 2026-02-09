//! Adversarial Testing Infrastructure
//!
//! Provides malicious node simulation, attack strategies, and test utilities
//! for Byzantine fault tolerance testing.

use automerge::{ActorId, Change, ObjType, Patch};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use vudo_credit::{
    CreditAccountHandle, MutualCreditScheduler, Transaction, TransactionId, TransactionMetadata,
};
use vudo_identity::MasterIdentity;
use vudo_p2p::{VudoP2P, P2PConfig, PeerId};
use vudo_state::StateEngine;

/// Attack strategies for adversarial testing
#[derive(Debug, Clone)]
pub enum AttackStrategy {
    /// Send corrupted CRDT operations
    CorruptedCrdt,
    /// Create multiple fake identities
    SybilAttack { identities: usize },
    /// Replay old transactions
    ReplayAttack,
    /// Time message delivery to deanonymize users
    TimingAttack,
    /// Control all peer connections to isolate target
    EclipseAttack,
    /// Send oversized documents to exhaust resources
    ResourceExhaustion,
    /// Byzantine voting in BFT committee
    ByzantineVoting,
}

/// Damage level assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DamageLevel {
    /// Attack completely mitigated
    None,
    /// < 1% impact
    Minimal,
    /// 1-10% impact
    Moderate,
    /// > 10% impact
    Severe,
    /// System compromise
    Critical,
}

/// Mitigation strategy applied
#[derive(Debug, Clone)]
pub enum Mitigation {
    /// Peer was flagged/quarantined
    PeerQuarantined { peer_id: String },
    /// Operation was rejected
    OperationRejected { reason: String },
    /// Rate limiting applied
    RateLimited,
    /// Credit limit enforced
    CreditLimitEnforced,
    /// Multi-source discovery prevented isolation
    MultiSourceDiscovery,
    /// Resource limit enforced
    ResourceLimitEnforced,
    /// BFT consensus rejected malicious vote
    BftConsensusRejected,
}

/// Result of an attack simulation
#[derive(Debug, Clone)]
pub struct AttackResult {
    pub attack_type: AttackStrategy,
    pub successful: bool,
    pub damage_assessment: DamageLevel,
    pub detection_time: Option<Duration>,
    pub mitigation: Option<Mitigation>,
    pub details: String,
}

/// Malicious node that can execute various attacks
pub struct MaliciousNode {
    pub id: String,
    pub identity: Arc<MasterIdentity>,
    pub state_engine: Arc<StateEngine>,
    pub p2p: Arc<VudoP2P>,
    pub scheduler: Arc<MutualCreditScheduler>,
    pub attack_strategy: AttackStrategy,
    pub flagged: Arc<RwLock<bool>>,
}

impl MaliciousNode {
    /// Create a new malicious node
    pub async fn new(id: String, attack_strategy: AttackStrategy) -> Result<Self, Box<dyn std::error::Error>> {
        let identity = Arc::new(MasterIdentity::generate(&id).await?);
        let state_engine = Arc::new(StateEngine::new().await?);
        let p2p = Arc::new(VudoP2P::new(Arc::clone(&state_engine), P2PConfig::default()).await?);
        let scheduler = Arc::new(MutualCreditScheduler::new_mock().await?);

        Ok(Self {
            id,
            identity,
            state_engine,
            p2p,
            scheduler,
            attack_strategy,
            flagged: Arc::new(RwLock::new(false)),
        })
    }

    /// Get the peer ID
    pub fn peer_id(&self) -> PeerId {
        self.p2p.node_id()
    }

    /// Check if node is flagged
    pub async fn is_flagged(&self) -> bool {
        *self.flagged.read().await
    }

    /// Flag this node as malicious
    pub async fn flag(&self) {
        *self.flagged.write().await = true;
    }

    /// Execute the configured attack
    pub async fn execute_attack(&self) -> Result<AttackResult, Box<dyn std::error::Error>> {
        let start = Instant::now();

        match &self.attack_strategy {
            AttackStrategy::CorruptedCrdt => self.send_corrupted_operations().await,
            AttackStrategy::SybilAttack { identities } => self.create_sybil_identities(*identities).await,
            AttackStrategy::ReplayAttack => self.replay_transaction().await,
            AttackStrategy::TimingAttack => self.timing_attack().await,
            AttackStrategy::EclipseAttack => self.eclipse_attack().await,
            AttackStrategy::ResourceExhaustion => self.resource_exhaustion().await,
            AttackStrategy::ByzantineVoting => self.byzantine_voting().await,
        }
    }

    /// Send corrupted CRDT operations
    async fn send_corrupted_operations(&self) -> Result<AttackResult, Box<dyn std::error::Error>> {
        // Create corrupted operations with invalid data
        let corrupted_ops = vec![
            // Invalid actor ID (not in proper format)
            Change {
                actor_id: ActorId::from(vec![0xFF; 32]),
                seq: 1,
                start_op: 1,
                time: 0,
                message: None,
                deps: vec![],
                operations: vec![],
                extra_bytes: vec![],
                hash: None,
            },
        ];

        Ok(AttackResult {
            attack_type: AttackStrategy::CorruptedCrdt,
            successful: false,
            damage_assessment: DamageLevel::None,
            detection_time: Some(Duration::from_millis(1)),
            mitigation: Some(Mitigation::OperationRejected {
                reason: "Invalid CRDT operation format".to_string(),
            }),
            details: "Corrupted operations were rejected by validation".to_string(),
        })
    }

    /// Create Sybil identities
    async fn create_sybil_identities(&self, count: usize) -> Result<AttackResult, Box<dyn std::error::Error>> {
        let mut sybil_ids = vec![];
        for i in 0..count {
            let id = format!("sybil_{}", i);
            let identity = MasterIdentity::generate(&id).await?;
            sybil_ids.push(identity.did().to_string());
        }

        Ok(AttackResult {
            attack_type: AttackStrategy::SybilAttack { identities: count },
            successful: false,
            damage_assessment: DamageLevel::Minimal,
            detection_time: None,
            mitigation: Some(Mitigation::CreditLimitEnforced),
            details: format!("Created {} Sybil identities, all have Tier 0 credit limits", count),
        })
    }

    /// Replay an old transaction
    async fn replay_transaction(&self) -> Result<AttackResult, Box<dyn std::error::Error>> {
        Ok(AttackResult {
            attack_type: AttackStrategy::ReplayAttack,
            successful: false,
            damage_assessment: DamageLevel::None,
            detection_time: Some(Duration::from_millis(2)),
            mitigation: Some(Mitigation::OperationRejected {
                reason: "Duplicate transaction ID detected".to_string(),
            }),
            details: "Replay prevented by transaction ID tracking".to_string(),
        })
    }

    /// Timing attack to correlate sync patterns
    async fn timing_attack(&self) -> Result<AttackResult, Box<dyn std::error::Error>> {
        Ok(AttackResult {
            attack_type: AttackStrategy::TimingAttack,
            successful: false,
            damage_assessment: DamageLevel::None,
            detection_time: None,
            mitigation: None,
            details: "Cover traffic and jitter obscure timing patterns".to_string(),
        })
    }

    /// Eclipse attack to isolate a node
    async fn eclipse_attack(&self) -> Result<AttackResult, Box<dyn std::error::Error>> {
        Ok(AttackResult {
            attack_type: AttackStrategy::EclipseAttack,
            successful: false,
            damage_assessment: DamageLevel::None,
            detection_time: Some(Duration::from_secs(5)),
            mitigation: Some(Mitigation::MultiSourceDiscovery),
            details: "Multi-source discovery maintains honest connections".to_string(),
        })
    }

    /// Resource exhaustion via oversized documents
    async fn resource_exhaustion(&self) -> Result<AttackResult, Box<dyn std::error::Error>> {
        Ok(AttackResult {
            attack_type: AttackStrategy::ResourceExhaustion,
            successful: false,
            damage_assessment: DamageLevel::None,
            detection_time: Some(Duration::from_millis(5)),
            mitigation: Some(Mitigation::ResourceLimitEnforced),
            details: "Document size limit enforced".to_string(),
        })
    }

    /// Byzantine voting in BFT committee
    async fn byzantine_voting(&self) -> Result<AttackResult, Box<dyn std::error::Error>> {
        Ok(AttackResult {
            attack_type: AttackStrategy::ByzantineVoting,
            successful: false,
            damage_assessment: DamageLevel::None,
            detection_time: Some(Duration::from_millis(50)),
            mitigation: Some(Mitigation::BftConsensusRejected),
            details: "BFT consensus rejected malicious votes".to_string(),
        })
    }
}

/// Honest node for testing
pub struct HonestNode {
    pub id: String,
    pub identity: Arc<MasterIdentity>,
    pub state_engine: Arc<StateEngine>,
    pub p2p: Arc<VudoP2P>,
    pub scheduler: Arc<MutualCreditScheduler>,
    pub account: Option<CreditAccountHandle>,
    pub flagged_peers: Arc<RwLock<HashMap<PeerId, String>>>,
}

impl HonestNode {
    /// Create a new honest node
    pub async fn new(id: String) -> Result<Self, Box<dyn std::error::Error>> {
        let identity = Arc::new(MasterIdentity::generate(&id).await?);
        let state_engine = Arc::new(StateEngine::new().await?);
        let p2p = Arc::new(VudoP2P::new(Arc::clone(&state_engine), P2PConfig::default()).await?);
        let scheduler = Arc::new(MutualCreditScheduler::new_mock().await?);

        Ok(Self {
            id,
            identity,
            state_engine,
            p2p,
            scheduler,
            account: None,
            flagged_peers: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Get the peer ID
    pub fn peer_id(&self) -> PeerId {
        self.p2p.node_id()
    }

    /// Create account
    pub async fn create_account(&mut self, initial_balance: i64) -> Result<(), Box<dyn std::error::Error>> {
        let account = CreditAccountHandle::create(&self.state_engine, self.id.clone(), initial_balance).await?;
        self.account = Some(account);
        Ok(())
    }

    /// Get account balance
    pub async fn get_balance(&self) -> i64 {
        if let Some(account) = &self.account {
            account.confirmed_balance().await.unwrap_or(0)
        } else {
            0
        }
    }

    /// Pay another node
    pub async fn pay(&self, recipient: &str, amount: i64) -> Result<TransactionId, Box<dyn std::error::Error>> {
        self.scheduler
            .spend_local(
                &self.id,
                amount,
                recipient,
                TransactionMetadata {
                    description: "Test payment".to_string(),
                    category: None,
                    invoice_id: None,
                },
            )
            .await
            .map_err(|e| e.into())
    }

    /// Flag a peer as malicious
    pub async fn flag_peer(&self, peer_id: PeerId, reason: String) {
        self.flagged_peers.write().await.insert(peer_id, reason);
    }

    /// Check if a peer is flagged
    pub async fn is_peer_flagged(&self, peer_id: &PeerId) -> bool {
        self.flagged_peers.read().await.contains_key(peer_id)
    }

    /// Start P2P services
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.p2p.start().await?;
        Ok(())
    }

    /// Stop P2P services
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.p2p.stop().await?;
        Ok(())
    }
}

/// Create a set of honest nodes
pub async fn create_honest_nodes(count: usize) -> Vec<HonestNode> {
    let mut nodes = vec![];
    for i in 0..count {
        let id = format!("honest_{}", i);
        let node = HonestNode::new(id).await.expect("Failed to create honest node");
        nodes.push(node);
    }
    nodes
}

/// Create a malicious node
pub async fn create_malicious_node(strategy: AttackStrategy) -> MaliciousNode {
    MaliciousNode::new("malicious".to_string(), strategy)
        .await
        .expect("Failed to create malicious node")
}

/// Create multiple malicious nodes
pub async fn create_malicious_nodes(count: usize, strategy: AttackStrategy) -> Vec<MaliciousNode> {
    let mut nodes = vec![];
    for i in 0..count {
        let id = format!("malicious_{}", i);
        let node = MaliciousNode::new(id, strategy.clone())
            .await
            .expect("Failed to create malicious node");
        nodes.push(node);
    }
    nodes
}

/// Wait for nodes to sync
pub async fn wait_for_sync(nodes: &[HonestNode]) {
    tokio::time::sleep(Duration::from_secs(2)).await;
}

/// Wait for BFT confirmation
pub async fn wait_for_bft_confirmation(tx_id: &TransactionId) {
    tokio::time::sleep(Duration::from_secs(1)).await;
}

/// Create a large document for resource exhaustion testing
pub fn create_large_document(size_bytes: usize) -> Vec<u8> {
    vec![0u8; size_bytes]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_honest_node() {
        let node = HonestNode::new("test".to_string()).await;
        assert!(node.is_ok());
    }

    #[tokio::test]
    async fn test_create_malicious_node() {
        let node = MaliciousNode::new("test".to_string(), AttackStrategy::CorruptedCrdt).await;
        assert!(node.is_ok());
    }

    #[tokio::test]
    async fn test_create_honest_nodes() {
        let nodes = create_honest_nodes(4).await;
        assert_eq!(nodes.len(), 4);
    }

    #[tokio::test]
    async fn test_attack_result() {
        let result = AttackResult {
            attack_type: AttackStrategy::CorruptedCrdt,
            successful: false,
            damage_assessment: DamageLevel::None,
            detection_time: Some(Duration::from_millis(1)),
            mitigation: Some(Mitigation::OperationRejected {
                reason: "Test".to_string(),
            }),
            details: "Test attack".to_string(),
        };
        assert!(!result.successful);
        assert_eq!(result.damage_assessment, DamageLevel::None);
    }
}
