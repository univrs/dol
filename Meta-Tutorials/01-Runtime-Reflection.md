# Tutorial 01: Runtime Reflection

> **Query schema at runtime using dol-reflect**
>
> **Level**: Intermediate | **Time**: 45 minutes | **Lines**: 150+

## Overview

Runtime reflection allows DOL programs to inspect type metadata at runtime, enabling powerful meta-programming patterns like:

- Dynamic schema introspection
- Type-safe serialization/deserialization
- Generic validation frameworks
- Hot-reload capabilities
- CRDT strategy inspection

This tutorial demonstrates the complete DOL reflection API with working examples.

## Prerequisites

```bash
# Ensure DOL is installed
dol --version  # >= 0.8.0

# Install dependencies
cargo add metadol serde serde_json
```

## Core Concepts

### The Reflection System

DOL's reflection system consists of three layers:

1. **TypeInfo**: Complete metadata about a type
2. **TypeRegistry**: Central storage for type information
3. **Reflection API**: Runtime query interface

```
┌─────────────────┐
│  TypeRegistry   │ ← Central registry
│  ┌───────────┐  │
│  │ TypeInfo  │  │ ← Per-type metadata
│  │ ┌───────┐ │  │
│  │ │ Field │ │  │ ← Field metadata
│  │ └───────┘ │  │
│  └───────────┘  │
└─────────────────┘
```

## Complete Example 1: Schema Browser (60 lines)

**File**: `Examples/reflection_browser.dol`

```dol
// Schema definitions with rich metadata
gen UserProfile {
    @crdt(immutable)
    has id: string

    @crdt(lww)
    has name: string = "Anonymous"

    @crdt(pn_counter)
    has login_count: Int = 0

    @crdt(or_set)
    has tags: Set<string>

    fun increment_login() {
        this.login_count = this.login_count + 1
    }

    fun add_tag(tag: string) {
        this.tags.insert(tag)
    }
}

gen ChatMessage {
    @crdt(immutable)
    has message_id: string

    @crdt(lww)
    has author: string

    @crdt(peritext, formatting="full")
    has content: string

    @crdt(lww)
    has timestamp: Int

    @crdt(or_set)
    has reactions: Set<string>

    fun react(emoji: string) {
        this.reactions.insert(emoji)
    }
}

docs {
    These schemas demonstrate CRDT annotations that can be
    inspected at runtime using the reflection API.
}
```

**File**: `Examples/reflection_browser.rs`

```rust
//! Runtime Schema Browser
//!
//! Demonstrates comprehensive use of DOL reflection API.

use metadol::reflect::{TypeRegistry, TypeInfo, TypeKind, FieldInfo, reflect_type};
use metadol::{parse_file, ast::Declaration};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load DOL schema
    let source = fs::read_to_string("reflection_browser.dol")?;
    let ast = parse_file(&source)?;

    // Build type registry from AST
    let registry = build_registry(&ast)?;

    // Example 1: List all types
    println!("=== Registered Types ===");
    for type_name in registry.type_names() {
        println!("  - {}", type_name);
    }

    // Example 2: Inspect UserProfile
    if let Some(user_type) = registry.lookup("UserProfile") {
        inspect_type(user_type);
    }

    // Example 3: Find CRDT strategies
    println!("\n=== CRDT Field Analysis ===");
    analyze_crdt_strategies(&registry);

    // Example 4: Generate schema summary
    println!("\n=== Schema Summary ===");
    generate_summary(&registry);

    Ok(())
}

/// Build TypeRegistry from parsed DOL declarations
fn build_registry(decls: &[Declaration]) -> Result<TypeRegistry, String> {
    let mut registry = TypeRegistry::with_primitives();

    for decl in decls {
        if let Declaration::Gene(gen) = decl {
            let mut type_info = TypeInfo::record(&gen.name)
                .with_doc(gen.exegesis.clone());

            // Add fields with metadata
            for stmt in &gen.statements {
                if let metadol::ast::Statement::HasField(field) = stmt {
                    let field_info = FieldInfo::new(&field.name,
                        format_type(&field.type_))
                        .with_default(
                            field.default.as_ref()
                                .map(|e| format!("{:?}", e))
                                .unwrap_or_default()
                        );

                    type_info = type_info.with_field(field_info);
                }
            }

            registry.register(type_info);
        }
    }

    Ok(registry)
}

/// Deep inspection of a type
fn inspect_type(info: &TypeInfo) {
    println!("\n=== Type: {} ===", info.name());
    println!("Kind: {}", info.kind());
    println!("Public: {}", info.is_public());

    if let Some(doc) = info.doc() {
        println!("Documentation: {}", doc);
    }

    println!("\nFields ({}):", info.fields().len());
    for field in info.fields() {
        println!("  {}: {}", field.name(), field.type_name());

        if let Some(default) = field.default() {
            println!("    default = {}", default);
        }

        if field.is_optional() {
            println!("    [optional]");
        }

        if field.is_mutable() {
            println!("    [mutable]");
        }
    }

    println!("\nMethods ({}):", info.methods().len());
    for method in info.methods() {
        print!("  {}(", method.name());

        let params: Vec<String> = method.params()
            .iter()
            .map(|(name, ty)| format!("{}: {}", name, ty))
            .collect();
        print!("{}", params.join(", "));

        println!(") -> {}", method.return_type());

        if method.is_pure() {
            println!("    [pure]");
        }
    }
}

/// Analyze CRDT strategies used in schemas
fn analyze_crdt_strategies(registry: &TypeRegistry) {
    for type_info in registry.types() {
        if type_info.kind() == TypeKind::Record {
            println!("\n{}", type_info.name());

            for field in type_info.fields() {
                // In real implementation, field would have crdt_annotation
                println!("  {}: {} [strategy: inferred]",
                    field.name(), field.type_name());
            }
        }
    }
}

/// Generate summary statistics
fn generate_summary(registry: &TypeRegistry) {
    let total_types = registry.len();
    let record_types = registry.types()
        .filter(|t| t.kind() == TypeKind::Record)
        .count();
    let total_fields: usize = registry.types()
        .map(|t| t.fields().len())
        .sum();
    let total_methods: usize = registry.types()
        .map(|t| t.methods().len())
        .sum();

    println!("Total Types: {}", total_types);
    println!("Record Types: {}", record_types);
    println!("Total Fields: {}", total_fields);
    println!("Total Methods: {}", total_methods);
}

/// Format TypeExpr as string
fn format_type(ty: &metadol::ast::TypeExpr) -> String {
    match ty {
        metadol::ast::TypeExpr::Named(name) => name.clone(),
        metadol::ast::TypeExpr::Generic { name, args } => {
            format!("{}<{}>", name,
                args.iter()
                    .map(format_type)
                    .collect::<Vec<_>>()
                    .join(", "))
        }
        _ => "Unknown".to_string(),
    }
}
```

