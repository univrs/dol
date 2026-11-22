# ADR-003: DOL v0.8 "Clarity" Syntax Evolution

| Field | Value |
|-------|-------|
| **Status** | Accepted. Keyword and type renames are implemented (dol 0.8.0). Removal of the old forms is planned for v0.9 and is not implemented. |
| **Date** | 2026-01-15 |
| **Deciders** | VUDO Core Team |
| **Supersedes** | Pre-0.8 keyword set (`gene`, `exegesis`, `evolves`, `constraint`, PascalCase types) |
| **Superseded by** | N/A |

> **Version note.** The filename says "V081" because this ADR was first written against v0.8.1.
> The renames actually shipped in **dol 0.8.0**: the parser's deprecation warnings say
> "deprecated in v0.8.0", and so does `CHANGELOG.md` (0.8.0 "Clarity"). This ADR uses 0.8.0.
> The filename is kept so existing links still work.

## Context

DOL (Design Ontology Language) renamed several keywords and its primitive type names in the 0.8 release line. Four things drove the change:

1. **Terminology conflicts.** "gene" clashes with bioinformatics domain modeling.
2. **Verbosity.** Common constructs needed shorter keywords.
3. **Plain English.** Documentation syntax should read naturally.
4. **Consistency.** Names should follow one convention across the language.

### Problems with the pre-0.8 keywords

| Old keyword | Problem |
|----------------|---------|
| `gene` | Clashes with the bioinformatics domain (DNA genes) |
| `exegesis` | Too obscure; most developers don't know the word |
| `evolves` | Verbose; `evo` is shorter |
| `constraint` | Verbose; `rule` is more intuitive |

## Decision

**Adopt simplified, conflict-free keywords and Rust-style lowercase primitive types.**

### Keyword changes (implemented in dol 0.8.0)

| Old (deprecated) | New | Rationale |
|--------|--------|-----------|
| `gene` | `gen` | Avoids the biology clash |
| `exegesis` | `docs` | Widely understood; matches Rust and Python usage |
| `evolves` | `evo` | Shorter; obvious abbreviation of "evolution" |
| `constraint` | `rule` | Simpler, more intuitive validation semantics |

### Type changes (implemented in dol 0.8.0)

| Old (deprecated) | New | Rationale |
|--------|--------|-----------|
| `String` | `string` | Lowercase primitives |
| `Bool` | `bool` | Consistency with Rust primitives |
| `Int8`/`Int16`/`Int32`/`Int64` | `i8`/`i16`/`i32`/`i64` | Rust-style numeric types |
| `UInt8`/`UInt16`/`UInt32`/`UInt64` | `u8`/`u16`/`u32`/`u64` | Rust-style numeric types |
| `Float32`/`Float64` | `f32`/`f64` | Rust-style numeric types |
| `Void` | `()` | Unit type |
| `List<T>` | `Vec<T>` | Rust alignment |
| `Optional<T>` | `Option<T>` | Rust alignment |

`i128` and `u128` are also accepted.

**Current behaviour (dol 0.8.2):** the old keywords and types still parse. Each one emits a warning such as
`warning: 'gene' keyword is deprecated in v0.8.0, use 'gen' instead`. The CHANGELOG plans to turn these warnings into errors in v0.9, which has not been released.

### Syntax comparison

Both blocks below parse with dol 0.8.2. The first one emits deprecation warnings.

**Deprecated (pre-0.8 keywords and types):**
```dol
gene Container {
    has id: UInt64
    has name: String
    has running: Bool
}

exegesis {
    A container is an isolated execution environment.
}

evolves Container @ 0.2.0 > 0.1.0 {
    adds namespace: String
    because "multi-tenancy needs namespaces"
}

exegesis {
    Adds namespace support for multi-tenancy.
}
```

**Current (0.8 syntax):**
```dol
gen Container {
    has id: u64
    has name: string
    has running: bool

    docs {
        A container is an isolated execution environment.
    }
}

evo Container @ 0.2.0 > 0.1.0 {
    adds namespace: string
    because "multi-tenancy needs namespaces"

    docs {
        Adds namespace support for multi-tenancy.
    }
}
```

An evolution is written `evo <Name> @ <new-version> > <parent-version> { … }`. Its body accepts:
- `adds <statement>`
- `deprecates <statement>`
- `removes <name>`
- `because "<rationale>"`
- `migrate { … }`

