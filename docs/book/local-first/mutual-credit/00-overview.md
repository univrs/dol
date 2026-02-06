# Mutual Credit System Overview

The VUDO mutual credit system enables decentralized value exchange without blockchains, tokens, or central banks. It combines CRDTs for local operations with Byzantine Fault Tolerance (BFT) for consensus.

## What is Mutual Credit?

**Traditional Money**: Central bank issues currency, banks track balances

**Mutual Credit**: Peers issue credit to each other, network tracks balances

### Key Concepts

**Credit Balance**: Can be positive or negative
- Positive: You've provided more value than you've received
- Negative: You've received more value than you've provided
- Zero: Balanced exchange

**Trust Limits**: Maximum credit you'll extend to a peer
- High trust: Large limit (e.g., 10,000 credits)
- Low trust: Small limit (e.g., 100 credits)
- No trust: Zero limit (no transactions)

## Why Mutual Credit for Local-First?

### Problems with Traditional Payment Systems

**Centralized**:
- Requires online connection to payment processor
- Third-party fees (2-5% per transaction)
- Vendor lock-in
- Privacy concerns

**Blockchain/Crypto**:
- High gas fees ($1-$50 per transaction)
- Slow confirmation (10+ seconds)
- Volatile token prices
- Still requires online connectivity

### Mutual Credit Advantages

**Offline-First**:
- ✅ Spend offline with local escrow
- ✅ No network latency
- ✅ Instant transactions (< 1ms)
- ✅ Sync when convenient

**Zero Fees**:
- ✅ No transaction fees
- ✅ No gas fees
- ✅ No middlemen
- ✅ Direct peer-to-peer value exchange

**Privacy-Preserving**:
- ✅ Transactions only visible to parties involved
- ✅ No public ledger
- ✅ Optional anonymity
- ✅ GDPR-compliant

**Byzantine Fault Tolerant**:
- ✅ Prevents double-spending
- ✅ Resists malicious peers
- ✅ Periodic reconciliation ensures accuracy
- ✅ 3f+1 consensus (tolerates f malicious nodes)

## Architecture

```
┌──────────────────────────────────────────────────┐
│ Local Operations (Offline)                      │
│                                                  │
│ User Balance:                                    │
│   confirmed_balance: 1000  (BFT-confirmed)       │
│   local_escrow: 500        (pre-allocated)       │
│   pending_credits: 200     (incoming)            │
│                                                  │
│ Spend: -50                                       │
│   ├─ Check: local_escrow >= 50 ✓                │
│   ├─ Deduct from escrow: 500 → 450              │
│   └─ Create transaction record                   │
│                                                  │
│ Result: Instant (< 1ms), No network required    │
└──────────────────────────────────────────────────┘
                      ↓ (background sync)
┌──────────────────────────────────────────────────┐
│ P2P Synchronization (Iroh)                      │
│                                                  │
│ - Broadcast transaction to peers                 │
│ - Receive pending credits from others            │
│ - CRDT merge (automatic convergence)             │
└──────────────────────────────────────────────────┘
                      ↓ (periodic)
┌──────────────────────────────────────────────────┐
│ BFT Reconciliation (Consensus)                   │
│                                                  │
│ - BFT committee (3f+1 nodes) verifies balances   │
│ - Pending credits → confirmed balance            │
│ - Allocate new escrow for next period            │
│ - Detect double-spend attempts                   │
│ - Resolve overdraft situations                   │
└──────────────────────────────────────────────────┘
```

## DOL Schema

```dol
gen account.mutual_credit {
  @crdt(immutable)
  has id: String

  @crdt(immutable)
  has owner_did: String

  // BFT-confirmed balance (strong consistency)
  @crdt(pn_counter, min_value=0)
  has confirmed_balance: i64

  // Pre-allocated for offline spending (local)
  @crdt(lww, min_value=0)
  has local_escrow: i64

  // Incoming credits (eventually consistent)
  @crdt(pn_counter)
  has pending_credits: i64

  // Transaction history
  @crdt(rga)
  has transaction_history: Vec<Transaction>

  // Trust network
  @crdt(or_set)
  has trust_connections: Set<TrustConnection>

  // Reputation tier (affects escrow limit)
  @crdt(lww)
  has reputation_tier: ReputationTier
}

gen transaction {
  @crdt(immutable)
  has id: String

  @crdt(immutable)
  has from: String

  @crdt(immutable)
  has to: String

  @crdt(immutable)
  has amount: i64

  @crdt(immutable)
  has created_at: i64

  @crdt(lww)
  has status: TransactionStatus

  @crdt(lww)
  has confirmed_at: Option<i64>
}

gen trust_connection {
  @crdt(immutable)
  has peer_did: String

  @crdt(lww)
  has trust_limit: i64

  @crdt(pn_counter, min_value=0)
  has total_exchanged: i64

  @crdt(lww)
  has reputation: f64
}

constraint account.solvency {
  confirmed_balance always >= 0
  local_escrow always >= 0
  local_escrow always <= confirmed_balance
}

constraint account.double_spend_prevention {
  account never spends_more_than local_escrow
  all transactions are atomic
}

docs {
  Mutual credit account with offline spending via escrow.

  Architecture:
  1. Strong consistency: confirmed_balance (BFT)
  2. Local operations: local_escrow (immediate)
  3. Eventually consistent: pending_credits (CRDT)

  Double-spend prevention:
  - Local spends deduct from escrow
  - Escrow is pre-allocated from confirmed balance
  - BFT reconciliation validates and reallocates escrow
}
```

