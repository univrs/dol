# dol-codegen

Template-based code generation framework for DOL (Design Ontology Language).

## Overview

`dol-codegen` provides a flexible, extensible framework for generating code from DOL declarations. It supports multiple target languages and includes a powerful transformation pipeline for AST manipulation.

## Features

### M2.1: Template Engine
- **Handlebars & Tera Support**: Dual template engine support for maximum flexibility
- **AST-to-Template Mapping**: Automatic conversion of DOL AST to template-friendly data structures
- **Custom Helpers**: DOL-specific template helpers for case conversion, type mapping, etc.
- **Hot-Reloadable Templates**: Templates stored in separate `.hbs` files for easy customization

### M2.2: AST Transformation Pipeline
- **Visitor Pattern**: Clean, extensible visitor pattern for AST traversal
- **Type Inference**: Automatic type inference and elaboration
- **CRDT Expansion**: Automatic expansion of CRDT annotations into implementation code
- **Custom Transformations**: Easy to add custom transformation passes

### M2.3: Multi-Target Generation
- **Rust**: Idiomatic Rust with serde, builder patterns, and documentation
- **TypeScript**: Type-safe TypeScript with interfaces and classes
- **WIT**: WebAssembly Interface Types for component model
- **Python**: Python dataclasses with type hints
- **JSON Schema**: JSON Schema Draft 2020-12 with CRDT metadata

## Usage

### Basic Code Generation

```rust
use dol_codegen::{generate, CodegenContext, Target};
use dol::ast::DolFile;

// Parse or construct a DOL file
let file: DolFile = /* ... */;

// Create a codegen context
let context = CodegenContext::new(Target::Rust)
    .with_docs(true)
    .with_module_name("my_module");

// Generate code
let code = generate(&file, &context)?;
println!("{}", code);
```

### Multi-Target Generation

```rust
use dol_codegen::{generate, CodegenContext, Target};

for target in [Target::Rust, Target::TypeScript, Target::Python] {
    let context = CodegenContext::new(target);
    let code = generate(&file, &context)?;

    let filename = format!("output.{}", target.extension());
    std::fs::write(filename, code)?;
}
```

### Custom Transformations

```rust
use dol_codegen::transforms::{Visitor, TransformPipeline};

// Create a custom visitor
struct MyCustomVisitor;

impl Visitor for MyCustomVisitor {
    fn visit_gen(&mut self, gen: &mut Gen) -> Result<()> {
        // Custom transformation logic
        Ok(())
    }
}

// Add to pipeline
let mut pipeline = TransformPipeline::new()
    .add_visitor(Box::new(MyCustomVisitor));

pipeline.execute(&mut file)?;
```

## Architecture

```
dol-codegen/
├── src/
│   ├── lib.rs              # Main entry point
│   ├── template_engine.rs  # Template engine implementation
│   ├── transforms.rs       # AST transformation pipeline
│   └── targets/            # Target-specific generators
│       ├── rust.rs
│       ├── typescript.rs
│       ├── wit.rs
│       ├── python.rs
│       └── json_schema.rs
├── templates/              # Handlebars templates
│   ├── rust_module.hbs
│   ├── typescript_module.hbs
│   ├── wit_module.hbs
│   ├── python_module.hbs
│   └── json_schema.hbs
└── tests/                  # Integration tests
    ├── template_engine_tests.rs
    ├── transforms_tests.rs
    ├── targets_tests.rs
    └── integration_tests.rs
```

## Target Languages

### Rust
- Generates idiomatic Rust structs with derives
- Optional builder pattern generation
- Serde support for serialization
- Full documentation comments
- CRDT annotations as compile-time metadata

### TypeScript
- Interfaces and class implementations
- Full type safety with generics
- JSDoc comments
- Optional/null handling
- Compatible with modern TypeScript (5.0+)

### WIT (WebAssembly Interface Types)
- Component model records
- Proper type mapping (s32, f64, string, etc.)
- Interface definitions
- Package declarations

### Python
- Python 3.10+ dataclasses
- Full type hints (PEP 484)
- Support for List, Optional, Dict, Set, Tuple
- Docstrings from exegesis
- Compatible with mypy

### JSON Schema
- JSON Schema Draft 2020-12
- Full validation rules
- CRDT metadata in extensions
- GDPR annotations for personal data
- Proper handling of arrays, objects, tuples

## Type Mapping

| DOL Type | Rust | TypeScript | WIT | Python | JSON Schema |
|----------|------|------------|-----|--------|-------------|
| String | String | string | string | str | string |
| i32 | i32 | number | s32 | int | integer |
| f64 | f64 | number | f64 | float | number |
| bool | bool | boolean | bool | bool | boolean |
| Vec\<T\> | Vec\<T\> | T[] | list\<T\> | List[T] | array |
| Option\<T\> | Option\<T\> | T \| null | option\<T\> | Optional[T] | anyOf |
| Map\<K,V\> | HashMap\<K,V\> | Map\<K,V\> | - | Dict[K,V] | object |

## Testing

Run tests with:
```bash
cargo test
```

Run tests with coverage:
```bash
cargo test --all-features
```

## Performance

The codegen pipeline is designed for efficiency:
- AST transformations are in-place where possible
- Template compilation is cached
- Parallel code generation for multiple targets
- Zero-copy template data where feasible

## Extension Points

### Custom Template Helpers

```rust
use handlebars::Handlebars;

let mut hb = Handlebars::new();
hb.register_helper("my_helper", Box::new(my_helper_fn));
```

### Custom Target Languages

Implement the target generation interface:

```rust
pub fn generate_my_language(
    file: &DolFile,
    context: &CodegenContext
) -> Result<String> {
    // Custom generation logic
    Ok(generated_code)
}
```

### Custom AST Visitors

Implement the `Visitor` trait:

```rust
impl Visitor for MyVisitor {
    fn visit_gen(&mut self, gen: &mut Gen) -> Result<()> {
        // Custom logic
        Ok(())
    }
}
```

## License

MIT OR Apache-2.0
