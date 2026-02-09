/**
 * Storage Adapter Factory
 *
 * Provides a unified interface for creating storage adapters.
 */

export * from './base.js';
export * from './indexeddb.js';
export * from './opfs-sqlite.js';
export * from './cr-sqlite.js';

import { StorageAdapter, StorageAdapterType, StorageAdapterConfig } from './base.js';
import { IndexedDBAdapter } from './indexeddb.js';
import { OPFSSQLiteAdapter } from './opfs-sqlite.js';
import { CRSQLiteAdapter } from './cr-sqlite.js';

/**
 * Create a storage adapter based on type
 */
export async function createStorageAdapter(config: StorageAdapterConfig): Promise<StorageAdapter> {
  let adapter: StorageAdapter;

  switch (config.type) {
    case 'indexeddb':
      adapter = new IndexedDBAdapter(config.dbName, config.version);
      break;

    case 'opfs-sqlite':
      adapter = new OPFSSQLiteAdapter(config.dbName, config.version);
      break;

    case 'cr-sqlite':
      adapter = new CRSQLiteAdapter(config.dbName, config.version);
      break;

    default:
      throw new Error(`Unknown storage adapter type: ${config.type}`);
  }

  await adapter.initialize();
  return adapter;
}

/**
 * Get all available adapter types
 */
export function getAvailableAdapters(): StorageAdapterType[] {
  return ['indexeddb', 'opfs-sqlite', 'cr-sqlite'];
}

/**
 * Check browser support for each adapter type
 */
export async function checkBrowserSupport(): Promise<Record<StorageAdapterType, boolean>> {
  const support: Record<StorageAdapterType, boolean> = {
    'indexeddb': false,
    'opfs-sqlite': false,
    'cr-sqlite': false,
  };

  // Check IndexedDB support
  support.indexeddb = 'indexedDB' in globalThis;

  // Check OPFS support
  if ('storage' in navigator && 'getDirectory' in navigator.storage) {
    try {
      await navigator.storage.getDirectory();
      support['opfs-sqlite'] = true;
    } catch (error) {
      support['opfs-sqlite'] = false;
    }
  }

  // cr-sqlite support depends on OPFS + WASM
  support['cr-sqlite'] = support['opfs-sqlite'] && typeof WebAssembly !== 'undefined';

  return support;
}

/**
 * Get recommended adapter for current browser
 */
export async function getRecommendedAdapter(): Promise<StorageAdapterType> {
  const support = await checkBrowserSupport();

  // Preference order: cr-sqlite > opfs-sqlite > indexeddb
  if (support['cr-sqlite']) {
    return 'cr-sqlite';
  } else if (support['opfs-sqlite']) {
    return 'opfs-sqlite';
  } else if (support.indexeddb) {
    return 'indexeddb';
  }

  throw new Error('No supported storage adapter available in this browser');
}
