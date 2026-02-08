# DOL Reflect

Runtime reflection API for DOL schemas with hot-reload support and CRDT introspection.

## Features

- **Schema Reflection API**: Query Gen structures, fields, constraints at runtime
- **Dynamic Loading**: Load .dol files at runtime with hot-reload support
- **CRDT Introspection**: Analyze CRDT strategies and compatibility
- **Type-safe API**: No stringly-typed operations
- **High Performance**: < 1ms reflection queries

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
dol-reflect = "0.1.0"
```

## Quick Start

### Basic Reflection

```rust
use dol_reflect::schema_api::SchemaRegistry;

let mut registry = SchemaRegistry::new();
registry.load_schema(r#"
gen user.profile {
  user has name: String
  user has age: Int32
}

exegesis { User profile schema }
"#)?;

// Query the schema
let gen = registry.get_gen("user.profile").unwrap();
println!("Gen: {}", gen.name());
for field in gen.fields() {
    println!("  Field: {} : {}", field.name(), field.type_name());
}
```

### Dynamic Loading with Hot-Reload

```rust
use dol_reflect::dynamic_load::SchemaLoader;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut loader = SchemaLoader::new();

    // Load all schemas from a directory
    loader.load_directory(Path::new("schemas/")).await?;

    // Enable hot-reload
    let (watcher, mut rx) = loader.watch_directory(Path::new("schemas/")).await?;

    // React to schema changes
    while let Some(event) = rx.recv().await {
        match event {
            SchemaEvent::Modified { path, .. } => {
                println!("Schema modified: {}", path.display());
            }
            _ => {}
        }
    }

    Ok(())
}
```

### CRDT Introspection

```rust
use dol_reflect::crdt_introspection::{CrdtIntrospector, MergeSemantics};
use dol::CrdtStrategy;

let mut introspector = CrdtIntrospector::new();

// Check merge semantics
let semantics = MergeSemantics::for_strategy(CrdtStrategy::Lww);
assert!(semantics.is_commutative());
assert!(semantics.is_sec()); // Strong Eventual Consistency

// Get recommended strategy for a type
let strategy = introspector.recommend_strategy("Set<String>");
assert_eq!(strategy, Some(CrdtStrategy::OrSet));

// Validate CRDT annotations
let schema = r#"
gen chat.message {
  @crdt(immutable)
  message has id: String

  @crdt(peritext)
  message has content: String
}

exegesis { Chat message }
"#;

let mut registry = SchemaRegistry::new();
registry.load_schema(schema)?;

let gen = registry.get_gen("chat.message").unwrap();
let analyses = introspector.analyze_gen(gen);

for analysis in analyses {
    println!("Field: {}", analysis.field_name);
    println!("  Strategy: {:?}", analysis.strategy);
    println!("  SEC: {}", analysis.semantics.is_sec());
}
```

## Architecture

The reflection system consists of three main components:

1. **SchemaRegistry**: Central registry for parsed schemas with fast lookup
2. **SchemaLoader**: Dynamic loading with file watching and hot-reload
3. **CrdtIntrospector**: CRDT analysis, validation, and recommendations

```text
┌─────────────────────────────────────────┐
│         Application Code                │
└─────────────────────────────────────────┘
             │         │         │
             ▼         ▼         ▼
   ┌─────────────┬──────────┬────────────┐
   │   Schema    │ Dynamic  │   CRDT     │
   │  Registry   │  Loader  │Introspector│
   └─────────────┴──────────┴────────────┘
             │         │         │
             └─────────┼─────────┘
                       ▼
             ┌──────────────────┐
             │   DOL Parser     │
             │   (metadol)      │
             └──────────────────┘
```

## Performance

All reflection queries complete in < 1ms for typical schemas:

- `get_gen()`: ~10-50 µs (microseconds)
- `get_field()`: ~5-20 µs
- `analyze_field()`: ~50-200 µs
- `load_schema()`: Depends on file size (~1-10ms for typical schemas)

## Use Cases

### 1. Runtime Schema Validation

```rust
// Load schema and validate data at runtime
let mut registry = SchemaRegistry::new();
registry.load_schema(&schema_source)?;

let gen = registry.get_gen("user.profile")?;
for field in gen.fields() {
    // Validate field against runtime data
    validate_field(field, &user_data)?;
}
```

### 2. Schema Migration

```rust
// Compare schema versions and migrate data
let mut loader = SchemaLoader::new();
loader.load_file(Path::new("schema_v1.dol")).await?;
let v1_registry = loader.registry();

loader.clear().await;
loader.load_file(Path::new("schema_v2.dol")).await?;
let v2_registry = loader.registry();

// Compare and migrate...
```

### 3. CRDT Configuration

```rust
// Analyze and recommend CRDT strategies
let mut introspector = CrdtIntrospector::new();

for gen in registry.gens() {
    for field in gen.fields() {
        if let Some(strategy) = field.crdt_strategy() {
            let analysis = introspector.analyze_field(field)?;
            if !analysis.compatible {
                eprintln!("Warning: {} uses incompatible CRDT strategy", field.name());
            }
        } else {
            // Recommend a strategy
            let recommended = introspector.recommend_strategy(field.type_name());
            println!("Recommended CRDT for {}: {:?}", field.name(), recommended);
        }
    }
}
```

### 4. Documentation Generation

```rust
// Generate documentation from schemas
let registry = SchemaRegistry::new();
registry.load_schema(&schema)?;

for gen in registry.gens() {
    println!("## {}", gen.name());
    println!("{}", gen.exegesis());
    println!("\n### Fields\n");
    for field in gen.fields() {
        println!("- `{}`: {}", field.name(), field.type_name());
    }
}
```

## Testing

Run the test suite:

```bash
cargo test
```

Run with benchmarks:

```bash
cargo test --release
```

## License

MIT OR Apache-2.0

## Contributing

Contributions are welcome! Please see the main DOL repository for contribution guidelines.
