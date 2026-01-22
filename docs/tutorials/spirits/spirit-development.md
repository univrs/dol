# Spirit Development Guide

> Creating, organizing, and publishing DOL Spirit packages

## Overview

A **Spirit** is DOL's package unit—a shareable collection of modules with metadata, dependencies, and entry points. This guide covers Spirit anatomy, module organization, and visibility rules.

## Spirit Anatomy

### Core Principle

> **"Spirit is just a gene with package semantics—no new concepts to learn."**

The same syntax patterns used for genes apply to Spirits:

| Gene Concept | Spirit Equivalent |
|--------------|-------------------|
| `has field: Type` | `has name: "my-spirit"` |
| `constraint valid { }` | `constraint valid_version { }` |
| `docs { }` | `docs { }` |
| — | `requires @org/pkg: "^1.0"` |

### Spirit.dol Structure

Every Spirit has a `Spirit.dol` manifest file:

```dol
spirit PhysicsLib {
    // ═══════════════════════════════════════════════════════════
    // IDENTITY (required)
    // ═══════════════════════════════════════════════════════════
    has name: "physics-lib"
    has version: "0.9.0"

    // ═══════════════════════════════════════════════════════════
    // METADATA (optional)
    // ═══════════════════════════════════════════════════════════
    has authors: ["VUDO Team <team@univrs.io>"]
    has license: "MIT"
    has repository: "https://github.com/univrs/physics-lib"
    has description: "Physics constants, particles, and mechanics"

    // ═══════════════════════════════════════════════════════════
    // DEPENDENCIES
    // ═══════════════════════════════════════════════════════════
    requires @univrs/std: "^1.0"

    // ═══════════════════════════════════════════════════════════
    // ENTRY POINTS
    // ═══════════════════════════════════════════════════════════
    has lib: "src/lib.dol"

    // ═══════════════════════════════════════════════════════════
    // DOCUMENTATION
    // ═══════════════════════════════════════════════════════════
    docs {
        Physics library providing fundamental constants, particle models,
        and classical mechanics calculations.

        Quick start:
            use @univrs/physics.{ SPEED_OF_LIGHT, kinetic_energy }

            energy = kinetic_energy(10.0, 100.0)
    }
}
```

### Directory Structure

```
physics-spirit/
├── Spirit.dol              # Package manifest
├── src/
│   ├── lib.dol             # Library entry point (pub exports)
│   ├── constants.dol       # mod constants
│   ├── particles.dol       # mod particles
│   ├── mechanics.dol       # mod mechanics
│   ├── quantum/            # Submodule directory
│   │   ├── wavefunctions.dol
│   │   └── operators.dol
│   └── internal/           # Internal utilities
│       └── helpers.dol
└── tests/
    ├── constants_test.dol
    ├── particles_test.dol
    └── mechanics_test.dol
```

## Module Organization

### Module Paths

Module paths are derived from file paths relative to `src/`:

| File Path | Module Path |
|-----------|-------------|
| `src/constants.dol` | `mod constants` |
| `src/particles.dol` | `mod particles` |
| `src/quantum/wavefunctions.dol` | `mod quantum.wavefunctions` |
| `src/internal/helpers.dol` | `mod internal.helpers` |

### The lib.dol Entry Point

The `lib.dol` file defines the Spirit's public API:

```dol
// src/lib.dol - Public API exports

// Re-export public items from internal modules
pub use constants::*
pub use particles::{ Particle, electron, proton, neutron }
pub use mechanics::{ kinetic_energy, gravitational_force }

// Selective re-export from quantum submodule
pub use quantum::wavefunctions::WaveFunction
pub use quantum::operators::{ momentum_operator, position_operator }
```

### Module Declaration

Modules are implicitly declared by file path. No explicit declaration needed:

```dol
// src/particles.dol
// This file IS mod particles — no declaration needed

docs {
    Particle physics module providing fundamental particles.
}

pub gen Particle {
    has name: string
    has symbol: string
    has mass: f64       // kg
    has charge: f64     // Coulombs
    has spin: f64       // dimensionless
}

pub fun electron() -> Particle {
    return Particle {
        name: "Electron",
        symbol: "e⁻",
        mass: 9.1093837015e-31,
        charge: -1.602176634e-19,
        spin: 0.5
    }
}
```

