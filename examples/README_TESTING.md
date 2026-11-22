# DOL Examples Testing Documentation

This document tracks testing procedures, fixes, and known issues for the DOL examples.

## Testing Procedures

### 1. Build DOL Toolchain

```bash
# Build with CLI feature (required for dol-parse, dol-check, dol-test)
cargo build --release --features cli
```

### 2. Parse All Examples

```bash
# Run dol-parse on all .dol files
find examples -name "*.dol" -type f -exec \
  cargo run --release --features cli --bin dol-parse -- {} \;
```

> **Caveat:** `dol-parse` and `dol-check` parse only the **first declaration** of each file and silently ignore
> the rest, including garbage (`src/parser.rs`, `Parser::parse`; see `docs/DOL-STATUS-2026-11.md` §5.1).
> A green result from them does not mean the whole file parses. For a real check, call
> `metadol::parse_file_all` on the file contents (as `dol-codegen` does).

### 3. Check All Examples (with validation)

```bash
# Run dol-check for validation and exegesis checking
cargo run --release --features cli --bin dol-check -- examples/
```

### 4. Run Full Test Suite

```bash
# Standard tests
cargo test

# With WASM compilation tests
cargo test --features wasm
```

### 5. Test DOL Test Generator

```bash
# Show what tests would be generated
cargo run --release --features cli --bin dol-test -- plan examples/container.lifecycle.dol.test

# Generate stub tests
cargo run --release --features cli --bin dol-test -- generate examples/container.lifecycle.dol.test --stubs --force
```

---

## Fixes Applied

### 2026-01-03: echo.dol - Reserved Keyword Fix

**File**: `examples/echo.dol`

**Issue**: Variable name `from` is a reserved keyword in DOL, causing parse error:
```
Parse error: expected expression, found 'from' at line 13, column 18
```

**Fix**: Renamed variable `from` to `sender_id`:
```text
// Before (line 10-13):
let from = sender()
println("Echo received: " + msg)
send(from, msg)

// After:
let sender_id = sender()
println("Echo received: " + msg)
send(sender_id, msg)
```

**Verification**:
```bash
cargo run --release --features cli --bin dol-parse -- examples/echo.dol
# Output: ✓ examples/echo.dol (EchoSpirit)
```

---

## Test Results Summary

### Examples Parsing (whole-file, dol 0.8.2, 2026-11-21)

The 2026-01-03 report of "37/37 passed" was produced with `dol-parse`, which only checks the first declaration of each file. A whole-file parse (`metadol::parse_file_all`) of every `.dol` file under `examples/` gives:

| Location | Whole-file parse |
|----------|------------------|
| `examples/*.dol` (top level) | 10/10 |
| `examples/genes/` | 10/11 |
| `examples/traits/` | 7/8 |
| `examples/constraints/` | 4/4 |
| `examples/systems/` | 4/4 |
| `examples/evolutions/` | 2/2 |
| `examples/crdt/` | 4/4 |
| `examples/native/` | 12/12 |
| `examples/v0.8/` | 1/4 |
| `examples/stdlib/` | 2/8 |
| `examples/spirits/` | 12/53 |
| `examples/minimal-test/` | 0/1 |
| **Total** | **68/121** |

Most failures come from syntax that is documented but not implemented in 0.8.2, e.g. `let mut`, `+=`, `fun f<T>`, `impl`, top-level `spirit`, `type X is enum`, and float exponents such as `1e9` (status audit §5.4). Spirit manifests (`Spirit.dol`) use a separate manifest parser (`metadol::manifest::parse_spirit_manifest`) and are not expected to pass the main parser.

### Unit Tests (cargo test)

Per the 2026-11 status audit: 2,512 tests pass with `--features cli,serde,wasm`, including doctests, with 0 failures (root crate only).

### WASM Tests

The counts previously listed here (47 total, 2026-01-03) are stale. The WASM backend has grown since then; see `docs/DOL-STATUS-2026-11.md` §3.2.

---

## Known Issues

### 1. WASM Enum Comparison Bug

**Test**: `test_enum_type_mapping`
**Location**: `tests/wasm_execution.rs:944`
**Error**: `WasmError { message: "Wasmtime error: WebAssembly translation error" }`

**Description**: Comparing enum types with equality operator (`a == b`) in a function produces invalid WASM bytecode. This is an edge case in the WASM compiler.

**Workaround**: Avoid direct enum comparison in functions compiled to WASM. Use match expressions instead:
<!-- fragment -->
```dol
// Instead of: if a == b { ... }
// Use:
match a {
    AccountType.Node => match b {
        AccountType.Node => 1,
        _ => 0,
    },
    // ...
}
```

**Status**: Reported 2026-01-03 as a known limitation. Not re-verified for 0.8.2.

---

## Reserved Keywords

The following words are keywords (`src/lexer.rs`, `keyword_kind`). Do not use them as variable names. The parser accepts some of them as identifiers in a few positions (field names, parameter names), but `let from = …` fails, for example.

| Keyword | Category |
|---------|----------|
| `from` | `derives from` |
| `in` | Loop iteration |
| `is`, `has`, `derives`, `requires`, `uses`, `emits`, `matches`, `never` | Ontology predicates |
| `if`, `else`, `match` | Control flow |
| `for`, `while`, `loop` | Loops |
| `break`, `continue`, `return` | Control flow |
| `gen`, `trait`, `rule`, `system`, `evo`, `docs`, `fun`, `type`, `const` | Declarations |
| `gene`, `constraint`, `evolves`, `exegesis` | Deprecated declaration keywords (since v0.8.0) |
| `let`, `val`, `var`, `sex`, `extern`, `mut` | Bindings and effects |
| `true`, `false`, `null` | Literals |
| `pub`, `use`, `module`, `mod` | Visibility/imports |

---

## CLI Quick Reference

| Command | Description |
|---------|-------------|
| `dol-parse <file>` | Parse DOL file, output AST summary |
| `dol-check <dir>` | Validate DOL files with semantic checks |
| `dol-check --require-exegesis <dir>` | Require `docs` (exegesis) blocks |
| `dol-check --typecheck <dir>` | Run the type checker (type errors are not enforced in 0.8.2; see the status audit §5.2) |
| `dol-test plan <file>` | Show test generation plan |
| `dol-test generate <file>` | Generate Rust tests from .dol.test |

---

## Adding New Examples

When adding new `.dol` examples:

1. **Avoid reserved keywords** as variable names
2. **Include `docs` blocks** for documentation
3. **Test parsing**: `cargo run --release --features cli --bin dol-parse -- <file>`. This checks only the first declaration; also check the whole file with `metadol::parse_file_all`
4. **Run validation**: `cargo run --release --features cli --bin dol-check -- <file>`
5. **Update this document** if fixes or workarounds are needed
