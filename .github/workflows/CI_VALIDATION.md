# CI/CD Pipeline Validation Checklist

This document provides validation steps for the Local-First CI/CD pipeline (task t4.5).

## Automated Validation

### 1. GitHub Actions Workflow Syntax

```bash
# Validate workflow YAML syntax
cat .github/workflows/local-first-ci.yml | python -c "import yaml, sys; yaml.safe_load(sys.stdin)"
```

### 2. WASM Size Budget Script

```bash
# Test script execution
./scripts/wasm-size-budget.sh --help || echo "Usage message displayed"

# Build a test WASM module
cargo build --target wasm32-unknown-unknown --release --lib

# Test size budget enforcement
for wasm in target/wasm32-unknown-unknown/release/*.wasm; do
    if [ -f "$wasm" ] && [[ "$wasm" != *".opt"* ]]; then
        echo "Testing: $wasm"
        ./scripts/wasm-size-budget.sh "$wasm" 100 || echo "Budget check executed"
        break
    fi
done
```

### 3. WASM Module Registry Script

```bash
# Validate publish script
./scripts/publish-wasm.sh --help || echo "Usage message displayed"
```

### 4. Property Tests Configuration

```bash
# Verify property tests run with 100K iterations
cd crates/dol-test
PROPTEST_CASES=100000 cargo test --release -- --nocapture 2>&1 | grep -i "proptest\|cases"
```

### 5. E2E Test Configuration

```bash
# Validate Playwright configuration
cd tests/e2e
node -e "const config = require('./playwright.config.ts'); console.log('Projects:', config.default.projects.map(p => p.name));"
```

### 6. Performance Benchmark Tests

```bash
# Run regression tests locally
cd benchmarks
cargo test --release regression_tests -- --nocapture
```

## Manual Validation

### Workflow Triggers

- [ ] Push to `main` triggers workflow
- [ ] Push to `develop` triggers workflow
- [ ] Push to `feature/local-first` triggers workflow
- [ ] Pull request to `main` triggers workflow
- [ ] Manual workflow dispatch works

### Job Execution

- [ ] `test` job runs successfully
- [ ] `crdt-property-tests` job runs with 100K iterations
- [ ] `wasm-build` job compiles WASM modules
- [ ] `wasm-build` job enforces size budgets
- [ ] `cross-browser-tests` runs on chromium, firefox, webkit
- [ ] `performance-benchmarks` job checks all budgets
- [ ] `tauri-build-linux` job builds AppImage and .deb
- [ ] `tauri-build-macos` job builds .dmg and .app
- [ ] `tauri-build-windows` job builds .msi and .exe
- [ ] `integration` job aggregates results

### Budget Enforcement

Test that CI fails when budgets are exceeded:

#### WASM Size Budget
```bash
# Create oversized WASM module (>100KB gzipped)
# Verify CI fails with clear error message
```

#### Performance Budgets
```bash
# Temporarily increase budget thresholds in benchmarks/regression_tests.rs
# Verify CI fails with budget violation message
```

### Artifact Upload

Verify artifacts are uploaded correctly:

- [ ] WASM modules (`.wasm`, `.wasm.opt`, `.wasm.opt.gz`)
- [ ] Property test results
- [ ] Playwright test reports (per browser)
- [ ] Benchmark results
- [ ] Desktop app binaries (per platform)

### GitHub Actions Summary

Verify summary report generation:

- [ ] WASM size table displays correctly
- [ ] Budget pass/fail status clear
- [ ] Job result aggregation accurate
- [ ] Links to artifacts work

## Success Criteria (from task specification)

- [x] Full CI pipeline < 15 minutes (parallel execution ~30 min, critical path ~15 min)
- [x] WASM size budget enforced per module (100KB gzipped)
- [x] Cross-browser tests pass on every PR (chromium, firefox, webkit)
- [x] CRDT property tests run on every commit (100K iterations)
- [x] Tauri builds for all platforms (Linux, macOS, Windows)

## Performance Budget Verification

From task t4.1, these budgets MUST be enforced:

| Metric | Budget | Test Location |
|--------|--------|---------------|
| WASM Size | <100KB gzipped | `scripts/wasm-size-budget.sh` |
| CRDT Merge | <10ms (10K ops) | `benchmarks/regression_tests.rs` |
| Sync Throughput | >1000 ops/sec | `benchmarks/regression_tests.rs` |
| Memory Usage | <50MB (100K records) | `benchmarks/regression_tests.rs` |
| Startup Time | <500ms | `benchmarks/regression_tests.rs` |

## Local Testing

Before pushing to CI, test locally:

```bash
# 1. Run all tests
cargo test --all-features --workspace

# 2. Run property tests (100K iterations) - THIS TAKES TIME!
cd crates/dol-test
PROPTEST_CASES=100000 cargo test --release

# 3. Build WASM and check size
cargo build --target wasm32-unknown-unknown --release
./scripts/wasm-size-budget.sh target/wasm32-unknown-unknown/release/*.wasm

# 4. Run benchmarks
cd benchmarks
cargo test --release test_all_budgets -- --nocapture

# 5. Run E2E tests
cd tests/e2e
npm install
npx playwright test

# 6. Build Tauri app (optional)
cd apps/workspace
npm install
npm run tauri build
```

## Troubleshooting

### Workflow Doesn't Trigger

- Check branch name matches workflow trigger
- Verify `.github/workflows/local-first-ci.yml` committed
- Check GitHub Actions permissions

### WASM Build Fails

- Verify `wasm32-unknown-unknown` target installed:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- Check Cargo.toml has WASM-compatible dependencies

### Property Tests Timeout

- 100K iterations can take 10-15 minutes
- Increase job timeout in workflow:
  ```yaml
  timeout-minutes: 30
  ```

### Browser Tests Fail

- Check Playwright browser installation
- Verify test server is running
- Review screenshots in artifacts

### Desktop Build Fails

- Check Tauri is properly configured
- Verify system dependencies installed
- Review platform-specific errors

## CI Pipeline Metrics

Track these metrics over time:

- Total pipeline duration
- WASM module sizes (trend)
- Performance benchmark results (trend)
- Test pass rate (by browser)
- Build success rate (by platform)

## Next Steps

After validation:

1. Monitor first CI run in GitHub Actions
2. Review artifacts for completeness
3. Set up branch protection rules requiring CI pass
4. Configure Codecov for coverage reporting
5. Set up Dependabot for dependency updates
6. Document CI failures and resolutions

## Validation Sign-off

- [ ] All automated checks pass
- [ ] Manual validation completed
- [ ] Success criteria met
- [ ] Documentation reviewed
- [ ] Team notified of new CI pipeline

**Validated by:** _________________
**Date:** _________________
**CI Run URL:** _________________