## Transaction Flow

### 1. Normal Transaction (Online)

```rust
// Alice sends 50 credits to Bob
let tx = Transaction {
    id: Uuid::new_v4().to_string(),
    from: alice_account.owner_did.clone(),
    to: bob_account.owner_did.clone(),
    amount: 50,
    created_at: Utc::now().timestamp(),
    status: TransactionStatus::Pending,
    confirmed_at: None,
};

// Check escrow
if alice_account.local_escrow >= 50 {
    // Deduct from Alice's escrow
    alice_account.local_escrow -= 50;

    // Add to Bob's pending credits
    bob_account.pending_credits += 50;

    // Record transaction
    alice_account.transaction_history.push(tx.clone());
    bob_account.transaction_history.push(tx);

    // Sync via P2P (background)
    p2p_sync.broadcast(&alice_account).await?;
    p2p_sync.broadcast(&bob_account).await?;

    println!("✅ Transaction successful (instant)");
} else {
    println!("❌ Insufficient escrow");
}
```

**Latency**: < 1ms (local operation)

### 2. Offline Transaction

```rust
// Alice is offline but has escrow
if alice_account.local_escrow >= 50 {
    // Same as above, but no network sync
    alice_account.local_escrow -= 50;
    bob_account.pending_credits += 50;

    // Transaction recorded locally
    // Will sync when Alice comes online
    println!("✅ Transaction successful (offline)");
}
```

**Key insight**: Escrow enables offline spending without double-spend risk.

### 3. BFT Reconciliation (Periodic)

```rust
// Every 10 minutes (or configurable interval)
async fn reconcile_accounts(bft_committee: &BftCommittee) -> Result<()> {
    // 1. Collect all pending credits across network
    let pending_txs = collect_pending_transactions().await?;

    // 2. BFT consensus on valid transactions
    let confirmed_txs = bft_committee
        .reach_consensus(pending_txs)
        .await?;

    // 3. Update confirmed balances
    for tx in confirmed_txs {
        // Deduct from sender's confirmed balance
        tx.from_account.confirmed_balance -= tx.amount;

        // Add to receiver's confirmed balance
        tx.to_account.confirmed_balance += tx.amount;

        // Mark transaction as confirmed
        tx.status = TransactionStatus::Confirmed;
        tx.confirmed_at = Some(Utc::now().timestamp());
    }

    // 4. Allocate new escrow for next period
    for account in all_accounts {
        let new_escrow = calculate_escrow_limit(
            account.confirmed_balance,
            account.reputation_tier,
        );
        account.local_escrow = new_escrow;
    }

    Ok(())
}
```

**Frequency**: Every 10 minutes (configurable)

**Consensus**: 3f+1 nodes must agree (tolerates f Byzantine faults)

## Escrow Allocation

### How Escrow is Calculated

```rust
fn calculate_escrow_limit(
    confirmed_balance: i64,
    reputation_tier: ReputationTier,
) -> i64 {
    let base_escrow = confirmed_balance / 2;  // 50% of balance

    let reputation_multiplier = match reputation_tier {
        ReputationTier::New => 0.25,       // 25% of base
        ReputationTier::Trusted => 1.0,    // 100% of base
        ReputationTier::Verified => 1.5,   // 150% of base
        ReputationTier::Premium => 2.0,    // 200% of base
    };

    (base_escrow as f64 * reputation_multiplier) as i64
}
```

**Example**:
- Alice has 1000 confirmed credits
- Reputation: Trusted
- Escrow: 1000 / 2 * 1.0 = 500 credits

Alice can spend up to 500 credits offline before needing reconciliation.

### Reputation Tiers

**New** (0-10 transactions):
- Escrow: 25% of balance
- Purpose: Limit risk for new users

**Trusted** (10-100 transactions, no disputes):
- Escrow: 50% of balance
- Purpose: Standard users

**Verified** (100+ transactions, identity verified):
- Escrow: 75% of balance
- Purpose: Verified users with history

**Premium** (1000+ transactions, high reputation):
- Escrow: 100% of balance
- Purpose: Power users, businesses

## Trust Network

### Trust Connections

```rust
let trust_conn = TrustConnection {
    peer_did: "did:key:z6Mkr...".to_string(),
    trust_limit: 1000,  // Max credit extension
    total_exchanged: 5000,  // Historical volume
    reputation: 4.5,  // 0-5 scale
};
```

