# DOL CLI Guide

> Command-line tools for parsing, validating, and building DOL projects

## Overview

DOL provides three primary CLI tools for working with DOL source files:

| Tool | Purpose |
|------|---------|
| `dol-parse` | Parse DOL files and output AST as JSON |
| `dol-check` | Validate DOL files and check for errors |
| `vudo` | Full-featured development environment |

## Installation

### From Cargo

```bash
# Install from crates.io
cargo install dol-tools

# Or build from source
git clone https://github.com/univrs/dol
cd dol
cargo install --path .
```

### Verify Installation

```bash
dol-parse --version
# dol-parse 0.9.0

dol-check --version
# dol-check 0.9.0

vudo --version
# vudo 0.9.0
```

## dol-parse: AST Generation

The `dol-parse` tool parses DOL source files and outputs the Abstract Syntax Tree (AST) in JSON format.

### Basic Usage

```bash
# Parse a single file
dol-parse examples/spirits/physics/Spirit.dol

# Parse and save to file
dol-parse examples/spirits/physics/Spirit.dol > physics-ast.json

# Parse with pretty printing (default)
dol-parse --pretty examples/spirits/physics/particles.dol

# Parse with compact JSON
dol-parse --compact examples/spirits/physics/particles.dol
```

### Command Options

```bash
dol-parse [OPTIONS] <FILE>

Arguments:
  <FILE>    The DOL file to parse

Options:
  -o, --output <FILE>    Write output to file instead of stdout
  -f, --format <FMT>     Output format: json (default), yaml, ron
      --pretty           Pretty-print output (default)
      --compact          Compact output (no whitespace)
      --include-spans    Include source location spans in AST
      --include-docs     Include doc comments in AST
  -h, --help             Print help
  -V, --version          Print version
```

### Example: Parsing a Physics Module

```bash
$ dol-parse examples/spirits/physics/particles.dol --include-spans

{
  "module": {
    "name": "physics.particles",
    "version": "0.9.0",
    "span": { "start": 0, "end": 45 }
  },
  "declarations": [
    {
      "type": "Gene",
      "name": "Particle",
      "visibility": "pub",
      "fields": [
        {
          "name": "name",
          "type": "string",
          "span": { "start": 120, "end": 132 }
        },
        {
          "name": "symbol",
          "type": "string"
        },
        {
          "name": "mass",
          "type": "f64"
        },
        {
          "name": "charge",
          "type": "f64"
        },
        {
          "name": "spin",
          "type": "f64"
        }
      ]
    },
    {
      "type": "Function",
      "name": "electron",
      "visibility": "pub",
      "params": [],
      "return_type": "Particle"
    }
  ]
}
```

### Parsing Entire Spirits

```bash
# Parse all files in a Spirit
dol-parse --recursive examples/spirits/physics/

# Output combined AST
dol-parse --recursive --merge examples/spirits/physics/ > physics-complete.json
```

## dol-check: Validation

The `dol-check` tool validates DOL files for syntax errors, type errors, and constraint violations.

### Basic Usage

```bash
# Check a single file
dol-check examples/spirits/physics/Spirit.dol

# Check an entire Spirit
dol-check examples/spirits/physics/

# Check with specific rules
dol-check --require-docs examples/spirits/physics/

# Check for CI (exit code 1 on errors)
dol-check --strict examples/spirits/physics/
```

### Command Options

```bash
dol-check [OPTIONS] <PATH>

Arguments:
  <PATH>    File or directory to check

Options:
      --strict           Exit with error code 1 on any warning
      --require-docs     Require doc comments on all public items
      --require-tests    Require test files for all modules
      --allow <RULE>     Allow specific warnings (can be repeated)
      --deny <RULE>      Treat specific warnings as errors
  -j, --jobs <N>         Number of parallel jobs (default: CPU count)
      --format <FMT>     Output format: human (default), json, sarif
  -h, --help             Print help
  -V, --version          Print version
```

### Example: Validating a Spirit

