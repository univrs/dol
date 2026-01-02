# DOL → WASM Compiler: Remaining Work Plan

**Generated**: 2026-01-02
**Based on**: Cross-repo assessment of DOL, ENR, Identity, and VUDO

## Executive Summary

| Component | Status | Completion |
|-----------|--------|------------|
| DOL → WASM Compiler | Partial | ~35% for full ENR |
| ENR Specs (entropy_core.dol) | **DONE** | ✅ First target compiled |
| Ed25519 Identity | Native only | 0% WASM |
| VUDO Runtime | Production-ready | 95% (missing fuel metering) |

---

## 1. Current State (After entropy_core.dol Success)

### What Now Compiles to WASM

| Feature | Status | Test Coverage |
|---------|--------|---------------|
| Function declarations | ✅ | 15+ tests |
| Local variables (`let`) | ✅ | Tested |
| Variable reassignment | ✅ | Tested |
| If/Else control flow | ✅ | Tested |
| Match expressions | ✅ | Tested |
| While loops | ✅ | Tested |
| For loops (range) | ✅ | Tested |
| Genes (struct-like) | ✅ | Tested |
| Gene methods | ✅ | Tested |
| Gene inheritance | ✅ | Tested |
| SEX functions/globals | ✅ | Tested |
| String literals | ✅ | Tested |
| **f64 binary operations** | ✅ **NEW** | entropy_core.dol |
| **Type inference for ops** | ✅ **NEW** | entropy_core.dol |

### What Does NOT Compile (Blocking ENR)

| Feature | ENR Usage | Complexity |
|---------|-----------|------------|
| `provides` keyword | 27 uses | Low |
| `Option<T>` handling | 12 uses | Medium |
| `Result<T,E>` handling | 7 uses | Medium |
| Enum with data variants | 9 uses | Medium |
| `List<T>` operations | 22 uses | High |
| `Map<K,V>` operations | 10 uses | High |
| Closures/lambdas | 17+ uses | High |
| Iterator chains | 23+ uses | Very High |
| `impl Trait` blocks | 8 uses | High |

---

## 2. ENR Compilation Order (By Complexity)

Based on ENR spec analysis:

### Tier 1: entropy.dol (DONE ✅)
- **Status**: entropy_core.dol compiles and executes
- **Functions working**: add_entropy, scale_entropy, clamp_entropy, weighted_sum, entropy_price_multiplier
- **File size**: 675 bytes WASM

### Tier 2: core.dol
- **Lines**: 528
- **Blockers**: `provides` keyword, `Option<T>`, enum definitions
- **Key types**: NodeId, Credits, EntropyAccount, NexusRole

### Tier 3: septal.dol
- **Lines**: 463
- **Blockers**: Enum match, state transitions, Result handling
- **Key types**: SeptalGateState, transition_gate

### Tier 4: pricing.dol
- **Lines**: 650
- **Blockers**: `impl Trait`, match on ResourceType, Result handling
- **Key types**: PricingModel trait, FixedRatePricing

### Tier 5: revival.dol
- **Lines**: 520
- **Blockers**: List operations, Map<String,String>, iterator chains
- **Key types**: DecompositionState, RevivalManager

### Tier 6: nexus.dol (Most Complex)
- **Lines**: 524
- **Blockers**: Closures, full iterator chains, weighted aggregations
- **Key patterns**: `.iter().map(|r| r.gradient.cpu_available * r.weight).sum()`

---

## 3. Implementation Phases

### Phase 1: P0 Blockers (Required for any ENR spec)

#### 1.1 `provides` Keyword Codegen
**Files to modify**: `src/wasm/compiler.rs`
**Complexity**: Low
**Description**: `provides` creates static constructor methods

```dol
gene Counter {
    has value: i64

    provides new() -> Counter {
        return Counter { value: 0 }
    }
}
```

**Implementation**:
- Detect `provides` blocks in gene declarations
- Emit as regular functions with naming convention: `Counter__new`
- Handle struct construction in return

#### 1.2 `Option<T>` Type
**Files to modify**: `src/wasm/compiler.rs`, `src/wasm/layout.rs`
**Complexity**: Medium
**Memory layout**: Tagged union `[tag: i32, value: T]`

