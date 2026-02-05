/**
 * Yrs (Y-CRDT) TodoList Implementation
 *
 * Yjs/Yrs is the most mature and widely-used CRDT library for web applications.
 * Features:
 * - Battle-tested in production (Notion, Linear, etc.)
 * - Excellent performance
 * - Rich ecosystem of providers and plugins
 * - Strong TypeScript support
 */

import * as Y from 'yjs';
import type { CRDTTodoList, Todo } from '../../common/domain';

export class YrsTodoList implements CRDTTodoList {
  private doc: Y.Doc;
  private todosMap: Y.Map<Y.Map<any>>;
  private meta: Y.Map<any>;
  private operationCount: number = 0;

  constructor() {
    this.doc = new Y.Doc();
    this.todosMap = this.doc.getMap('todos');
    this.meta = this.doc.getMap('meta');
    this.meta.set('title', '');
    this.meta.set('lastModified', Date.now());
  }

  create(title: string): void {
    this.doc.transact(() => {
      this.meta.set('title', title);
      this.meta.set('lastModified', Date.now());
    });
    this.operationCount++;
  }

  addTodo(id: string, text: string): void {
    this.doc.transact(() => {
      const todoMap = new Y.Map();
      todoMap.set('id', id);
      todoMap.set('text', text);
      todoMap.set('completed', false);
      todoMap.set('createdAt', Date.now());
      todoMap.set('updatedAt', Date.now());

      this.todosMap.set(id, todoMap);
      this.meta.set('lastModified', Date.now());
    });
    this.operationCount++;
  }

  updateTodoText(id: string, text: string): void {
    this.doc.transact(() => {
      const todoMap = this.todosMap.get(id);
      if (todoMap) {
        todoMap.set('text', text);
        todoMap.set('updatedAt', Date.now());
        this.meta.set('lastModified', Date.now());
      }
    });
    this.operationCount++;
  }

  toggleTodo(id: string): void {
    this.doc.transact(() => {
      const todoMap = this.todosMap.get(id);
      if (todoMap) {
        const completed = todoMap.get('completed') as boolean;
        todoMap.set('completed', !completed);
        todoMap.set('updatedAt', Date.now());
        this.meta.set('lastModified', Date.now());
      }
    });
    this.operationCount++;
  }

  deleteTodo(id: string): void {
    this.doc.transact(() => {
      this.todosMap.delete(id);
      this.meta.set('lastModified', Date.now());
    });
    this.operationCount++;
  }

  setTitle(title: string): void {
    this.doc.transact(() => {
      this.meta.set('title', title);
      this.meta.set('lastModified', Date.now());
    });
    this.operationCount++;
  }

  getTodo(id: string): Todo | undefined {
    const todoMap = this.todosMap.get(id);
    if (!todoMap) return undefined;

    return {
      id: todoMap.get('id') as string,
      text: todoMap.get('text') as string,
      completed: todoMap.get('completed') as boolean,
      createdAt: todoMap.get('createdAt') as number,
      updatedAt: todoMap.get('updatedAt') as number
    };
  }

  getAllTodos(): Todo[] {
    const todos: Todo[] = [];

    this.todosMap.forEach((todoMap, id) => {
      todos.push({
        id: todoMap.get('id') as string,
        text: todoMap.get('text') as string,
        completed: todoMap.get('completed') as boolean,
        createdAt: todoMap.get('createdAt') as number,
        updatedAt: todoMap.get('updatedAt') as number
      });
    });

    return todos;
  }

  getTitle(): string {
    return this.meta.get('title') as string || '';
  }

  merge(other: unknown): void {
    if (other instanceof YrsTodoList) {
      const update = Y.encodeStateAsUpdate(other.doc);
      Y.applyUpdate(this.doc, update);
    } else if (other instanceof Uint8Array) {
      Y.applyUpdate(this.doc, other);
    }
  }

  serialize(): Uint8Array {
    return Y.encodeStateAsUpdate(this.doc);
  }

  deserialize(data: Uint8Array): void {
    Y.applyUpdate(this.doc, data);
  }

  clone(): CRDTTodoList {
    const cloned = new YrsTodoList();
    const update = Y.encodeStateAsUpdate(this.doc);
    Y.applyUpdate(cloned.doc, update);
    cloned.operationCount = this.operationCount;
    return cloned;
  }

  getByteSize(): number {
    return Y.encodeStateAsUpdate(this.doc).byteLength;
  }

  getOperationCount(): number {
    return this.operationCount;
  }

  // Yjs-specific helpers

  getDoc(): Y.Doc {
    return this.doc;
  }

  observeDeep(callback: (events: Y.YEvent<any>[], transaction: Y.Transaction) => void): void {
    this.doc.on('update', callback as any);
  }

  unobserveDeep(callback: (events: Y.YEvent<any>[], transaction: Y.Transaction) => void): void {
    this.doc.off('update', callback as any);
  }

  getStateVector(): Uint8Array {
    return Y.encodeStateVector(this.doc);
  }

  getDiff(stateVector: Uint8Array): Uint8Array {
    return Y.encodeStateAsUpdate(this.doc, stateVector);
  }
}
