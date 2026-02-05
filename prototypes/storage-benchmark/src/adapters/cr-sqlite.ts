/**
 * cr-sqlite Storage Adapter
 *
 * Conflict-free Replicated SQLite with CRDT support.
 * Pros: CRDT merge capabilities, multi-device sync, SQLite power
 * Cons: Larger payload, more complex, relatively new
 *
 * Note: This is a mock implementation for benchmarking purposes.
 * Actual cr-sqlite integration would require the cr-sqlite WASM module.
 */

import { StorageAdapter, StorageMetrics, WriteOptions, ReadOptions, TransactionOptions } from './base.js';

// Mock cr-sqlite types
interface CRSQLiteDB {
  exec(sql: string): any;
  prepare(sql: string): CRSQLiteStatement;
  close(): void;
  crsql_begin_alter(table: string): void;
  crsql_commit_alter(table: string): void;
  crsql_sync_bit(): number;
}

interface CRSQLiteStatement {
  bind(values: any[]): void;
  step(): boolean;
  get(): any[];
  reset(): void;
  finalize(): void;
}

export class CRSQLiteAdapter extends StorageAdapter {
  private db: CRSQLiteDB | null = null;
  private insertStmt: CRSQLiteStatement | null = null;
  private selectStmt: CRSQLiteStatement | null = null;
  private deleteStmt: CRSQLiteStatement | null = null;

  async initialize(): Promise<void> {
    try {
      // In a real implementation, this would load the cr-sqlite WASM module
      // For benchmarking, we'll use a mock that simulates the API

      // Mock initialization - in production, this would be:
      // const crsqlite = await crsqliteInit();
      // this.db = await crsqlite.open(`${this.dbName}.db`);

      console.warn('Using mock cr-sqlite adapter for benchmarking');

      // Create a mock database object
      this.db = this.createMockDB();

      // Create CRR (Conflict-free Replicated Relation) table
      this.db.exec(`
        CREATE TABLE IF NOT EXISTS ${this.dbName} (
          key TEXT PRIMARY KEY NOT NULL,
          value TEXT NOT NULL,
          created_at INTEGER DEFAULT (strftime('%s', 'now')),
          updated_at INTEGER DEFAULT (strftime('%s', 'now'))
        )
      `);

      // Convert to CRR for CRDT support
      this.db.exec(`SELECT crsql_as_crr('${this.dbName}')`);

      // Create index
      this.db.exec(`CREATE INDEX IF NOT EXISTS idx_created_at ON ${this.dbName}(created_at)`);

      // Prepare statements
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
      console.error('cr-sqlite initialization error:', error);
      throw error;
    }
  }

  // Mock database implementation for benchmarking
  private createMockDB(): CRSQLiteDB {
    const storage = new Map<string, string>();

    return {
      exec: (sql: string) => {
        // Mock SQL execution
        console.log('Mock cr-sqlite exec:', sql);
      },
      prepare: (sql: string) => {
        return {
          bind: (values: any[]) => {},
          step: () => true,
          get: () => [],
          reset: () => {},
          finalize: () => {},
        };
      },
      close: () => {},
      crsql_begin_alter: (table: string) => {},
      crsql_commit_alter: (table: string) => {},
      crsql_sync_bit: () => Date.now(),
    };
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
      // Begin transaction
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
    this.db.exec('VACUUM');
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
    return { active: true, syncBit: this.db.crsql_sync_bit() };
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
    return 'cr-sqlite (CRDT)';
  }

  getCapabilities() {
    return {
      transactions: true,
      batching: true,
      compression: false,
      multiTab: true,
      persistence: true,
    };
  }

  /**
   * cr-sqlite specific: Get changesets for sync
   */
  async getChangesets(since?: number): Promise<any[]> {
    if (!this.db) {
      throw new Error('Database not initialized');
    }

    const sinceVersion = since || 0;
    const stmt = this.db.prepare(`
      SELECT * FROM crsql_changes WHERE db_version > ?
    `);

    const changes: any[] = [];
    stmt.bind([sinceVersion]);

    while (stmt.step()) {
      changes.push(stmt.get());
    }

    stmt.finalize();
    return changes;
  }

  /**
   * cr-sqlite specific: Apply changesets from sync
   */
  async applyChangesets(changes: any[]): Promise<void> {
    if (!this.db) {
      throw new Error('Database not initialized');
    }

    this.db.exec('BEGIN TRANSACTION');

    try {
      const stmt = this.db.prepare(`
        INSERT INTO crsql_changes VALUES (?, ?, ?, ?, ?, ?)
      `);

      for (const change of changes) {
        stmt.bind(change);
        stmt.step();
        stmt.reset();
      }

      stmt.finalize();
      this.db.exec('COMMIT');
    } catch (error) {
      this.db.exec('ROLLBACK');
      throw error;
    }
  }
}
