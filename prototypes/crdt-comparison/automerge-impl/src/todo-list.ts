/**
 * Automerge 3.0 TodoList Implementation
 *
 * Uses Automerge's automatic CRDT synchronization with:
 * - LWW (Last-Write-Wins) for scalar fields
 * - OR-Set semantics for the todo map
 * - Text CRDT for todo text fields (future: use Automerge.Text)
 */

import * as Automerge from '@automerge/automerge';
import type { CRDTTodoList, Todo } from '../../common/domain';

interface AutomergeTodoList {
  title: string;
  lastModified: number;
  todos: { [id: string]: Todo };
}

export class AutomergeTodoList implements CRDTTodoList {
  private doc: Automerge.Doc<AutomergeTodoList>;
  private operationCount: number = 0;

  constructor() {
    this.doc = Automerge.from<AutomergeTodoList>({
      title: '',
      lastModified: Date.now(),
      todos: {}
    });
  }

  create(title: string): void {
    this.doc = Automerge.change(this.doc, (doc) => {
      doc.title = title;
      doc.lastModified = Date.now();
    });
    this.operationCount++;
  }

  addTodo(id: string, text: string): void {
    this.doc = Automerge.change(this.doc, (doc) => {
      doc.todos[id] = {
        id,
        text,
        completed: false,
        createdAt: Date.now(),
        updatedAt: Date.now()
      };
      doc.lastModified = Date.now();
    });
    this.operationCount++;
  }

  updateTodoText(id: string, text: string): void {
    this.doc = Automerge.change(this.doc, (doc) => {
      if (doc.todos[id]) {
        doc.todos[id].text = text;
        doc.todos[id].updatedAt = Date.now();
        doc.lastModified = Date.now();
      }
    });
    this.operationCount++;
  }

  toggleTodo(id: string): void {
    this.doc = Automerge.change(this.doc, (doc) => {
      if (doc.todos[id]) {
        doc.todos[id].completed = !doc.todos[id].completed;
        doc.todos[id].updatedAt = Date.now();
        doc.lastModified = Date.now();
      }
    });
    this.operationCount++;
  }

  deleteTodo(id: string): void {
    this.doc = Automerge.change(this.doc, (doc) => {
      delete doc.todos[id];
      doc.lastModified = Date.now();
    });
    this.operationCount++;
  }

  setTitle(title: string): void {
    this.doc = Automerge.change(this.doc, (doc) => {
      doc.title = title;
      doc.lastModified = Date.now();
    });
    this.operationCount++;
  }

  getTodo(id: string): Todo | undefined {
    return this.doc.todos[id];
  }

  getAllTodos(): Todo[] {
    return Object.values(this.doc.todos);
  }

  getTitle(): string {
    return this.doc.title;
  }

  merge(other: unknown): void {
    if (other instanceof AutomergeTodoList) {
      this.doc = Automerge.merge(this.doc, other.doc);
    } else if (other instanceof Uint8Array) {
      const [merged] = Automerge.applyChanges(this.doc, [other]);
      this.doc = merged;
    }
  }

  serialize(): Uint8Array {
    return Automerge.save(this.doc);
  }

  deserialize(data: Uint8Array): void {
    this.doc = Automerge.load<AutomergeTodoList>(data);
  }

  clone(): CRDTTodoList {
    const cloned = new AutomergeTodoList();
    cloned.doc = Automerge.clone(this.doc);
    cloned.operationCount = this.operationCount;
    return cloned;
  }

  getByteSize(): number {
    return Automerge.save(this.doc).byteLength;
  }

  getOperationCount(): number {
    return this.operationCount;
  }

  // Automerge-specific helpers

  getDoc(): Automerge.Doc<AutomergeTodoList> {
    return this.doc;
  }

  getChanges(since?: Automerge.Heads): Uint8Array[] {
    return Automerge.getChanges(since ? this.doc : Automerge.init(), this.doc);
  }

  applyChanges(changes: Uint8Array[]): void {
    const [newDoc] = Automerge.applyChanges(this.doc, changes);
    this.doc = newDoc;
  }

  getHeads(): Automerge.Heads {
    return Automerge.getHeads(this.doc);
  }
}