```bash
$ dol-check examples/spirits/chemistry/

Checking chemistry Spirit...

✓ Spirit.dol
✓ lib.dol
✓ elements.dol (1153 lines)
✓ reactions.dol (892 lines)
✓ bonds.dol (654 lines)

Summary:
  Files checked: 5
  Total lines: 3142
  Errors: 0
  Warnings: 2

Warnings:
  elements.dol:445:1: warning[unused-function]: `internal_helper` is never used
  reactions.dol:203:5: warning[missing-docs]: Missing documentation for `balance_equation`

All checks passed!
```

### Checking for Exegesis Coverage

DOL requires exegesis (documentation) for all public declarations:

```bash
$ dol-check --require-docs examples/spirits/biology/

error[missing-exegesis]: Public gene `DNA` is missing exegesis
  --> examples/spirits/biology/genetics.dol:92:1
   |
92 | pub gen DNA {
   | ^^^^^^^^^^^^ add `docs { ... }` before this declaration
   |
   = help: Add exegesis describing what this gene represents

error: Found 3 errors, aborting
```

### CI Integration

```yaml
# .github/workflows/ci.yml
name: DOL CI

on: [push, pull_request]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: univrs/dol-action@v1

      - name: Check DOL files
        run: |
          dol-check --strict --require-docs examples/

      - name: Validate Spirits
        run: |
          for spirit in examples/spirits/*/; do
            echo "Checking $spirit..."
            dol-check --strict "$spirit"
          done
```

### Error Codes

| Code | Name | Description |
|------|------|-------------|
| `E001` | `syntax-error` | Invalid DOL syntax |
| `E002` | `unknown-type` | Reference to undefined type |
| `E003` | `type-mismatch` | Type incompatibility |
| `E004` | `missing-field` | Required field not provided |
| `E005` | `constraint-violation` | Constraint not satisfied |
| `W001` | `unused-function` | Function defined but never called |
| `W002` | `missing-docs` | Missing documentation |
| `W003` | `deprecated` | Using deprecated feature |

## vudo: Development Environment

The `vudo` CLI provides a full-featured development environment for DOL projects.

### Project Initialization

```bash
# Create a new Spirit project
vudo new my-spirit

# Create with template
vudo new my-spirit --template library
vudo new my-spirit --template service
vudo new my-spirit --template cli

# Initialize in existing directory
vudo init
```

### Project Structure

```bash
$ vudo new physics-spirit --template library
Creating Spirit: physics-spirit

physics-spirit/
├── Spirit.dol           # Package manifest
├── src/
│   └── lib.dol          # Library entry point
└── tests/
    └── lib_test.dol     # Test file

Spirit created! Run `cd physics-spirit && vudo build` to get started.
```

### Building Spirits

```bash
# Build current Spirit
vudo build

# Build with release optimizations
vudo build --release

# Build specific target
vudo build --target wasm32-unknown-unknown

# Build and show generated Rust
vudo build --emit rust
```

### Running Tests

```bash
# Run all tests
vudo test

# Run specific test file
vudo test tests/particles_test.dol

# Run tests matching pattern
vudo test --filter "particle"

# Run tests with coverage
vudo test --coverage
```

### Development Server

```bash
# Start REPL with current Spirit loaded
vudo repl

# Start web development server
vudo serve
# => Server running at http://localhost:3000

# Start with hot reloading
vudo serve --watch
```

### Spirit Management

```bash
# Add a dependency
vudo add @univrs/std
vudo add @univrs/http --version "^2.0"

# Remove a dependency
vudo remove @univrs/http

# Update dependencies
vudo update

# Show dependency tree
vudo tree
```

### Example: Complete Workflow

