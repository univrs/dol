/**
 * Test Data Generation
 *
 * Utilities for generating realistic test data for benchmarks.
 */

export interface TestRecord {
  id: string;
  type: string;
  data: any;
  metadata: {
    created: number;
    updated: number;
    version: number;
  };
}

/**
 * Generate a random string of specified length
 */
export function randomString(length: number): string {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  let result = '';
  for (let i = 0; i < length; i++) {
    result += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return result;
}

/**
 * Generate a random integer between min and max
 */
export function randomInt(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

/**
 * Generate a random test record
 */
export function generateTestRecord(id?: string, size: 'small' | 'medium' | 'large' = 'medium'): TestRecord {
  const recordId = id || `record-${randomString(16)}`;
  const timestamp = Date.now();

  let dataSize: number;
  switch (size) {
    case 'small':
      dataSize = 100; // ~100 bytes
      break;
    case 'medium':
      dataSize = 1000; // ~1KB
      break;
    case 'large':
      dataSize = 10000; // ~10KB
      break;
  }

  return {
    id: recordId,
    type: 'test-record',
    data: {
      content: randomString(dataSize),
      number: randomInt(1, 1000000),
      boolean: Math.random() > 0.5,
      array: Array.from({ length: 10 }, () => randomInt(1, 100)),
      nested: {
        field1: randomString(50),
        field2: randomInt(1, 1000),
        field3: {
          deep: randomString(20),
        },
      },
    },
    metadata: {
      created: timestamp,
      updated: timestamp,
      version: 1,
    },
  };
}

/**
 * Generate a batch of test records
 */
export function generateTestRecords(
  count: number,
  size: 'small' | 'medium' | 'large' = 'medium',
  idPrefix?: string
): TestRecord[] {
  const records: TestRecord[] = [];

  for (let i = 0; i < count; i++) {
    const id = idPrefix ? `${idPrefix}-${i}` : undefined;
    records.push(generateTestRecord(id, size));
  }

  return records;
}

/**
 * Generate records with specific size in bytes
 */
export function generateRecordsWithSize(count: number, targetBytesPerRecord: number): TestRecord[] {
  const records: TestRecord[] = [];

  for (let i = 0; i < count; i++) {
    const id = `record-${i}`;
    const timestamp = Date.now();

    // Calculate approximate string length needed
    const baseSize = 200; // Approximate overhead
    const contentLength = Math.max(0, targetBytesPerRecord - baseSize);

    const record: TestRecord = {
      id,
      type: 'sized-record',
      data: {
        content: randomString(contentLength),
      },
      metadata: {
        created: timestamp,
        updated: timestamp,
        version: 1,
      },
    };

    records.push(record);
  }

  return records;
}

/**
 * Generate Automerge-compatible test data
 */
export function generateAutomergeData(sizeInBytes: number): any {
  const targetContentSize = Math.max(0, sizeInBytes - 500); // Account for metadata

  return {
    id: randomString(32),
    type: 'automerge-doc',
    timestamp: Date.now(),
    content: randomString(targetContentSize),
    metadata: {
      created: Date.now(),
      version: 1,
    },
    // Simulate some structured data
    items: Array.from({ length: 10 }, (_, i) => ({
      id: `item-${i}`,
      value: randomString(50),
      count: randomInt(1, 100),
    })),
  };
}

/**
 * Generate realistic VUDO Runtime data
 */
export function generateVUDORuntimeData(type: 'spirit' | 'memory' | 'effect'): TestRecord {
  const id = `${type}-${randomString(16)}`;
  const timestamp = Date.now();

  let data: any;

  switch (type) {
    case 'spirit':
      data = {
        id,
        name: `Spirit_${randomString(8)}`,
        state: {
          health: randomInt(0, 100),
          energy: randomInt(0, 100),
          position: {
            x: randomInt(0, 1000),
            y: randomInt(0, 1000),
            z: randomInt(0, 1000),
          },
        },
        properties: Array.from({ length: 5 }, () => ({
          key: randomString(10),
          value: randomString(20),
        })),
      };
      break;

    case 'memory':
      data = {
        id,
        address: randomInt(0, 0xFFFFFFFF),
        size: randomInt(64, 4096),
        data: randomString(256),
        type: 'linear-memory',
        allocated: true,
      };
      break;

    case 'effect':
      data = {
        id,
        type: randomInt(1, 10),
        payload: randomString(128),
        timestamp,
        handled: false,
        result: null,
      };
      break;
  }

  return {
    id,
    type,
    data,
    metadata: {
      created: timestamp,
      updated: timestamp,
      version: 1,
    },
  };
}

/**
 * Generate a dataset with specific characteristics
 */
export interface DatasetConfig {
  count: number;
  size: 'small' | 'medium' | 'large';
  distribution?: 'uniform' | 'skewed' | 'random';
  keyPattern?: 'sequential' | 'random' | 'uuid';
}

export function generateDataset(config: DatasetConfig): Array<{ key: string; value: TestRecord }> {
  const dataset: Array<{ key: string; value: TestRecord }> = [];

  for (let i = 0; i < config.count; i++) {
    let key: string;

    switch (config.keyPattern || 'sequential') {
      case 'sequential':
        key = `key-${String(i).padStart(10, '0')}`;
        break;
      case 'random':
        key = `key-${randomString(16)}`;
        break;
      case 'uuid':
        key = crypto.randomUUID();
        break;
    }

    const value = generateTestRecord(undefined, config.size);
    dataset.push({ key, value });
  }

  // Apply distribution if skewed
  if (config.distribution === 'skewed') {
    // Sort by key to create hot spots
    dataset.sort((a, b) => a.key.localeCompare(b.key));
  } else if (config.distribution === 'random') {
    // Shuffle dataset
    for (let i = dataset.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [dataset[i], dataset[j]] = [dataset[j], dataset[i]];
    }
  }

  return dataset;
}

/**
 * Calculate actual size of serialized data
 */
export function calculateDataSize(data: any): number {
  const json = JSON.stringify(data);
  return new Blob([json]).size;
}

/**
 * Generate keys for read benchmarks
 */
export function generateReadKeys(
  existingKeys: string[],
  count: number,
  pattern: 'sequential' | 'random' | 'mixed'
): string[] {
  const keys: string[] = [];

  switch (pattern) {
    case 'sequential':
      // Read keys in order
      for (let i = 0; i < count && i < existingKeys.length; i++) {
        keys.push(existingKeys[i]);
      }
      break;

    case 'random':
      // Random access
      for (let i = 0; i < count; i++) {
        const randomIndex = Math.floor(Math.random() * existingKeys.length);
        keys.push(existingKeys[randomIndex]);
      }
      break;

    case 'mixed':
      // 80% sequential, 20% random
      const seqCount = Math.floor(count * 0.8);
      for (let i = 0; i < seqCount && i < existingKeys.length; i++) {
        keys.push(existingKeys[i]);
      }
      for (let i = seqCount; i < count; i++) {
        const randomIndex = Math.floor(Math.random() * existingKeys.length);
        keys.push(existingKeys[randomIndex]);
      }
      break;
  }

  return keys;
}
