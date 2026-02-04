# DOL ABI Integration Examples

This directory contains four comprehensive example DOL programs demonstrating how to use the host ABI (Application Binary Interface) to interact with the runtime system.

## Examples

### 1. `hello-world.dol` - Basic I/O Operations

**Size:** 107 lines | **Focus:** Fundamental I/O and string output

Demonstrates:
- `vudo_println()` - Output text to host standard output
- String concatenation and formatting
- Multiple I/O calls from a single spirit function
- Return code handling basics
- State management with I/O operations

**Key Concepts:**
- Host function declarations with `sex fun` keyword
- Automatic pointer/length conversion for strings
- Building output messages dynamically
- Maintaining spirit state while performing I/O

**Use Cases:**
- Logging and debugging output
- Status reporting
- User-facing messages

---

### 2. `messaging.dol` - Spirit-to-Spirit Communication

**Size:** 216 lines | **Focus:** Inter-spirit messaging patterns

Demonstrates:
- `vudo_send()` - Send messages to recipient spirits
- `vudo_recv()` - Receive incoming messages
- `vudo_sender()` - Identify the message sender
- `vudo_pending()` - Check for queued messages
- Result code interpretation (success, not found, queue full)
- Message relay and brokering patterns

**Key Concepts:**
- Queue-based asynchronous messaging
- Sender identification for routing
- Error handling specific to messaging failures
- Message broker/relay architecture
- State tracking for message statistics

**Use Cases:**
- Request/response patterns
- Publish/subscribe systems
- Message routing and relay
- Multi-agent communication

---

### 3. `effects.dol` - Effect System Usage

**Size:** 266 lines | **Focus:** Event-driven architecture via effects

Demonstrates:
- `vudo_emit_effect()` - Emit named effects into the system
- `vudo_subscribe()` - Subscribe to effects matching patterns
- `vudo_unsubscribe()` - Cancel subscriptions
- `vudo_get_effect()` - Receive subscribed effects
- Wildcard pattern matching (e.g., "state.*")
- Effect payload structure and metadata

**Key Concepts:**
- Effect-driven reactive architecture
- Decoupled spirit coordination via events
- Subscription pattern management
- StandardEffect structure with metadata
- Multiple emitters/subscribers in one system

**Use Cases:**
- State change notifications
- Event-driven workflows
- System monitoring and observability
- Reactive spirit systems

---

### 4. `error-handling.dol` - Robust Error Handling

**Size:** 374 lines | **Focus:** Production-quality error management

Demonstrates:
- Result code checking from all ABI functions
- `vudo_error()` - Report errors to the host
- ErrorCodes constant definitions
- Fallback strategies (shorter messages, retries)
- Retry logic with exponential backoff patterns
- Input validation before operations
- Comprehensive error logging

**Key Concepts:**
- Safe wrapper functions for ABI calls
- Differentiating transient vs. permanent failures
- Recovery strategies (retry, fallback, abort)
- Error reporting and diagnostics
- Statistics tracking for monitoring
- Input validation and bounds checking

**Use Cases:**
- Production spirit development
- Resilient system components
- Debugging and observability
- Graceful degradation patterns

---

## ABI Function Reference

### I/O Functions

| Function | Signature | Returns | Purpose |
|----------|-----------|---------|---------|
| `vudo_println` | `(ptr: i32, len: i32) -> i32` | ResultCode | Print string to output |

### Messaging Functions

| Function | Signature | Returns | Purpose |
|----------|-----------|---------|---------|
| `vudo_send` | `(recipient_id: i32, msg_ptr: i32, msg_len: i32) -> i32` | ResultCode | Send message to spirit |
| `vudo_recv` | `() -> String` | Message | Receive next message |
| `vudo_sender` | `() -> i32` | SpiritId | Get sender of current message |
| `vudo_pending` | `() -> Int` | Count | Check message queue |

### Effect Functions

| Function | Signature | Returns | Purpose |
|----------|-----------|---------|---------|
| `vudo_emit_effect` | `(type: String, payload: String) -> i32` | ResultCode | Emit an effect |
| `vudo_subscribe` | `(pattern: String) -> i32` | SubscriptionId | Subscribe to effects |
| `vudo_unsubscribe` | `(sub_id: i32) -> i32` | ResultCode | Cancel subscription |
| `vudo_get_effect` | `() -> String` | Effect | Receive subscribed effect |

### Error Functions

| Function | Signature | Returns | Purpose |
|----------|-----------|---------|---------|
| `vudo_error` | `(code: i32, msg: String) -> i32` | ResultCode | Report error to host |

---

## Common ResultCodes

| Code | Meaning | Transient? | Action |
|------|---------|-----------|--------|
| 0 | Success | N/A | Continue |
| 1 | Not found / Invalid | No | Abort, don't retry |
| 2 | Buffer/queue full | Yes | Retry with backoff |
| 3 | Invalid format | No | Abort, validate input |
| 100+ | I/O errors | Varies | See error-handling.dol |
| 200+ | Messaging errors | Varies | See error-handling.dol |
| 300+ | Operation errors | No | Abort |

---

## Learning Path

1. **Start with `hello-world.dol`** - Learn basic I/O and the `sex fun` syntax
2. **Progress to `messaging.dol`** - Understand inter-spirit communication
3. **Study `effects.dol`** - Learn event-driven patterns
4. **Deep dive into `error-handling.dol`** - Master production techniques

---

## Best Practices

### Memory Safety
- The runtime automatically converts DOL strings to pointer/length pairs
- Always declare `sex fun` imports for host functions
- Check return codes for all I/O operations

### Performance
- Batch messages when possible
- Use subscriptions instead of polling for effects
- Minimize string allocations in hot paths

### Reliability
- Always check ResultCodes from host functions
- Use retry logic for transient failures
- Validate inputs before sending to host
- Report critical errors with `vudo_error()`

### Debugging
- Use `vudo_println()` for logging
- Track statistics (messages sent/received, errors)
- Emit effects at key decision points
- Report errors with contextual information

---

## Running the Examples

To validate these examples:

```bash
# Parse the examples
cargo run --bin dol-parse -- docs/examples/hello-world.dol
cargo run --bin dol-parse -- docs/examples/messaging.dol
cargo run --bin dol-parse -- docs/examples/effects.dol
cargo run --bin dol-parse -- docs/examples/error-handling.dol

# Check for errors
cargo run --bin dol-check -- docs/examples/
```

---

## Documentation

Each example includes:
- **Comments** - Explaining each ABI function call
- **Exegesis block** - Human-readable description of concepts
- **Helper spirits** - Reusable patterns for common tasks
- **Error handling** - Proper handling of ABI return codes

See the individual `.dol` files for comprehensive documentation.