### Inline Submodules

For small, tightly-coupled code, declare submodules inline:

```dol
// src/particles.dol

pub gen Particle { ... }

// Inline submodule for particle categories
mod categories {
    pub gen Lepton {
        has particle: Particle
        has lepton_number: i8
    }

    pub gen Quark {
        has particle: Particle
        has color_charge: string
    }
}

// Access: particles.categories.Lepton
```

## Visibility Rules

DOL uses explicit visibility modifiers:

| Modifier | Scope | Rust Equivalent |
|----------|-------|-----------------|
| (none) | Private — same module only | (default) |
| `pub` | Public — accessible everywhere | `pub` |
| `pub(spirit)` | Package — within Spirit only | `pub(crate)` |
| `pub(parent)` | Parent — direct parent module | `pub(super)` |

### Visibility Boundaries

```
┌─────────────────────────────────────────────────────────────────┐
│                      VISIBILITY BOUNDARIES                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Spirit A                          Spirit B                      │
│  ┌─────────────────────────┐       ┌─────────────────────────┐  │
│  │                         │       │                         │  │
│  │  mod x                  │       │  mod y                  │  │
│  │  ┌───────────────────┐  │       │  ┌───────────────────┐  │  │
│  │  │ private item      │──┼───✗───┼──│ cannot access     │  │  │
│  │  │ pub(parent) item  │──┼───✗───┼──│ cannot access     │  │  │
│  │  │ pub(spirit) item  │──┼───✗───┼──│ cannot access     │  │  │
│  │  │ pub item          │──┼───────┼─►│ CAN access        │  │  │
│  │  └───────────────────┘  │       │  └───────────────────┘  │  │
│  │                         │       │                         │  │
│  └─────────────────────────┘       └─────────────────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Example: Visibility in Practice

```dol
// src/internal/helpers.dol
// Package-internal utilities — not exported from lib.dol

// Only accessible within this Spirit
pub(spirit) fun generate_id() -> u64 {
    return next_id()
}

// Only accessible from parent module (internal)
pub(parent) fun validate_input(x: f64) -> bool {
    return x >= 0.0
}

// Private — only this module
var ID_COUNTER: u64 = 0

fun next_id() -> u64 {
    ID_COUNTER += 1
    return ID_COUNTER
}
```

```dol
// src/particles.dol
use internal::helpers::generate_id  // ✓ Works (pub(spirit))
use internal::helpers::next_id      // ✗ Error: private
```

```dol
// Another Spirit
use @univrs/physics::internal::helpers::generate_id  // ✗ Error: pub(spirit)
use @univrs/physics::electron                         // ✓ Works (pub)
```

## Creating a New Spirit

### Step 1: Initialize Project

```bash
# Create new Spirit
vudo new chemistry-spirit --template library

# Navigate to project
cd chemistry-spirit
```

### Step 2: Define Spirit.dol

```dol
spirit ChemistryLib {
    has name: "chemistry-lib"
    has version: "0.9.0"
    has authors: ["Your Name <you@example.com>"]
    has license: "MIT"

    requires @univrs/physics-constants: "^0.9"

    has lib: "src/lib.dol"

    docs {
        Chemistry library providing elements, reactions, and molecular modeling.
    }
}
```

### Step 3: Create Module Structure

```
chemistry-spirit/
├── Spirit.dol
├── src/
│   ├── lib.dol           # Entry point
│   ├── elements.dol      # Periodic table
│   ├── reactions.dol     # Chemical reactions
│   └── bonds.dol         # Chemical bonding
└── tests/
    └── elements_test.dol
```

### Step 4: Implement Modules

```dol
// src/elements.dol

docs {
    Periodic table elements module.
}

pub gen Element {
    has atomic_number: u8
    has symbol: string
    has name: string
    has atomic_mass: f64
    has electronegativity: Option<f64>
}

docs {
    Hydrogen - lightest element.
}
pub fun hydrogen() -> Element {
    return Element {
        atomic_number: 1,
        symbol: "H",
        name: "Hydrogen",
        atomic_mass: 1.008,
        electronegativity: Some(2.20)
    }
}

