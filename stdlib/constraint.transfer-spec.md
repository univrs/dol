# Constraint Transfer Semantics

> **Design spec — not implemented.** There is no DOL standard library in dol 0.8.2: no stdlib `.dol` or `.rs` sources exist in `src/` or `stdlib/`, and the compiler loads none. This directory contains only Markdown specs. Code samples here may use syntax the 0.8.2 parser does not accept (see `docs/DOL-STATUS-2026-11.md` §5.4 and §7).

> Full specification for constraint preservation during gene transformations.
> This document uses extended DOL syntax that may not yet be supported by the parser.

## Overview

When transforming a gene from type `T` to type `U`, what happens to constraints?

```dol
-- Current: constraints are lost
function map<U>(self, f: Function<T, U>) -> Gene<U>

-- Proposed: optional constraint preservation
function map_preserving<U>(self, f: Function<T, U>) -> (Gene<U>, TransferResult)
function map_strict<U>(self, f: Function<T, U>) -> Gene<U> | Error
```

## Constraint Categories

```dol
gene constraint.category {
  category is one_of: structural, semantic, contextual

  structural:
    operates_on: shape
    examples: [non_null, non_empty, bounded_size, length_limit]
    transferability: high

  semantic:
    operates_on: meaning
    examples: [valid_email, positive_integer, sorted]
    transferability: conditional

  contextual:
    operates_on: external_state
    examples: [unique_in_collection, references_valid_id]
    transferability: none
}
```

## Transferability

```dol
gene constraint.transferability {
  transferability is one_of:
    direct       -- constraint transfers unchanged
    adapted      -- constraint transfers with modification
    incompatible -- constraint cannot transfer

  decision_tree:
    if constraint.category == structural:
      if target.shape == source.shape:
        return direct
      else:
        return incompatible

    if constraint.category == semantic:
      if types_are_compatible(source, target):
        return adapted
      else:
        return incompatible

    if constraint.category == contextual:
      return incompatible  -- always
}
```

## Transfer Result

```dol
gene constraint.transfer_result<T> {
  result has:
    transferred: List<Constraint>        -- preserved unchanged
    adapted: List<(Constraint, Constraint)>  -- (original, modified)
    dropped: List<(Constraint, Reason)>  -- lost with explanation
    warnings: List<Warning>              -- diagnostics
}

trait transfer_result.inspect {
  is_lossless: transferred.len() == source.constraints.len()
  has_adaptations: adapted.len() > 0
  has_losses: dropped.len() > 0

  to_report: String  -- human-readable summary
}
```

## Map Operations

### Basic Map (constraints dropped)

```dol
trait gene.map<T, U> {
  signature: (Gene<T>, Function<T, U>) -> Gene<U>

  behavior:
    new_gene = apply_transform(source, function)
    new_gene.constraints = []  -- all dropped
    return new_gene

  use_when:
    - constraints are irrelevant to target type
    - you will add new constraints manually
    - transformation changes semantics fundamentally
}
```

### Preserving Map (constraints carried with audit)

```dol
trait gene.map_preserving<T, U> {
  signature: (Gene<T>, Function<T, U>) -> (Gene<U>, TransferResult)

  behavior:
    new_gene = apply_transform(source, function)
    result = TransferResult.empty()

    for constraint in source.constraints:
      match check_compatibility(constraint, T, U):
        Direct =>
          new_gene.add_constraint(constraint)
          result.transferred.push(constraint)
        Adapted(new_constraint) =>
          new_gene.add_constraint(new_constraint)
          result.adapted.push((constraint, new_constraint))
        Incompatible(reason) =>
          result.dropped.push((constraint, reason))

    return (new_gene, result)

  use_when:
    - constraint preservation matters
    - you want visibility into what transferred
    - you will handle drops gracefully
}
```

### Strict Map (fail on any loss)

