# ADR-003: DOL v0.8.1 Syntax Evolution

| Field | Value |
|-------|-------|
| **Status** | Accepted |
| **Date** | 2026-01-15 |
| **Deciders** | VUDO Core Team |
| **Supersedes** | DOL v0.8.0 syntax |
| **Superseded by** | N/A |

## Context

DOL (Design Ontology Language) underwent keyword evolution from v0.8.0 to v0.8.1. The changes were driven by:

1. **Terminology Conflicts** - "gene" conflicts with biological domain modeling
2. **Verbosity Reduction** - Shorter keywords for common constructs
3. **Plain English Aesthetic** - More natural documentation syntax
4. **Consistency** - Align naming across the language

### Problems with v0.8.0

| v0.8.0 Keyword | Problem |
|----------------|---------|
| `gene` | Conflicts with bioinformatics domain (DNA genes) |
| `exegesis` | Too obscure; most developers don't know this word |
| `evolves` | Verbose; `evo` is clearer and shorter |
| `constraint` | Verbose; `rule` is more intuitive |

## Decision

**We adopted DOL v0.8.1 with simplified, conflict-free keywords.**

### Keyword Changes

| v0.8.0 | v0.8.1 | Rationale |
|--------|--------|-----------|
| `gene` | `gen` | Avoids biology conflict; short for "generate/generic" |
| `exegesis` | `docs` | Universal understanding; matches Rust/Python |
| `evolves` | `evo` | Shorter; clear abbreviation of "evolution" |
| `constraint` | `rule` | Simpler; more intuitive validation semantics |

### Type Changes (Lowercase Convention)

| v0.8.0 | v0.8.1 | Rationale |
|--------|--------|-----------|
| `String` | `string` | Lowercase primitives (like TypeScript) |
| `Bool` | `bool` | Consistency with Rust primitives |
| `Int32` | `i32` | Rust-style numeric types |
| `Int64` | `i64` | Rust-style numeric types |
| `UInt64` | `u64` | Rust-style numeric types |
| `Float32` | `f32` | Rust-style numeric types |
| `Float64` | `f64` | Rust-style numeric types |
| `List<T>` | `Vec<T>` | Rust alignment |

### Syntax Comparison

**v0.8.0 (Deprecated):**
```dol
gene Container {
    has id: UInt64
    has name: String
    has running: Bool
    
    constraint valid_id {
        this.id > 0
    }
    
    exegesis {
        A container is an isolated execution environment.
    }
}

Container evolves ContainerV2 {
    added namespace: String
    
    exegesis {
        Adds namespace support for multi-tenancy.
    }
}
```

**v0.8.1 (Current):**
```dol
gen Container {
    has id: u64
    has name: string
    has running: bool
    
    rule valid_id {
        this.id > 0
    }
    
    docs {
        A container is an isolated execution environment.
    }
}

evo Container > ContainerV2 {
    added namespace: string
    
    docs {
        Adds namespace support for multi-tenancy.
    }
}
```

### Other Keywords (Unchanged)

| Keyword | Purpose | Status |
|---------|---------|--------|
| `has` | Field declaration | ✅ Kept |
| `fun` | Function definition | ✅ Kept |
| `val` | Immutable binding | ✅ Kept |
| `var` | Mutable binding | ✅ Kept |
| `trait` | Behavior interface | ✅ Kept |
| `system` | Composition module | ✅ Kept |
| `pub` | Public visibility | ✅ Kept |
| `use` | Import statement | ✅ Kept |

## Consequences

### Positive

- **No Domain Conflicts** - `gen` doesn't clash with biology
- **Approachable** - `docs` is universally understood
- **Concise** - Shorter keywords reduce visual noise
- **Rust Alignment** - Type naming matches Rust conventions
- **Migration Tooling** - Automated codemods available

### Negative

- **Breaking Change** - All v0.8.0 files need updating
- **Documentation Updates** - Extensive doc rewrites required
- **Learning Curve** - Users of v0.8.0 need to relearn

### Migration Impact

| Component | Files | Status |
|-----------|-------|--------|
| Core compiler | 12 | ✅ Updated |
| Test fixtures | 48 | ⚠️ 38 updated, 10 pending |
| Documentation | 25 | ✅ Updated |
| Examples | 15 | ✅ Updated |
| Specs | 7 | ✅ Updated |

## Migration Guide

### Automated Migration

```bash
# Run the v0.8.1 migration tool
dol migrate --from 0.8.0 --to 0.8.1 src/

# Preview changes without applying
dol migrate --from 0.8.0 --to 0.8.1 --dry-run src/
```

### Manual Checklist

1. Replace `gene` → `gen`
2. Replace `exegesis` → `docs`
3. Replace `evolves` → `evo` (also update syntax: `A evolves B` → `evo A > B`)
4. Replace `constraint` → `rule`
5. Update types: `String` → `string`, `Int64` → `i64`, etc.
6. Verify all files parse: `dol check src/`

### Regex Patterns

```bash
# Quick sed replacements (use with caution)
sed -i 's/\bgene\b/gen/g' *.dol
sed -i 's/\bexegesis\b/docs/g' *.dol
sed -i 's/\bconstraint\b/rule/g' *.dol
sed -i 's/\bString\b/string/g' *.dol
sed -i 's/\bBool\b/bool/g' *.dol
sed -i 's/\bInt32\b/i32/g' *.dol
sed -i 's/\bInt64\b/i64/g' *.dol
sed -i 's/\bUInt64\b/u64/g' *.dol
```

## References

- [DOL 2.0 Specification](../docs/02-DOL-2.0-SPECIFICATION.md)
- [Migration Tool Source](../tools/dol-migrate/)
- [v0.8.1 Release Notes](../CHANGELOG.md#v081)

## Changelog

| Date | Change |
|------|--------|
| 2026-01-15 | Initial v0.8.1 syntax accepted |
| 2026-01-20 | Added migration tooling |
| 2026-02-05 | Completed core file migration |
