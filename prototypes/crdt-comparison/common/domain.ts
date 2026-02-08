/**
 * Common TodoList Domain Model
 *
 * All CRDT implementations must support this interface to ensure
 * fair comparison across libraries.
 */

export interface Todo {
  id: string;
  text: string;
  completed: boolean;
  createdAt: number;
  updatedAt: number;
}

export interface TodoList {
  todos: Map<string, Todo>;
  title: string;
  lastModified: number;
}

/**
 * Common operations that all CRDT implementations must support
 */
export interface CRDTTodoList {
  // Creation
  create(title: string): void;

  // Mutations
  addTodo(id: string, text: string): void;
  updateTodoText(id: string, text: string): void;
  toggleTodo(id: string): void;
  deleteTodo(id: string): void;
  setTitle(title: string): void;

  // Queries
  getTodo(id: string): Todo | undefined;
  getAllTodos(): Todo[];
  getTitle(): string;

  // CRDT operations
  merge(other: unknown): void;
  serialize(): Uint8Array;
  deserialize(data: Uint8Array): void;
  clone(): CRDTTodoList;

  // Metrics
  getByteSize(): number;
  getOperationCount(): number;
}

/**
 * Benchmark scenarios - standardized test cases
 */
export interface BenchmarkScenario {
  name: string;
  operationCount: number;
  setup: () => void;
  execute: (impl: CRDTTodoList) => void;
  validate: (impl: CRDTTodoList) => boolean;
}

/**
 * Performance metrics collected during benchmarks
 */
export interface BenchmarkMetrics {
  library: string;
  scenario: string;
  operationCount: number;

  // Time measurements (ms)
  totalTime: number;
  mergeTime: number;
  serializeTime: number;
  deserializeTime: number;

  // Size measurements (bytes)
  serializedSize: number;
  memoryUsage: number;
  wasmBundleSize?: number;

  // Throughput
  opsPerSecond: number;
  mergesPerSecond: number;

  // Convergence
  convergenceCorrect: boolean;

  platform: string; // Chrome, Firefox, Node, etc.
  timestamp: number;
}

/**
 * Standard benchmark scenarios
 */
export const SCENARIOS = {
  SEQUENTIAL_ADDS_1K: {
    name: 'Sequential Adds (1K)',
    operationCount: 1000,
    description: 'Single peer adding 1000 todos sequentially'
  },
  SEQUENTIAL_ADDS_10K: {
    name: 'Sequential Adds (10K)',
    operationCount: 10000,
    description: 'Single peer adding 10000 todos sequentially'
  },
  SEQUENTIAL_ADDS_100K: {
    name: 'Sequential Adds (100K)',
    operationCount: 100000,
    description: 'Single peer adding 100000 todos sequentially'
  },
  CONCURRENT_EDITS_2PEER: {
    name: 'Concurrent Edits (2 peers)',
    operationCount: 1000,
    description: 'Two peers concurrently editing different todos'
  },
  CONCURRENT_EDITS_10PEER: {
    name: 'Concurrent Edits (10 peers)',
    operationCount: 1000,
    description: 'Ten peers concurrently editing different todos'
  },
  CONFLICT_SAME_TODO: {
    name: 'Conflict Resolution (same todo)',
    operationCount: 100,
    description: 'Multiple peers editing the same todo text'
  },
  MIXED_OPERATIONS: {
    name: 'Mixed Operations',
    operationCount: 1000,
    description: 'Mix of adds, updates, toggles, and deletes'
  }
} as const;
