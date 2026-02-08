# DOL CRDT Examples

This directory contains comprehensive examples demonstrating all 7 CRDT strategies supported by DOL 2.0 CRDT annotations (RFC-001).

## Files

### 1. `chat-message.dol` - Collaborative Chat Message

**Demonstrates:**
- `@crdt(immutable)` - Immutable identity fields (id, author, created_at)
- `@crdt(lww)` - Last-write-wins for simple metadata (edited_at, pinned)
- `@crdt(peritext)` - Rich text collaborative editing (message content)
- `@crdt(or_set)` - Add-wins sets for reactions and attachments

**Use Case:** Real-time chat application where multiple users collaborate on message editing, add reactions, and attach files simultaneously.

**Key Concepts:**
- Immutability ensures message identity never changes
- LWW provides simple conflict resolution for metadata
- Peritext enables Google Docs-style collaborative text editing
- OR-Set preserves all additions (add-wins semantics)

---

### 2. `account-credit.dol` - Mutual Credit Account with Escrow

**Demonstrates:**
- `@crdt(pn_counter)` - Positive-Negative Counter for balances and metrics
- Escrow pattern for strong consistency under eventual consistency
- BFT reconciliation for critical operations

**Use Case:** Local-first mutual credit system where users can spend offline (up to escrow limit) and reconcile balances via BFT consensus when online.

**Key Concepts:**
- PN-Counter enables commutative numeric operations
- Escrow allocation prevents double-spend without coordination
- Three-tier architecture: strong consistency (BFT), local operations (escrow), eventual consistency (CRDT)
- Reputation system for dynamic escrow limits

**Architecture:**
```
┌─────────────────────────────────────────────────────────┐
│ Strong Consistency Layer (BFT)                          │
│ - confirmed_balance (BFT-confirmed)                     │
│ - Escrow allocation (BFT consensus)                     │
└─────────────────────────────────────────────────────────┘
           ↓ Periodic reconciliation
┌─────────────────────────────────────────────────────────┐
│ Local Operations Layer                                  │
│ - local_escrow (pre-allocated for offline spending)     │
│ - Immediate spend (< 1ms, no network)                   │
└─────────────────────────────────────────────────────────┘
           ↓ CRDT merge
┌─────────────────────────────────────────────────────────┐
│ Eventually Consistent Layer (CRDT)                      │
│ - pending_credits (PN-Counter CRDT)                     │
│ - transaction_history (OR-Set CRDT)                     │
└─────────────────────────────────────────────────────────┘
```

---

### 3. `task-board.dol` - Collaborative Kanban Board

**Demonstrates:**
- `@crdt(rga)` - Replicated Growable Array for ordered sequences
- Causal ordering for drag-and-drop operations
- Tombstone management for deletions

**Use Case:** Collaborative Kanban board where multiple users reorder tasks and columns concurrently, preserving intent through causal ordering.

**Key Concepts:**
- RGA maintains causal order: insertions reference "left origin"
- Concurrent insertions at same position ordered by timestamp
- Deletions use tombstones to preserve causal structure
- Topological sort ensures deterministic merge

**Example:**
```
Initial: [Task1, Task2, Task3]

Concurrent operations:
- User A: Insert TaskA between Task1 and Task2
- User B: Insert TaskB between Task1 and Task2

Both record left_origin = Task1.id

RGA merge (by timestamp):
  If A.timestamp < B.timestamp: [Task1, TaskA, TaskB, Task2, Task3]
  If B.timestamp < A.timestamp: [Task1, TaskB, TaskA, Task2, Task3]

Deterministic result across all replicas!
```

---

### 4. `config-app.dol` - Application Configuration

**Demonstrates:**
- `@crdt(mv_register)` - Multi-Value Register for conflict detection
- AI-assisted conflict resolution
- User-driven conflict resolution

**Use Case:** Application settings synchronized across multiple devices, with explicit conflict detection when users make incompatible changes concurrently.

