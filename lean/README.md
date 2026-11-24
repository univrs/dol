# dol-lean

Lean 4 models and proofs for DOL. See `docs/design/DOL-LEAN4-ALIGNMENT.md`.

This is the foundation for Phases L1–L2: hand-written Lean targets that the
future `dol-codegen -t lean` backend must reproduce, plus the verification gate.

## Setup

Install [elan](https://github.com/leanprover/elan). The toolchain is pinned in
`lean-toolchain` (Lean v4.36.0). No Mathlib dependency.

## Commands

```bash
cd lean
lake build                      # type-check everything
./scripts/verify.sh             # build + gate: reject sorry and non-standard axioms
./scripts/verify.sh --self-test # prove the gate rejects sorry and custom axioms
```

The gate (`scripts/CheckAxioms.lean`) walks every theorem in modules under
`DolLean` and allows only `propext`, `Classical.choice` and `Quot.sound`.

## Contents

| Module | What it is |
|---|---|
| `DolLean.Crdt.Lww` | Last-writer-wins register (`@crdt(lww)`): merge is idempotent, associative, and commutative for coherent writes. `not_comm_without_coherence` proves that commutativity fails if two distinct writes share a (timestamp, replica) pair. |
| `DolLean.Examples.Ledger` | The ledger example from the design doc (§5, Phase L1), kept type-checked. |
| `GateFixtures.*` | Deliberately bad modules for the gate self-test. Not built by default. |