docs {
    Calculate molar mass from atomic mass.
}
pub fun molar_mass(element: Element) -> f64 {
    return element.atomic_mass  // g/mol
}
```

### Step 5: Define Public API

```dol
// src/lib.dol

// Public API
pub use elements::{ Element, hydrogen, helium, carbon, oxygen }
pub use elements::molar_mass
pub use reactions::{ Reaction, balance_equation }
pub use bonds::{ Bond, BondType, bond_energy }
```

### Step 6: Write Tests

```dol
// tests/elements_test.dol

use elements::{ hydrogen, molar_mass }

test "hydrogen has correct properties" {
    let h = hydrogen()
    assert_eq(h.atomic_number, 1)
    assert_eq(h.symbol, "H")
    assert_eq(h.name, "Hydrogen")
}

test "molar mass equals atomic mass" {
    let h = hydrogen()
    assert_eq(molar_mass(h), 1.008)
}
```

### Step 7: Build and Test

```bash
# Check for errors
vudo check

# Run tests
vudo test

# Build WASM
vudo build --release
```

## Real-World Examples

### Example: Biology Spirit Structure

The Biology Spirit demonstrates complex module organization:

```
biology-spirit/
├── Spirit.dol
├── src/
│   ├── lib.dol
│   ├── genetics.dol        # DNA, RNA, proteins
│   ├── cell.dol            # Cell biology
│   ├── evolution.dol       # Evolutionary models
│   └── taxonomy/
│       ├── kingdom.dol
│       ├── phylum.dol
│       └── species.dol
└── tests/
    ├── genetics_test.dol
    └── evolution_test.dol
```

```dol
// Spirit.dol
spirit BiologyLib {
    has name: "biology-lib"
    has version: "0.9.0"

    requires @univrs/chemistry: "^0.9"

    has lib: "src/lib.dol"

    docs {
        Biology library providing genetics, cell biology, and taxonomy.
    }
}
```

### Example: Genetics Module

```dol
// src/genetics.dol

docs {
    Genetics module providing DNA, RNA, and protein synthesis.
}

use @univrs/chemistry::elements::{ carbon, hydrogen, nitrogen, oxygen }

// ═══════════════════════════════════════════════════════════
// TYPES
// ═══════════════════════════════════════════════════════════

docs {
    DNA - double-stranded nucleic acid.
}
pub gen DNA {
    has sequence: string    // A, T, G, C
    has length: u64
    has gc_content: f64
}

docs {
    RNA - single-stranded nucleic acid.
}
pub gen RNA {
    has sequence: string    // A, U, G, C
    has length: u64
    has is_coding: bool
}

docs {
    Protein - amino acid sequence.
}
pub gen Protein {
    has sequence: string    // Single-letter amino acid codes
    has length: u32
    has molecular_weight: f64
}

// ═══════════════════════════════════════════════════════════
// FUNCTIONS
// ═══════════════════════════════════════════════════════════

docs {
    Transcribe DNA to RNA.
}
pub fun transcribe(dna: DNA) -> RNA {
    let mut rna_seq = ""
    for c in dna.sequence.chars() {
        match c {
            'T' => rna_seq = rna_seq + "U",
            _ => rna_seq = rna_seq + c.to_string()
        }
    }
    return RNA {
        sequence: rna_seq,
        length: dna.length,
        is_coding: true
    }
}

docs {
    Translate RNA to protein.
}
pub fun translate(rna: RNA) -> Protein {
    let mut protein_seq = ""
    // Codon translation logic...
    return Protein {
        sequence: protein_seq,
        length: (rna.length / 3) as u32,
        molecular_weight: calculate_mw(protein_seq)
    }
}

// ═══════════════════════════════════════════════════════════
// RULES
// ═══════════════════════════════════════════════════════════

docs {
    Central Dogma: DNA → RNA → Protein
}
pub rule central_dogma {
    each Gene {
        this.dna can_transcribe_to RNA
        RNA can_translate_to Protein
    }
}

// ═══════════════════════════════════════════════════════════
// INTERNAL HELPERS
// ═══════════════════════════════════════════════════════════

pub(spirit) fun calculate_gc_content(seq: string) -> f64 {
    let gc_count = seq.chars().filter(|c| c == 'G' || c == 'C').count()
    return (gc_count as f64) / (seq.len() as f64) * 100.0
}

