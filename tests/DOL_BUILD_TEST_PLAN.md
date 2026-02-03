# DOL Build CLI Test Plan

## Overview

This document describes the comprehensive testing strategy for the `dol-build` CLI command, which orchestrates the DOL → Rust → WASM → JS build pipeline.

## Test Status

**Current Status**: ⏸️ BLOCKED - Awaiting Implementation

The `dol-build` binary does not exist yet. Test files are prepared and ready to run once implementation is complete.

**Implementation Requirements**:
- [ ] Create `src/bin/dol-build.rs`
- [ ] Add `[[bin]]` entry in `Cargo.toml` for `dol-build`
- [ ] Implement manifest parsing using `src/manifest.rs`
- [ ] Orchestrate build pipeline (DOL → Rust → WASM → JS)
- [ ] Generate `manifest.json` output

## Test Levels

### 1. Unit Tests (manifest_build_tests.rs)

**Purpose**: Verify manifest parsing and build configuration logic in isolation.

**Test Coverage**:
- ✅ Manifest parsing (standard format)
- ✅ Build config defaults
- ✅ Build config customization
- ✅ Entry file resolution
- ✅ Dependency parsing
- ✅ Module declarations
- ✅ Version parsing (including suffixes)
- ✅ Error handling (invalid manifests)
- ✅ JSON serialization
- ⚠️  Game of Life manifest format (non-standard format)

**Run Command**:
```bash
cargo test --test manifest_build_tests
```

### 2. Integration Tests (dol_build_tests.rs)

**Purpose**: Verify end-to-end build pipeline with real examples.

**Test Coverage**:
- 🔄 Binary existence and execution
- 🔄 Building game-of-life example
- 🔄 Verbose output verification
- 🔄 Error handling (missing manifest)
- 🔄 Custom output directory
- 🔄 Performance comparison with build.sh
- 🔄 Incremental builds
- 🔄 CLI flags (--help, --version)

**Run Command**:
```bash
# Build dol-build binary first
cargo build --release --features cli --bin dol-build

# Run tests (currently marked as #[ignore])
cargo test --test dol_build_tests -- --ignored
```

### 3. Manual Testing

**Purpose**: Verify behavior in real-world scenarios.

**Test Procedure**:

#### A. Basic Build Test

```bash
cd examples/spirits/game-of-life
dol-build --verbose
```

**Expected Results**:
- Exit code: 0
- Console output shows:
  - "Parsing Spirit.dol"
  - "Compiling DOL files"
  - "Building Rust crate"
  - "Compiling WASM"
  - "Generating JS bindings"
- Output files created:
  - `target/spirit/game_of_life.wasm` (~XXX KB)
  - `target/spirit/game_of_life.js` (JS bindings)
  - `target/spirit/manifest.json` (build metadata)

#### B. Comparison Test

Compare dol-build output with build.sh output:

```bash
# Build with build.sh
./build.sh
mv web/game_of_life_bg.wasm web/game_of_life_bg.wasm.buildsh
mv web/game_of_life.js web/game_of_life.js.buildsh

# Build with dol-build
dol-build

# Compare files
diff -u web/game_of_life_bg.wasm.buildsh target/spirit/game_of_life.wasm
diff -u web/game_of_life.js.buildsh target/spirit/game_of_life.js
```

**Expected**: Files should be functionally equivalent (binary diffs may differ due to timestamps).

#### C. Error Handling Test

```bash
# Test missing manifest
mkdir /tmp/test-spirit
cd /tmp/test-spirit
dol-build
# Expected: Error message mentioning "Spirit.dol not found"

# Test invalid manifest
echo "invalid content" > Spirit.dol
dol-build
# Expected: Parse error with line number
```

## Verification Checklist

### Build Pipeline Verification

- [ ] **Manifest Parsing**
  - [ ] Correctly reads Spirit.dol from current directory
  - [ ] Parses spirit name and version
  - [ ] Parses dependencies
  - [ ] Parses build configuration
  - [ ] Handles both manifest formats (standard and game-of-life style)

- [ ] **DOL → Rust Compilation**
  - [ ] Invokes dol-codegen for each .dol file
  - [ ] Generates .rs files in correct location
  - [ ] Handles nested directories (genes/, spells/, effects/)
  - [ ] Reports progress for each file

- [ ] **Rust → WASM Compilation**
  - [ ] Invokes cargo with correct target (wasm32-unknown-unknown)
  - [ ] Applies optimization flags from manifest
  - [ ] Uses correct Rust edition (from manifest or default 2021)
  - [ ] Produces .wasm binary

- [ ] **WASM → JS Bindings**
  - [ ] Invokes wasm-bindgen with correct arguments
  - [ ] Generates .js and .wasm files
  - [ ] Uses correct target (web, nodejs, bundler)
  - [ ] Omits default module path if specified

- [ ] **Manifest.json Generation**
  - [ ] Creates manifest.json in output directory
  - [ ] Includes spirit name and version
  - [ ] Includes build timestamp
  - [ ] Includes file hashes/sizes
  - [ ] Valid JSON format

### CLI Interface Verification

- [ ] **Arguments and Flags**
  - [ ] `--verbose` shows detailed progress
  - [ ] `--quiet` suppresses non-error output
  - [ ] `--output <dir>` uses custom output directory
  - [ ] `--help` shows usage information
  - [ ] `--version` shows version number

- [ ] **Error Handling**
  - [ ] Missing Spirit.dol: Clear error message
  - [ ] Invalid Spirit.dol: Parse error with location
  - [ ] Missing dependencies: Helpful error
  - [ ] Build failures: Propagates error with context
  - [ ] Non-zero exit code on failure

