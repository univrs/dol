/**
 * Loro CRDT TodoList Implementation
 *
 * Loro is a new CRDT library with excellent performance characteristics.
 * Features:
 * - Rust core compiled to WASM
 * - Optimized for real-time collaboration
 * - Rich text support with Peritext
 * - Time-travel capabilities
 */

import { Loro, LoroMap, LoroList } from 'loro-crdt';
import type { CRDTTodoList, Todo } from '../../common/domain';

export class LoroTodoList implements CRDTTodoList {
  private doc: Loro;
  private todosMap: LoroMap;
  private operationCount: number = 0;

  constructor() {
    this.doc = new Loro();
    this.todosMap = this.doc.getMap('todos');
    this.doc.getMap('meta').set('title', '');
    this.doc.getMap('meta').set('lastModified', Date.now());
  }

  create(title: string): void {
    this.doc.getMap('meta').set('title', title);
    this.doc.getMap('meta').set('lastModified', Date.now());
    this.operationCount++;
  }

  addTodo(id: string, text: string): void {
    const todoMap = this.todosMap.setContainer(id, new LoroMap()) as LoroMap;
    todoMap.set('id', id);
    todoMap.set('text', text);
    todoMap.set('completed', false);
    todoMap.set('createdAt', Date.now());
    todoMap.set('updatedAt', Date.now());

    this.doc.getMap('meta').set('lastModified', Date.now());
    this.operationCount++;
  }

  updateTodoText(id: string, text: string): void {
    const todoMap = this.todosMap.get(id) as LoroMap | undefined;
    if (todoMap) {
      todoMap.set('text', text);
      todoMap.set('updatedAt', Date.now());
      this.doc.getMap('meta').set('lastModified', Date.now());
    }
    this.operationCount++;
  }

  toggleTodo(id: string): void {
    const todoMap = this.todosMap.get(id) as LoroMap | undefined;
    if (todoMap) {
      const completed = todoMap.get('completed') as boolean;
      todoMap.set('completed', !completed);
      todoMap.set('updatedAt', Date.now());
      this.doc.getMap('meta').set('lastModified', Date.now());
    }
    this.operationCount++;
  }

  deleteTodo(id: string): void {
    this.todosMap.delete(id);
    this.doc.getMap('meta').set('lastModified', Date.now());
    this.operationCount++;
  }

  setTitle(title: string): void {
    this.doc.getMap('meta').set('title', title);
    this.doc.getMap('meta').set('lastModified', Date.now());
    this.operationCount++;
  }

  getTodo(id: string): Todo | undefined {
    const todoMap = this.todosMap.get(id) as LoroMap | undefined;
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
    const entries = this.todosMap.toJSON() as Record<string, any>;

    for (const [key, value] of Object.entries(entries)) {
      if (value && typeof value === 'object') {
        todos.push({
          id: value.id || key,
          text: value.text || '',
          completed: value.completed || false,
          createdAt: value.createdAt || 0,
          updatedAt: value.updatedAt || 0
        });
      }
    }

    return todos;
  }

  getTitle(): string {
    return this.doc.getMap('meta').get('title') as string || '';
  }

  merge(other: unknown): void {
    if (other instanceof LoroTodoList) {
      const updates = other.doc.exportFrom();
      this.doc.import(updates);
    } else if (other instanceof Uint8Array) {
      this.doc.import(other);
    }
  }

  serialize(): Uint8Array {
    return this.doc.exportFrom();
  }

  deserialize(data: Uint8Array): void {
    this.doc.import(data);
  }

  clone(): CRDTTodoList {
    const cloned = new LoroTodoList();
    const data = this.serialize();
    cloned.deserialize(data);
    cloned.operationCount = this.operationCount;
    return cloned;
  }

  getByteSize(): number {
    return this.doc.exportFrom().byteLength;
  }

  getOperationCount(): number {
    return this.operationCount;
  }

  // Loro-specific helpers

  getDoc(): Loro {
    return this.doc;
  }

  commit(): void {
    this.doc.commit();
  }

  checkout(frontiers: any): LoroTodoList {
    const cloned = this.clone() as LoroTodoList;
    cloned.doc.checkout(frontiers);
    return cloned;
  }
}
