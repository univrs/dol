# CRDT Validation Examples

This directory contains examples of CRDT (Conflict-Free Replicated Data Type) annotations in DOL, demonstrating both valid and invalid usage patterns.

## Overview

DOL's CRDT validation system (implemented in RFC-001) ensures that CRDT merge strategies are compatible with field types and that constraints are compatible with distributed merge semantics.

## Validation Features

### 1. Type-Strategy Compatibility (RFC-001 Section 4)

The validator checks that CRDT strategies are compatible with field types based on the 84-combination matrix:

**String Types** (`String`, `string`):
- ✅ `immutable` - Set once, never changes
- ✅ `lww` - Last-write-wins
- ✅ `peritext` - Rich text CRDT for collaborative editing
- ✅ `mv_register` - Multi-value register (exposes conflicts)
- ❌ `or_set`, `pn_counter`, `rga` - Incompatible

**Integer Types** (`i8`, `i16`, `i32`, `i64`, `i128`, `u8`, `u16`, `u32`, `u64`, `u128`, `Int`):
- ✅ `immutable` - Set once
- ✅ `lww` - Last-write-wins
- ✅ `pn_counter` - Positive-negative counter for +/- operations
- ✅ `mv_register` - Multi-value register
- ❌ `or_set`, `peritext`, `rga` - Incompatible

**Float Types** (`f32`, `f64`, `Float32`, `Float64`, `Float`):
- ✅ `immutable` - Set once
- ✅ `lww` - Last-write-wins
- ✅ `mv_register` - Multi-value register
- ❌ `or_set`, `pn_counter`, `peritext`, `rga` - Incompatible

**Bool Types** (`Bool`, `bool`):
- ✅ `immutable` - Set once
- ✅ `lww` - Last-write-wins
- ✅ `mv_register` - Multi-value register
- ❌ `or_set`, `pn_counter`, `peritext`, `rga` - Incompatible

**Set Types** (`Set<T>`):
- ✅ `immutable` - Set once
- ✅ `or_set` - Observed-remove set (add-wins)
- ✅ `mv_register` - Multi-value register
- ❌ `lww`, `pn_counter`, `peritext`, `rga` - Incompatible

**Vec/List Types** (`Vec<T>`, `List<T>`):
- ✅ `immutable` - Set once
- ✅ `lww` - Last-write-wins (replace entire list)
- ✅ `rga` - Replicated growable array (causal ordering)
- ✅ `mv_register` - Multi-value register
- ❌ `or_set`, `pn_counter`, `peritext` - Incompatible

**Map Types** (`Map<K, V>`):
- ✅ `immutable` - Set once
- ✅ `lww` - Last-write-wins
- ✅ `mv_register` - Multi-value register
- ❌ `or_set`, `pn_counter`, `peritext`, `rga` - Incompatible

### 2. Constraint-CRDT Compatibility (RFC-001 Section 5)

The validator categorizes constraints into three categories:

**Category A: CRDT-Safe**
- Constraints enforced by the CRDT strategy itself
- No warnings generated
- Examples: Immutability checks, monotonic properties

**Category B: Eventually Consistent**
- Constraints may temporarily violate during network partitions
- Will converge after partition heals
- ⚠️ Warning generated
- Examples: Bounds checks, cardinality limits

**Category C: Strong Consistency**
- Constraints require coordination mechanisms (escrow, BFT)
- ⚠️ Warning generated with coordination pattern suggestion
- Examples: Uniqueness constraints, escrow balances, quotas

### 3. Evolution CRDT Strategy Changes (RFC-001 Section 6)

The validator checks that CRDT strategy evolutions maintain semantic compatibility:

**Forbidden Changes**:
- ❌ `immutable` → any (immutable means never changes)
- ❌ `or_set` → `rga` (set → list semantic change)
- ❌ Type changes that break compatibility

**Allowed Changes**:
- ✅ `lww` → `mv_register` (exposes conflicts)
- ✅ Strategy changes that preserve semantics

## Examples

### Valid Examples

#### chat-message.dol
Demonstrates collaborative editing with multiple CRDT strategies:
- Immutable identity fields
- Peritext for rich text editing
- OR-Set for reactions
- LWW for edit timestamps

#### counter.dol
Demonstrates PN-Counter for distributed counting:
- Immutable identity
- PN-Counter for commutative increment/decrement

#### collaborative-list.dol
Demonstrates RGA for ordered sequences:
- Immutable identity
- RGA for causal ordering of list operations

### Invalid Examples

#### invalid-strategy.dol
Demonstrates incompatible CRDT strategies:
- ❌ OR-Set on String (should be Set<T>)
- ❌ PN-Counter on String (should be integer type)

## Running Validation

```bash
# Validate valid examples
dol-check examples/crdt/chat-message.dol
dol-check examples/crdt/counter.dol
dol-check examples/crdt/collaborative-list.dol

# Check invalid example (should fail)
dol-check examples/crdt/invalid-strategy.dol

# Validate all CRDT examples
dol-check examples/crdt/*.dol
```

## Error Messages

### Type Incompatibility
```
✗ examples/crdt/invalid-strategy.dol
  incompatible CRDT strategy for field 'name' at line 2, column 3: OrSet cannot be used with type string
```

### Suggestion
The validator provides suggestions for valid strategies:
```
Valid strategies for String: immutable, lww, peritext, mv_register
Valid strategies for integers: immutable, lww, pn_counter, mv_register
Valid strategies for Set: immutable, or_set, mv_register
```

## Implementation Details

### Validator Location
- **File**: `src/validator.rs`
- **Function**: `validate_crdt_type_compatibility()`
- **Helper Functions**:
  - `is_string_type()`, `is_integer_type()`, `is_float_type()`, `is_bool_type()`
  - `suggest_valid_strategies()`
  - `categorize_constraint()`
  - `validate_crdt_constraints()`

### Error Types
- **File**: `src/error.rs`
- **Error**: `ValidationError::IncompatibleCrdtStrategy`
- **Warnings**: 
  - `ValidationWarning::EventuallyConsistent`
  - `ValidationWarning::RequiresCoordination`

## References

- **RFC-001**: CRDT Annotation System (Section 4: Type Compatibility Matrix)
- **RFC-001**: Constraint-CRDT Interaction (Section 5: Three-Category Framework)
- **RFC-001**: Evolution CRDT Changes (Section 6: Strategy Migration Rules)

## Future Enhancements

1. **Constraint Expression Parsing**: Currently uses heuristics; could parse full constraint expressions
2. **Evolution Validation**: Add full support for validating CRDT strategy changes in `evo` declarations
3. **Custom CRDT Strategies**: Allow user-defined CRDT strategies with compatibility rules
4. **Performance Optimization**: Cache compatibility checks for large schemas
