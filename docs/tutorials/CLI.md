# VUDO CLI

The `vudo` command-line tool for working with DOL spirits.

## Installation

```bash
# From source
cargo install --path . --features vudo

# Or build locally
cargo build --bin vudo --features vudo --release
```

## Commands

### Run a Spirit

Execute a compiled WASM spirit:

```bash
# Run with default function
vudo run myspirit.wasm

# Call specific function
vudo run counter.wasm -f add_numbers -a '[3, 4]'

# Call gene method with pointer argument
vudo run counter.wasm -f Counter.increment -a '[1024]'

# Verbose output
vudo run myspirit.wasm -v

# Enable execution tracing
vudo run myspirit.wasm --trace
```

**Options:**
- `-f, --function <name>` - Function to call (default: main or first export)
- `-a, --args <json>` - Arguments as JSON array
- `--memory <pages>` - Initial memory pages (default: 16 = 1MB)
- `--trace` - Enable execution tracing

### Compile DOL to WASM

Compile DOL source files to WASM:

```bash
# Basic compilation (outputs counter.wasm)
vudo compile counter.dol

# Specify output file
vudo compile counter.dol -o build/counter.wasm

# With optimization
vudo compile counter.dol --optimize

# Verbose output
vudo compile counter.dol -v
```

**Options:**
- `-o, --output <file>` - Output file (default: `<input>.wasm`)
- `--optimize` - Enable optimization
- `--debug` - Include debug info

### Type Check

Check DOL files for errors:

```bash
# Check single file
vudo check counter.dol

# Check multiple files
vudo check src/*.dol

# Check directory recursively
vudo check src/ -r

# Strict mode (warnings are errors)
vudo check src/ --strict

# JSON output for tooling
vudo check src/ --json
```

**Options:**
- `-r, --recursive` - Recursively check directories
- `--strict` - Treat warnings as errors
- `--json` - Output as JSON

## Global Options

These options work with all commands:

- `-v, --verbose` - Enable verbose output
- `-q, --quiet` - Suppress non-error output
- `-h, --help` - Print help
- `-V, --version` - Print version

## Examples

### Full Development Workflow

```bash
# 1. Create a DOL file
cat > counter.dol << 'EOF'
gene Counter {
    has value: Int64

    fun increment() -> Int64 {
        return value + 1
    }

    fun add(n: Int64) -> Int64 {
        return value + n
    }
}

fun add_numbers(a: Int64, b: Int64) -> Int64 {
    return a + b
}
EOF

# 2. Check it compiles
vudo check counter.dol

# 3. Compile to WASM
vudo compile counter.dol -o counter.wasm

# 4. Test standalone function
vudo run counter.wasm -f add_numbers -a '[3, 4]'
# Output: 7

# 5. Test gene method
vudo run counter.wasm -f Counter.increment -a '[0]'
# Output: 1
```

### CI Integration

```bash
# Check all DOL files strictly
vudo check src/ -r --strict

# Compile all to WASM
for f in src/*.dol; do
    vudo compile "$f" -o "build/$(basename "$f" .dol).wasm"
done
```

### JSON Output for Tooling

```bash
vudo check src/ --json
# Output:
# {
#   "passed": 5,
#   "failed": 1,
#   "errors": [
#     {"file": "src/broken.dol", "error": "Parse error: ..."}
#   ]
# }
```

## Exit Codes

- `0` - Success
- `1` - Error (compilation failed, file not found, etc.)

## Feature Requirements

The `vudo` binary requires the `vudo` feature, which enables both `cli` and `wasm`:

```bash
cargo build --features vudo
```
