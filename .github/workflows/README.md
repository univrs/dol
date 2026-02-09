# CI/CD Workflows

This directory contains GitHub Actions workflows for the DOL 2.0 Local-First project.

## Workflows

### 1. `local-first-ci.yml` - Local-First CI Pipeline

Comprehensive CI/CD pipeline for local-first WASM artifacts with performance budget enforcement.

**Triggers:**
- Push to `main`, `develop`, `feature/local-first` branches
- Pull requests to `main`
- Manual workflow dispatch

**Jobs:**

#### Test Suite (`test`)
- Runs all Rust tests with `--all-features`
- Includes doc tests
- Duration: ~5 minutes

#### CRDT Property Tests (`crdt-property-tests`)
- Runs property-based tests with **100,000 iterations**
- Tests CRDT convergence properties
- Ensures eventually consistent behavior across all scenarios
- Duration: ~10-15 minutes
- Environment:
  - `PROPTEST_CASES=100000`
  - `PROPTEST_MAX_SHRINK_ITERS=10000`

#### WASM Build & Size Check (`wasm-build`)
- Compiles Rust to WASM target
- Optimizes with `wasm-opt -Oz`
- **Enforces 100KB gzipped size budget per module**
- Fails CI if budget exceeded
- Generates size report in GitHub Actions summary
- Duration: ~5-10 minutes

#### Cross-Browser E2E Tests (`cross-browser-tests`)
- Runs Playwright tests on:
  - Chromium (Chrome)
  - Firefox
  - WebKit (Safari)
- Tests browser sync, offline/online transitions, crash recovery
- Duration: ~10 minutes per browser
- Uploads test reports and screenshots on failure

#### Performance Benchmarks (`performance-benchmarks`)
- Runs regression tests with performance budgets:
  - CRDT merge: <10ms for 10K operations
  - Sync throughput: >1000 ops/sec
  - Memory usage: <50MB for 100K records
  - Startup time: <500ms
- Fails CI if any budget exceeded
- Duration: ~15-20 minutes

#### Tauri Desktop Builds (`tauri-build-*`)
- Builds desktop app for:
  - **Linux**: AppImage, .deb
  - **macOS**: .dmg, .app
  - **Windows**: .msi, .exe
- Only runs if Tauri is configured
- Duration: ~15-30 minutes per platform

#### Integration Summary (`integration`)
- Aggregates all job results
- Provides consolidated CI report
- Fails if any required job fails

**Performance Budgets (from t4.1):**

| Metric | Budget | Test |
|--------|--------|------|
| WASM Size (gzipped) | <100KB | Per module |
| CRDT Merge Latency | <10ms | 10K operations |
| Sync Throughput | >1000 ops/sec | Network sync |
| Memory Usage | <50MB | 100K records |
| Startup Time | <500ms | Cold start |

**Artifacts:**
- WASM modules (optimized + gzipped)
- Property test results
- Playwright test reports
- Benchmark results
- Desktop app binaries (Linux, macOS, Windows)

### 2. `ci.yml` - Main CI Pipeline

Standard Rust CI pipeline with:
- Code formatting (`cargo fmt`)
- Linting (`cargo clippy`)
- Cross-platform tests (Ubuntu, macOS, Windows)
- Documentation build
- Code coverage (Codecov)

### 3. `performance.yml` - Performance Tests

Detailed performance testing workflow:
- Regression tests
- Benchmark comparison (PR vs main)
- WASM size tracking
- Memory profiling with Valgrind
- Performance summary report

### 4. `hir-ci.yml` - HIR Compilation

Specialized workflow for HIR (High-Level Intermediate Representation) tests.

### 5. `npm.yml` - NPM Package CI

Workflow for JavaScript/TypeScript packages (if applicable).

### 6. `release.yml` - Release Automation

Automated release workflow:
- Triggered on version tags (`v*`)
- Creates GitHub releases
- Publishes to crates.io
- Generates release notes

## Scripts

### `scripts/wasm-size-budget.sh`

Enforces WASM size budgets in CI.

**Usage:**
```bash
./scripts/wasm-size-budget.sh <input.wasm> [budget_kb]
```

**Features:**
- Optimizes WASM with `wasm-opt -Oz`
- Gzip compresses output
- Checks against size budget (default: 100KB)
- Fails with exit code 1 if budget exceeded
- Provides detailed size breakdown

**Example:**
```bash
./scripts/wasm-size-budget.sh target/wasm32-unknown-unknown/release/vudo.wasm
```

**Output:**
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
WASM Size Budget Enforcement
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Original size:  250 KB
Optimized size: 120 KB (-52.0% reduction)
Gzipped size:   85 KB (-66.0% total reduction)

✅ Size budget met!
  Gzipped: 85 KB
  Budget:  100 KB
  Margin:  15 KB under budget

