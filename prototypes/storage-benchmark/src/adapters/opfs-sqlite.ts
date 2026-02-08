/**
 * OPFS + SQLite WASM Storage Adapter
 *
 * Uses Origin Private File System with SQLite compiled to WebAssembly.
 * Pros: SQL queries, good performance, ACID transactions
 * Cons: WASM overhead, browser support varies, setup complexity
 */

import { StorageAdapter, StorageMetrics, WriteOptions, ReadOptions, TransactionOptions } from './base.js';

// Type definitions for SQLite WASM (simplified)
interface SQLiteDB {
  exec(sql: string, options?: any): any;
  prepare(sql: string): SQLiteStatement;
  close(): void;
}

interface SQLiteStatement {
  bind(values: any[]): void;
  step(): boolean;
  get(): any[];
  reset(): void;
  finalize(): void;
}

interface SQLiteAPI {
  opfs?: {
    OpfsDb: new (filename: string) => SQLiteDB;
  };
}

export class OPFSSQLiteAdapter extends StorageAdapter {
  private db: SQLiteDB | null = null;
  private sqlite3: SQLiteAPI | null = null;
  private insertStmt: SQLiteStatement | null = null;
  private selectStmt: SQLiteStatement | null = null;
  private deleteStmt: SQLiteStatement | null = null;

  async initialize(): Promise<void> {
    try {
      // Load SQLite WASM module
      // Note: This is a simplified version - actual implementation would use the official SQLite WASM
      const sqlite3InitModule = (globalThis as any).sqlite3InitModule;

      if (!sqlite3InitModule) {
        throw new Error('SQLite WASM module not loaded. Include sqlite3.js in your HTML.');
      }

      this.sqlite3 = await sqlite3InitModule({
        print: console.log,
        printErr: console.error,
      });

      if (!this.sqlite3?.opfs) {
        throw new Error('OPFS support not available in SQLite WASM');
      }

      // Open database in OPFS
      this.db = new this.sqlite3.opfs.OpfsDb(`${this.dbName}.db`);

      // Create table
      this.db.exec(`
        CREATE TABLE IF NOT EXISTS ${this.dbName} (
          key TEXT PRIMARY KEY,
          value TEXT NOT NULL,
          created_at INTEGER DEFAULT (strftime('%s', 'now')),
          updated_at INTEGER DEFAULT (strftime('%s', 'now'))
        )
      `);

      // Create index for faster queries
      this.db.exec(`CREATE INDEX IF NOT EXISTS idx_created_at ON ${this.dbName}(created_at)`);

      // Prepare statements for reuse
      this.insertStmt = this.db.prepare(`
        INSERT OR REPLACE INTO ${this.dbName} (key, value, updated_at)
        VALUES (?, ?, strftime('%s', 'now'))
      `);

      this.selectStmt = this.db.prepare(`
        SELECT value FROM ${this.dbName} WHERE key = ?
      `);

      this.deleteStmt = this.db.prepare(`
        DELETE FROM ${this.dbName} WHERE key = ?
      `);

    } catch (error) {
      console.error('OPFS SQLite initialization error:', error);
      throw error;
    }
  }

  async write(key: string, value: any): Promise<StorageMetrics> {
    const startTime = performance.now();
    const valueStr = JSON.stringify(value);
    const bytesWritten = new Blob([valueStr]).size;

    if (!this.db || !this.insertStmt) {
      throw new Error('Database not initialized');
    }

    try {
      this.insertStmt.bind([key, valueStr]);
      this.insertStmt.step();
      this.insertStmt.reset();

      const operationTime = performance.now() - startTime;
      return {
        operationTime,
        bytesWritten,
        operationCount: 1,
      };
    } catch (error) {
      throw new Error(`Write error: ${error}`);
    }
  }

  async writeBatch(records: Array<{ key: string; value: any }>, options?: WriteOptions): Promise<StorageMetrics> {
    const startTime = performance.now();
    let bytesWritten = 0;
    const errors: string[] = [];

    if (!this.db) {
      throw new Error('Database not initialized');
    }

    try {
      // Use transaction for batch write
      this.db.exec('BEGIN TRANSACTION');

      const stmt = this.db.prepare(`
        INSERT OR REPLACE INTO ${this.dbName} (key, value, updated_at)
        VALUES (?, ?, strftime('%s', 'now'))
      `);

      for (const record of records) {
        try {
          const valueStr = JSON.stringify(record.value);
          bytesWritten += new Blob([valueStr]).size;

          stmt.bind([record.key, valueStr]);
          stmt.step();
          stmt.reset();
        } catch (error) {
          errors.push(`Error writing key ${record.key}: ${error}`);
        }
      }

      stmt.finalize();
      this.db.exec('COMMIT');

      const operationTime = performance.now() - startTime;
      return {
        operationTime,
        bytesWritten,
        operationCount: records.length,
        errors: errors.length > 0 ? errors : undefined,
      };
    } catch (error) {
      // Rollback on error
      try {
        this.db.exec('ROLLBACK');
      } catch (rollbackError) {
        console.error('Rollback error:', rollbackError);
      }
      throw new Error(`Batch write error: ${error}`);
    }
  }

  async read(key: string): Promise<{ value: any; metrics: StorageMetrics }> {
    const startTime = performance.now();

    if (!this.db || !this.selectStmt) {
      throw new Error('Database not initialized');
    }

    try {
      this.selectStmt.bind([key]);
      const hasRow = this.selectStmt.step();

      let value = null;
      let bytesRead = 0;

      if (hasRow) {
        const row = this.selectStmt.get();
        const valueStr = row[0] as string;
        value = JSON.parse(valueStr);
        bytesRead = new Blob([valueStr]).size;
      }

      this.selectStmt.reset();

      const operationTime = performance.now() - startTime;
      return {
        value,
        metrics: {
          operationTime,
          bytesRead,
          operationCount: 1,
        },
      };
    } catch (error) {
      this.selectStmt.reset();
      throw new Error(`Read error: ${error}`);
    }
  }

