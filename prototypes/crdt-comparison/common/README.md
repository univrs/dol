# Common Domain Model

This directory contains the shared interface and benchmark scenarios used across all CRDT implementations.

## Files

### `domain.ts`

Defines the common `TodoList` interface that all CRDT libraries must implement:

```typescript
export interface CRDTTodoList {
  // Mutations
  addTodo(id: string, text: string): void;
  updateTodoText(id: string, text: string): void;
  toggleTodo(id: string): void;
  deleteTodo(id: string): void;

  // Queries
  getTodo(id: string): Todo | undefined;
  getAllTodos(): Todo[];

  // CRDT operations
  merge(other: unknown): void;
  serialize(): Uint8Array;
  deserialize(data: Uint8Array): void;
  clone(): CRDTTodoList;

  // Metrics
  getByteSize(): number;
  getOperationCount(): number;
}
```

### `scenarios.ts`

Standardized benchmark scenarios:

- **Sequential Adds:** Single peer adding N todos
- **Concurrent Edits:** Multiple peers editing simultaneously
- **Conflict Resolution:** Same todo edited by multiple peers
- **Mixed Operations:** Realistic mix of adds/updates/deletes

## Why TodoList?

TodoList is a simple but representative domain for CRDT evaluation:

1. **CRUD Operations:** Create, Read, Update, Delete
2. **Conflicts:** Multiple users editing same item
3. **Deletions:** Tombstone handling
4. **Scalability:** Can test with 1K, 10K, 100K items
5. **Real-World:** Common use case

## Mapping to DOL

While TodoList is simple, it maps to DOL concepts:

```dol
gen Todo.exists v1.0.0 {
  @crdt(lww) id: String
  @crdt(lww) text: String
  @crdt(lww) completed: Bool
  @crdt(lww) createdAt: Timestamp
  @crdt(lww) updatedAt: Timestamp
}

gen TodoList.exists v1.0.0 {
  @crdt(lww) title: String
  @crdt(or_set) todos: Map<String, Todo>
}
```

This evaluates:
- Scalar CRDT (LWW): `text`, `completed`
- Collection CRDT (OR-Set): `todos` map
- Nested types: `Map<String, Todo>`

## Usage

```typescript
import { CRDTTodoList } from './domain';
import { sequentialAdds, verifyConvergence } from './scenarios';

// Run scenario
const impl: CRDTTodoList = createImplementation();
sequentialAdds(1000)(impl);

// Verify convergence
const impl1 = createImplementation();
const impl2 = createImplementation();
// ... concurrent edits ...
impl1.merge(impl2);
impl2.merge(impl1);
console.log(verifyConvergence(impl1, impl2)); // true
```
