# WIT (WebAssembly Interface Types) Examples

This directory contains example WIT interface definitions generated from DOL Gen declarations with CRDT annotations.

## Files

- **chat-message.wit**: Collaborative chat message with multiple CRDT strategies
- **counter.wit**: Distributed counter using PN-Counter
- **document.wit**: Collaborative document with text editing

## Usage

These WIT files can be used with the WASM Component Model to define clean component boundaries for distributed applications.

```bash
# Validate WIT with wasm-tools
wasm-tools component wit chat-message.wit

# Generate bindings for various languages
wit-bindgen rust chat-message.wit
wit-bindgen typescript chat-message.wit
```

## Generated from DOL

These interfaces were automatically generated from DOL Gen declarations using `dol-codegen-wit`.

> **Status (2026-11):** `crates/dol-codegen-wit` is not built in CI, and it currently fails to build (see `docs/DOL-STATUS-2026-11.md` §5.9). These `.wit` files are checked-in snapshots. They cannot be regenerated with the current tree, and the DOL compiler does not emit Component Model output.

See the corresponding `.dol` files in `../crdt/` for the source declarations.
