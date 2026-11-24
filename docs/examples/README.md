# DOL Host ABI Examples

Small DOL programs that call the VUDO VM through its host ABI. Each one
compiles to WebAssembly with `metadol::wasm::WasmCompiler` and imports only
functions the VM provides.

## The host ABI

A Spirit reaches the outside world only through host functions in WASM import
module `vudo`. The VM provides exactly 15, all named `host_*` and defined in
[`src/wasm/host_abi.rs`](../../src/wasm/host_abi.rs):

| Category | Functions |
|----------|-----------|
| Time / random | `host_time_now`, `host_random_bytes` |
| Logging | `host_log` |
| Storage | `host_storage_read`, `host_storage_write`, `host_storage_delete` |
| Network | `host_network_connect`, `host_network_listen`, `host_network_broadcast` |
| Credit | `host_credit_balance`, `host_credit_transfer`, `host_credit_reserve`, `host_credit_release`, `host_credit_consume`, `host_credit_available` |

Declare one with a bodiless `sex fun`, for example
`sex fun host_log(level: i32, msg: string) -> i32`. A DOL `string` argument
lowers to the ABI's `(ptr, len)` pair. Declaring a function the VM does not
provide, or one with the wrong arity or parameter types, is a compile error.
The older `vudo_*` names (`vudo_println`, `vudo_send`, `vudo_emit_effect`, ...)
are not part of the VM ABI.

The functions these examples use:

| Function | DOL declaration | Returns |
|----------|-----------------|---------|
| `host_log` | `(level: i32, msg: string) -> i32` | 0 on success, -1 on error |
| `host_storage_write` | `(key: string, value: string) -> i32` | 0 on success, -1 on error |
| `host_network_broadcast` | `(msg: string) -> i64` | peers reached, or -1 on error |

`host_log` levels: 0 = TRACE, 1 = DEBUG, 2 = INFO, 3 = WARN, 4 = ERROR.
Storage values are opaque bytes to the VM.

## Examples

| File | What it does |
|------|--------------|
| `hello-world.dol` | Logs a greeting at INFO and a message at DEBUG with `host_log`. |
| `error-handling.dol` | Checks the status codes of `host_storage_write` and `host_log`, logs failures at ERROR, and propagates -1. |
| `messaging.dol` | Sends a message to peers with `host_network_broadcast` and logs the peer count outcome. |
| `effects.dol` | Records a state change: persists it with `host_storage_write`, broadcasts it, and logs it. |

## Not available yet

- **Receiving messages.** The VM has no host function that delivers incoming
  messages to a Spirit, so `messaging.dol` only sends.
- **Effect publish/subscribe.** The VM has no emit/subscribe host functions.
  `effects.dol` shows side effects as plain host calls instead.
- **Reading storage into a buffer.** `host_storage_read(key, val_ptr, cap)`
  returns the number of bytes written (0 if the key is missing, -1 on error),
  but DOL cannot yet allocate a buffer or load bytes from memory, so the
  examples do not read storage.

## Checking the examples

```bash
# Compile every example and check its imports against the VM ABI,
# then run each `main` against host stubs
cargo test --features wasm --test docs_examples_abi

# Parse and validate
cargo run --features cli --bin dol-check -- docs/examples/
```
