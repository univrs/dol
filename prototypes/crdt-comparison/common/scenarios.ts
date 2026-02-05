/**
 * Benchmark Scenario Implementations
 *
 * Reusable test scenarios that exercise CRDT implementations
 * consistently across all libraries.
 */

import type { CRDTTodoList, BenchmarkMetrics } from './domain';

export class BenchmarkRunner {
  async runScenario(
    name: string,
    impl: CRDTTodoList,
    scenario: (impl: CRDTTodoList) => void
  ): Promise<Partial<BenchmarkMetrics>> {
    const startMem = this.getMemoryUsage();
    const startTime = performance.now();

    scenario(impl);

    const endTime = performance.now();
    const endMem = this.getMemoryUsage();

    return {
      totalTime: endTime - startTime,
      memoryUsage: endMem - startMem,
      serializedSize: impl.getByteSize(),
      operationCount: impl.getOperationCount()
    };
  }

  async measureMerge(
    impl1: CRDTTodoList,
    impl2: CRDTTodoList
  ): Promise<number> {
    const startTime = performance.now();
    const serialized = impl2.serialize();
    impl1.deserialize(serialized);
    const endTime = performance.now();

    return endTime - startTime;
  }

  async measureSerialization(impl: CRDTTodoList): Promise<{
    serializeTime: number;
    deserializeTime: number;
    size: number;
  }> {
    const startSerialize = performance.now();
    const data = impl.serialize();
    const endSerialize = performance.now();

    const clone = impl.clone();
    const startDeserialize = performance.now();
    clone.deserialize(data);
    const endDeserialize = performance.now();

    return {
      serializeTime: endSerialize - startSerialize,
      deserializeTime: endDeserialize - startDeserialize,
      size: data.byteLength
    };
  }

  private getMemoryUsage(): number {
    if (typeof performance !== 'undefined' && 'memory' in performance) {
      return (performance as any).memory.usedJSHeapSize;
    }
    if (typeof process !== 'undefined' && process.memoryUsage) {
      return process.memoryUsage().heapUsed;
    }
    return 0;
  }
}

/**
 * Scenario 1: Sequential Adds
 */
export function sequentialAdds(count: number) {
  return (impl: CRDTTodoList) => {
    impl.create('Sequential Adds Test');

    for (let i = 0; i < count; i++) {
      impl.addTodo(`todo-${i}`, `Task ${i}: Do something important`);
    }
  };
}

/**
 * Scenario 2: Concurrent Edits (2 peers)
 */
export function concurrentEdits2Peer(count: number) {
  return (impl: CRDTTodoList) => {
    impl.create('Concurrent Test');

    // Peer 1: Add even-numbered todos
    for (let i = 0; i < count; i += 2) {
      impl.addTodo(`todo-${i}`, `Peer 1 Task ${i}`);
    }
  };
}

export function concurrentEdits2PeerSecond(count: number) {
  return (impl: CRDTTodoList) => {
    impl.create('Concurrent Test');

    // Peer 2: Add odd-numbered todos
    for (let i = 1; i < count; i += 2) {
      impl.addTodo(`todo-${i}`, `Peer 2 Task ${i}`);
    }
  };
}

/**
 * Scenario 3: Conflict Resolution (same todo edited by multiple peers)
 */
export function conflictResolution(count: number) {
  return (impl: CRDTTodoList) => {
    impl.create('Conflict Test');

    // First add some todos
    for (let i = 0; i < 10; i++) {
      impl.addTodo(`todo-${i}`, `Initial task ${i}`);
    }

    // Then edit the same todos multiple times (simulating conflicts)
    for (let i = 0; i < count; i++) {
      const todoId = `todo-${i % 10}`;
      impl.updateTodoText(todoId, `Updated ${i} times by peer`);
    }
  };
}

/**
 * Scenario 4: Mixed Operations
 */
export function mixedOperations(count: number) {
  return (impl: CRDTTodoList) => {
    impl.create('Mixed Operations Test');

    for (let i = 0; i < count; i++) {
      const op = i % 4;
      const todoId = `todo-${i}`;

      switch (op) {
        case 0: // Add
          impl.addTodo(todoId, `Task ${i}`);
          break;
        case 1: // Update
          if (i > 0) {
            impl.updateTodoText(`todo-${i - 1}`, `Updated task ${i - 1}`);
          }
          break;
        case 2: // Toggle
          if (i > 0) {
            impl.toggleTodo(`todo-${i - 1}`);
          }
          break;
        case 3: // Delete (occasionally)
          if (i > 10 && i % 20 === 0) {
            impl.deleteTodo(`todo-${i - 10}`);
          }
          break;
      }
    }
  };
}

/**
 * Convergence test: Verify that two replicas converge to same state
 */
export function verifyConvergence(
  impl1: CRDTTodoList,
  impl2: CRDTTodoList
): boolean {
  const todos1 = impl1.getAllTodos().sort((a, b) => a.id.localeCompare(b.id));
  const todos2 = impl2.getAllTodos().sort((a, b) => a.id.localeCompare(b.id));

  if (todos1.length !== todos2.length) {
    console.error(`Convergence failed: length mismatch ${todos1.length} vs ${todos2.length}`);
    return false;
  }

  for (let i = 0; i < todos1.length; i++) {
    const t1 = todos1[i];
    const t2 = todos2[i];

    if (t1.id !== t2.id || t1.text !== t2.text || t1.completed !== t2.completed) {
      console.error(`Convergence failed at index ${i}:`, t1, t2);
      return false;
    }
  }

  return true;
}
