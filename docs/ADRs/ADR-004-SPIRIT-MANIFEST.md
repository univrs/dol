# ADR-004: Spirit.dol Manifest Format

| Field | Value |
|-------|-------|
| **Status** | Accepted |
| **Date** | 2025-12-01 |
| **Deciders** | VUDO Core Team |
| **Supersedes** | N/A |
| **Superseded by** | N/A |

## Context

DOL packages (called "Spirits") need a manifest file for:
- Package metadata (name, version, authors)
- Dependency declarations
- Build configuration
- Entry point specification

### Requirements

1. **Self-Describing** - Manifest should use DOL syntax, not a separate format
2. **Type-Safe** - Manifest structure should be validatable
3. **Extensible** - Support custom metadata without breaking changes
4. **Readable** - Human-friendly format for manual editing
5. **Tooling-Friendly** - Easy to parse programmatically

### Options Considered

| Format | Self-Describing | Type-Safe | Familiar | Tooling |
|--------|-----------------|-----------|----------|---------|
| **Spirit.dol** | ✅ | ✅ | ⚠️ | ✅ |
| **TOML** | ❌ | ❌ | ✅ | ✅ |
| **JSON** | ❌ | ❌ | ✅ | ✅ |
| **YAML** | ❌ | ❌ | ✅ | ⚠️ |

## Decision

**We chose DOL syntax for Spirit manifests (Spirit.dol).**

### Rationale

1. **Dogfooding** - Use DOL to describe DOL packages
2. **Type Safety** - Manifest is a `gen` definition, validated by compiler
3. **Consistency** - One syntax to learn, not DOL + TOML
4. **Extensibility** - Custom fields are just additional `has` declarations
5. **Tooling Reuse** - Same parser for code and manifest

### Why Not TOML?

While TOML is familiar (Cargo.toml), it:
- Introduces a second syntax for users to learn
- Lacks type checking (typos silently accepted)
- Can't express complex types (e.g., conditional dependencies)
- Misses opportunity to demonstrate DOL's capabilities

## Spirit.dol Specification

### Minimal Manifest

```dol
spirit MySpirit {
    name: "my-spirit"
    version: "1.0.0"
}
```

### Complete Manifest

```dol
spirit MySpirit {
    // ═══════════════════════════════════════════════════════════════
    // METADATA
    // ═══════════════════════════════════════════════════════════════
    
    name: "my-spirit"
    version: "1.0.0"
    authors: ["Alice <alice@example.com>", "Bob <bob@example.com>"]
    license: "MIT"
    repository: "https://github.com/org/my-spirit"
    homepage: "https://my-spirit.dev"
    keywords: ["container", "orchestration", "local-first"]
    
    docs {
        A Spirit for container orchestration with CRDT-based state.
        
        Features:
        - Offline-first operation
        - Automatic conflict resolution
        - P2P synchronization
    }
    
    // ═══════════════════════════════════════════════════════════════
    // DEPENDENCIES
    // ═══════════════════════════════════════════════════════════════
    
    requires {
        @univrs/std: "^1.0"
        @univrs/network: "^0.5"
        @community/logging: "^2.0"
    }
    
    dev_requires {
        @univrs/test: "^1.0"
        @univrs/bench: "^0.3"
    }
    
    // ═══════════════════════════════════════════════════════════════
    // BUILD CONFIGURATION
    // ═══════════════════════════════════════════════════════════════
    
    targets {
        wasm: {
            optimize: true
            target: "wasm32-wasi"
            features: ["simd"]
        }
        rust: {
            edition: "2024"
            features: ["async"]
        }
        typescript: {
            esm: true
            runtime: "deno"
        }
    }
    
    // ═══════════════════════════════════════════════════════════════
    // ENTRY POINTS
    // ═══════════════════════════════════════════════════════════════
    
    lib: "src/lib.dol"
    
    bin: [
        { name: "my-cli", path: "src/main.dol" },
        { name: "my-daemon", path: "src/daemon.dol" }
    ]
    
    // ═══════════════════════════════════════════════════════════════
    // FEATURES (Optional Compilation)
    // ═══════════════════════════════════════════════════════════════
    
    features {
        default: ["logging"]
        logging: { requires: ["@community/logging"] }
        metrics: { requires: ["@univrs/metrics"] }
        full: { includes: ["logging", "metrics"] }
    }
}
```

### Schema Definition

The manifest itself is defined as a `gen`:

```dol
gen SpiritManifest {
    has name: string
    has version: string
    has authors: Option<Vec<string>>
    has license: Option<string>
    has repository: Option<string>
    has homepage: Option<string>
    has keywords: Option<Vec<string>>
    
    has requires: Option<Map<string, string>>
    has dev_requires: Option<Map<string, string>>
    
    has targets: Option<TargetConfig>
    has lib: Option<string>
    has bin: Option<Vec<BinEntry>>
    has features: Option<Map<string, FeatureConfig>>
    
    rule valid_name {
        this.name.matches(r"^[a-z][a-z0-9-]*$")
    }
    
    rule valid_version {
        this.version.matches(r"^\d+\.\d+\.\d+(-[a-z0-9.]+)?$")
    }
    
    docs {
        Spirit package manifest schema.
        Validated by the DOL compiler during build.
    }
}
```

## Consequences

### Positive

- **Single Syntax** - Users learn only DOL
- **Type-Safe** - Compiler catches manifest errors
- **Expressive** - Complex configurations possible
- **Self-Documenting** - `docs` blocks in manifest
- **Extensible** - Add fields without breaking parsers

### Negative

- **Unfamiliar** - Developers expect TOML/JSON
- **Verbose** - More syntax than minimal TOML
- **Tooling Gap** - Editors don't highlight Spirit.dol specially

### Neutral

- **Migration** - Projects starting fresh avoid TOML habits
- **Ecosystem** - Other tools need DOL parser for manifest reading

## Implementation Notes

### CLI Commands

```bash
# Initialize new Spirit
dol init my-spirit
# Creates: my-spirit/Spirit.dol with minimal manifest

# Validate manifest
dol check Spirit.dol

# Add dependency
dol add @univrs/network@^0.5
# Updates: Spirit.dol requires block

# Build all targets
dol build
# Reads: Spirit.dol for configuration
```

### Programmatic Access

```rust
use dol_manifest::SpiritManifest;

let manifest = SpiritManifest::load("Spirit.dol")?;
println!("Building {} v{}", manifest.name, manifest.version);

for (dep, version) in manifest.requires.unwrap_or_default() {
    println!("  Requires: {} @ {}", dep, version);
}
```

## References

- [Cargo.toml Reference](https://doc.rust-lang.org/cargo/reference/manifest.html) (inspiration)
- [DOL Package Registry](../apps/gen-registry/)
- [Spirit Packaging Guide](../docs/SPIRIT-PACKAGING.md)

## Changelog

| Date | Change |
|------|--------|
| 2025-12-01 | Initial Spirit.dol format accepted |
| 2025-12-15 | Added features system |
| 2026-01-20 | Updated examples to v0.8.1 syntax |
