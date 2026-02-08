# Storage Benchmark Suite

Comprehensive browser storage evaluation for VUDO Runtime local-first mode.

## Storage Options Evaluated

1. **OPFS + SQLite WASM** - Origin Private File System with SQLite compiled to WebAssembly
2. **cr-sqlite** - Conflict-free Replicated SQLite with CRDT support
3. **IndexedDB** - Native browser key-value storage

## Benchmark Scenarios

### 1. Write Throughput
- Sequential writes (1K, 10K, 100K records)
- Batch writes (100, 1000, 10000 records per batch)
- Random writes with updates

### 2. Read Throughput
- Sequential reads
- Random access reads
- Range queries
- Full table scans

### 3. Automerge Document Lifecycle
- Save/load documents (100KB, 1MB, 10MB, 100MB)
- Operation log persistence
- Document sync performance
- Merge conflict resolution

### 4. Multi-Tab Coordination
- 2, 5, 10 concurrent tabs
- Concurrent write safety
- Lock coordination
- Data corruption detection
- Conflict resolution

### 5. Persistence & Reliability
- Browser restart persistence
- Crash recovery
- Data integrity verification
- Transaction rollback

### 6. Storage Quota Management
- Fill to 50MB, 100MB, 200MB
- Quota exceeded handling
- Storage eviction behavior
- Compression effectiveness

## Running Benchmarks

```bash
# Install dependencies
npm install

# Run all benchmarks
npm run test:all

# Run individual benchmarks
npm run test:write
npm run test:read
npm run test:automerge
npm run test:multi-tab
npm run test:persistence
npm run test:quota

# Start development server
npm run dev
```

## Browser Testing

Test on:
- Chrome/Chromium (latest)
- Firefox (latest)
- Safari (latest)

## Results

Results are saved to `results/` directory organized by:
- Browser type
- Storage adapter
- Benchmark scenario
- Dataset size

## Architecture

```
src/
├── adapters/           # Storage adapter implementations
│   ├── base.ts        # Base adapter interface
│   ├── opfs-sqlite.ts # OPFS + SQLite WASM adapter
│   ├── cr-sqlite.ts   # cr-sqlite adapter
│   └── indexeddb.ts   # IndexedDB adapter
├── benchmarks/         # Benchmark implementations
│   ├── write.ts       # Write throughput tests
│   ├── read.ts        # Read throughput tests
│   ├── automerge.ts   # Automerge lifecycle tests
│   ├── multi-tab.ts   # Multi-tab coordination tests
│   ├── persistence.ts # Persistence tests
│   └── quota.ts       # Quota management tests
├── multi-tab/          # Multi-tab coordination
│   ├── coordinator.ts # Main coordinator tab
│   └── worker.ts      # Worker tab implementation
└── utils/              # Utility functions
    ├── metrics.ts     # Performance metrics collection
    └── data-gen.ts    # Test data generation
```

## Acceptance Criteria

- Benchmarks for 1K, 10K, 100K record datasets
- Multi-tab write coordination tested (no data corruption)
- Persistence across browser restart verified
- Storage quota behavior documented per browser
- 108 test combinations (3 storage × 3 browsers × 4 ops × 3 dataset sizes)
