# Host Functions Implementation - Phases 2-4

This document describes the implementation of I/O, Memory, and Time host functions for the vudo-runtime package.

## Overview

Implemented 10 host functions across 3 categories:
- **I/O Functions (4)**: print, println, log, error
- **Memory Functions (3)**: alloc, free, realloc
- **Time Functions (3)**: now, sleep, monotonic_now

## Architecture

### Core Interfaces

All implementations use dependency injection through interfaces for testability:

```typescript
interface IWasmMemory {
  buffer: ArrayBuffer;
  u8: Uint8Array;
  i32: Int32Array;
  f64: Float64Array;
  decodeString(ptr: number, len: number): string;
  encodeString(str: string): number;
  alloc(size: number): number;
  free(ptr: number, size: number): void;
  realloc(ptr: number, oldSize: number, newSize: number): number;
}

interface ILogger {
  log(level: LogLevel, message: string): void;
  debug(message: string): void;
  info(message: string): void;
  warn(message: string): void;
  error(message: string): void;
  print(message: string): void;
  println(message: string): void;
}

interface ITimeProvider {
  now(): bigint;
  monotonicNow(): bigint;
  sleep(ms: number): Promise<void>;
}
```

### File Structure

```
packages/vudo-runtime/src/host/
├── interfaces.ts       # Core interfaces and default implementations
├── io.ts              # I/O host functions
├── memory-ops.ts      # Memory management host functions
├── time.ts            # Time host functions
└── index.ts           # Module exports

tests/host/
├── test-utils.ts      # Mock implementations for testing
├── io.test.ts         # I/O function tests (29 tests)
├── memory-ops.test.ts # Memory function tests (36 tests)
└── time.test.ts       # Time function tests (37 tests)
```

## Implementation Details

### 1. I/O Functions (`src/host/io.ts`)

#### `vudo_print(ptr: i32, len: i32) -> void`
- Prints UTF-8 string without newline
- Validates pointer and length
- Handles invalid UTF-8 gracefully
- Configurable max string length (default 1MB)

#### `vudo_println(ptr: i32, len: i32) -> void`
- Prints UTF-8 string with newline
- Same validation as print

#### `vudo_log(level: i32, ptr: i32, len: i32) -> void`
- Structured logging with severity levels
- Supports DEBUG (0), INFO (1), WARN (2), ERROR (3)
- Invalid levels default to ERROR

#### `vudo_error(ptr: i32, len: i32) -> void`
- Convenience function for error logging
- Equivalent to `vudo_log(ERROR, ptr, len)`

**Error Handling:**
- Invalid pointers/lengths are caught and logged
- Out of bounds access is prevented
- UTF-8 decode failures are handled gracefully
- All errors are logged but don't crash the runtime

### 2. Memory Functions (`src/host/memory-ops.ts`)

#### `vudo_alloc(size: i32) -> i32`
- Allocates memory from host allocator
- Returns pointer on success, 0 on failure
- Validates size (must be > 0)
- Configurable max allocation (default 10MB)
- Tracks allocation statistics

#### `vudo_free(ptr: i32, size: i32) -> void`
- Frees previously allocated memory
- Accepts null pointer (0) as no-op
- Validates pointer and size
- Updates statistics

#### `vudo_realloc(ptr: i32, old_size: i32, new_size: i32) -> i32`
- Resizes existing allocation
- Returns new pointer on success, 0 on failure
- Preserves data up to min(old_size, new_size)
- Original pointer remains valid on failure

**Features:**
- Allocation statistics tracking (count, total bytes, peak usage)
- Debug logging mode
- Resilient error handling (returns 0 on failure)
- Memory bounds validation

### 3. Time Functions (`src/host/time.ts`)

#### `vudo_now() -> i64`
- Returns current Unix timestamp in milliseconds
- Uses ITimeProvider for testability
- Fallback to Date.now() on error

#### `vudo_sleep(ms: i32) -> void`
- Sleeps for specified duration
- Validates duration (max 1 hour by default)
- Negative durations treated as 0
- Tracks sleep statistics

#### `vudo_monotonic_now() -> i64`
- Returns monotonic time in nanoseconds
- Never goes backward
- Uses performance.now() or process.hrtime.bigint()
- Suitable for high-precision measurements

**Features:**
- Sleep statistics (count, total time, average)
- Configurable max sleep duration
- Debug logging mode
- Fallback implementations for compatibility

## Testing

### Test Coverage

- **102 total tests** across all modules
- **Comprehensive edge case coverage**
- **Mock implementations** for all interfaces
- **Error handling verification**

### Mock Implementations

#### MockWasmMemory
- Simulates WASM linear memory
- Bump allocator with tracking
- UTF-8 encoding/decoding
- Allocation statistics

#### MockLogger
- Captures all log messages
- Pattern matching support
- Message retrieval by level
- Test helper methods

#### MockTimeProvider
- Controllable time advancement
- Sleep tracking
- Configurable timestamps
- Test utilities