```bash
# 1. Create a new Spirit
$ vudo new mechanics --template library
Creating Spirit: mechanics

# 2. Navigate to project
$ cd mechanics

# 3. Edit Spirit.dol
$ cat Spirit.dol
spirit Mechanics {
    has name: "mechanics"
    has version: "0.1.0"
    has lib: "src/lib.dol"

    requires @univrs/physics-constants: "^0.9"

    docs {
        Classical mechanics calculations.
    }
}

# 4. Write some code
$ cat src/lib.dol
use @univrs/physics-constants::GRAVITATIONAL_CONSTANT as G

docs {
    Calculate gravitational force between two masses.
}
pub fun gravitational_force(m1: f64, m2: f64, r: f64) -> f64 {
    return G * m1 * m2 / (r * r)
}

docs {
    Calculate orbital velocity for circular orbit.
}
pub fun orbital_velocity(central_mass: f64, radius: f64) -> f64 {
    return (G * central_mass / radius).sqrt()
}

# 5. Check for errors
$ vudo check
✓ All checks passed

# 6. Run tests
$ vudo test
Running tests...
  ✓ gravitational_force_test (2 assertions)
  ✓ orbital_velocity_test (3 assertions)

All tests passed!

# 7. Build
$ vudo build --release
   Compiling mechanics v0.1.0
   Generating Rust code...
   Compiling to WASM...
    Finished release [optimized] target(s)

Output: target/wasm32-unknown-unknown/release/mechanics.wasm (12.4 KB)

# 8. Publish (optional)
$ vudo publish
Publishing mechanics v0.1.0 to registry.univrs.io...
✓ Published successfully!
```

## Working with Multiple Spirits

### Workspace Configuration

Create a workspace to manage multiple related Spirits:

```bash
# Create workspace
$ vudo workspace new science-spirits

# Add Spirits to workspace
$ cd science-spirits
$ vudo new physics
$ vudo new chemistry
$ vudo new biology
```

```dol
// Workspace.dol
workspace ScienceSpirits {
    has members: [
        "physics",
        "chemistry",
        "biology"
    ]

    // Shared dependencies
    requires @univrs/std: "^1.0"
}
```

### Workspace Commands

```bash
# Build all Spirits in workspace
vudo build --workspace

# Test all Spirits
vudo test --workspace

# Check all Spirits
vudo check --workspace
```

## Output Formats

### JSON Output

```bash
# Parse to JSON
dol-parse file.dol --format json

# Check with JSON output (for tooling)
dol-check file.dol --format json
```

### SARIF Output (for IDE integration)

```bash
# Output in SARIF format for VS Code, etc.
dol-check file.dol --format sarif > results.sarif
```

### Human-Readable Output

```bash
# Default format with colors
dol-check file.dol

# Plain text (no colors)
dol-check file.dol --color=never

# Verbose output
dol-check file.dol --verbose
```

## Configuration

### .dolrc Configuration

Create `.dolrc` in your project root:

```toml
# .dolrc

[check]
strict = true
require-docs = true

[build]
target = "wasm32-unknown-unknown"
optimize = "release"

[format]
indent = 4
max-line-length = 100
```

### Environment Variables

```bash
# Set default target
export DOL_TARGET=wasm32-unknown-unknown

# Set registry URL
export DOL_REGISTRY=https://registry.univrs.io

# Enable debug output
export DOL_LOG=debug
```

## Summary

| Tool | Purpose | Key Commands |
|------|---------|--------------|
| `dol-parse` | Parse to AST | `dol-parse file.dol` |
| `dol-check` | Validate code | `dol-check --strict .` |
| `vudo` | Development | `vudo new`, `vudo build`, `vudo test` |

### Quick Reference

```bash
# Parse
dol-parse file.dol                    # Parse to JSON
dol-parse --recursive spirit/         # Parse all files

# Check
dol-check file.dol                    # Validate file
dol-check --strict --require-docs .   # Strict CI checks

# Development
vudo new my-spirit                    # Create project
vudo build                            # Build Spirit
vudo test                             # Run tests
vudo check                            # Validate
vudo serve --watch                    # Dev server
vudo publish                          # Publish to registry
```

### Next Steps

- **[REPL Guide](repl-guide.md)** - Interactive DOL exploration
- **[WASM Guide](wasm-guide.md)** - WebAssembly compilation
- **[Spirit Development](spirit-development.md)** - Building Spirits