A `docs { }` block can go either inside the braces or directly after them.

A `rule name { … }` nested inside a `gen` body is accepted, but its body is discarded by the parser (it is recorded only as a requirement name). See `docs/DOL-STATUS-2026-11.md` §5.3.

### Proposed — not implemented

The original draft of this ADR showed the forms below. **None of them parse in dol 0.8.2.**

```text
Container evolves ContainerV2 { ... }     // infix "evolves" between two names
evo Container > ContainerV2 { ... }        // evo between names, without "@ version"
added namespace: string                    // "added" (the keyword is "adds")
```

### Other keywords (unchanged)

| Keyword | Purpose |
|---------|---------|
| `has` | Field / property declaration |
| `fun` | Function definition |
| `val` | Immutable binding |
| `var` | Mutable binding |
| `trait` | Behavior interface |
| `system` | Composition module |
| `pub` | Visibility (parsed but currently dropped from the AST; see the status audit §5.3) |
| `use` | Import statement |

## Consequences

### Positive

- **No domain conflicts.** `gen` doesn't clash with biology.
- **Approachable.** `docs` is universally understood.
- **Concise.** Shorter keywords reduce visual noise.
- **Rust alignment.** Type names match Rust conventions.
- **Migration tooling.** `dol-migrate 0.7-to-0.8` rewrites the old forms automatically.

### Negative

- **Breaking change at v0.9.** Pre-0.8 files will stop parsing once the warnings become errors. As of 2026-11, more than 127 ontology `.dol` files in other repos still use pre-0.8 syntax (see the status audit §4).
- **Documentation churn.** Many docs and tutorials still use the old forms and are being corrected.
- **Learning curve.** Existing users have to relearn the keywords.

## Migration guide

### Automated migration

```bash
# Rewrite pre-0.8 keywords and types to 0.8 syntax
cargo run --features cli --bin dol-migrate -- 0.7-to-0.8 src/

# Preview changes without applying them
cargo run --features cli --bin dol-migrate -- 0.7-to-0.8 --dry-run src/
cargo run --features cli --bin dol-migrate -- 0.7-to-0.8 --diff src/
```

There is no `dol migrate` command: no `dol` binary exists. The tool is the `dol-migrate` binary in this crate (`src/bin/dol-migrate.rs`).

### Manual checklist

1. Replace `gene` with `gen`.
2. Replace `exegesis` with `docs`.
3. Replace `evolves` with `evo`. The header form is the same for both: `evo Name @ new > old { … }`.
4. Replace `constraint` with `rule`.
5. Update the types, e.g. `String` → `string`, `Int64` → `i64`, `List<T>` → `Vec<T>`, `Void` → `()`.
6. Check that every file parses: `cargo run --features cli --bin dol-check -- src/`. Note that `dol-check` and `dol-parse` currently parse only the first declaration in each file (status audit §5.1).

### Regex patterns

```bash
# Quick sed replacements (use with caution; dol-migrate is safer)
sed -i 's/\bgene\b/gen/g' *.dol
sed -i 's/\bexegesis\b/docs/g' *.dol
sed -i 's/\bconstraint\b/rule/g' *.dol
sed -i 's/\bevolves\b/evo/g' *.dol
sed -i 's/\bString\b/string/g' *.dol
sed -i 's/\bBool\b/bool/g' *.dol
sed -i 's/\bInt32\b/i32/g' *.dol
sed -i 's/\bInt64\b/i64/g' *.dol
sed -i 's/\bUInt64\b/u64/g' *.dol
```

## References

- [Grammar: DOL language as implemented by dol 0.8.2](../grammar.ebnf)
- [CHANGELOG: 0.8.0 "Clarity"](../../CHANGELOG.md)
- [Migration tool source](../../src/bin/dol-migrate.rs)
- [DOL status audit (2026-11)](../DOL-STATUS-2026-11.md)

There is no separate "DOL 2.0 specification" document. An earlier version of this ADR linked to `docs/02-DOL-2.0-SPECIFICATION.md`, which does not exist.

## Changelog

| Date | Change |
|------|--------|
| 2026-01-15 | Initial syntax decision accepted |
| 2026-01-20 | Added migration tooling |
| 2026-02-05 | Completed core file migration |
| 2026-11-21 | Corrected to match the implementation: renames shipped in 0.8.0; evolution syntax is `evo X @ new > old`; unimplemented draft forms marked as proposed; broken spec link removed |