  async readBatch(keys: string[], options?: ReadOptions): Promise<{ values: any[]; metrics: StorageMetrics }> {
    const startTime = performance.now();
    let bytesRead = 0;
    const values: any[] = [];
    const errors: string[] = [];

    if (!this.db) {
      throw new Error('Database not initialized');
    }

    try {
      const stmt = this.db.prepare(`
        SELECT value FROM ${this.dbName} WHERE key = ?
      `);

      for (const key of keys) {
        try {
          stmt.bind([key]);
          const hasRow = stmt.step();

          if (hasRow) {
            const row = stmt.get();
            const valueStr = row[0] as string;
            const value = JSON.parse(valueStr);
            values.push(value);
            bytesRead += new Blob([valueStr]).size;
          } else {
            values.push(null);
          }

          stmt.reset();
        } catch (error) {
          errors.push(`Error reading key ${key}: ${error}`);
          values.push(null);
          stmt.reset();
        }
      }

      stmt.finalize();

      const operationTime = performance.now() - startTime;
      return {
        values,
        metrics: {
          operationTime,
          bytesRead,
          operationCount: keys.length,
          errors: errors.length > 0 ? errors : undefined,
        },
      };
    } catch (error) {
      throw new Error(`Batch read error: ${error}`);
    }
  }

  async delete(key: string): Promise<StorageMetrics> {
    const startTime = performance.now();

    if (!this.db || !this.deleteStmt) {
      throw new Error('Database not initialized');
    }

    try {
      this.deleteStmt.bind([key]);
      this.deleteStmt.step();
      this.deleteStmt.reset();

      const operationTime = performance.now() - startTime;
      return {
        operationTime,
        operationCount: 1,
      };
    } catch (error) {
      this.deleteStmt.reset();
      throw new Error(`Delete error: ${error}`);
    }
  }

  async deleteBatch(keys: string[]): Promise<StorageMetrics> {
    const startTime = performance.now();
    const errors: string[] = [];

    if (!this.db) {
      throw new Error('Database not initialized');
    }

    try {
      this.db.exec('BEGIN TRANSACTION');

      const stmt = this.db.prepare(`DELETE FROM ${this.dbName} WHERE key = ?`);

      for (const key of keys) {
        try {
          stmt.bind([key]);
          stmt.step();
          stmt.reset();
        } catch (error) {
          errors.push(`Error deleting key ${key}: ${error}`);
          stmt.reset();
        }
      }

      stmt.finalize();
      this.db.exec('COMMIT');

      const operationTime = performance.now() - startTime;
      return {
        operationTime,
        operationCount: keys.length,
        errors: errors.length > 0 ? errors : undefined,
      };
    } catch (error) {
      try {
        this.db.exec('ROLLBACK');
      } catch (rollbackError) {
        console.error('Rollback error:', rollbackError);
      }
      throw new Error(`Batch delete error: ${error}`);
    }
  }

  async getAllKeys(): Promise<string[]> {
    if (!this.db) {
      throw new Error('Database not initialized');
    }

    const keys: string[] = [];
    const stmt = this.db.prepare(`SELECT key FROM ${this.dbName}`);

    while (stmt.step()) {
      const row = stmt.get();
      keys.push(row[0] as string);
    }

    stmt.finalize();
    return keys;
  }

  async getStorageSize(): Promise<number> {
    if (!this.db) {
      throw new Error('Database not initialized');
    }

    // SQLite page_count * page_size
    const stmt = this.db.prepare('PRAGMA page_count');
    stmt.step();
    const pageCount = stmt.get()[0] as number;
    stmt.finalize();

    const stmt2 = this.db.prepare('PRAGMA page_size');
    stmt2.step();
    const pageSize = stmt2.get()[0] as number;
    stmt2.finalize();

    return pageCount * pageSize;
  }

  async clear(): Promise<void> {
    if (!this.db) {
      throw new Error('Database not initialized');
    }

    this.db.exec(`DELETE FROM ${this.dbName}`);
    this.db.exec('VACUUM'); // Reclaim space
  }

  async close(): Promise<void> {
    if (this.insertStmt) {
      this.insertStmt.finalize();
      this.insertStmt = null;
    }
    if (this.selectStmt) {
      this.selectStmt.finalize();
      this.selectStmt = null;
    }
    if (this.deleteStmt) {
      this.deleteStmt.finalize();
      this.deleteStmt = null;
    }
    if (this.db) {
      this.db.close();
      this.db = null;
    }
  }

  supportsTransactions(): boolean {
    return true;
  }

  async beginTransaction(options?: TransactionOptions): Promise<any> {
    if (!this.db) {
      throw new Error('Database not initialized');
    }

    this.db.exec('BEGIN TRANSACTION');
    return { active: true };
  }

  async commitTransaction(transaction: any): Promise<void> {
    if (!this.db) {
      throw new Error('Database not initialized');
    }

    this.db.exec('COMMIT');
  }

  async rollbackTransaction(transaction: any): Promise<void> {
    if (!this.db) {
      throw new Error('Database not initialized');
    }

    this.db.exec('ROLLBACK');
  }

  getName(): string {
    return 'OPFS + SQLite WASM';
  }

  getCapabilities() {
    return {
      transactions: true,
      batching: true,
      compression: false,
      multiTab: true, // OPFS supports multi-tab with proper locking
      persistence: true,
    };
  }
}
