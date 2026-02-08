# Storage Benchmark Suite - Quick Start Guide

Get up and running with the storage benchmark suite in 5 minutes.

## Prerequisites

- Node.js 18+ installed
- Modern browser (Chrome, Firefox, or Safari)
- ~500MB free disk space for test data

## Installation

```bash
cd /home/ardeshir/repos/univrs-dol/prototypes/storage-benchmark
npm install
```

## Running Benchmarks

### Option 1: Web Interface (Recommended)

```bash
npm run dev
```

This will:
1. Start a local server at http://localhost:3000
2. Open your browser automatically
3. Display the benchmark interface

Click **"Run All Benchmarks"** to start testing.

### Option 2: Command Line

```bash
# Run specific benchmarks
npm run test:write
npm run test:read
npm run test:automerge
npm run test:multi-tab
npm run test:persistence
npm run test:quota

# Run all benchmarks
npm run test:all
```

## Understanding Results

### Real-Time Display

The web interface shows:
- **Tests Run**: Total number of benchmark tests completed
- **Avg Throughput**: Operations per second (higher is better)
- **Total Time**: Cumulative test duration
- **Error Rate**: Percentage of failed operations (lower is better)

### Log Output

Color-coded log messages:
- 🟦 **Blue (Info)**: General information
- 🟩 **Green (Success)**: Test completed successfully
- 🟥 **Red (Error)**: Test failed or error occurred

### Performance Baselines

Expected results (Chrome on modern hardware):

| Test | IndexedDB | OPFS + SQLite | cr-sqlite |
|------|-----------|---------------|-----------|
| Write (10K records) | ~1,100 ops/s | ~3,000 ops/s | ~2,700 ops/s |
| Read (10K records) | ~2,000 ops/s | ~4,200 ops/s | ~3,900 ops/s |
| Automerge save (10MB) | ~1,400ms | ~600ms | ~700ms |

## Exporting Results

### From Web Interface

1. Click **"Export JSON"** or **"Export CSV"**
2. File downloads automatically
3. Save to `results/` directory

### From Browser Console

```javascript
// After running benchmarks
await exportResults('json');  // Download JSON
await exportResults('csv');   // Download CSV
```

## Testing Different Adapters

### Select Adapters

In the web interface:
1. Uncheck adapters you don't want to test
2. Click "Run All Benchmarks"

### Check Browser Support

Before testing, verify which adapters are supported:

```javascript
// In browser console
const support = await checkBrowserSupport();
console.log(support);
// Output: { indexeddb: true, 'opfs-sqlite': true, 'cr-sqlite': true }
```

## Common Issues

### "OPFS not supported" Error

**Cause:** Browser doesn't support Origin Private File System

**Solution:**
- Use Chrome 102+ or Edge 102+
- OR test with IndexedDB only (uncheck OPFS adapters)

### "QuotaExceeded" Error

**Cause:** Browser storage quota exceeded

**Solution:**
1. Clear browser storage: DevTools → Application → Storage → Clear site data
2. Request persistent storage (benchmark does this automatically)

### Slow Performance

**Causes:**
- Running in debug mode
- Other browser tabs consuming resources
- System under load

**Solutions:**
- Close other tabs
- Run in production build: `npm run build && npm run preview`
- Restart browser before testing

## Test-Specific Quick Starts

### Test Write Performance Only

```javascript
import { createStorageAdapter } from './src/adapters/index.js';
import { runWriteBenchmark } from './src/benchmarks/write.js';

const adapter = await createStorageAdapter({ type: 'indexeddb' });
await runWriteBenchmark({
  adapter,
  datasetSizes: [1000, 10000],
  patterns: ['sequential', 'batch'],
});
```

### Test Automerge Documents Only

```javascript
import { runAutomergeBenchmark } from './src/benchmarks/automerge.js';

await runAutomergeBenchmark({
  adapter,
  documentSizes: [1024 * 1024, 10 * 1024 * 1024], // 1MB, 10MB
  operations: ['save', 'load'],
  iterations: 3,
});
```

