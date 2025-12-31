# DOL Feature: Constraints

**Generated:** $(date)
**Status:** PARSING & VALIDATION WORKING | WASM NOT YET SUPPORTED

---

## Overview

Constraints in DOL define invariants and rules that must always hold. They express business logic, safety requirements, and behavioral guarantees.

### Constraint Syntax

```dol
constraint <name> {
    requires <condition>
    ensures <condition>
}
```

---

## Building DOL

```bash
cargo build --features cli 2>&1 | tail -3
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
```

## Constraint Examples

### Example 1: Identity Immutable Constraint

Ensures identity properties cannot change:

```dol
constraint identity.immutable {
  keypair never changes
  public_key never changes
  signature matches content
}

exegesis {
  Cryptographic identity must be immutable because any mutation invalidates the
  entire trust chain built upon it. When a keypair changes, all signatures
  created with the previous private key become orphaned, and all verification
  relationships established with the public key break. This is not merely a
  technical inconvenience but a fundamental violation of cryptographic assumptions
  that underpin distributed systems. The immutability of the public key ensures
  that identity verification remains consistent across time and space, allowing
  remote parties to trust that the entity they authenticated yesterday is the
  same entity they interact with today. Requiring signatures to match content
  completes the integrity guarantee, ensuring that identity cannot be separated
  from the authentic work product of that identity. These constraints make
  identity a bedrock upon which other system properties can be safely built.
}
```

**Parse Result:**
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/dol-parse examples/constraints/identity.immutable.dol`
✓ examples/constraints/identity.immutable.dol (identity.immutable)
    identity.immutable constraint with 3 rules

Summary
  Total:    1
  Success:  1
```

### Example 2: Container Integrity Constraint

Ensures container state consistency:

```dol
constraint container.integrity {
  state matches declared
  identity never changes
  boundaries never expand
  resources never exceeds
}

exegesis {
  Container integrity constraints ensure that runtime behavior exactly matches
  the ontological declarations made at creation time. This prevents the common
  failure mode where containers drift from their intended specifications through
  mutation, resource creep, or boundary violations. By requiring state to match
  declared properties, we guarantee that what the system believes about a
  container remains true throughout its lifecycle. The immutability of identity
  ensures containers cannot masquerade as other entities, while fixed boundaries
  prevent unauthorized expansion that could compromise isolation. Resource limits
  protect the system from container overreach that could destabilize other
  components. Together, these constraints form an invariant bridge between
  ontology and runtime, making the abstract concrete and the declared enforceable.
}
```

**Parse Result:**
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running `target/debug/dol-parse examples/constraints/container.integrity.dol`
✓ examples/constraints/container.integrity.dol (container.integrity)
    container.integrity constraint with 4 rules

Summary
  Total:    1
  Success:  1
```

### Example 3: Counter Bounds Constraint

Ensures counter stays within limits:

```dol
constraint counter.bounds_valid {
  value never overflows
  value never underflows
  bounds never inverted
}

exegesis {
  The counter.bounds_valid constraint ensures the counter state is
  always valid. Three invariants must hold:

  1. value >= minimum: can't go below the lower bound
  2. value <= maximum: can't exceed the upper bound
  3. minimum <= maximum: bounds must be sensible

  Constraints are checked after every operation. If violated, the
  operation must be rejected or the system is in an invalid state.
}
```

**Parse Result:**
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/dol-parse examples/constraints/counter_bounds.dol`
✓ examples/constraints/counter_bounds.dol (counter.bounds_valid)
    counter.bounds_valid constraint with 3 rules

Summary
  Total:    1
  Success:  1
```

### Example 4: Greeting Protocol Constraint

```dol
constraint greeting.protocol {
  greeting requires sender
  greeting requires recipient
  sender differs.from recipient
  response follows request
}

exegesis {
  The greeting.protocol constraint defines the rules for valid greetings:

  1. greeting requires sender: every greeting must have a sender
  2. greeting requires recipient: every greeting must have a recipient
  3. sender differs.from recipient: can't greet yourself
  4. response follows request: responses must follow initial greetings

  This demonstrates how constraints encode business rules and
  protocol requirements that must always be satisfied.
}
```


---

## Constraint Feature Status

| Feature | Status | Notes |
|---------|--------|-------|
| Constraint declaration | WORKING | `constraint name { }` |
| Requires clause | WORKING | Preconditions |
| Ensures clause | WORKING | Postconditions |
| Invariants | WORKING | Always-true conditions |
| Parse to AST | WORKING | Complete |
| Validate | WORKING | Semantic checks pass |
| WASM compile | NOT YET | Requires assertion lowering |

---

## Constraint Semantics

Constraints are checked at:
1. **Compile time** - Static analysis where possible
2. **Runtime** - Dynamic assertions for complex conditions

### Constraint Types

| Type | Description | Check Time |
|------|-------------|------------|
| Invariant | Always true | Compile + Runtime |
| Precondition | True before | Runtime |
| Postcondition | True after | Runtime |
| Type constraint | Type bounds | Compile |

---

*Generated by DOL Feature Demo Script*