PASS
```

### `scripts/publish-wasm.sh`

Publishes WASM modules to registry.

**Usage:**
```bash
./scripts/publish-wasm.sh <module-name> <version> [registry-type]
```

**Registry Types:**
- `s3`: AWS S3 bucket
- `npm`: npm registry
- `http`: Custom HTTP endpoint

**Example:**
```bash
# Publish to S3
WASM_REGISTRY_S3_BUCKET=vudo-wasm-modules \
  ./scripts/publish-wasm.sh vudo-state 0.1.0 s3

# Publish to npm
WASM_REGISTRY_NPM_SCOPE=@vudo \
  ./scripts/publish-wasm.sh vudo-state 0.1.0 npm
```

## Running Locally

### Run Full CI Suite
```bash
# Install dependencies
cargo build --all-features

# Run tests
cargo test --all-features --workspace

# Run property tests (100K iterations)
cd crates/dol-test
PROPTEST_CASES=100000 cargo test --release

# Build WASM
cargo build --target wasm32-unknown-unknown --release

# Check WASM size
./scripts/wasm-size-budget.sh target/wasm32-unknown-unknown/release/*.wasm

# Run benchmarks
cd benchmarks
cargo test --release regression_tests -- --nocapture
cargo bench

# Run E2E tests
cd tests/e2e
npm install
npx playwright test
```

### Build Desktop App
```bash
cd apps/workspace
npm install
npm run tauri build
```

## CI Pipeline Duration

| Job | Duration | Can Fail CI |
|-----|----------|-------------|
| Test Suite | ~5 min | Yes |
| CRDT Property Tests | ~15 min | Yes |
| WASM Build & Size | ~10 min | Yes |
| Cross-Browser E2E | ~30 min | Yes |
| Performance Benchmarks | ~20 min | Yes |
| Tauri Linux | ~20 min | No |
| Tauri macOS | ~25 min | No |
| Tauri Windows | ~25 min | No |
| **Total (parallel)** | **~30 min** | - |

## Performance Budget Enforcement

The CI pipeline **fails** if any performance budget is exceeded:

1. **WASM Size Budget**: Checks each compiled WASM module
   - Budget: <100KB gzipped
   - Fails if: `gzipped_size >= 100KB`

2. **CRDT Merge Latency**: Tests merge performance
   - Budget: <10ms for 10K operations
   - Fails if: `merge_time >= 10ms`

3. **Sync Throughput**: Tests P2P sync speed
   - Budget: >1000 ops/sec
   - Fails if: `throughput <= 1000 ops/sec`

4. **Memory Usage**: Tests memory footprint
   - Budget: <50MB for 100K records
   - Fails if: `memory_delta >= 50MB`

5. **Startup Time**: Tests cold start performance
   - Budget: <500ms
   - Fails if: `startup_time >= 500ms`

## Troubleshooting

### WASM Size Budget Failure

If WASM size exceeds budget:

1. **Enable LTO** in `Cargo.toml`:
   ```toml
   [profile.release]
   opt-level = 'z'  # Optimize for size
   lto = true
   codegen-units = 1
   strip = true
   panic = 'abort'
   ```

2. **Remove unused dependencies**:
   ```bash
   cargo tree --duplicate
   cargo udeps
   ```

3. **Use feature flags** to minimize included code:
   ```bash
   cargo build --target wasm32-unknown-unknown --release --no-default-features --features minimal
   ```

4. **Review imports** and remove unused code

### Property Test Failures

If property tests fail:

1. Check `proptest-regressions/` for reproducible failures
2. Review failing test case in CI logs
3. Run locally with same seed:
   ```bash
   PROPTEST_CASES=100000 cargo test -- --nocapture
   ```

### E2E Test Failures

If browser tests fail:

1. Download artifacts from GitHub Actions
2. Review screenshots and videos
3. Run locally with headed browser:
   ```bash
   cd tests/e2e
   npx playwright test --headed --project=chromium
   ```

### Performance Regression

If performance tests fail:

1. Review benchmark results in artifacts
2. Compare with baseline in `main` branch
3. Profile locally:
   ```bash
   cd benchmarks
   cargo bench
   cargo flamegraph --bench regression_tests
   ```

## Maintenance

### Updating Performance Budgets

Edit budgets in `benchmarks/regression_tests.rs`:

```rust
const WASM_SIZE_BUDGET_KB: usize = 100;
const MERGE_LATENCY_BUDGET_MS: u128 = 10;
const SYNC_THROUGHPUT_BUDGET_OPS_PER_SEC: f64 = 1000.0;
const MEMORY_BUDGET_MB: usize = 50;
const STARTUP_TIME_BUDGET_MS: u128 = 500;
```

### Adding New WASM Modules

To add size checks for new WASM modules:

1. Build module for `wasm32-unknown-unknown` target
2. CI automatically checks all `.wasm` files
3. Ensure module size <100KB gzipped

### Updating Browser Matrix

Edit `local-first-ci.yml`:

```yaml
strategy:
  matrix:
    browser: [chromium, firefox, webkit]
```

Add mobile browsers:
```yaml
browser: [chromium, firefox, webkit, 'Mobile Chrome', 'Mobile Safari']
```

## Contact

For CI/CD issues, contact the DevOps team or open an issue on GitHub.