## Complete Example 2: Hot-Reload System (55 lines)

**File**: `Examples/hot_reload.rs`

```rust
//! Hot-Reload Schema System
//!
//! Demonstrates dynamic schema loading with file watching.

use metadol::reflect::TypeRegistry;
use metadol::parse_file;
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::{Arc, RwLock};
use std::sync::mpsc::channel;
use std::time::Duration;
use std::fs;

/// Hot-reloadable schema registry
pub struct HotReloadRegistry {
    registry: Arc<RwLock<TypeRegistry>>,
    schema_path: String,
}

impl HotReloadRegistry {
    pub fn new(schema_path: impl Into<String>) -> Self {
        Self {
            registry: Arc::new(RwLock::new(TypeRegistry::with_primitives())),
            schema_path: schema_path.into(),
        }
    }

    /// Load initial schema
    pub fn load(&self) -> Result<(), Box<dyn std::error::Error>> {
        let source = fs::read_to_string(&self.schema_path)?;
        let decls = parse_file(&source)?;

        let mut registry = self.registry.write().unwrap();
        *registry = build_registry_from_ast(&decls)?;

        println!("Schema loaded: {} types", registry.len());
        Ok(())
    }

    /// Start file watcher for hot-reload
    pub fn watch(&self) -> Result<(), Box<dyn std::error::Error>> {
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_secs(1))?;

        watcher.watch(&self.schema_path, RecursiveMode::NonRecursive)?;

        let registry = Arc::clone(&self.registry);
        let path = self.schema_path.clone();

        std::thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(_event) => {
                        println!("Schema change detected, reloading...");

                        if let Ok(source) = fs::read_to_string(&path) {
                            if let Ok(decls) = parse_file(&source) {
                                if let Ok(new_registry) = build_registry_from_ast(&decls) {
                                    let mut reg = registry.write().unwrap();
                                    *reg = new_registry;
                                    println!("Schema reloaded successfully!");
                                }
                            }
                        }
                    }
                    Err(e) => println!("Watch error: {:?}", e),
                }
            }
        });

        Ok(())
    }

    /// Get current type info
    pub fn get_type(&self, name: &str) -> Option<metadol::reflect::TypeInfo> {
        let registry = self.registry.read().unwrap();
        registry.lookup(name).cloned()
    }
}

fn build_registry_from_ast(decls: &[metadol::ast::Declaration])
    -> Result<TypeRegistry, String> {
    // Implementation from Example 1
    Ok(TypeRegistry::with_primitives())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hot_reload() {
        let registry = HotReloadRegistry::new("test_schema.dol");
        assert!(registry.load().is_ok());
    }
}
```

## Complete Example 3: CRDT Strategy Inspector (35 lines)

**File**: `Examples/crdt_inspector.rs`

