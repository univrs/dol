/**
 * Type bridge utilities for DOL ↔ JavaScript conversion
 *
 * Handles encoding/decoding of DOL types in WASM linear memory.
 * Based on vudo-runtime-spec.md string and gene helpers.
 */

import type { GeneLayout, GeneField } from '../types.js';
import type { BumpAllocator } from '../memory.js';

// ============================================================================
// String Encoding/Decoding
// ============================================================================

/**
 * Encode a JavaScript string into WASM memory
 *
 * Format: 4-byte length prefix (u32, little-endian) + UTF-8 bytes
 *
 * @param memory - WASM memory instance
 * @param allocator - Allocator for memory allocation
 * @param str - String to encode
 * @returns Pointer to encoded string
 */
export function encodeString(
  memory: WebAssembly.Memory,
  allocator: { alloc: (size: number, align?: number) => number },
  str: string
): number {
  const encoder = new TextEncoder();
  const bytes = encoder.encode(str);
  const len = bytes.length;

  // Allocate: 4 bytes for length + string bytes
  const ptr = allocator.alloc(4 + len, 4);
  const view = new DataView(memory.buffer);

  // Write length prefix (u32, little-endian)
  view.setUint32(ptr, len, true);

  // Write string bytes
  new Uint8Array(memory.buffer, ptr + 4, len).set(bytes);

  return ptr;
}

/**
 * Decode a string from WASM memory
 *
 * Expects format: 4-byte length prefix (u32, little-endian) + UTF-8 bytes
 *
 * @param memory - WASM memory instance
 * @param ptr - Pointer to encoded string
 * @returns Decoded JavaScript string
 */
export function decodeString(memory: WebAssembly.Memory, ptr: number): string {
  const view = new DataView(memory.buffer);
  const len = view.getUint32(ptr, true);
  const bytes = new Uint8Array(memory.buffer, ptr + 4, len);
  return new TextDecoder('utf-8').decode(bytes);
}

// ============================================================================
// Gene Read/Write Operations
// ============================================================================

/**
 * Write a gene struct to WASM memory
 *
 * @param memory - WASM memory instance
 * @param ptr - Pointer to write location
 * @param layout - Gene layout definition
 * @param values - Field values to write
 */
export function writeGene(
  memory: WebAssembly.Memory,
  ptr: number,
  layout: GeneLayout,
  values: Record<string, unknown>
): void {
  const view = new DataView(memory.buffer);

  for (const field of layout.fields) {
    const value = values[field.name];
    const fieldPtr = ptr + field.offset;

    writeField(view, fieldPtr, field, value);
  }
}

/**
 * Read a gene struct from WASM memory
 *
 * @param memory - WASM memory instance
 * @param ptr - Pointer to gene location
 * @param layout - Gene layout definition
 * @returns Object with field values
 */
export function readGene<T extends GeneLayout>(
  memory: WebAssembly.Memory,
  ptr: number,
  layout: T
): Record<string, unknown> {
  const view = new DataView(memory.buffer);
  const result: Record<string, unknown> = {};

  for (const field of layout.fields) {
    const fieldPtr = ptr + field.offset;
    result[field.name] = readField(view, memory, fieldPtr, field);
  }

  return result;
}

/**
 * Write a single field value
 * @internal
 */
function writeField(
  view: DataView,
  ptr: number,
  field: GeneField,
  value: unknown
): void {
  switch (field.type) {
    case 'i32':
      view.setInt32(ptr, Number(value), true);
      break;
    case 'bool':
      view.setInt32(ptr, value ? 1 : 0, true);
      break;
    case 'i64':
      view.setBigInt64(ptr, BigInt(value as number | bigint), true);
      break;
    case 'f32':
      view.setFloat32(ptr, Number(value), true);
      break;
    case 'f64':
      view.setFloat64(ptr, Number(value), true);
      break;
    case 'string':
      // String is stored as pointer - value should be pre-allocated ptr
      view.setInt32(ptr, Number(value), true);
      break;
  }
}

/**
 * Read a single field value
 * @internal
 */
function readField(
  view: DataView,
  memory: WebAssembly.Memory,
  ptr: number,
  field: GeneField
): unknown {
  switch (field.type) {
    case 'i32':
      return view.getInt32(ptr, true);
    case 'bool':
      return view.getInt32(ptr, true) !== 0;
    case 'i64':
      return view.getBigInt64(ptr, true);
    case 'f32':
      return view.getFloat32(ptr, true);
    case 'f64':
      return view.getFloat64(ptr, true);
    case 'string': {
      const strPtr = view.getInt32(ptr, true);
      return strPtr !== 0 ? decodeString(memory, strPtr) : '';
    }
  }
}

// ============================================================================
// Type Size Helpers
// ============================================================================

/**
 * Get byte size for a field type
 */
export function getTypeSize(type: GeneField['type']): number {
  switch (type) {
    case 'i32':
    case 'bool':
    case 'f32':
    case 'string': // pointer
      return 4;
    case 'i64':
    case 'f64':
      return 8;
  }
}

/**
 * Get alignment for a field type
 */
export function getTypeAlignment(type: GeneField['type']): number {
  switch (type) {
    case 'i32':
    case 'bool':
    case 'f32':
    case 'string':
      return 4;
    case 'i64':
    case 'f64':
      return 8;
  }
}

/**
 * Calculate gene layout from field definitions
 */
export function calculateLayout(name: string, fields: Omit<GeneField, 'offset'>[]): GeneLayout {
  let offset = 0;
  let maxAlignment = 1;
  const layoutFields: GeneField[] = [];

  for (const field of fields) {
    const alignment = getTypeAlignment(field.type);
    const size = getTypeSize(field.type);

    // Align offset
    offset = Math.ceil(offset / alignment) * alignment;

    layoutFields.push({
      ...field,
      offset,
    });

    offset += size;
    maxAlignment = Math.max(maxAlignment, alignment);
  }

  // Align total size to max alignment
  const totalSize = Math.ceil(offset / maxAlignment) * maxAlignment;

  return {
    name,
    fields: layoutFields,
    size: totalSize,
    alignment: maxAlignment,
  };
}
