# CRDT Library Comparison Prototypes

This directory contains hands-on implementations and benchmarks comparing four CRDT libraries for DOL's local-first implementation.

## Overview

**Task:** t0.1 - Technology Evaluation Matrix
**Phase:** 0 (SPORE) - Foundation & Research
**Goal:** Determine the best CRDT library for MYCELIUM-SYNC

## Libraries Evaluated

1. **Automerge 3.0** - Recommended choice
2. **Loro** - High-performance alternative
3. **Yrs (Yjs)** - Mature ecosystem leader
4. **cr-sqlite** - SQL-based CRDT approach

## Directory Structure

```
crdt-comparison/
├── common/                  # Shared domain model and scenarios
│   ├── domain.ts           # TodoList interface and types
│   └── scenarios.ts        # Benchmark scenarios
├── automerge-impl/         # Automerge implementation
│   ├── src/todo-list.ts    # TodoList using Automerge
│   └── package.json
├── loro-impl/              # Loro implementation
│   ├── src/todo-list.ts    # TodoList using Loro
│   └── package.json
├── yrs-impl/               # Yrs implementation
│   ├── src/todo-list.ts    # TodoList using Yjs
│   └── package.json
├── cr-sqlite-impl/         # cr-sqlite implementation
│   ├── src/todo-list.ts    # TodoList using cr-sqlite
│   └── package.json
├── benchmarks/             # Benchmark harness
│   ├── src/harness.ts      # Core benchmarking logic
│   ├── src/run-node.ts     # Node.js runner
│   └── src/run-browser.ts  # Browser runner (Playwright)
└── results/                # Benchmark results (JSON)
    ├── node-*.json         # Node.js results
    ├── chrome-*.json       # Chrome results
    └── firefox-*.json      # Firefox results
```

## Quick Start

### Prerequisites

- Node.js 20+
- pnpm (or npm/yarn)
- Modern browser (Chrome/Firefox)

### Installation

```bash
# Install dependencies for all implementations
cd automerge-impl && pnpm install && cd ..
cd loro-impl && pnpm install && cd ..
cd yrs-impl && pnpm install && cd ..
cd cr-sqlite-impl && pnpm install && cd ..
cd benchmarks && pnpm install && cd ..
```

### Build All

```bash
# Build TypeScript for all implementations
for dir in automerge-impl loro-impl yrs-impl cr-sqlite-impl benchmarks; do
  cd $dir
  pnpm build
  cd ..
done
```

### Run Benchmarks

#### Node.js

```bash
cd benchmarks
pnpm benchmark:node
```

#### Browser (Chrome, Firefox, Safari)

```bash
cd benchmarks
pnpm benchmark:browser
```

This will:
1. Start local HTTP server
2. Launch Playwright
3. Run benchmarks in each browser
4. Collect results

### View Results

```bash
cd benchmarks
pnpm results
```

This analyzes all result files and generates comparison tables.

## Benchmark Scenarios

All implementations run identical scenarios:

1. **Sequential Adds (1K, 10K, 100K)**
   - Single peer adding todos sequentially
   - Measures: throughput, memory, serialized size

2. **Concurrent Edits (2 peers)**
   - Two peers editing different todos simultaneously
   - Measures: merge latency, convergence correctness

3. **Conflict Resolution**
   - Multiple peers editing the same todo
   - Measures: conflict resolution time, final state

4. **Mixed Operations**
   - Adds, updates, toggles, deletes
   - Measures: realistic usage performance

## Key Metrics

- **Merge Latency:** Time to merge 10K operations
- **Bundle Size:** WASM + JS glue (gzipped)
- **Memory Usage:** Heap allocation during operations
- **Throughput:** Operations per second
- **Convergence:** Do replicas reach same state?

## Implementation Notes

### Automerge

```typescript
import * as Automerge from '@automerge/automerge';

let doc = Automerge.from({ todos: {} });
doc = Automerge.change(doc, d => {
  d.todos['1'] = { text: 'Learn Automerge', done: false };
});
```

- Functional API (immutable updates)
- LWW for scalars, OR-Set for collections
- Excellent Rust support via `automerge-rs`

### Loro

```typescript
import { Loro } from 'loro-crdt';

const doc = new Loro();
const todos = doc.getMap('todos');
const todo = todos.setContainer('1', new LoroMap());
todo.set('text', 'Learn Loro');
```

- Imperative API (mutable updates)
- Fastest merge performance
- Time-travel built-in

### Yrs (Yjs)

```typescript
import * as Y from 'yjs';

const doc = new Y.Doc();
const todos = doc.getMap('todos');
const todo = new Y.Map();
todo.set('text', 'Learn Yjs');
todos.set('1', todo);
```

- Mature ecosystem
- Smallest bundle size
- Battle-tested in production

### cr-sqlite

```typescript
// Note: Mock implementation for prototype
// Real usage requires WASM setup

db.exec('CREATE TABLE todos (...)');
db.exec('SELECT crsql_as_crr("todos")');
db.prepare('INSERT INTO todos VALUES (?, ?)').run('1', 'Learn cr-sqlite');
```

- SQL interface
- Familiar transactions
- Large WASM bundle

## Evaluation Criteria

| Criterion | Weight | Notes |
|-----------|--------|-------|
| DOL Integration | 30% | Natural type mapping |
| Constraint Enforcement | 25% | Custom merge logic |
| Performance | 15% | Acceptable for ontologies |
| Bundle Size | 10% | Browser-first priority |
| Ecosystem | 10% | Maturity, docs, support |
| Rust Support | 10% | DOL compiler integration |

## Results Summary

**Recommendation: Automerge 3.0**

- ✅ Best DOL type mapping
- ✅ Constraint enforcement support
- ✅ Rust-first architecture
- ⚠️ Larger bundle (450KB vs 120-180KB)
- ⚠️ Slower merge (45ms vs 12-18ms)

Trade-offs acceptable for DOL use case.

See [`docs/research/crdt-evaluation-matrix.md`](../../docs/research/crdt-evaluation-matrix.md) for full analysis.

## Contributing

To add a new CRDT library:

1. Create `<library>-impl/` directory
2. Implement `CRDTTodoList` interface from `common/domain.ts`
3. Add to benchmark harness
4. Run benchmarks
5. Update evaluation matrix

## License

MIT (aligned with DOL project)

## References

- [Automerge](https://automerge.org/)
- [Loro](https://loro.dev/)
- [Yjs](https://docs.yjs.dev/)
- [cr-sqlite](https://vlcn.io/)
- [DOL Project](https://github.com/univrs/dol)