### Test Categories

1. **Happy Path Tests**
   - Valid operations
   - Expected outputs
   - Common use cases

2. **Error Handling Tests**
   - Invalid parameters
   - Out of bounds access
   - Resource exhaustion
   - UTF-8 decode failures

3. **Edge Cases**
   - Empty strings
   - Zero allocations
   - Maximum values
   - Boundary conditions

4. **Configuration Tests**
   - Custom limits
   - Debug mode
   - Provider injection

## Usage Examples

### Creating Host Functions

```typescript
import {
  createIOHostFunctions,
  createMemoryHostFunctions,
  createTimeHostFunctions,
  ConsoleLogger,
  SystemTimeProvider,
} from '@vudo/runtime/host';

// Create with defaults
const memory = new SpiritMemoryManager(wasmMemory);
const logger = new ConsoleLogger();
const timeProvider = new SystemTimeProvider();

const io = createIOHostFunctions(memory, logger);
const memOps = createMemoryHostFunctions(memory);
const time = createTimeHostFunctions(timeProvider);

// Build imports object
const imports = {
  vudo: {
    ...io.buildImports(),
    ...memOps.buildImports(),
    ...time.buildImports(),
  },
};

// Instantiate WASM
const instance = await WebAssembly.instantiate(wasmBytes, imports);
```

### Custom Configuration

```typescript
// Custom I/O with smaller max string length
const io = new IOHostFunctions({
  memory,
  logger,
  maxStringLength: 512 * 1024, // 512KB
});

// Memory ops with debug mode
const memOps = new MemoryHostFunctions({
  memory,
  debug: true,
  maxAllocationSize: 5 * 1024 * 1024, // 5MB
});

// Time ops with shorter max sleep
const time = new TimeHostFunctions({
  timeProvider,
  maxSleepDuration: 30 * 60 * 1000, // 30 minutes
  debug: true,
});
```

### Getting Statistics

```typescript
// Memory allocation statistics
const memStats = memOps.getStats();
console.log(`Allocations: ${memStats.allocationCount}`);
console.log(`Total allocated: ${memStats.totalAllocated} bytes`);
console.log(`Peak usage: ${memStats.peakMemoryUsage} bytes`);

// Sleep statistics
const timeStats = time.getStats();
console.log(`Sleep calls: ${timeStats.sleepCount}`);
console.log(`Total sleep time: ${timeStats.totalSleepTime}ms`);
console.log(`Average sleep: ${timeStats.averageSleepTime}ms`);
```

## Design Decisions

### 1. Interface-Based Design
- Enables dependency injection
- Facilitates testing with mocks
- Allows custom implementations
- Decouples from platform specifics

### 2. Error Resilience
- I/O functions never throw
- Memory functions return 0 on failure
- Time functions use fallbacks
- All errors are logged

### 3. Validation First
- All parameters validated
- Bounds checking before access
- Size limits enforced
- Clear error messages

### 4. Statistics Tracking
- Optional debug mode
- Performance monitoring
- Resource usage tracking
- Helpful for optimization

### 5. Testability
- 100% mock implementations
- Comprehensive test coverage
- Edge case verification
- Error path testing

## Performance Considerations

### I/O Functions
- UTF-8 decoding is optimized by TextDecoder
- String length validation prevents large allocations
- Error handling overhead is minimal

### Memory Functions
- Delegates to underlying allocator
- Minimal validation overhead
- Statistics tracking is O(1)
- Debug logging is conditional

### Time Functions
- now() is a simple wrapper over Date.now()
- monotonic_now() uses platform-optimized implementations
- sleep() is synchronous in host (async in runtime)

## Future Enhancements

1. **Streaming I/O**
   - Support for large string streams
   - Chunked output

2. **Advanced Memory**
   - Different allocator strategies
   - Memory pools
   - Alignment control

3. **Timer Management**
   - Timeout cancellation
   - Timer scheduling
   - High-resolution timers

4. **Metrics**
   - Performance profiling
   - Histogram tracking
   - Resource alerts

## Test Results

```
✓ tests/host/io.test.ts         (29 tests)
✓ tests/host/memory-ops.test.ts (36 tests)  
✓ tests/host/time.test.ts       (37 tests)

Test Files  3 passed (3)
Tests      102 passed (102)
```

All tests pass with comprehensive coverage of:
- Happy paths
- Error conditions
- Edge cases
- Configuration options

## Summary

The host functions implementation provides:

✅ **10 fully-tested host functions**
✅ **Interface-based architecture for testability**
✅ **Comprehensive error handling**
✅ **102 passing tests with edge case coverage**
✅ **Statistics tracking for debugging**
✅ **Configurable limits and behavior**
✅ **Mock implementations for testing**
✅ **Clear documentation and examples**

The implementation is production-ready and follows best practices for:
- Type safety
- Error resilience
- Testability
- Performance
- Maintainability