- [ ] **Performance**
  - [ ] Comparable speed to build.sh (within 20%)
  - [ ] Incremental builds skip unchanged work
  - [ ] Parallel compilation where possible

## Expected Outputs

### Console Output (Verbose Mode)

```
══════════════════════════════════════════════════════════════
  Building Game of Life Spirit
══════════════════════════════════════════════════════════════

Step 1: Parsing Spirit.dol
────────────────────────────────────────────────────────────────
  ✓ Spirit: GameOfLife @ 0.1.0
  ✓ Entry: src/lib.dol
  ✓ Target: wasm32-unknown-unknown

Step 2: DOL → Rust
────────────────────────────────────────────────────────────────
  Compiling genes/cell.dol...
  Compiling genes/grid.dol...
  Compiling spells/rules.dol...
  Compiling spells/grid_ops.dol...
  Compiling spells/patterns.dol...
  Compiling effects/browser.dol...
  ✓ DOL compilation complete (6 files)

Step 3: Rust → WASM
────────────────────────────────────────────────────────────────
  Building with cargo...
  ✓ WASM binary compiled (123 KB)

Step 4: WASM → JS bindings
────────────────────────────────────────────────────────────────
  Generating JS bindings...
  ✓ JS bindings generated

Step 5: Manifest generation
────────────────────────────────────────────────────────────────
  ✓ manifest.json written

══════════════════════════════════════════════════════════════
  Build complete! (2.3s)
  Output: target/spirit/
══════════════════════════════════════════════════════════════
```

### manifest.json Format

```json
{
  "spirit": {
    "name": "GameOfLife",
    "version": "0.1.0",
    "qualified_name": "GameOfLife @ 0.1.0"
  },
  "build": {
    "timestamp": "2024-01-25T22:30:00Z",
    "dol_version": "0.8.1",
    "rust_edition": "2024",
    "wasm_target": "wasm32-unknown-unknown",
    "optimized": true
  },
  "outputs": {
    "wasm": {
      "path": "game_of_life.wasm",
      "size_bytes": 125830,
      "sha256": "abc123..."
    },
    "js": {
      "path": "game_of_life.js",
      "size_bytes": 45620,
      "sha256": "def456..."
    }
  },
  "source_files": [
    "src/genes/cell.dol",
    "src/genes/grid.dol",
    "src/spells/rules.dol",
    "src/spells/grid_ops.dol",
    "src/spells/patterns.dol",
    "src/effects/browser.dol"
  ]
}
```

## Test Execution Guide

### Prerequisites

1. Build DOL compiler with CLI features:
   ```bash
   cargo build --release --features cli
   ```

2. Ensure dol-build binary exists:
   ```bash
   ls -la target/release/dol-build
   ```

3. Install wasm-bindgen-cli (for JS bindings):
   ```bash
   cargo install wasm-bindgen-cli
   ```

### Running Tests

1. **Unit tests** (can run now):
   ```bash
   cargo test --test manifest_build_tests
   ```

2. **Integration tests** (require implementation):
   ```bash
   cargo test --test dol_build_tests -- --ignored
   ```

3. **Manual tests**:
   ```bash
   cd examples/spirits/game-of-life
   ../../../target/release/dol-build --verbose
   ```

### Validation Steps

1. **Verify WASM output**:
   ```bash
   file target/spirit/game_of_life.wasm
   # Should show: WebAssembly (wasm) binary module

   wasm-objdump -h target/spirit/game_of_life.wasm
   # Should show valid WASM sections
   ```

2. **Verify JS bindings**:
   ```bash
   head -20 target/spirit/game_of_life.js
   # Should show wasm-bindgen generated code
   ```

3. **Verify manifest.json**:
   ```bash
   cat target/spirit/manifest.json | jq .
   # Should be valid JSON with expected structure
   ```

## Performance Benchmarks

Target performance metrics (compared to build.sh):

| Metric | Target | Notes |
|--------|--------|-------|
| Build time (clean) | ≤ 120% of build.sh | Allow 20% overhead for orchestration |
| Build time (incremental) | ≤ 50% of clean build | Skip unchanged files |
| Memory usage | ≤ 500 MB | Peak memory during build |
| Binary size (WASM) | ≈ Same as build.sh | Within 5% difference |

## Success Criteria

The dol-build CLI is considered complete when:

1. ✅ All unit tests pass
2. ✅ All integration tests pass (remove #[ignore] markers)
3. ✅ Manual testing produces expected outputs
4. ✅ Error handling is robust and helpful
5. ✅ Performance is within target metrics
6. ✅ Documentation is complete (--help, README)
7. ✅ Works on Linux, macOS, Windows (CI verification)

## Known Issues / Future Work

- [ ] **Manifest format mismatch**: The game-of-life Spirit.dol uses a different format than the current manifest parser expects. Need to support both formats or migrate examples.
- [ ] **Incremental builds**: Current tests assume full rebuilds. Need to implement change detection.
- [ ] **Parallel compilation**: DOL file compilation could be parallelized for better performance.
- [ ] **Build caching**: Consider caching DOL → Rust compilation results.
- [ ] **Cross-platform paths**: Ensure paths work on Windows (use `std::path::Path`).
- [ ] **Error recovery**: Consider continuing after non-critical errors.

## Contact

For questions about testing:
- Review test files: `tests/cli/dol_build_tests.rs`, `tests/cli/manifest_build_tests.rs`
- Check coordination memory: `npx @claude-flow/cli@latest memory retrieve --key "dol-build-cli/tester-status" --namespace coordination`
- See implementation requirements: `npx @claude-flow/cli@latest memory retrieve --key "build-requirements" --namespace dol-build-cli`