fun calculate_mw(seq: string) -> f64 {
    // Internal implementation
    return seq.len() as f64 * 110.0  // Average amino acid MW
}
```

## Dependencies

### Adding Dependencies

In `Spirit.dol`:

```dol
spirit MySpirit {
    // Registry packages
    requires @univrs/std: "^1.0"
    requires @univrs/physics: "^0.9"

    // Git repository
    requires @git:github.com/org/repo: "main"

    // HTTP single-file
    requires @https://example.com/utils.dol: { sha256: "abc123..." }
}
```

### Version Specifiers

| Specifier | Meaning |
|-----------|---------|
| `"^1.0"` | Compatible with 1.x |
| `"~1.2"` | Patch updates only (1.2.x) |
| `"=1.2.3"` | Exact version |
| `"main"` | Git branch |
| `"v1.0.0"` | Git tag |

### Using Dependencies

```dol
// src/lib.dol

// Import from dependencies
use @univrs/std::{ Option, Result }
use @univrs/physics::constants::SPEED_OF_LIGHT
use @univrs/physics::mechanics::kinetic_energy
```

## Publishing Spirits

### Prepare for Publication

```bash
# Verify package
vudo check --strict --require-docs

# Run all tests
vudo test

# Build release
vudo build --release
```

### Publish to Registry

```bash
# Login to registry
vudo login

# Publish
vudo publish

# Publishing my-spirit v0.9.0 to registry.univrs.io...
# ✓ Published successfully!
```

### Version Updates

```bash
# Bump patch version (0.9.0 → 0.9.1)
vudo version patch

# Bump minor version (0.9.1 → 0.10.0)
vudo version minor

# Bump major version (0.10.0 → 1.0.0)
vudo version major
```

## Best Practices

### Module Organization

1. **Keep modules focused** — One concept per module
2. **Use submodules for related code** — `quantum/wavefunctions.dol`, `quantum/operators.dol`
3. **Internal helpers go in `internal/`** — Not exported from `lib.dol`
4. **Tests mirror source structure** — `tests/particles_test.dol` tests `src/particles.dol`

### Visibility

1. **Default to private** — Only expose what's needed
2. **Use `pub(spirit)` for internal APIs** — Shared within package
3. **Document all `pub` items** — Required exegesis
4. **Minimize public API surface** — Easier to maintain

### Documentation

```dol
// Every public item needs docs { }
docs {
    Brief description on first line.

    Detailed explanation follows after blank line.
    Can include:
    - Bullet points
    - Code examples
    - Mathematical formulas

    Example:
        let result = my_function(42)
}
pub fun my_function(x: i64) -> i64 {
    return x * 2
}
```

### Testing

```dol
// tests/particles_test.dol

// Group related tests
test "electron has correct mass" {
    let e = electron()
    assert_approx_eq(e.mass, 9.1093837015e-31, 1e-40)
}

test "electron has correct charge" {
    let e = electron()
    assert_approx_eq(e.charge, -1.602176634e-19, 1e-28)
}

// Test error conditions
test "divide by zero returns None" {
    assert_eq(safe_divide(10.0, 0.0), None)
}
```

## Summary

### Spirit Structure

```
my-spirit/
├── Spirit.dol          # Manifest
├── src/
│   ├── lib.dol         # Public API
│   ├── module.dol      # Modules
│   └── internal/       # Internal utilities
└── tests/
    └── module_test.dol # Tests
```

### Key Concepts

| Concept | Description |
|---------|-------------|
| Spirit | Package unit with manifest |
| Module | Single `.dol` file |
| Visibility | `pub`, `pub(spirit)`, `pub(parent)`, private |
| lib.dol | Public API entry point |

### Commands

```bash
vudo new my-spirit      # Create Spirit
vudo check              # Validate
vudo test               # Run tests
vudo build --release    # Build WASM
vudo publish            # Publish to registry
```

### Next Steps

- **[REPL Guide](repl-guide.md)** - Interactive exploration
- **[CLI Guide](cli-guide.md)** - Command-line tools
- **[WASM Guide](wasm-guide.md)** - WebAssembly compilation
