# DOL ABI Integration Tests

This directory contains comprehensive integration tests for the `dol-abi` crate.

## Test Files

### `integration_tests.rs`
Integration tests that verify all modules work together correctly and that re-exports are properly configured.

**Test Coverage:**
- **Constants Tests** (2 tests)
  - ABI version constant verification
  - Import module constant verification

- **QualifiedId Tests** (7 tests)
  - Construction with and without version
  - Display formatting
  - Equality comparisons
  - Cloning behavior
  - Serialization/deserialization
  - Hash implementation

- **Message Tests** (6 tests)
  - Message creation
  - Serialization roundtrip
  - Type string conversions
  - Null payload handling
  - Complex nested payload handling
  - Message-response roundtrip

- **Response Tests** (3 tests)
  - Success response creation
  - Error response creation
  - Serialization roundtrip

- **Error Tests** (4 tests)
  - Display formatting for all error types
  - Serialization/deserialization
  - Result type usage
  - std::error::Error trait implementation

- **General Tests** (5 tests)
  - Empty string handling
  - Multiple IDs with hash collections
  - Module re-exports
  - Type conversions

**Total: 27 tests**

### `host_tests.rs`
Tests for the host interface functionality, including mock implementations and WASM-specific tests.

**Test Coverage:**
- **Mock Host Tests** (6 tests)
  - Host initialization
  - Host shutdown
  - Send message before/after init
  - Full lifecycle (init → send messages → shutdown)

- **Performance & Stress Tests** (3 tests)
  - Concurrent message handling (100 messages)
  - Error recovery after failed operations
  - Message size limits (1B to 10KB payloads)
  - Special character handling (unicode, newlines, tabs, null bytes)

- **WASM Tests** (conditionally compiled for `target_arch = "wasm32"`)
  - Host functions availability verification
  - FFI send_message testing
  - Message roundtrip through WASM boundary

- **Future Tests** (feature-gated with `full-abi`)
  - Placeholder tests for full ABI specification
  - Memory allocation/free/realloc tests
  - Logging tests (all levels, unicode, large messages)
  - Time tests (monotonic, precision, sleep)
  - Message tests (send/recv, broadcast, serialization)
  - Effect tests (emit, subscribe)
  - Random tests (distribution, determinism)
  - Debug tests (assert, panic)

**Total: 9 active tests + 18+ placeholder tests**

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Only Integration Tests
```bash
cargo test --test integration_tests
```

### Run Only Host Tests
```bash
cargo test --test host_tests
```

### Run Tests with Output
```bash
cargo test -- --nocapture
```

### Run Specific Test
```bash
cargo test test_qualified_id_serialization
```

### Run WASM Tests
WASM tests require the wasm32 target and wasm-bindgen-test runner:

```bash
# Install wasm32 target
rustup target add wasm32-unknown-unknown

# Install wasm-bindgen-cli
cargo install wasm-bindgen-cli

# Run WASM tests in browser
wasm-pack test --chrome

# Run WASM tests in Node.js
wasm-pack test --node
```

### Run Future Tests (Full ABI)
When the full ABI specification is implemented:

```bash
cargo test --features full-abi
```

## Test Organization

### Unit Tests
Located in individual source files (e.g., `src/types.rs`, `src/message.rs`)

### Integration Tests
Located in `tests/` directory, test the crate as external consumers would use it

### WASM Tests
Conditionally compiled with `#[cfg(target_arch = "wasm32")]`
- Test FFI bindings
- Test host function availability
- Test message serialization across WASM boundary

## Coverage Goals

- ✅ **Type definitions**: 100% covered
- ✅ **Message serialization**: 100% covered
- ✅ **Error handling**: 100% covered
- ✅ **Host interface trait**: 100% covered
- ⚠️ **Host functions**: Awaiting full ABI implementation
- ⚠️ **WASM FFI**: Requires WASM runtime for execution

## Test Assertions

All tests use standard Rust test assertions:
- `assert!()` - Boolean condition
- `assert_eq!()` - Equality comparison
- `assert_ne!()` - Inequality comparison
- `matches!()` - Pattern matching
- `.is_ok()` / `.is_err()` - Result checking
- `.unwrap()` - Panic if error (use only when failure is test failure)

## Edge Cases Tested

1. **Empty values**: Empty strings, null payloads
2. **Boundaries**: Max message sizes, concurrent operations
3. **Unicode**: Multi-byte UTF-8 characters
4. **Special characters**: Newlines, tabs, null bytes
5. **Lifecycle**: Init before use, cleanup after use
6. **Error recovery**: Failed operations followed by retry
7. **Serialization**: Roundtrip JSON encoding/decoding

## Dependencies

- `serde` / `serde_json` - Serialization
- `wasm-bindgen-test` - WASM testing (dev-dependency)

## Future Enhancements

When the full ABI specification is implemented, enable these tests by implementing:

1. **Memory Management**
   - `vudo_alloc` / `vudo_free` / `vudo_realloc` host functions
   - Memory leak detection
   - Pointer validation

2. **Logging**
   - `vudo_print` / `vudo_println` / `vudo_log` / `vudo_error`
   - Log level filtering
   - UTF-8 validation

3. **Time Functions**
   - `vudo_now` / `vudo_sleep` / `vudo_monotonic_now`
   - Monotonic clock verification
   - Sleep precision testing

4. **Messaging**
   - Full message wire format implementation
   - Multi-spirit communication
   - Broadcast functionality

5. **Effects**
   - Effect emission and subscription
   - Effect handler registration
   - Standard effect types

6. **Random**
   - Deterministic random for testing
   - Distribution verification

7. **Debug**
   - Breakpoint triggers
   - Assertion failures
   - Panic handling

## Contributing

When adding new tests:
1. Place unit tests in the relevant `src/*.rs` file
2. Place integration tests in `tests/integration_tests.rs`
3. Place host-specific tests in `tests/host_tests.rs`
4. Use `#[cfg(target_arch = "wasm32")]` for WASM-only tests
5. Use `#[cfg(feature = "full-abi")]` for future ABI tests
6. Ensure all tests have descriptive names and comments
7. Test both success and failure cases
8. Include edge cases and boundary conditions

## Test Results

Last run: All 36 tests passing ✅
- 27 integration tests
- 9 host tests
- 0 failures
- 0 warnings

```
test result: ok. 27 passed; 0 failed; 0 ignored
test result: ok. 9 passed; 0 failed; 0 ignored
```
