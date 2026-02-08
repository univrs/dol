# ADR-006: SEX (Side Effect eXecution) System Design

| Field | Value |
|-------|-------|
| **Status** | Accepted |
| **Date** | 2025-12-10 |
| **Deciders** | VUDO Core Team |
| **Supersedes** | N/A |
| **Superseded by** | N/A |

## Context

DOL needs a mechanism to distinguish pure code from code with side effects. This enables:
- Compile-time effect tracking
- Safe parallelization of pure code
- Clear boundaries for I/O, mutation, FFI
- Better tooling and optimization opportunities

### Requirements

1. **Pure by Default** - Most code should be side-effect free
2. **Explicit Effects** - Side effects must be marked
3. **Effect Propagation** - Calling effectful code requires effect context
4. **No Runtime Overhead** - Effect tracking is compile-time only
5. **Gradual Adoption** - Don't require annotation everywhere

### Options Considered

| Approach | Pure Default | Explicit | Propagation | Overhead | Ergonomics |
|----------|--------------|----------|-------------|----------|------------|
| **SEX keyword** | ✅ | ✅ | ✅ | None | ⚠️ |
| **Effect types (Haskell IO)** | ✅ | ✅ | ✅ | None | ❌ |
| **Implicit effects (Rust)** | ❌ | ❌ | ❌ | None | ✅ |
| **Async/await style** | ⚠️ | ⚠️ | ✅ | ⚠️ | ✅ |

## Decision

**We chose the SEX (Side Effect eXecution) system with compiler-inferred effects.**

### Design Philosophy

> *"Pure code is the default — safe, predictable, parallelizable.*  
> *Sex code is explicit — tracked, contained, documented.*  
> *Because sometimes, to create something new, boundaries must be crossed."*

The biological metaphor of "sex" (recombination, mutation, crossing barriers) maps intuitively to:
- **Mutation** - Changing global state
- **I/O** - Crossing process boundaries
- **FFI** - Mixing with external code
- **Non-determinism** - Random, time, network

### Key Principles

1. **Everything Pure by Default**
   - Functions without `sex` are referentially transparent
   - Safe to memoize, parallelize, inline

2. **Compiler-Inferred Effects**
   - Write `print("hello")`, compiler knows it's effectful
   - No verbose annotations on every call

3. **Effect Propagation**
   - Pure functions can't call `sex` functions
   - Effects bubble up automatically

4. **Explicit Boundaries**
   - `sex fun` marks effectful functions
   - `sex { }` blocks for inline effects
   - `sex var` for mutable globals

### Why This Name?

The term "SEX" was chosen deliberately:
- **Memorable** - Developers won't forget it
- **Metaphorically Apt** - Sex in biology enables genetic mixing across boundaries
- **Distinct** - Unlike "unsafe" (memory), "impure", or "effectful" (verbose)
- **Honest** - Side effects are sometimes necessary and shouldn't be shameful

## SEX System Specification

### Function-Level Effects

```dol
// Pure function - no side effects
fun add(a: i64, b: i64) -> i64 {
    return a + b
}

// Effectful function - marked with sex
sex fun log_and_add(a: i64, b: i64) -> i64 {
    print("Adding " + a + " and " + b)  // I/O effect
    return a + b
}
```

### Effect Propagation

```dol
// ❌ ERROR: Cannot call sex function from pure context
fun calculate() -> i64 {
    log_and_add(1, 2)  // Compile error!
}

// ✅ OK: Sex function can call other sex functions
sex fun process() -> i64 {
    return log_and_add(1, 2)  // Fine
}
```

### Inline Sex Blocks

```dol
// Mostly pure function with contained effect
fun compute_with_logging(x: i64) -> i64 {
    val result = x * 2
    
    sex {
        // Side effects contained here
        debug_log("Computed: " + result)
    }
    
    return result
}
```

### Mutable Global State

```dol
// sex/globals.dol - Must be in sex file

sex var COUNTER: i64 = 0

sex fun increment() -> i64 {
    COUNTER += 1
    return COUNTER
}
```

### FFI (Foreign Function Interface)

```dol
// sex/ffi.dol - External calls require sex

sex extern "C" fun malloc(size: u64) -> Ptr<u8>
sex extern "C" fun free(ptr: Ptr<u8>)

sex fun allocate<T>() -> Ptr<T> {
    val size = size_of<T>()
    return malloc(size).cast<T>()
}
```

## Consequences

### Positive

- **Clear Boundaries** - Know exactly where effects occur
- **Safe Parallelization** - Pure code is parallelizable
- **Better Optimization** - Compiler can inline/memoize pure functions
- **Self-Documenting** - Effect signatures are documentation
- **Gradual Strictness** - Start permissive, tighten over time

### Negative

- **Learning Curve** - New concept for most developers
- **Annotation Burden** - Must mark effectful code
- **Infectious** - One effect can require changes up the call stack

### Neutral

- **Cultural** - The name may cause reaction (intended to be memorable)
- **Tooling** - IDEs need to highlight sex code differently

## Implementation Notes

### Compiler Error Messages

```
error[E001]: sex in pure context
  --> src/main.dol:12:5
   |
12 |     print("hello")
   |     ^^^^^^^^^^^^^^ cannot call effectful function from pure context
   |
help: mark this function as `sex fun` or wrap in `sex { }`
   |
10 | sex fun greet() {
   | +++
```

### Effect Inference Algorithm

```
1. Mark built-in I/O functions as sex
2. Mark extern functions as sex
3. Mark functions accessing mutable globals as sex
4. For each function:
   a. If calls any sex function → mark as sex
   b. If modifies any sex var → mark as sex
   c. Otherwise → pure
5. Report errors for sex calls in pure context
```

### What Requires SEX

| Operation | Requires SEX | Rationale |
|-----------|--------------|-----------|
| `print()` | ✅ | I/O |
| `read_file()` | ✅ | I/O |
| `random()` | ✅ | Non-deterministic |
| `now()` | ✅ | Non-deterministic |
| `http_get()` | ✅ | Network I/O |
| `extern fn` | ✅ | FFI |
| Mutable global | ✅ | Shared state |
| Pure math | ❌ | Deterministic |
| Immutable data | ❌ | No mutation |

## References

- [DOL 2.0 Runtime Architecture](../docs/DOLRAC.md)
- [Effect Systems Survey](https://arxiv.org/abs/1907.01257)
- [Koka Language Effects](https://koka-lang.github.io/koka/doc/index.html)
- [Unison Abilities](https://www.unison-lang.org/learn/abilities/)

## Changelog

| Date | Change |
|------|--------|
| 2025-12-10 | Initial SEX system design |
| 2026-01-15 | Added compiler inference |
| 2026-02-01 | Removed user-visible annotations (auto-inferred) |
