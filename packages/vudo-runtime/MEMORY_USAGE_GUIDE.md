# WASM Memory Interface Usage Guide

Quick reference for using the Phase 1 memory interfaces in host functions.

## Basic Setup

```typescript
import {
  WasmMemory,
  HostBumpAllocator,
  HostStackAllocator,
  MEMORY_LAYOUT,
} from '@vudo/runtime/host';

// Create the memory interface
const wasmMem = new WasmMemory(memory);

// Create heap allocator for persistent data
const heapAlloc = new HostBumpAllocator(wasmMem);

// Create stack allocator for temporary data
const stackAlloc = new HostStackAllocator(wasmMem);
```

## String Operations

```typescript
// Write a string
const str = "Hello, World!";
const ptr = heapAlloc.alloc(64);  // Allocate space
wasmMem.writeString(ptr, str);

// Read a string
const readStr = wasmMem.readString(ptr, str.length);
console.log(readStr);  // "Hello, World!"

// UTF-8 strings work too
const utf8 = "Hello, 世界! 🌍";
const encoded = new TextEncoder().encode(utf8);
const ptr2 = heapAlloc.alloc(encoded.length);
wasmMem.writeString(ptr2, utf8);
const decoded = wasmMem.readString(ptr2, encoded.length);
```

## Numeric Operations

```typescript
// Write numbers
const basePtr = heapAlloc.alloc(32);
wasmMem.writeI32(basePtr, 42);
wasmMem.writeI64(basePtr + 4, 9007199254740992n);
wasmMem.writeF32(basePtr + 12, 3.14);
wasmMem.writeF64(basePtr + 16, 3.141592653589793);

// Read them back
const i32 = wasmMem.readI32(basePtr);         // 42
const i64 = wasmMem.readI64(basePtr + 4);    // 9007199254740992n
const f32 = wasmMem.readF32(basePtr + 12);   // 3.140000104904175
const f64 = wasmMem.readF64(basePtr + 16);   // 3.141592653589793
```

## Byte Operations

```typescript
// Write raw bytes
const data = new Uint8Array([1, 2, 3, 4, 5]);
const ptr = heapAlloc.alloc(10);
wasmMem.writeBytes(ptr, data);

// Read raw bytes
const read = wasmMem.readBytes(ptr, 5);
console.log(read);  // Uint8Array(5) [1, 2, 3, 4, 5]
```

## Heap Allocator

```typescript
// Basic allocation
const ptr1 = heapAlloc.alloc(256);           // Default 8-byte alignment
const ptr2 = heapAlloc.alloc(128, 16);       // 16-byte alignment
const ptr3 = heapAlloc.alloc(64, 4);         // 4-byte alignment

// String allocation (calculates size for UTF-8)
const strPtr = heapAlloc.allocString("test");

// Check statistics
const stats = heapAlloc.getStats();
console.log(`Allocated: ${stats.totalAllocated} bytes`);
console.log(`Active allocations: ${stats.activeAllocations}`);
console.log(`Peak usage: ${stats.peakUsage} bytes`);
console.log(`Memory grown: ${stats.growthCount} times`);

// Get allocation info
const info = heapAlloc.getAllocationInfo(ptr1);
console.log(`Size: ${info?.size}`);
console.log(`Tagged as: ${info?.tag}`);

// List all allocations
const allocs = heapAlloc.getAllocations();
allocs.forEach(a => console.log(`${a.ptr}: ${a.size} bytes`));

// Reset all allocations
heapAlloc.reset();
```

## Stack Allocator

```typescript
// Allocate from stack
const tempPtr = stackAlloc.alloc(256);

// Mark position and allocate
stackAlloc.mark();
const nested1 = stackAlloc.alloc(100);
const nested2 = stackAlloc.alloc(100);

// Pop back to mark
stackAlloc.pop();

// Nested marks
stackAlloc.mark();  // Mark 1
stackAlloc.alloc(100);
stackAlloc.mark();  // Mark 2
stackAlloc.alloc(100);
stackAlloc.pop();   // Back to Mark 2
stackAlloc.pop();   // Back to Mark 1

// Check usage
console.log(`Used: ${stackAlloc.used} bytes`);
console.log(`Capacity: ${stackAlloc.capacity} bytes`);
console.log(`Top: ${stackAlloc.top}`);

// Reset completely
stackAlloc.reset();
```

## Memory Growth

```typescript
// Memory grows automatically when needed
const ptr = heapAlloc.alloc(1000000);  // May trigger growth

// Manual growth if needed
wasmMem.grow(2);  // Grow by 2 pages (128KB)

// Query memory size
const bytes = wasmMem.size();   // Total bytes
const pages = wasmMem.pages();  // Total 64KB pages
```

## Bounds Checking

```typescript
import { MemoryError } from '@vudo/runtime/host';

try {
  // This will throw MemoryError if out of bounds
  wasmMem.readString(1000000, 100);
} catch (error) {
  if (error instanceof MemoryError) {
    console.log(`Error: ${error.message}`);
    console.log(`Pointer: ${error.ptr}`);
    console.log(`Length: ${error.length}`);
  }
}

// Check before accessing (optional)
if (wasmMem.isValidRange(ptr, len)) {
  const str = wasmMem.readString(ptr, len);
}
```