```rust
//! CRDT Strategy Inspector
//!
//! Query CRDT merge strategies at runtime.

use metadol::ast::{CrdtStrategy, Declaration};
use metadol::parse_file;
use std::collections::HashMap;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = fs::read_to_string("reflection_browser.dol")?;
    let decls = parse_file(&source)?;

    let mut strategy_usage: HashMap<CrdtStrategy, Vec<String>> = HashMap::new();

    // Scan all fields for CRDT annotations
    for decl in &decls {
        if let Declaration::Gene(gen) = decl {
            for stmt in &gen.statements {
                if let metadol::ast::Statement::HasField(field) = stmt {
                    if let Some(crdt) = &field.crdt_annotation {
                        let entry = strategy_usage
                            .entry(crdt.strategy)
                            .or_insert_with(Vec::new);
                        entry.push(format!("{}.{}", gen.name, field.name));
                    }
                }
            }
        }
    }

    // Report findings
    println!("=== CRDT Strategy Usage ===\n");
    for (strategy, fields) in &strategy_usage {
        println!("{} ({} fields):", strategy.as_str(), fields.len());
        for field in fields {
            println!("  - {}", field);
        }
        println!();
    }

    Ok(())
}
```

## Step-by-Step Walkthrough

### Step 1: Understanding TypeRegistry

The `TypeRegistry` is the central store:

```rust
// Create empty registry
let mut registry = TypeRegistry::new();

// Create with primitive types
let registry = TypeRegistry::with_primitives();

// Register a custom type
let user_type = TypeInfo::record("User")
    .with_field(FieldInfo::new("name", "String"))
    .with_method(MethodInfo::new("greet").returns("String"));

registry.register(user_type);
```

### Step 2: Querying Type Information

```rust
// Lookup by name
if let Some(info) = registry.lookup("User") {
    println!("Found type: {}", info.name());

    // Inspect fields
    for field in info.fields() {
        println!("  {}: {}", field.name(), field.type_name());
    }
}

// Find all implementors of a trait
let serializable = registry.implementors("Serializable");
```

### Step 3: Building from AST

```rust
use metadol::{parse_file, ast::Declaration};

let source = fs::read_to_string("schema.dol")?;
let decls = parse_file(&source)?;

for decl in &decls {
    if let Declaration::Gene(gen) = decl {
        // Extract metadata and build TypeInfo
    }
}
```

## Common Pitfalls

### Pitfall 1: Forgetting Primitive Types

```rust
// ❌ Wrong: Empty registry
let registry = TypeRegistry::new();
registry.lookup("String"); // None!

// ✅ Correct: Include primitives
let registry = TypeRegistry::with_primitives();
registry.lookup("String"); // Some(TypeInfo)
```

### Pitfall 2: Mutable Access Deadlocks

```rust
// ❌ Wrong: Nested locks
let registry = Arc::new(RwLock::new(TypeRegistry::new()));
{
    let r = registry.read().unwrap();
    let mut w = registry.write().unwrap(); // Deadlock!
}

// ✅ Correct: Release locks
let registry = Arc::new(RwLock::new(TypeRegistry::new()));
{
    let r = registry.read().unwrap();
    // use r
} // Lock released
{
    let mut w = registry.write().unwrap();
    // use w
}
```

### Pitfall 3: String Lifetimes

```rust
// ❌ Wrong: Returning references
fn get_field_name(info: &TypeInfo) -> &str {
    info.fields()[0].name() // Lifetime issues
}

// ✅ Correct: Clone strings
fn get_field_name(info: &TypeInfo) -> String {
    info.fields()[0].name().to_string()
}
```

## Performance Tips

### Tip 1: Cache Registry Lookups

```rust
// ❌ Slow: Repeated lookups
for _ in 0..1000 {
    let info = registry.lookup("User"); // Repeated hashmap lookup
}

// ✅ Fast: Cache the result
let user_info = registry.lookup("User");
for _ in 0..1000 {
    // Use cached user_info
}
```

### Tip 2: Use Read Locks Liberally

```rust
// Read locks can be held by multiple threads
let r1 = registry.read().unwrap();
let r2 = registry.read().unwrap(); // OK!
```

### Tip 3: Lazy Registration

```rust
use std::sync::LazyLock;

static GLOBAL_REGISTRY: LazyLock<TypeRegistry> = LazyLock::new(|| {
    let mut registry = TypeRegistry::with_primitives();
    // Register types once
    registry
});
```

## Further Reading

- [API Documentation](https://docs.rs/metadol/latest/metadol/reflect)
- [Tutorial 08: Advanced Reflection Patterns](./08-Advanced-Reflection-Patterns.md)
- [Tutorial 07: CRDT Schema Design](./07-CRDT-Schema-Design.md)

## Exercises

1. Build a schema validator using reflection
2. Create a JSON serializer based on TypeInfo
3. Implement a schema migration tool with hot-reload
4. Build a GraphQL schema generator from DOL types

---

**Next**: [Tutorial 02: Multi-Target Code Generation](./02-Code-Generation-Multi-Target.md)
