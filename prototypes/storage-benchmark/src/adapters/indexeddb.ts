/**
 * IndexedDB Storage Adapter
 *
 * Native browser key-value storage implementation.
 * Pros: Native, widely supported, no dependencies
 * Cons: Async-only, complex API, limited query capabilities
 */

import { StorageAdapter, StorageMetrics, WriteOptions, ReadOptions, TransactionOptions } from './base.js';

export class IndexedDBAdapter extends StorageAdapter {
  private db: IDBDatabase | null = null;
  private storeName = 'storage';

  async initialize(): Promise<void> {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(this.dbName, this.version);

      request.onerror = () => reject(request.error);
      request.onsuccess = () => {
        this.db = request.result;
        resolve();
      };

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;
        if (!db.objectStoreNames.contains(this.storeName)) {
          db.createObjectStore(this.storeName);
        }
      };
    });
  }

  async write(key: string, value: any): Promise<StorageMetrics> {
    const startTime = performance.now();
    const valueStr = JSON.stringify(value);
    const bytesWritten = new Blob([valueStr]).size;

    return new Promise((resolve, reject) => {
      if (!this.db) {
        reject(new Error('Database not initialized'));
        return;
      }

      const transaction = this.db.transaction([this.storeName], 'readwrite');
      const store = transaction.objectStore(this.storeName);
      const request = store.put(value, key);

      request.onsuccess = () => {
        const operationTime = performance.now() - startTime;
        resolve({
          operationTime,
          bytesWritten,
          operationCount: 1,
        });
      };

      request.onerror = () => reject(request.error);
    });
  }

  async writeBatch(records: Array<{ key: string; value: any }>, options?: WriteOptions): Promise<StorageMetrics> {
    const startTime = performance.now();
    let bytesWritten = 0;
    const errors: string[] = [];

    return new Promise((resolve, reject) => {
      if (!this.db) {
        reject(new Error('Database not initialized'));
        return;
      }

      const transaction = this.db.transaction([this.storeName], 'readwrite');
      const store = transaction.objectStore(this.storeName);

      // Calculate bytes written
      for (const record of records) {
        const valueStr = JSON.stringify(record.value);
        bytesWritten += new Blob([valueStr]).size;
      }

      // Write all records in the transaction
      let completed = 0;
      for (const record of records) {
        const request = store.put(record.value, record.key);

        request.onsuccess = () => {
          completed++;
          if (completed === records.length) {
            const operationTime = performance.now() - startTime;
            resolve({
              operationTime,
              bytesWritten,
              operationCount: records.length,
              errors: errors.length > 0 ? errors : undefined,
            });
          }
        };

        request.onerror = () => {
          errors.push(`Error writing key ${record.key}: ${request.error}`);
          completed++;
          if (completed === records.length) {
            const operationTime = performance.now() - startTime;
            resolve({
              operationTime,
              bytesWritten,
              operationCount: records.length,
              errors,
            });
          }
        };
      }
    });
  }

  async read(key: string): Promise<{ value: any; metrics: StorageMetrics }> {
    const startTime = performance.now();

    return new Promise((resolve, reject) => {
      if (!this.db) {
        reject(new Error('Database not initialized'));
        return;
      }

      const transaction = this.db.transaction([this.storeName], 'readonly');
      const store = transaction.objectStore(this.storeName);
      const request = store.get(key);

      request.onsuccess = () => {
        const value = request.result;
        const valueStr = JSON.stringify(value);
        const bytesRead = new Blob([valueStr]).size;
        const operationTime = performance.now() - startTime;

        resolve({
          value,
          metrics: {
            operationTime,
            bytesRead,
            operationCount: 1,
          },
        });
      };

      request.onerror = () => reject(request.error);
    });
  }

  async readBatch(keys: string[], options?: ReadOptions): Promise<{ values: any[]; metrics: StorageMetrics }> {
    const startTime = performance.now();
    let bytesRead = 0;
    const values: any[] = [];
    const errors: string[] = [];

    return new Promise((resolve, reject) => {
      if (!this.db) {
        reject(new Error('Database not initialized'));
        return;
      }

      const transaction = this.db.transaction([this.storeName], 'readonly');
      const store = transaction.objectStore(this.storeName);

      let completed = 0;
      for (const key of keys) {
        const request = store.get(key);

        request.onsuccess = () => {
          const value = request.result;
          values.push(value);
          if (value) {
            const valueStr = JSON.stringify(value);
            bytesRead += new Blob([valueStr]).size;
          }

          completed++;
          if (completed === keys.length) {
            const operationTime = performance.now() - startTime;
            resolve({
              values,
              metrics: {
                operationTime,
                bytesRead,
                operationCount: keys.length,
                errors: errors.length > 0 ? errors : undefined,
              },
            });
          }
        };

        request.onerror = () => {
          errors.push(`Error reading key ${key}: ${request.error}`);
          values.push(null);
          completed++;
          if (completed === keys.length) {
            const operationTime = performance.now() - startTime;
            resolve({
              values,
              metrics: {
                operationTime,
                bytesRead,
                operationCount: keys.length,
                errors,
              },
            });
          }
        };
      }
    });
  }

  async delete(key: string): Promise<StorageMetrics> {
    const startTime = performance.now();

    return new Promise((resolve, reject) => {
      if (!this.db) {
        reject(new Error('Database not initialized'));
        return;
      }

      const transaction = this.db.transaction([this.storeName], 'readwrite');
      const store = transaction.objectStore(this.storeName);
      const request = store.delete(key);

      request.onsuccess = () => {
        const operationTime = performance.now() - startTime;
        resolve({
          operationTime,
          operationCount: 1,
        });
      };

      request.onerror = () => reject(request.error);
    });
  }

  async deleteBatch(keys: string[]): Promise<StorageMetrics> {
    const startTime = performance.now();
    const errors: string[] = [];

    return new Promise((resolve, reject) => {
      if (!this.db) {
        reject(new Error('Database not initialized'));
        return;
      }

      const transaction = this.db.transaction([this.storeName], 'readwrite');
      const store = transaction.objectStore(this.storeName);

      let completed = 0;
      for (const key of keys) {
        const request = store.delete(key);

        request.onsuccess = () => {
          completed++;
          if (completed === keys.length) {
            const operationTime = performance.now() - startTime;
            resolve({
              operationTime,
              operationCount: keys.length,
              errors: errors.length > 0 ? errors : undefined,
            });
          }
        };

        request.onerror = () => {
          errors.push(`Error deleting key ${key}: ${request.error}`);
          completed++;
          if (completed === keys.length) {
            const operationTime = performance.now() - startTime;
            resolve({
              operationTime,
              operationCount: keys.length,
              errors,
            });
          }
        };
      }
    });
  }

  async getAllKeys(): Promise<string[]> {
    return new Promise((resolve, reject) => {
      if (!this.db) {
        reject(new Error('Database not initialized'));
        return;
      }

      const transaction = this.db.transaction([this.storeName], 'readonly');
      const store = transaction.objectStore(this.storeName);
      const request = store.getAllKeys();

      request.onsuccess = () => {
        resolve(request.result.map(key => String(key)));
      };

      request.onerror = () => reject(request.error);
    });
  }

  async getStorageSize(): Promise<number> {
    if ('estimate' in navigator.storage) {
      const estimate = await navigator.storage.estimate();
      return estimate.usage || 0;
    }
    return 0;
  }

  async clear(): Promise<void> {
    return new Promise((resolve, reject) => {
      if (!this.db) {
        reject(new Error('Database not initialized'));
        return;
      }

      const transaction = this.db.transaction([this.storeName], 'readwrite');
      const store = transaction.objectStore(this.storeName);
      const request = store.clear();

      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  }

  async close(): Promise<void> {
    if (this.db) {
      this.db.close();
      this.db = null;
    }
  }

  supportsTransactions(): boolean {
    return true;
  }

  async beginTransaction(options?: TransactionOptions): Promise<IDBTransaction> {
    if (!this.db) {
      throw new Error('Database not initialized');
    }

    const mode = options?.readOnly ? 'readonly' : 'readwrite';
    return this.db.transaction([this.storeName], mode);
  }

  async commitTransaction(transaction: IDBTransaction): Promise<void> {
    // IndexedDB transactions auto-commit when all requests are complete
    return new Promise((resolve, reject) => {
      transaction.oncomplete = () => resolve();
      transaction.onerror = () => reject(transaction.error);
      transaction.onabort = () => reject(new Error('Transaction aborted'));
    });
  }

  async rollbackTransaction(transaction: IDBTransaction): Promise<void> {
    transaction.abort();
  }

  getName(): string {
    return 'IndexedDB';
  }
}