### Test Multi-Tab Safety

```javascript
import { runMultiTabBenchmark } from './src/benchmarks/multi-tab.js';

await runMultiTabBenchmark({
  adapter,
  tabCounts: [2, 5],
  writesPerTab: 100,
  coordinationStrategy: 'optimistic',
});
```

## Interpreting Benchmark Results

### Write Throughput

**Metrics:**
- `ops/s`: Records written per second
- `MB/s`: Data written per second
- `avgOperationTime`: Average time per write

**What to look for:**
- Batch writes should be 40%+ faster than sequential
- Performance should not degrade significantly with dataset size

### Read Throughput

**Metrics:**
- `ops/s`: Records read per second
- `cache hit rate`: Percentage of cached reads

**What to look for:**
- Random reads slower than sequential (expected)
- Cache should improve performance by 15-25%

### Multi-Tab Safety

**Metrics:**
- `conflicts`: Number of write conflicts detected
- `corruption`: Data integrity failures (should be 0!)
- `integrity score`: Percentage of correct data

**What to look for:**
- **CRITICAL:** corruption must be 0%
- Conflicts are acceptable if properly resolved
- Integrity score should be 97-100%

### Persistence

**Metrics:**
- `recovery rate`: Percentage of data recovered after restart/crash
- `integrity score`: Correctness of recovered data

**What to look for:**
- Recovery rate should be 97-100%
- No data corruption after restart

## Next Steps

### 1. Analyze Results

Open your exported results in `results/` directory:

```bash
cat results/Chrome-*.json | jq '.results[] | {adapter, scenario, throughput: .metrics.throughput}'
```

### 2. Compare Adapters

Use the comparison table in the web interface or:

```javascript
globalMetrics.generateComparisonTable();
```

### 3. Read Full Evaluation

See comprehensive analysis:
- `/docs/research/storage-evaluation.md`

### 4. Integrate with VUDO Runtime

Follow implementation guide:
- `/prototypes/storage-benchmark/README.md`
- Section: "VUDO Runtime Integration"

## Advanced Usage

### Custom Dataset Sizes

```javascript
await runCompleteBenchmarkSuite({
  datasetSizes: [500, 5000, 50000],
  documentSizes: [500 * 1024, 5 * 1024 * 1024],
  iterations: 5,
});
```

### Test Specific Browser

```javascript
const browserInfo = getBrowserInfo();
console.log(`Testing on ${browserInfo.name} ${browserInfo.version}`);

// Adjust test parameters based on browser
const datasetSizes = browserInfo.name === 'Safari'
  ? [1000, 10000]  // Smaller datasets for Safari
  : [1000, 10000, 100000];  // Full suite for Chrome/Firefox
```

### Continuous Testing

```bash
# Run benchmarks every hour
while true; do
  npm run test:all
  sleep 3600
done
```

## Getting Help

- **Full Documentation:** `/prototypes/storage-benchmark/README.md`
- **Evaluation Report:** `/docs/research/storage-evaluation.md`
- **Implementation Status:** `/prototypes/storage-benchmark/IMPLEMENTATION_STATUS.md`
- **API Reference:** See TypeDoc comments in source files

## Quick Reference

```bash
# Setup
npm install

# Run (interactive)
npm run dev

# Run (automated)
npm run test:all

# Build for production
npm run build

# Preview production build
npm run preview
```

**Estimated Runtime:**
- Quick test (1K-10K records): 2-5 minutes
- Full test (1K-100K records): 15-30 minutes
- Comprehensive (all scenarios): 45-60 minutes

**Disk Space:**
- Benchmark code: ~2MB
- Test data: ~200MB during tests (auto-cleared)
- Results export: ~5-10MB per full run

---

**Ready to start?** Run `npm run dev` and click "Run All Benchmarks"!
