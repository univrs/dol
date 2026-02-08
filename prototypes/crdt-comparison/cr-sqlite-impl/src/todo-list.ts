/**
 * cr-sqlite TodoList Implementation
 *
 * cr-sqlite brings CRDT capabilities to SQLite, combining:
 * - Familiar SQL interface
 * - SQLite's proven reliability
 * - CRDT conflict resolution
 * - Efficient delta sync
 *
 * Note: This is a unique approach - SQL + CRDTs
 */

import type { CRDTTodoList, Todo } from '../../common/domain';

// Mock interface for cr-sqlite since it requires native setup
// In a real implementation, this would use @vlcn.io/crsqlite-wasm
interface CRSQLiteDB {
  exec(sql: string): void;
  prepare(sql: string): {
    run(...params: any[]): void;
    get(...params: any[]): any;
    all(...params: any[]): any[];
  };
  createCRR(tableName: string): void;
  getChanges(since: number): Uint8Array;
  applyChanges(changes: Uint8Array): void;
}

export class CRSQLiteTodoList implements CRDTTodoList {
  private db: CRSQLiteDB | null = null;
  private operationCount: number = 0;
  private mockData: Map<string, Todo> = new Map();
  private title: string = '';
  private lastModified: number = Date.now();

  constructor() {
    // In a real implementation, initialize cr-sqlite here
    // For this prototype, we'll use a mock implementation
    this.initializeMockDB();
  }

  private initializeMockDB(): void {
    // Mock initialization - real implementation would use:
    // import initWasm from '@vlcn.io/crsqlite-wasm';
    // const sqlite = await initWasm();
    // this.db = await sqlite.open(':memory:');

    // Mock schema creation
    // CREATE TABLE todos (
    //   id TEXT PRIMARY KEY NOT NULL,
    //   text TEXT NOT NULL,
    //   completed INTEGER NOT NULL,
    //   createdAt INTEGER NOT NULL,
    //   updatedAt INTEGER NOT NULL
    // );
    // SELECT crsql_as_crr('todos');
  }

  create(title: string): void {
    this.title = title;
    this.lastModified = Date.now();
    this.operationCount++;
  }

  addTodo(id: string, text: string): void {
    // Real SQL: INSERT INTO todos (id, text, completed, createdAt, updatedAt)
    //           VALUES (?, ?, 0, ?, ?)
    this.mockData.set(id, {
      id,
      text,
      completed: false,
      createdAt: Date.now(),
      updatedAt: Date.now()
    });
    this.lastModified = Date.now();
    this.operationCount++;
  }

  updateTodoText(id: string, text: string): void {
    // Real SQL: UPDATE todos SET text = ?, updatedAt = ? WHERE id = ?
    const todo = this.mockData.get(id);
    if (todo) {
      todo.text = text;
      todo.updatedAt = Date.now();
      this.mockData.set(id, todo);
    }
    this.lastModified = Date.now();
    this.operationCount++;
  }

  toggleTodo(id: string): void {
    // Real SQL: UPDATE todos SET completed = NOT completed, updatedAt = ? WHERE id = ?
    const todo = this.mockData.get(id);
    if (todo) {
      todo.completed = !todo.completed;
      todo.updatedAt = Date.now();
      this.mockData.set(id, todo);
    }
    this.lastModified = Date.now();
    this.operationCount++;
  }

  deleteTodo(id: string): void {
    // Real SQL: DELETE FROM todos WHERE id = ?
    this.mockData.delete(id);
    this.lastModified = Date.now();
    this.operationCount++;
  }

  setTitle(title: string): void {
    this.title = title;
    this.lastModified = Date.now();
    this.operationCount++;
  }

  getTodo(id: string): Todo | undefined {
    // Real SQL: SELECT * FROM todos WHERE id = ?
    return this.mockData.get(id);
  }

  getAllTodos(): Todo[] {
    // Real SQL: SELECT * FROM todos ORDER BY createdAt
    return Array.from(this.mockData.values());
  }

  getTitle(): string {
    return this.title;
  }

  merge(other: unknown): void {
    if (other instanceof CRSQLiteTodoList) {
      // Real implementation: apply changesets from other DB
      // const changes = other.db.getChanges(this.lastSyncVersion);
      // this.db.applyChanges(changes);

      // Mock merge: last-write-wins based on updatedAt
      for (const [id, todo] of other.mockData.entries()) {
        const existing = this.mockData.get(id);
        if (!existing || todo.updatedAt > existing.updatedAt) {
          this.mockData.set(id, { ...todo });
        }
      }
    } else if (other instanceof Uint8Array) {
      // Real implementation: apply binary changeset
      // this.db.applyChanges(other);
    }
  }

  serialize(): Uint8Array {
    // Real implementation: SELECT * FROM crsql_changes WHERE version > ?
    // Returns binary changeset

    // Mock: Serialize to JSON then to Uint8Array
    const data = {
      title: this.title,
      lastModified: this.lastModified,
      todos: Array.from(this.mockData.entries())
    };
    const json = JSON.stringify(data);
    return new TextEncoder().encode(json);
  }

  deserialize(data: Uint8Array): void {
    // Real implementation: INSERT INTO crsql_changes ...
    // Applies binary changeset

    // Mock: Deserialize from JSON
    const json = new TextDecoder().decode(data);
    const parsed = JSON.parse(json);
    this.title = parsed.title;
    this.lastModified = parsed.lastModified;
    this.mockData = new Map(parsed.todos);
  }

  clone(): CRDTTodoList {
    const cloned = new CRSQLiteTodoList();
    cloned.title = this.title;
    cloned.lastModified = this.lastModified;
    cloned.mockData = new Map(this.mockData);
    cloned.operationCount = this.operationCount;
    return cloned;
  }

  getByteSize(): number {
    return this.serialize().byteLength;
  }

  getOperationCount(): number {
    return this.operationCount;
  }

  // cr-sqlite specific helpers

  getChanges(since: number): Uint8Array {
    // Real: SELECT * FROM crsql_changes WHERE version > ?
    return new Uint8Array();
  }

  applyChanges(changes: Uint8Array): void {
    // Real: INSERT INTO crsql_changes ...
  }

  compact(): void {
    // Real: SELECT crsql_compact()
    // Garbage collect tombstones
  }
}