```
Option<i64>:
  None  = [0, 0]      (8 bytes)
  Some  = [1, value]  (8 bytes)
```

**Operations needed**:
- `Some(x)` construction
- `None` construction
- `is_some()` / `is_none()` checks
- `unwrap()` (with trap on None)
- `unwrap_or(default)`
- `map(|x| ...)` (requires closures - defer)

#### 1.3 Enum with Data Variants
**Files to modify**: `src/wasm/compiler.rs`, `src/ast.rs`
**Complexity**: Medium

```dol
type NexusRole: enum {
    Provider,
    Consumer,
    Validator { stake: Credits }
}
```

**Memory layout**:
```
NexusRole:
  Provider  = [0]          (4 bytes, discriminant only)
  Consumer  = [1]          (4 bytes)
  Validator = [2, stake]   (12 bytes: 4 + Credits size)
```

**Implementation**:
- Parse enum variants with optional data
- Compute max variant size for allocation
- Emit discriminant-based branching for match

---

### Phase 2: Core Types (core.dol target)

#### 2.1 `Result<T, E>` Type
**Complexity**: Medium
**Memory layout**: Tagged union like Option

```
Result<i64, Error>:
  Ok  = [0, value]   (discriminant + T)
  Err = [1, error]   (discriminant + E)
```

**Operations**:
- `Ok(x)`, `Err(e)` construction
- `is_ok()`, `is_err()` checks
- `unwrap()`, `unwrap_err()`
- `?` operator (try expression) - syntax sugar for early return

#### 2.2 `const` Declarations
**Complexity**: Low
**Current state**: Partially working

```dol
const MAX_ENTROPY: f64 = 10.0
```

**Implementation**:
- Emit as WASM globals (immutable)
- Or inline at usage sites for simple values

---

### Phase 3: Collections (revival.dol target)

#### 3.1 `List<T>` Type
**Complexity**: High
**Memory layout**: `[len: i32, cap: i32, ptr: i32]` (12 bytes header)

```
List<i64> with 3 elements:
  Header: [3, 4, ptr]
  Data at ptr: [elem0, elem1, elem2, _unused]
```

**Operations needed**:
- `List::new()` - allocate empty list
- `push(x)` - append with possible reallocation
- `get(i)` - bounds-checked access
- `len()` - return length
- `iter()` - return iterator (Phase 4)

#### 3.2 `Map<K, V>` Type
**Complexity**: High
**Memory layout**: Hash table with linear probing

```
Map<String, i64>:
  Header: [len, cap, entries_ptr]
  Entries: [(hash, key_ptr, value), ...]
```

**Operations needed**:
- `Map::new()`
- `insert(k, v)`
- `get(k)` -> Option<V>
- `contains_key(k)`
- `remove(k)`

---

### Phase 4: Functional Features (nexus.dol target)

#### 4.1 Closures/Lambdas
**Complexity**: Very High
**Syntax**: `|x| x + 1` or `|a, b| a + b`

**Implementation approaches**:
1. **Inline expansion** (simple): Desugar to explicit loops
2. **Function references** (complex): Capture environment, create closure struct

For ENR, inline expansion may suffice since most closures are simple transforms.

#### 4.2 Iterator Chains
**Complexity**: Very High
**Pattern**: `.iter().map(f).filter(g).collect()`

**Implementation approaches**:
1. **Eager evaluation**: Each step materializes a new list
2. **Lazy evaluation**: Iterator trait with next() method
3. **Loop fusion**: Compile chain to single loop (optimal)

**Recommended**: Loop fusion for performance

```dol
// This:
list.iter().map(|x| x * 2).filter(|x| x > 5).collect()

// Compiles to:
let result = List::new()
for x in list {
    let mapped = x * 2
    if mapped > 5 {
        result.push(mapped)
    }
}
```

---

### Phase 5: Traits and Dispatch (pricing.dol target)

#### 5.1 `impl Trait` Blocks
**Complexity**: High

```dol
impl PricingModel for FixedRatePricing {
    fun calculate_price(self, request: ResourceRequest) -> Credits {
        // ...
    }
}
```