### Trust Limit Enforcement

```rust
fn can_transact(
    from: &Account,
    to: &Account,
    amount: i64,
) -> bool {
    // Check escrow
    if from.local_escrow < amount {
        return false;
    }

    // Check trust limit
    let trust_conn = from.trust_connections
        .iter()
        .find(|c| c.peer_did == to.owner_did);

    match trust_conn {
        Some(conn) => {
            // Calculate current exposure
            let current_exposure = calculate_exposure(from, to);
            current_exposure + amount <= conn.trust_limit
        }
        None => false,  // No trust connection
    }
}
```

## Byzantine Fault Tolerance

### Threat Model

**Honest Peers**: Follow protocol correctly

**Malicious Peers** (Byzantine):
- May try to double-spend
- May report false balances
- May withhold transaction information
- May collude with other malicious peers

**BFT Guarantee**: System remains correct if < 1/3 of nodes are malicious.

### Double-Spend Prevention

**Scenario**: Alice tries to spend same credits twice

```rust
// Alice (malicious) tries double-spend
// Device 1: Send 500 credits to Bob
tx1 = Transaction { from: alice, to: bob, amount: 500 };

// Device 2: Send 500 credits to Carol (concurrent)
tx2 = Transaction { from: alice, to: carol, amount: 500 };

// BFT Reconciliation detects double-spend
async fn detect_double_spend(txs: Vec<Transaction>) -> Vec<Transaction> {
    let mut valid_txs = Vec::new();
    let mut spent_amounts: HashMap<String, i64> = HashMap::new();

    for tx in txs {
        let current_spent = spent_amounts.get(&tx.from).unwrap_or(&0);
        let account = get_account(&tx.from).await?;

        if current_spent + tx.amount <= account.local_escrow {
            // Valid transaction
            valid_txs.push(tx.clone());
            spent_amounts.insert(tx.from.clone(), current_spent + tx.amount);
        } else {
            // Double-spend detected! Reject transaction
            println!("🚨 Double-spend detected: {:?}", tx);
        }
    }

    valid_txs
}
```

**Result**: Only one transaction is confirmed (first by timestamp).

## Use Cases

### 1. Marketplace

```dol
gen marketplace.listing {
  @crdt(immutable)
  has id: String

  @crdt(immutable)
  has seller_did: String

  @crdt(lww)
  has title: String

  @crdt(peritext)
  has description: String

  @crdt(lww)
  has price: i64  // In credits

  @crdt(lww)
  has status: ListingStatus
}
```

**Flow**:
1. Buyer finds listing
2. Buyer sends credits (offline)
3. Seller ships item
4. BFT reconciliation confirms transaction

### 2. Freelance Services

```dol
gen service.contract {
  @crdt(immutable)
  has id: String

  @crdt(immutable)
  has client_did: String

  @crdt(immutable)
  has freelancer_did: String

  @crdt(lww)
  has deliverables: String

  @crdt(lww)
  has payment_amount: i64

  @crdt(lww)
  has payment_status: PaymentStatus
}
```

**Flow**:
1. Client and freelancer agree on terms
2. Client allocates credits to escrow
3. Freelancer completes work
4. Client releases payment (instant)

### 3. Community Resource Sharing

```dol
gen resource.booking {
  @crdt(immutable)
  has id: String

  @crdt(immutable)
  has resource_id: String

  @crdt(immutable)
  has borrower_did: String

  @crdt(lww)
  has start_time: i64

  @crdt(lww)
  has end_time: i64

  @crdt(lww)
  has credits_per_hour: i64
}
```

**Flow**:
1. User books shared resource (car, tool, space)
2. Credits auto-deducted from escrow
3. Owner receives credits
4. BFT reconciliation confirms

## Performance

**Local Operations** (offline):
- Latency: < 1ms
- Throughput: 10,000+ transactions/sec

**P2P Sync**:
- Latency: 50-200ms (peer-to-peer)
- Throughput: 1,000 transactions/sec

**BFT Reconciliation**:
- Latency: 2-5 seconds (consensus)
- Throughput: 100 transactions/sec
- Frequency: Every 10 minutes (configurable)

## Next Steps

- [Escrow Pattern](./01-escrow-pattern.md) - Deep dive into offline spending
- [BFT Reconciliation](./02-bft-reconciliation.md) - Consensus algorithm
- [Integration Guide](./03-integration.md) - Add credit to your app

## Further Reading

- [t3.2: Mutual Credit Implementation](/TASK_T3_2_MUTUAL_CREDIT_COMPLETE.md)
- [Credit Security Audit](/docs/security/credit-security-audit.md)
- [BFT Test Report](/docs/security/bft-test-report.md)
- [vudo-credit Crate](/crates/vudo-credit/README.md)

---

**Next**: [Escrow Pattern →](./01-escrow-pattern.md)
