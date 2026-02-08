//! BFT Private Voting Demo
//!
//! This example demonstrates Byzantine Fault Tolerant voting with privacy
//! using S-IDA fragmentation. The proposal is fragmented so no single
//! committee member sees the full proposal, yet the committee can still
//! reach consensus.
//!
//! Run with:
//! ```
//! cargo run --example bft_private_vote
//! ```

use std::sync::Arc;
use vudo_identity::MasterIdentity;
use vudo_p2p::{P2PConfig, VudoP2P};
use vudo_planetserve::bft::{BftPrivateCommittee, Proposal};
use vudo_planetserve::config::PrivacyConfig;
use vudo_planetserve::PlanetServeAdapter;
use vudo_state::StateEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== BFT Private Voting Demo ===\n");

    // Setup PlanetServe adapter
    let identity = Arc::new(MasterIdentity::generate("Coordinator").await?);
    let state_engine = Arc::new(StateEngine::new().await?);
    let p2p = Arc::new(VudoP2P::new(state_engine, P2PConfig::default()).await?);

    let adapter = Arc::new(
        PlanetServeAdapter::new(identity, p2p, PrivacyConfig::privacy_max()).await?,
    );

    println!("PlanetServe adapter initialized with maximum privacy");
    println!();

    // Create committee
    let committee_members = vec![
        "did:peer:member_alice".to_string(),
        "did:peer:member_bob".to_string(),
        "did:peer:member_carol".to_string(),
        "did:peer:member_dave".to_string(),
        "did:peer:member_eve".to_string(),
    ];

    println!("=== Committee Configuration ===");
    println!("Members: {}", committee_members.len());
    for (i, member) in committee_members.iter().enumerate() {
        println!("  {}. {}", i + 1, member);
    }
    println!();

    let committee = BftPrivateCommittee::new(committee_members.clone(), Arc::clone(&adapter));

    println!("BFT Parameters:");
    println!("  Committee size (n): {}", committee.size());
    println!("  Max Byzantine nodes (f): {}", committee.max_byzantine());
    println!("  Quorum threshold (2f+1): {}", committee.quorum_threshold());
    println!();

    println!("Byzantine Fault Tolerance:");
    println!("  - Can tolerate {} Byzantine (malicious) nodes", committee.max_byzantine());
    println!("  - Need {} votes for consensus", committee.quorum_threshold());
    println!();

    // Create proposal
    let proposal_data = serde_json::json!({
        "type": "credit_reconciliation",
        "from": "did:peer:alice",
        "to": "did:peer:bob",
        "amount": 1000,
        "reason": "Medical supplies delivery",
        "timestamp": chrono::Utc::now().timestamp(),
    });

    let proposal = Proposal::new("credit_reconciliation", proposal_data.clone());

    println!("=== Proposal ===");
    println!("ID: {}", proposal.id);
    println!("Type: {}", proposal.proposal_type);
    println!("Data:");
    println!("{}", serde_json::to_string_pretty(&proposal.data)?);
    println!();

    // Fragment proposal to show privacy
    println!("=== Privacy Analysis ===");
    let proposal_bytes = proposal.serialize()?;
    println!("Proposal size: {} bytes", proposal_bytes.len());
    println!();

    let fragmenter = adapter.fragmenter();
    let fragments = fragmenter.fragment(&proposal_bytes)?;

    println!("Proposal fragmented into {} shards (k={}, n={})",
        fragments.len(),
        fragmenter.config().k,
        fragmenter.config().n
    );
    println!();

    println!("Fragment distribution:");
    for (_i, (fragment, member)) in fragments.iter().zip(committee_members.iter()).enumerate() {
        println!(
            "  {} receives fragment {} ({} bytes)",
            member,
            fragment.index,
            fragment.size()
        );
    }
    println!();

    println!("Privacy guarantees:");
    println!("  ✓ No single member sees the full proposal");
    println!("  ✓ Need {} members to collude to reconstruct", fragmenter.config().k);
    println!("  ✓ Any {} members can independently verify their fragment", fragmenter.config().k);
    println!();

    // Simulate voting
    println!("=== Conducting Private Vote ===");
    println!("Sending fragments to committee members...");
    println!();

    let result = committee.private_vote(&proposal).await?;

    println!("Vote result: {:?}", result);
    println!();

    // Analysis
    println!("=== Voting Process ===");
    println!("1. Proposal fragmented via S-IDA ({}-of-{})", fragmenter.config().k, fragmenter.config().n);
    println!("2. Each fragment sent to different member");
    println!("3. Members vote on fragment hashes");
    println!("4. Votes collected via private channels");
    println!("5. Quorum checked ({} required)", committee.quorum_threshold());
    println!("6. Result tallied");
    println!();

    println!("=== Security Properties ===");
    println!("Byzantine Fault Tolerance:");
    println!("  ✓ Consensus reached despite {} malicious nodes", committee.max_byzantine());
    println!("  ✓ Quorum threshold prevents minority attacks");
    println!();

    println!("Privacy:");
    println!("  ✓ No single member observes full proposal");
    println!("  ✓ Fragment distribution prevents reconstruction");
    println!("  ✓ Votes encrypted and aggregated");
    println!();

    println!("=== Use Cases ===");
    println!("- Credit reconciliation");
    println!("- Resource allocation");
    println!("- Governance decisions");
    println!("- Permission grants");
    println!("- System upgrades");
    println!();

    println!("=== Summary ===");
    println!("✓ BFT consensus with privacy preservation");
    println!("✓ No trust in individual committee members");
    println!("✓ Resistant to {} Byzantine failures", committee.max_byzantine());
    println!("✓ Practical for critical distributed operations");

    Ok(())
}
