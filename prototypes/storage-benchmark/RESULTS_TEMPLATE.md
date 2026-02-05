# Benchmark Results Analysis

**Browser:** [Chrome/Firefox/Safari] [Version]
**Date:** YYYY-MM-DD
**Tester:** [Name]
**Environment:** [OS, Hardware specs]

## Test Configuration

**Adapters Tested:**
- [ ] IndexedDB
- [ ] OPFS + SQLite WASM
- [ ] cr-sqlite

**Scenarios Run:**
- [ ] Write Throughput
- [ ] Read Throughput
- [ ] Automerge Lifecycle
- [ ] Multi-Tab Coordination
- [ ] Persistence & Reliability
- [ ] Storage Quota Management

## Results Summary

### Write Throughput (10K records)

| Adapter | Sequential (ops/s) | Batch (ops/s) | Random (ops/s) |
|---------|-------------------|---------------|----------------|
| IndexedDB | | | |
| OPFS + SQLite | | | |
| cr-sqlite | | | |

### Read Throughput (10K records)

| Adapter | Sequential (ops/s) | Random (ops/s) | Cache Hit Rate |
|---------|-------------------|----------------|----------------|
| IndexedDB | | | |
| OPFS + SQLite | | | |
| cr-sqlite | | | |

### Automerge Document Performance

| Document Size | IndexedDB Save | OPFS Save | cr-sqlite Save |
|--------------|---------------|-----------|----------------|
| 100KB | | | |
| 1MB | | | |
| 10MB | | | |
| 100MB | | | |

### Multi-Tab Safety

| Adapter | Conflicts Detected | Data Corruption | Integrity Score |
|---------|-------------------|-----------------|-----------------|
| IndexedDB | | | |
| OPFS + SQLite | | | |
| cr-sqlite | | | |

### Persistence Tests

| Adapter | Restart Recovery | Crash Recovery | Transaction Rollback |
|---------|-----------------|----------------|---------------------|
| IndexedDB | % | % | Pass/Fail |
| OPFS + SQLite | % | % | Pass/Fail |
| cr-sqlite | % | % | Pass/Fail |

### Storage Quota

| Adapter | 50MB Fill Time | 100MB Fill Time | Quota Exceeded Handling |
|---------|---------------|----------------|------------------------|
| IndexedDB | | | |
| OPFS + SQLite | | | |
| cr-sqlite | | | |

## Observations

### Performance Notes
-
-
-

### Browser-Specific Behavior
-
-
-

### Issues Encountered
-
-
-

### Recommendations
-
-
-

## Raw Data

Attach JSON/CSV export files:
- `results/[browser]-[timestamp].json`
- `results/[browser]-[timestamp].csv`

## Comparison to Expected Results

| Metric | Expected | Actual | Variance |
|--------|----------|--------|----------|
| Write ops/s (IndexedDB) | ~1,100 | | |
| Write ops/s (OPFS) | ~3,000 | | |
| Multi-tab integrity | 97-100% | | |
| Crash recovery | 97-100% | | |

## Conclusion

**Recommended Adapter for this Browser:**

**Reasoning:**

**Issues to Address:**
