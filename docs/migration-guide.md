# DOL Migration Guide - v0.7.x to v0.8.0

## Overview

The `dol-migrate` CLI tool automates the migration of DOL files from v0.7.x syntax to v0.8.0 syntax, including keyword changes, type system updates, and optional return statement modernization.

## Installation

The migration tool is built as part of the DOL crate:

```bash
cargo build --features cli --bin dol-migrate
```

## Usage

### Basic Migration

```bash
# Migrate a single file
dol-migrate 0.7-to-0.8 src/container.dol

# Migrate a directory (recursive)
dol-migrate 0.7-to-0.8 src/
```

### Preview Changes

```bash
# Dry run - preview changes without applying
dol-migrate 0.7-to-0.8 --dry-run src/

# Show diff of changes
dol-migrate 0.7-to-0.8 --diff src/
```

### Advanced Options

```bash
# Modernize return statements (remove 'return' before final expression)
dol-migrate 0.7-to-0.8 --modernize src/

# Combine options
dol-migrate 0.7-to-0.8 --dry-run --modernize src/
```

## Transformations

### 1. Keyword Changes

| v0.7.x | v0.8.0 | Description |
|--------|--------|-------------|
| `gene` | `gen` | Gene declarations |
| `constraint` | `rule` | Constraint declarations |
| `evolves` | `evo` | Evolution statements |
| `exegesis` | `docs` | Documentation blocks |

#### Example

**Before (v0.7.x):**
```dol
gene container.exists {
    container has identity
}

exegesis {
    A container is the fundamental unit.
}

constraint must_exist {
    container.identity never empty
}
```

**After (v0.8.0):**
```dol
gen container.exists {
    container has identity
}

docs {
    A container is the fundamental unit.
}

rule must_exist {
    container.identity never empty
}
```

### 2. Type System Changes

All type names are migrated to Rust-like conventions:

#### Integer Types

| v0.7.x | v0.8.0 |
|--------|--------|
| `Int8` | `i8` |
| `Int16` | `i16` |
| `Int32` | `i32` |
| `Int64` | `i64` |
| `UInt8` | `u8` |
| `UInt16` | `u16` |
| `UInt32` | `u32` |
| `UInt64` | `u64` |

#### Floating Point Types

| v0.7.x | v0.8.0 |
|--------|--------|
| `Float32` | `f32` |
| `Float64` | `f64` |

#### Other Primitive Types

| v0.7.x | v0.8.0 |
|--------|--------|
| `Bool` | `bool` |
| `String` | `string` |
| `Void` | `()` |

#### Generic Types

| v0.7.x | v0.8.0 |
|--------|--------|
| `List<T>` | `Vec<T>` |
| `Optional<T>` | `Option<T>` |

#### Example

**Before (v0.7.x):**
```dol
gen container.state {
    container has port: UInt16
    container has status: Int32
    container has uptime: Float64
    container has running: Bool
    container has name: String
    container has tags: List<String>
    container has parent: Optional<String>
}
```

**After (v0.8.0):**
```dol
gen container.state {
    container has port: u16
    container has status: i32
    container has uptime: f64
    container has running: bool
    container has name: string
    container has tags: Vec<string>
    container has parent: Option<string>
}
```

### 3. Return Statement Modernization (Optional)

When using the `--modernize` flag, the tool removes the `return` keyword before final expressions in function blocks, following Rust conventions.

**Before:**
```dol
fun get_status() -> i32 {
    return status
}

fun calculate(x: i32, y: i32) -> i32 {
    return x + y
}
```

**After (with `--modernize`):**
```dol
fun get_status() -> i32 {
    status
}

fun calculate(x: i32, y: i32) -> i32 {
    x + y
}
```

## Complete Migration Example

**Before (v0.7.x):**
```dol
gene container.exists {
    container has identity: String
    container has status: Int32
    container has ports: List<UInt16>
}

exegesis {
    A container is the fundamental unit of workload isolation.
}

constraint must_have_id {
    container.identity never empty
}

fun get_status() -> Int32 {
    return status
}
```

**After (v0.8.0 with `--modernize`):**
```dol
gen container.exists {
    container has identity: string
    container has status: i32
    container has ports: Vec<u16>
}

docs {
    A container is the fundamental unit of workload isolation.
}

rule must_have_id {
    container.identity never empty
}

fun get_status() -> i32 {
    status
}
```

## Migration Command

To migrate the above file:

```bash
# Preview changes
dol-migrate 0.7-to-0.8 --dry-run --modernize container.dol

# Apply migration
dol-migrate 0.7-to-0.8 --modernize container.dol
```

## Output Format

The migration tool provides clear, colored output:

```
→ Migrating from v0.7.x to v0.8.0
(modernizing return statements)

✓ src/container.dol
  → gene → gen
  → exegesis → docs
  → constraint → rule
  → String → string
  → Int32 → i32
  → List<T> → Vec<T>
  → UInt16 → u16
  → return <expr> → <expr> (final expression)

DONE: 1 files processed, 1 files updated
```

### Dry Run Output

```
DRY RUN: 1 files scanned, 1 would be changed
Run without --dry-run to apply changes.
```

### Diff Output

```bash
dol-migrate 0.7-to-0.8 --diff container.dol
```

```
--- src/container.dol
+++ src/container.dol
   1 - gene container.exists {
   1 + gen container.exists {
   2 -     container has identity: String
   2 +     container has identity: string
   3 -     container has status: Int32
   3 +     container has status: i32
```

## Best Practices

1. **Always use `--dry-run` first** to preview changes before applying them
2. **Use version control** - commit your files before running migration
3. **Review the diff** - use `--diff` to see exactly what will change
4. **Test after migration** - run `dol-check` and your test suite after migration
5. **Modernize return statements** - use `--modernize` for cleaner, more idiomatic code

## Troubleshooting

### No Changes Detected

If the tool reports "no changes", your files may already be using v0.8.0 syntax or don't contain any transformable patterns.

### Partial Matches

The tool uses word boundaries (`\b`) in regex patterns to avoid partial matches. For example, `gene_name` will not be changed to `gen_name`.

### Complex Return Statements

The `--modernize` flag uses regex-based detection which may not handle all complex cases. Review the changes carefully, especially for:
- Multi-line return statements
- Return statements with complex expressions
- Nested blocks with multiple return points

## Legacy Migration (v0.2 to v0.3)

The tool also supports migrating from v0.2 to v0.3 for legacy codebases:

```bash
dol-migrate 0.2-to-0.3 old_code/
```

See the help output for details:
```bash
dol-migrate --help
```

## Testing

The migration tool includes comprehensive tests covering all transformation rules:

```bash
cargo test --features cli --bin dol-migrate
```

All 17 tests pass successfully, covering:
- Keyword transformations (gene, constraint, evolves, exegesis)
- All type transformations (integers, floats, primitives, generics)
- Return statement modernization
- Edge cases (partial matches, nested generics)
- Full integration scenarios
