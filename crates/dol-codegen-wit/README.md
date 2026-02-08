# dol-codegen-wit

WIT (WebAssembly Interface Types) generator for DOL Gen declarations with CRDT annotations.

## Overview

This crate transforms DOL Gen declarations into WIT interface definitions for the WASM Component Model, enabling clean component boundaries for distributed applications with CRDT-based conflict resolution.

## Features

- **CRDT-Aware Generation**: Preserves `@crdt(...)` annotations as documentation comments
- **Type Mapping**: Maps DOL types to WIT types (including collections, options, results)
- **Merge Functions**: Generates CRDT merge functions for distributed synchronization
- **Serialization**: Generates to-bytes/from-bytes functions for Automerge format
- **Validation**: Ensures generated WIT validates with `wasm-tools component wit`

## Type Mapping

| DOL Type | WIT Type | Notes |
|----------|----------|-------|
| `String` | `string` | |
| `Bool` | `bool` | |
| `Int`, `i32` | `s32` | Signed 32-bit |
| `i64` | `s64` | Signed 64-bit |
| `u32` | `u32` | Unsigned 32-bit |
| `u64` | `u64` | Unsigned 64-bit |
| `f32` | `f32` | 32-bit float |
| `f64` | `f64` | 64-bit float |
| `Option<T>` | `option<T>` | |
| `Result<T, E>` | `result<T, E>` | |
| `List<T>`, `Vec<T>` | `list<T>` | |
| `Set<T>` | `list<T>` | WIT has no native set |
| `Map<K, V>` | `list<tuple<K, V>>` | WIT has no native map |
| `(T1, T2, ...)` | `tuple<T1, T2, ...>` | |

## CRDT Strategy Support

All 7 CRDT strategies are supported and preserved as documentation:

- `@crdt(immutable)` - Immutable values set exactly once
- `@crdt(lww)` - Last-Write-Wins for single values
- `@crdt(or_set)` - Observed-Remove Set for collections
- `@crdt(pn_counter)` - Positive-Negative Counter for numeric values
- `@crdt(peritext)` - Rich text CRDT for collaborative editing
- `@crdt(rga)` - Replicated Growable Array for ordered sequences
- `@crdt(mv_register)` - Multi-Value Register for concurrent values

## Usage

```rust
use dol_codegen_wit::{generate_wit, WitOptions};
use metadol::ast::{Declaration, DolFile, Gen};

// Create or parse a DOL file
let file: DolFile = /* ... */;

// Configure generation options
let mut options = WitOptions::default();
options.package_name = Some("univrs:my-app".to_string());
options.package_version = Some("1.0.0".to_string());
options.generate_merge_functions = true;
options.generate_serialization = true;

// Generate WIT
let wit_code = generate_wit(&file, &options)?;

// Write to file
std::fs::write("output.wit", wit_code)?;
```

## Example Output

### Input DOL

```dol
gen ChatMessage {
  @crdt(immutable)
  has id: String

  @crdt(peritext)
  has content: String

  @crdt(or_set)
  has reactions: Set<String>

  @crdt(lww)
  has edited_at: Option<i64>
}

docs {
  A collaborative chat message with CRDT merge strategies.
}
```

### Generated WIT

```wit
package univrs:chat-message@1.0.0;

interface chat-message {
  /// A collaborative chat message with CRDT merge strategies.
  record chat-message {
    /// @crdt(immutable)
    id: string,
    /// @crdt(peritext)
    content: string,
    /// @crdt(or_set)
    reactions: list<string>,
    /// @crdt(lww)
    edited-at: option<s64>,
  }

  /// Merge two chat-message instances using CRDT semantics
  merge: func(a: chat-message, b: chat-message) -> chat-message;

  /// Serialize chat-message to bytes (Automerge format)
  to-bytes: func(value: chat-message) -> list<u8>;

  /// Deserialize chat-message from bytes (Automerge format)
  from-bytes: func(data: list<u8>) -> result<chat-message, string>;
}
```

## Validation

Generated WIT files can be validated with `wasm-tools`:

```bash
wasm-tools component wit output.wit
```

All generated WIT files are guaranteed to validate successfully.

## Examples

See the `/examples/wit/` directory for complete examples:

- `chat-message.wit` - Collaborative chat with multiple CRDT strategies
- `counter.wit` - Distributed counter using PN-Counter
- `document.wit` - Collaborative document with rich text editing

## Component Model Integration

The generated WIT interfaces can be used with:

- **WASM Component Model**: Define clean component boundaries
- **wit-bindgen**: Generate language bindings (Rust, TypeScript, Python, etc.)
- **wasmtime**: Execute components with proper interface contracts

```bash
# Generate Rust bindings
wit-bindgen rust output.wit

# Generate TypeScript bindings
wit-bindgen typescript output.wit
```

## API Reference

### `generate_wit(file: &DolFile, options: &WitOptions) -> Result<String, WitError>`

Generate WIT code from a complete DOL file with all Gen declarations.

### `generate_gen_interface(gen: &Gen, options: &WitOptions) -> Result<String, WitError>`

Generate WIT interface for a single Gen declaration.

### `generate_world(file: &DolFile, world_name: &str, options: &WitOptions) -> Result<String, WitError>`

Generate a WIT world declaration that exports all interfaces.

### `WitOptions`

Configuration for WIT generation:

- `package_name: Option<String>` - Package name (e.g., "univrs:my-app")
- `package_version: Option<String>` - Package version (default: "1.0.0")
- `include_docs: bool` - Include documentation comments (default: true)
- `generate_merge_functions: bool` - Generate CRDT merge functions (default: true)
- `generate_serialization: bool` - Generate to-bytes/from-bytes (default: true)

## Testing

The crate includes comprehensive tests covering:

- All 7 CRDT strategies
- Complex nested types (Option<List<T>>, Map<K,V>)
- All integer types (i8, i16, i32, i64, u8, u16, u32, u64)
- Documentation generation
- Kebab-case conversion
- Interface and world generation

Run tests:

```bash
cargo test
```

All tests include validation that generated WIT is valid.

## License

MIT OR Apache-2.0