**Key Concepts:**
- MV-Register keeps ALL concurrent values (doesn't lose data)
- Vector clocks track causality
- Application chooses resolution strategy:
  - User prompt
  - AI suggestion
  - Latest timestamp
  - Union/blend
- Formal conflict detection (not hidden like LWW)

**Resolution Strategies:**
1. **User Choose**: Prompt user to pick preferred value
2. **AI Suggest**: Machine learning predicts user intent
3. **Latest**: Use most recent wall-clock timestamp
4. **Union**: Blend or combine values (if meaningful)
5. **Default**: Revert to known good state

---

## CRDT Strategy Reference

| Strategy | Use Case | Conflict Resolution | Space Overhead |
|----------|----------|---------------------|----------------|
| **immutable** | IDs, creation timestamps | First-write-wins | O(1) |
| **lww** | Single-valued fields | Last timestamp wins | O(1) |
| **or_set** | Collections (tags, reactions) | Add-wins | O(n × k) tags |
| **pn_counter** | Numeric counters | Sum all operations | O(m) actors |
| **peritext** | Rich text documents | Causal character ordering | O(n log n) |
| **rga** | Ordered sequences | Causal insertion order | O(n) + tombstones |
| **mv_register** | Conflict detection | Keep all, resolve later | O(k) conflicts |

---

## Constraint Categories

DOL constraints interact with CRDT merges in three ways:

### Category A: CRDT-Safe (Compile-Time)
**Enforced by CRDT strategy itself.** No runtime validation needed.

Examples:
- Immutability: `field never changes` (enforced by `@crdt(immutable)`)
- Monotonicity: `counter always increases` (enforced by PN-Counter)
- Set membership: `set has element` (enforced by OR-Set)

### Category B: Eventually-Consistent (Merge-Time)
**May be temporarily violated during concurrent operations but converge to valid state.**

Examples:
- Uniqueness: `all users have unique email`
- Resource bounds: `used <= quota`
- Referential integrity: `post.author exists in users`

Handling:
- Soft validation during merge
- Flag violations for reconciliation
- Application resolves conflicts

### Category C: Strong-Consistency (Operation-Time)
**Require coordination (BFT consensus, locks, or escrow) to maintain.**

Examples:
- Double-spend prevention: `balance >= 0 always`
- Atomic transfers: `A.balance -= X AND B.balance += X`
- Access control: `user can_edit IFF user in collaborators`

Handling:
- Escrow allocation (local pre-authorization)
- BFT consensus for critical operations
- Reject invalid operations immediately

---

## Compilation Pipeline

```
DOL Source                      Typed AST
(with @crdt)           ──────►  with CRDT annotations
     │                               │
     │    dol-check                  │
     │  ◄─────────────────────────   │
     │  Validate type compat,        │
     │  constraint categories        │
     │                               │
     ▼                               ▼
dol-codegen-rust              Generated Rust Code
                              - Structs with #[derive(Reconcile, Hydrate)]
                              - CRDT operation methods
                              - Constraint validation
                              - Merge functions
                              │
                              ▼
                         cargo build --target wasm32-unknown-unknown
                              │
                              ▼
                         WASM Module
                         - < 200KB compressed per Gene
                         - wasm-bindgen JS bindings
                         - Component Model WIT interface
```

---

## Testing

All CRDT strategies verified via property-based testing:

1. **Convergence**: N replicas with random operations → same state
2. **Commutativity**: merge(A, B) = merge(B, A)
3. **Associativity**: merge(merge(A, B), C) = merge(A, merge(B, C))
4. **Idempotency**: merge(A, A) = A
5. **Constraint Preservation**: Constraints hold after merge (Category A, B)
6. **Partition Tolerance**: Network partition + heal → convergence

Target: 1,000,000+ random operation sequences for each strategy.

---

## Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| WASM module size | < 200KB compressed | wasm-opt -Oz |
| CRDT merge latency | < 10ms for 10K ops | criterion benchmarks |
| Local operation latency | < 1ms | wasm-bindgen perf trace |
| Sync throughput | > 1000 ops/sec | P2P benchmark harness |
| Convergence time | < 5 seconds after heal | Integration tests |

---

## References

- **RFC-001**: DOL 2.0 CRDT Annotations
  - `/rfcs/RFC-001-dol-crdt-annotations.md`

- **RFC-001 Formal Proofs**: Convergence guarantees and theorems
  - `/rfcs/RFC-001-formal-proofs.md`

- **CRDT Literature**:
  - Shapiro et al. (2011): "Conflict-free Replicated Data Types"
  - Kleppmann & Beresford (2017): "A Conflict-Free Replicated JSON Datatype" (Automerge)
  - Litt et al. (2022): "Peritext: A CRDT for Collaborative Rich Text Editing"
  - Kleppmann (2025): "eg-walker" (EuroSys Best Artifact)

- **Implementation**:
  - Automerge 3.0: https://github.com/automerge/automerge
  - Iroh (P2P): https://iroh.computer
  - Willow Protocol: https://willowprotocol.org

---

## Next Steps

1. **Phase 1** (Months 2-5): Implement parser, validator, and code generator
   - `dol-parse`: Recognize @crdt annotations
   - `dol-check`: Validate type compatibility and constraints
   - `dol-codegen-rust`: Generate Automerge-backed Rust code

2. **Phase 2** (Months 5-9): Integrate with VUDO Runtime
   - Local-first state engine
   - P2P sync via Iroh
   - Offline operation queue

3. **Phase 3** (Months 9-13): Identity, credit, and privacy
   - Peer DIDs + UCANs
   - Escrow-based mutual credit
   - PlanetServe privacy integration

4. **Phase 4** (Months 13-16): Production hardening
   - Performance optimization
   - Reference application
   - Comprehensive documentation

---

**Status:** Phase 0 (SPORE) - Foundation & Research
**Next Milestone:** RFC approval and Phase 1 implementation kickoff
**Project:** MYCELIUM-SYNC - Univrs Local-First Architecture
