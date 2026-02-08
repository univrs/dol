# ADR-005: WASM-First Runtime Target

| Field | Value |
|-------|-------|
| **Status** | Accepted |
| **Date** | 2025-10-20 |
| **Deciders** | VUDO Core Team |
| **Supersedes** | N/A |
| **Superseded by** | N/A |

## Context

DOL needs a primary compilation target for executing Spirits. The choice affects:
- Platform reach (browser, server, desktop, embedded)
- Security model (sandboxing, capabilities)
- Performance characteristics
- Ecosystem integration

### Requirements

1. **Universal Execution** - Run on browser, server, desktop, mobile
2. **Sandboxed Security** - Isolated execution, no arbitrary system access
3. **Near-Native Performance** - Acceptable overhead for most workloads
4. **Capability-Based** - Explicit permission grants for resources
5. **Portable** - Single binary runs everywhere
6. **Embeddable** - Host languages can invoke DOL code

### Options Considered

| Target | Universal | Sandboxed | Performance | Portable | Embeddable |
|--------|-----------|-----------|-------------|----------|------------|
| **WASM** | ✅ | ✅ | ⚠️ 0.8-1.2x | ✅ | ✅ |
| **Native (LLVM)** | ❌ | ❌ | ✅ 1.0x | ❌ | ⚠️ |
| **JVM** | ⚠️ | ⚠️ | ⚠️ | ✅ | ⚠️ |
| **JavaScript** | ✅ | ⚠️ | ❌ | ✅ | ✅ |

## Decision

**WebAssembly (WASM) is DOL's primary compilation target.**

### Rationale

1. **Universal Runtime**
   - Browsers: V8, SpiderMonkey, JavaScriptCore
   - Servers: Wasmtime, Wasmer, WasmEdge
   - Desktop: Tauri (WebView + Rust)
   - Edge: Cloudflare Workers, Fastly Compute

2. **Security by Default**
   - Linear memory isolation
   - No ambient authority (capabilities must be granted)
   - Deterministic execution (same inputs → same outputs)
   - No syscall access without explicit host imports

3. **WASI for System Access**
   - Standardized interface for file system, network, etc.
   - Capability-based: grant only what's needed
   - Preview 2 (Component Model) for better composition

4. **Performance Acceptable**
   - ~0.8-1.2x native speed for compute-bound code
   - Suitable for DOL's use cases (not HPC)
   - JIT compilation available in all major runtimes

5. **Ecosystem Momentum**
   - Adopted by Rust, Go, C++, AssemblyScript
   - Component Model enables language interop
   - Growing tooling and debugging support

### Secondary Targets

While WASM is primary, we support:
- **Rust** - For native performance when needed
- **TypeScript** - For JavaScript ecosystem integration
- **JSON Schema** - For validation in any language

## Consequences

### Positive

- **Write Once, Run Anywhere** - True portability
- **Secure by Design** - Can't escape sandbox
- **Browser-Native** - No plugins needed
- **Edge-Ready** - Deploy to Cloudflare/Fastly instantly
- **Future-Proof** - WASM adoption accelerating

### Negative

- **Performance Ceiling** - Not suitable for HPC/games
- **Debugging Challenges** - Source maps less mature than native
- **Binary Size** - Larger than native binaries
- **Tooling Gaps** - Profilers less advanced

### Neutral

- **Learning Curve** - Team needs WASM expertise
- **Runtime Choice** - Must pick/support multiple runtimes

## Implementation Notes

### Compilation Pipeline

```
DOL Source (.dol)
       │
       ▼
   DOL Parser
       │
       ▼
  Typed AST + HIR
       │
       ▼
  ┌────┴────┐
  │         │
  ▼         ▼
WASM      Rust
Codegen   Codegen
  │         │
  ▼         ▼
.wasm     .rs
binary    files
```

### WASM Target Configuration

```dol
// Spirit.dol
spirit MySpirit {
    targets {
        wasm: {
            optimize: true
            target: "wasm32-wasi"      // or "wasm32-unknown-unknown"
            features: ["simd", "bulk-memory"]
            stack_size: 1048576        // 1MB stack
        }
    }
}
```

### Runtime Integration

```rust
// Rust host embedding WASM Spirit
use wasmtime::*;

let engine = Engine::default();
let module = Module::from_file(&engine, "spirit.wasm")?;

let mut linker = Linker::new(&engine);
// Add WASI capabilities
wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

// Add DOL host functions
linker.func_wrap("dol", "log", |msg: &str| {
    println!("[Spirit] {}", msg);
})?;

let mut store = Store::new(&engine, WasiCtxBuilder::new().build());
let instance = linker.instantiate(&mut store, &module)?;

// Call Spirit entry point
let main = instance.get_typed_func::<(), ()>(&mut store, "main")?;
main.call(&mut store, ())?;
```

### Browser Execution

```typescript
// TypeScript host in browser
const response = await fetch('spirit.wasm');
const bytes = await response.arrayBuffer();

const imports = {
    dol: {
        log: (ptr: number, len: number) => {
            const msg = readString(memory, ptr, len);
            console.log('[Spirit]', msg);
        },
    },
};

const { instance } = await WebAssembly.instantiate(bytes, imports);
instance.exports.main();
```

### Host Function Bindings

DOL provides 22 standard host functions:

| Category | Functions |
|----------|-----------|
| I/O | `print`, `read_line`, `read_file`, `write_file` |
| Time | `now`, `sleep` |
| Random | `random_u64`, `random_f64` |
| Crypto | `hash_sha256`, `sign_ed25519`, `verify_ed25519` |
| Network | `http_get`, `http_post` |
| Storage | `store_get`, `store_set`, `store_delete` |
| CRDT | `crdt_merge`, `crdt_diff` |
| Debug | `debug_log`, `debug_break` |

## Performance Benchmarks

| Operation | Native | WASM | Overhead |
|-----------|--------|------|----------|
| Fibonacci(40) | 1.0x | 1.1x | +10% |
| JSON parse 1MB | 1.0x | 1.3x | +30% |
| CRDT merge 10K ops | 1.0x | 1.15x | +15% |
| Regex match | 1.0x | 1.4x | +40% |

Performance is acceptable for DOL's intended use cases.

## References

- [WebAssembly Specification](https://webassembly.github.io/spec/)
- [WASI Preview 2](https://github.com/WebAssembly/WASI/blob/main/preview2)
- [Wasmtime Runtime](https://wasmtime.dev/)
- [Component Model](https://component-model.bytecodealliance.org/)

## Changelog

| Date | Change |
|------|--------|
| 2025-10-20 | Initial WASM-first decision |
| 2025-11-15 | Added host function catalog |
| 2026-01-10 | Performance benchmarks added |
