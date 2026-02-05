/**
 * Base Storage Adapter Interface
 *
 * All storage adapters must implement this interface to ensure
 * consistent benchmarking across different storage mechanisms.
 */

export interface StorageMetrics {
  operationTime: number;       // Time in milliseconds
  bytesWritten?: number;        // Bytes written (if applicable)
  bytesRead?: number;           // Bytes read (if applicable)
  storageUsed?: number;         // Total storage used (bytes)
  operationCount: number;       // Number of operations performed
  errors?: string[];            // Any errors encountered
}

export interface WriteOptions {
  batch?: boolean;              // Use batch writes
  batchSize?: number;           // Records per batch
  overwrite?: boolean;          // Overwrite existing records
}

export interface ReadOptions {
  cache?: boolean;              // Use caching if available
  range?: {                     // Read a range of records
    start: string;
    end: string;
  };
}

export interface TransactionOptions {
  readOnly?: boolean;
  durability?: 'default' | 'strict' | 'relaxed';
}

export abstract class StorageAdapter {
  protected dbName: string;
  protected version: number;

  constructor(dbName: string = 'benchmark-db', version: number = 1) {
    this.dbName = dbName;
    this.version = version;
  }

  /**
   * Initialize the storage adapter
   */
  abstract initialize(): Promise<void>;

  /**
   * Write a single record
   */
  abstract write(key: string, value: any): Promise<StorageMetrics>;

  /**
   * Write multiple records
   */
  abstract writeBatch(records: Array<{ key: string; value: any }>, options?: WriteOptions): Promise<StorageMetrics>;

  /**
   * Read a single record
   */
  abstract read(key: string): Promise<{ value: any; metrics: StorageMetrics }>;

  /**
   * Read multiple records
   */
  abstract readBatch(keys: string[], options?: ReadOptions): Promise<{ values: any[]; metrics: StorageMetrics }>;

  /**
   * Delete a record
   */
  abstract delete(key: string): Promise<StorageMetrics>;

  /**
   * Delete multiple records
   */
  abstract deleteBatch(keys: string[]): Promise<StorageMetrics>;

  /**
   * Get all keys in storage
   */
  abstract getAllKeys(): Promise<string[]>;

  /**
   * Get storage size in bytes
   */
  abstract getStorageSize(): Promise<number>;

  /**
   * Clear all data
   */
  abstract clear(): Promise<void>;

  /**
   * Close/cleanup the storage connection
   */
  abstract close(): Promise<void>;

  /**
   * Check if storage supports transactions
   */
  abstract supportsTransactions(): boolean;

  /**
   * Begin a transaction
   */
  abstract beginTransaction(options?: TransactionOptions): Promise<any>;

  /**
   * Commit a transaction
   */
  abstract commitTransaction(transaction: any): Promise<void>;

  /**
   * Rollback a transaction
   */
  abstract rollbackTransaction(transaction: any): Promise<void>;

  /**
   * Get adapter name
   */
  abstract getName(): string;

  /**
   * Get adapter capabilities
   */
  getCapabilities(): {
    transactions: boolean;
    batching: boolean;
    compression: boolean;
    multiTab: boolean;
    persistence: boolean;
  } {
    return {
      transactions: this.supportsTransactions(),
      batching: true,
      compression: false,
      multiTab: true,
      persistence: true,
    };
  }

  /**
   * Test multi-tab safety
   */
  async testMultiTabSafety(tabId: string, writeCount: number): Promise<{
    success: boolean;
    conflicts: number;
    errors: string[];
  }> {
    const errors: string[] = [];
    let conflicts = 0;

    try {
      for (let i = 0; i < writeCount; i++) {
        const key = `multi-tab-test-${i}`;
        const value = { tabId, timestamp: Date.now(), index: i };
        await this.write(key, value);
      }
      return { success: true, conflicts, errors };
    } catch (error) {
      errors.push(String(error));
      return { success: false, conflicts, errors };
    }
  }

  /**
   * Verify data integrity
   */
  async verifyIntegrity(expectedRecords: Array<{ key: string; value: any }>): Promise<{
    valid: boolean;
    missing: string[];
    corrupted: string[];
  }> {
    const missing: string[] = [];
    const corrupted: string[] = [];

    for (const expected of expectedRecords) {
      try {
        const { value } = await this.read(expected.key);
        if (!value) {
          missing.push(expected.key);
        } else if (JSON.stringify(value) !== JSON.stringify(expected.value)) {
          corrupted.push(expected.key);
        }
      } catch (error) {
        missing.push(expected.key);
      }
    }

    return {
      valid: missing.length === 0 && corrupted.length === 0,
      missing,
      corrupted,
    };
  }
}

/**
 * Storage Adapter Factory
 */
export type StorageAdapterType = 'opfs-sqlite' | 'cr-sqlite' | 'indexeddb';

export interface StorageAdapterConfig {
  type: StorageAdapterType;
  dbName?: string;
  version?: number;
  options?: Record<string, any>;
}