## Direct View Access

```typescript
// Create a DataView for direct access
const view = wasmMem.createDataView(ptr, 16);
view.setInt32(0, 42, true);  // little-endian
const value = view.getInt32(0, true);

// Create a Uint8Array view
const bytes = wasmMem.createByteView(ptr, 16);
bytes[0] = 255;
bytes[1] = 0;
```

## Complete Example: Host Function

```typescript
// Host function that reads a struct from WASM memory
function readUserStruct(memory: WebAssembly.Memory, ptr: number): User {
  const wasmMem = new WasmMemory(memory);

  // Read name (4-byte pointer to string)
  const namePtr = wasmMem.readI32(ptr);
  const nameLen = wasmMem.readI32(ptr + 4);
  const name = wasmMem.readString(namePtr, nameLen);

  // Read age
  const age = wasmMem.readI32(ptr + 8);

  // Read score
  const score = wasmMem.readF64(ptr + 12);

  return { name, age, score };
}

// Host function that writes data to WASM memory
function writeResultStruct(
  memory: WebAssembly.Memory,
  ptr: number,
  result: Result
): void {
  const wasmMem = new WasmMemory(memory);
  const heapAlloc = new HostBumpAllocator(wasmMem);

  // Allocate space for result string
  const messagePtr = heapAlloc.alloc(256);
  wasmMem.writeString(messagePtr, result.message);

  // Write result struct
  wasmMem.writeI32(ptr, messagePtr);              // message pointer
  wasmMem.writeI32(ptr + 4, result.message.length);  // message length
  wasmMem.writeI32(ptr + 8, result.code);        // result code
  wasmMem.writeF64(ptr + 12, result.value);      // result value
}

// Host function with temporary stack allocation
function processData(memory: WebAssembly.Memory, ptr: number): void {
  const wasmMem = new WasmMemory(memory);
  const stackAlloc = new HostStackAllocator(wasmMem);

  // Use stack for temporary buffer
  stackAlloc.mark();
  const tempBuffer = stackAlloc.alloc(1024);

  // Do work with temp buffer
  const data = wasmMem.readBytes(ptr, 512);
  wasmMem.writeBytes(tempBuffer, data);

  // Clean up
  stackAlloc.pop();
}
```

## Memory Layout Constants

```typescript
import { MEMORY_LAYOUT } from '@vudo/runtime/host';

console.log(MEMORY_LAYOUT.HEAP_BASE);   // 0x10000 (65536)
console.log(MEMORY_LAYOUT.STACK_SIZE);  // 0x8000 (32768)
console.log(MEMORY_LAYOUT.PAGE_SIZE);   // 0x10000 (65536)
```

## Common Patterns

### Temporary Buffer
```typescript
stackAlloc.mark();
try {
  const tempPtr = stackAlloc.alloc(1024);
  // Use tempPtr
} finally {
  stackAlloc.pop();
}
```

### String Interop
```typescript
// Write string to WASM
const str = "Hello";
const ptr = heapAlloc.allocString(str);
wasmMem.writeString(ptr, str);

// Read string from WASM
const len = wasmMem.readI32(ptr);  // Assuming length stored first
const read = wasmMem.readString(ptr + 4, len);
```

### Struct Mapping
```typescript
// Define memory layout
const STRUCT_SIZE = 24;
const FIELD_A_OFFSET = 0;
const FIELD_B_OFFSET = 8;
const FIELD_C_OFFSET = 16;

// Write struct
wasmMem.writeI64(ptr + FIELD_A_OFFSET, 42n);
wasmMem.writeF32(ptr + FIELD_B_OFFSET, 3.14);
wasmMem.writeI32(ptr + FIELD_C_OFFSET, 100);

// Read struct
const a = wasmMem.readI64(ptr + FIELD_A_OFFSET);
const b = wasmMem.readF32(ptr + FIELD_B_OFFSET);
const c = wasmMem.readI32(ptr + FIELD_C_OFFSET);
```

## Debugging

```typescript
// Check bounds before accessing
if (!wasmMem.isValidRange(ptr, len)) {
  console.error(`Invalid range: ${ptr}+${len}`);
  return;
}

// Track allocations
const allocations = heapAlloc.getAllocations();
console.table(allocations);

// Monitor statistics
setInterval(() => {
  const stats = heapAlloc.getStats();
  console.log(`Memory: ${stats.currentOffset}/${wasmMem.size()} bytes`);
}, 1000);
```

## Performance Tips

1. **Reuse Allocators**: Create once, reuse across calls
2. **Use Stack for Temporaries**: Much faster than heap
3. **Batch Operations**: Reduce crossing WASM boundary
4. **Prefer Direct Views**: For repeated access to same region
5. **Alignment**: Use correct alignment (8 for i64/f64, 4 for i32/f32)

## Error Handling

```typescript
function safeReadString(
  memory: WebAssembly.Memory,
  ptr: number,
  len: number
): string | null {
  try {
    const wasmMem = new WasmMemory(memory);
    return wasmMem.readString(ptr, len);
  } catch (error) {
    if (error instanceof MemoryError) {
      console.error(`Memory error: ${error.message}`);
      return null;
    }
    throw error;
  }
}
```
