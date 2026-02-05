# dol-codegen-rust

Automerge-backed Rust code generation for DOL (Design Ontology Language).

## Overview

This crate transforms DOL declarations with `@crdt(...)` annotations into Rust code with [Automerge](https://automerge.org/) CRDT backing, enabling local-first, collaborative applications.

## Features

- **CRDT-Backed Structs**: Generate Rust structs with automatic conflict resolution
- **Type-Safe Merge Strategies**: Map DOL annotations to Automerge data types
- **WASM Bindings**: Generate JavaScript/TypeScript bindings for browser use
- **Constraint Enforcement**: Validate DOL rules during merge operations (planned)

## Supported CRDT Strategies

| DOL Annotation | Automerge Type | Use Case |
|----------------|----------------|----------|
| `@crdt(immutable)` | Immutable | Set-once fields (IDs, timestamps) |
| `@crdt(lww)` | Last-Write-Wins | Simple values (usernames, settings) |
| `@crdt(peritext)` | Rich Text | Collaborative text editing |
| `@crdt(or_set)` | Observed-Remove Set | Tags, reactions, votes |
| `@crdt(pn_counter)` | PN-Counter | Distributed counters (likes, views) |
| `@crdt(rga)` | Replicated Growable Array | Ordered lists, paragraphs |
| `@crdt(mv_register)` | Multi-Value Register | Conflict preservation |

## Example

**Input DOL:**

```dol
gen ChatMessage {
  @crdt(immutable)
  message has id: String

  @crdt(peritext)
  message has content: String

  @crdt(or_set)
  message has reactions: Set<String>

  @crdt(pn_counter)
  message has likes: i64
}

exegesis {
  A collaborative chat message with reactions and likes.
}
```

**Generated Rust:**

```rust
#[derive(Debug, Clone, Reconcile, Hydrate)]
pub struct ChatMessage {
    #[autosurgeon(immutable)]
    pub id: String,

    #[autosurgeon(text)]
    pub content: String,

    #[autosurgeon(set)]
    pub reactions: HashSet<String>,

    #[autosurgeon(counter)]
    pub likes: i64,
}

impl ChatMessage {
    pub fn merge(&mut self, other: &Self) -> Result<(), ReconcileError> {
        autosurgeon::reconcile(self, other)
    }

    pub fn from_automerge(doc: &Automerge) -> Result<Self, HydrateError> {
        autosurgeon::hydrate(doc)
    }

    pub fn to_automerge(&self) -> Result<Automerge, ReconcileError> {
        let mut doc = Automerge::new();
        autosurgeon::reconcile(&mut doc, self)?;
        Ok(doc)
    }
}
```

## Usage

### In Rust

```rust
use dol_codegen_rust::{generate_rust, CodegenOptions, Target};
use dol::parse_file;

let source = std::fs::read_to_string("chat.dol")?;
let file = parse_file(&source)?;

let options = CodegenOptions {
    target: Target::AutomergeRust,
    derive_serde: true,
    ..Default::default()
};

let code = generate_rust(&file, &options)?;
std::fs::write("generated.rs", code)?;
```

### WASM Target

Enable the `wasm` feature to generate JavaScript bindings:

```toml
[dependencies]
dol-codegen-rust = { version = "0.1", features = ["wasm"] }
```

```rust
let options = CodegenOptions {
    target: Target::Wasm,
    ..Default::default()
};

let code = generate_rust(&file, &options)?;
```

### Build Script

Use the generated code in a build script:

```bash
#!/bin/bash
# build-wasm.sh
wasm-pack build --target web --release
wasm-opt -Oz -o pkg/output_bg.wasm pkg/output_bg.wasm
```

## Architecture

- **`automerge_backend.rs`**: Core Automerge struct generation
- **`wasm_bindings.rs`**: WASM/JavaScript wrapper generation
- **`type_mapper.rs`**: DOL → Rust type conversion
- **`templates/`**: Handlebars templates for code generation

## Dependencies

- [`automerge`](https://crates.io/crates/automerge): CRDT implementation
- [`autosurgeon`](https://crates.io/crates/autosurgeon): Rust derive macros for Automerge
- [`wasm-bindgen`](https://crates.io/crates/wasm-bindgen): WASM bindings (optional)
- [`quote`](https://crates.io/crates/quote): Rust code generation
- [`syn`](https://crates.io/crates/syn): Rust parsing

## Testing

```bash
cargo test
cargo test --features wasm
```

## License

MIT OR Apache-2.0