```dol
trait gene.map_strict<T, U> {
  signature: (Gene<T>, Function<T, U>) -> Result<Gene<U>, TransferError>

  behavior:
    (new_gene, result) = map_preserving(source, function)

    if result.dropped.len() > 0:
      return Error(TransferError {
        incompatible: result.dropped,
        message: "Constraints cannot transfer"
      })

    if result.adapted.len() > 0:
      return Error(TransferError {
        requires_adaptation: result.adapted,
        message: "Constraints require modification"
      })

    return Ok(new_gene)

  use_when:
    - constraint preservation is mandatory
    - you need compile-time guarantees
    - any loss indicates a bug
}
```

## Compatibility Rules

### Numeric Types

```
Int -> Float:
  ">= 0"      => adapted to ">= 0.0"
  "<= 100"    => adapted to "<= 100.0"
  "is_even"   => incompatible (no parity in Float)
  "is_prime"  => incompatible (no integer primality in Float)

Float -> Int:
  ">= 0.0"    => adapted to ">= 0" (if floor/ceil specified)
  "is_finite" => incompatible (all Int are finite)
```

### Container Types

```
List<A> -> List<B>:
  "is_non_empty"   => direct (structural)
  "length <= 100"  => direct (structural)
  "all is_valid"   => incompatible (A-specific)
  "is_sorted"      => incompatible (requires A comparison)

List<A> -> Set<A>:
  "is_non_empty"   => direct
  "length <= 100"  => adapted to "size <= 100"
  "has_duplicates" => incompatible (Set has no duplicates)
```

### Custom Types

```
User -> String (via user.name):
  "email_valid"     => incompatible (User-specific)
  "is_non_null"     => direct (structural)
  "age >= 18"       => incompatible (User-specific)

CustomerId -> String:
  "matches_uuid"    => direct (if CustomerId wraps UUID string)
  "is_active"       => incompatible (requires lookup)
```

## Invariants

```dol
constraint transfer.soundness {
  -- Core invariant: never claim success incorrectly

  forall constraint C, types T U:
    if map_preserving(gene<T>, f).result.transferred.contains(C):
      then C.holds_on(map_preserving(gene<T>, f).new_gene)

  -- Conservative: false negatives OK, false positives NOT OK

  may_drop_transferable_constraints: true   -- acceptable
  may_claim_incompatible_transferred: false -- unacceptable
}

constraint transfer.explicitness {
  -- Default behavior drops all constraints

  map.behavior: drop_all_constraints
  map_preserving.behavior: attempt_transfer_with_audit
  map_strict.behavior: fail_on_any_loss

  -- User must opt-in to constraint transfer
  -- This prevents accidental semantic leakage
}
```

## Examples

### Example 1: Numeric Widening

```dol
gene counter {
  value is Int
  value >= 0
  value <= 100
  value is even
}

-- Map to Float
let (float_counter, result) = counter.map_preserving(|v| v as Float)

-- Result:
--   transferred: []
--   adapted: [(>= 0, >= 0.0), (<= 100, <= 100.0)]
--   dropped: [(is_even, "Float has no integer parity")]
```

### Example 2: Container Mapping

```dol
gene user_list {
  users is List<User>
  users is non_empty
  users has length <= 1000
  all users has valid_email
}

-- Map to list of names
let (names, result) = user_list.map_preserving(|u| u.name)

-- Result:
--   transferred: [non_empty, length <= 1000]
--   adapted: []
--   dropped: [(all has valid_email, "String has no email concept")]
```

### Example 3: Strict Mode Failure

```dol
gene validated_id {
  id is CustomerId
  id is non_null
  id is active  -- requires DB lookup
}

-- Attempt strict map to String
let result = validated_id.map_strict(|id| id.to_string())

-- Result: Error
--   message: "Constraints cannot transfer"
--   incompatible: [(is_active, "contextual constraint")]
```

## Implementation Notes

1. **Parser Extension Needed**: This spec uses syntax not yet in DOL
   - Generics: `<T, U>`
   - Match expressions
   - For loops
   - Function signatures

2. **Type System Integration**: Constraint transfer should integrate with:
   - Trait bounds
   - Where clauses
   - Subtyping rules

3. **Rust Codegen**: Generate proper constraint checks in Rust output

4. **Parseable Specs**: See individual `.dol` files in this directory for
   parser-compatible versions of each concept.