**Implementation**:
- Static dispatch: Monomorphize at compile time
- Generate mangled function names: `FixedRatePricing__PricingModel__calculate_price`

#### 5.2 Dynamic Dispatch (Optional)
- Vtable approach for trait objects
- Likely not needed for ENR v1

---

## 4. Cross-Repo Integration

### 4.1 Ed25519 WASM Bindings
**Repository**: univrs-identity
**Current state**: Native-only (45 tests passing)
**Required changes**:

```toml
# Cargo.toml additions
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
getrandom = { version = "0.2", features = ["js"] }
```

**Functions to export**:
- `generate_keypair()` -> `[public_key, private_key]`
- `sign(private_key, message)` -> `signature`
- `verify(public_key, message, signature)` -> `bool`

### 4.2 VUDO Fuel Metering
**Repository**: packages/vudo-runtime
**Current state**: No metering (105 tests passing)
**Required changes**:

Option A: WASM instrumentation
- Add fuel counter instructions during DOL→WASM compilation
- Decrement fuel on branches, calls, memory ops

Option B: External timeout
```typescript
const result = await Promise.race([
  spirit.call('expensive_function', args),
  timeout(5000, new Error('Execution timeout'))
]);
```

---

## 5. Priority Matrix

| Task | Priority | Effort | Unlocks |
|------|----------|--------|---------|
| `provides` keyword | P0 | 1 day | core.dol constructors |
| `Option<T>` basic | P0 | 2 days | core.dol, pricing.dol |
| Enum variants | P0 | 2 days | core.dol, septal.dol |
| `const` declarations | P1 | 0.5 days | All ENR files |
| `Result<T,E>` | P1 | 1 day | Error handling |
| `List<T>` operations | P1 | 3 days | revival.dol |
| `Map<K,V>` operations | P2 | 3 days | nexus.dol |
| Closures (inline) | P2 | 3 days | All iterator usage |
| Iterator fusion | P2 | 4 days | Performance |
| `impl Trait` | P2 | 2 days | pricing.dol |
| Ed25519 WASM | P3 | 2 days | Identity in browser |
| VUDO fuel metering | P3 | 3 days | Production safety |

---

## 6. Test Cases to Add

For each new feature, add to `/home/ardeshir/repos/univrs-dol/test-cases/`:

### Level 5 (Advanced)
- `option_basic.dol` - Option construction and unwrap
- `result_handling.dol` - Result Ok/Err and ? operator
- `enum_variants.dol` - Enum with data and match
- `list_operations.dol` - List push, get, len

### Level 6 (Collections)
- `map_operations.dol` - Map insert, get, contains
- `iterator_basic.dol` - Simple .iter() usage

### ENR Subset
- `core_subset.dol` - Types from core.dol
- `pricing_subset.dol` - PricingModel trait impl
- `nexus_subset.dol` - Simplified nexus operations

---

## 7. Success Metrics

### Phase 1 Complete
- [ ] `provides` keyword compiles
- [ ] `Option<i64>` Some/None/unwrap works
- [ ] Simple enum variants compile
- [ ] core.dol subset compiles

### Phase 2 Complete
- [ ] `Result<T,E>` with ? operator
- [ ] Full core.dol compiles
- [ ] septal.dol state machine works

### Phase 3 Complete
- [ ] `List<T>` with push/get/len
- [ ] `Map<K,V>` with insert/get
- [ ] revival.dol compiles

### Phase 4 Complete
- [ ] Inline closures in iterator chains
- [ ] Loop fusion optimization
- [ ] pricing.dol compiles

### Phase 5 Complete
- [ ] Full nexus.dol compiles
- [ ] All 6 ENR specs → WASM
- [ ] Ed25519 in browser
- [ ] VUDO fuel metering active

---

## 8. Next Immediate Steps

1. **Now**: Test the existing f64 support with more entropy functions
2. **Next**: Implement `provides` keyword (lowest hanging fruit)
3. **Then**: Add `Option<T>` type with tagged union layout
4. **After**: Tackle enum variants with discriminant-based match

The path to full ENR compilation is clear. Each phase builds on the previous, with entropy.dol proving the foundation is solid.
