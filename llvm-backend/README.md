# 🔧 DOL LLVM Backend — Native Compilation

**Project:** DOL → LLVM IR → Native binaries (arm64, RISC-V, x86)  
**Framework:** LLM-TEAMS.md (hive-mind agentic development)  
**Goal:** Compile DOL Spirits to native machine code for hardware targets beyond WASM

---

## 🎯 End State (Measurable)

A working `dol-llvm` crate and CLI that:

1. **Parses** → DOL source files (existing parser)
2. **Lowers** → DOL HIR to LLVM IR
3. **Optimizes** → LLVM optimization passes (O0-O3)
4. **Emits** → Native code for target architectures:
   - `aarch64-unknown-linux-gnu` (ARM64 Linux)
   - `aarch64-apple-darwin` (ARM64 macOS / Apple Silicon)
   - `riscv64gc-unknown-linux-gnu` (RISC-V 64-bit)
   - `x86_64-unknown-linux-gnu` (x86-64 Linux)
   - `x86_64-pc-windows-msvc` (x86-64 Windows)
5. **Links** → Against VUDO runtime library for host functions
6. **Produces** → Standalone executables or shared libraries

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      DOL Source (.dol)                       │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    DOL Parser (existing)                     │
│                         AST → HIR                            │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                  LLVM IR Generator (NEW)                     │
│              dol-codegen-llvm / hir_llvm.rs                  │
│                                                              │
│  • Type lowering (DOL types → LLVM types)                   │
│  • Function codegen (DOL fun → LLVM function)               │
│  • Gen codegen (DOL gen → LLVM struct)                      │
│  • ABI bridge (host function imports)                       │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      LLVM Backend                            │
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │  ARM64   │  │  RISC-V  │  │  x86-64  │  │  WASM*   │    │
│  │ aarch64  │  │ riscv64  │  │  x86_64  │  │ wasm32   │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │
│       │             │             │             │           │
│       ▼             ▼             ▼             ▼           │
│    .o / .a      .o / .a       .o / .a      .wasm           │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     System Linker                            │
│              (ld / lld / link.exe)                          │
│                                                              │
│  + libvudo_runtime.a (host function implementations)        │
│  + libc / system libraries                                  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   Native Executable                          │
│           Spirit runs directly on hardware                   │
└─────────────────────────────────────────────────────────────┘

* WASM via LLVM is optional alternative to existing wasm-bindgen path
```

---

## 📁 Project Structure

```
llvm-backend/
├── README.md                 # This file
├── PROGRESS.md               # Current status (agent-maintained)
├── DECISIONS.log             # Architectural decisions (YAML)
├── FAILED_APPROACHES.md      # What didn't work
├── INVARIANTS.md             # Global invariants
├── CONTRACTS.md              # Interface contracts
├── current_tasks/            # Agent task locks
│
├── crates/
│   ├── dol-codegen-llvm/     # LLVM IR generator
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs      # DOL → LLVM type mapping
│   │       ├── functions.rs  # Function codegen
│   │       ├── structs.rs    # Gen/struct codegen
│   │       ├── abi.rs        # Host function ABI
│   │       └── targets.rs    # Target-specific codegen
│   │
│   └── vudo-runtime-native/  # Native runtime library
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── host.rs       # host_* — the VUDO VM host ABI (15 functions)
│           ├── io.rs         # vudo_print, vudo_println (intrinsics)
│           └── memory.rs     # vudo_alloc, vudo_free (intrinsics)
│
├── cli/
│   └── dol-native/           # CLI for native compilation
│       ├── Cargo.toml
│       └── src/main.rs
│
├── tests/
│   ├── fixtures/             # Test DOL programs
│   └── integration/          # End-to-end tests
│
└── docs/
    ├── LLVM_MAPPING.md       # DOL → LLVM type/instruction mapping
    └── ABI_SPEC.md           # Native ABI specification
```

---

## 🔧 Build & Test

```bash
# Build the LLVM codegen crate
cargo build -p dol-codegen-llvm

# Compile a DOL file to native (future)
dol-native build ./spirit.dol --target aarch64-apple-darwin -o spirit

# Run native Spirit
./spirit

# Cross-compile for RISC-V
dol-native build ./spirit.dol --target riscv64gc-unknown-linux-gnu
```

---

## 🔌 Native Host ABI

The Spirit host ABI is the VUDO VM's 15 `host_*` functions (canonical table:
`src/wasm/host_abi.rs` at the repository root; VM linker:
`univrs/vudo` `vudo_vm/src/linker.rs`). A DOL Spirit declares the ones it uses
as bodiless `sex fun`s:

```dol
sex fun host_log(level: i32, msg: string) -> i32

sex fun main() {
    host_log(2, "hello")   // prints "[VUDO:INFO] hello"
}
```

The WASM backend turns these into imports from module `vudo`. This backend
calls the C-ABI functions of the same names in `vudo-runtime-native`, so the
same source runs in the VM and natively.

Native signatures keep the VM's `i32`/`i64` types. A VM guest address becomes
a native pointer, and a DOL `string` argument becomes `(ptr, i32 len)` where the VM
has a `(ptr, len)` pair. `crates/dol-codegen-llvm/src/abi.rs` documents the full
mapping. A bodiless function that is not a host function, a `string` in a
non-pair slot, or a wrong arity is a compile error, as in the WASM backend.

| Host function | Native behaviour |
|---------------|------------------|
| `host_time_now() -> i64` | Real: Unix time in nanoseconds |
| `host_random_bytes(buf, len) -> i32` | Real: OS CSPRNG; 0, or -1 if `len` is not in 1..=1 MiB |
| `host_log(level, msg, len) -> i32` | Real: `[VUDO:LEVEL] msg`. Levels 0 trace, 1 debug and 4 error go to stderr; 2 info and 3 warn go to stdout. Returns -1 for an unknown level, invalid UTF-8 or more than 64 KiB |
| `host_storage_read(key, key_len, buf, cap) -> i32` | In-process map: bytes written, 0 if missing, -1 on error (incl. value > `cap`) |
| `host_storage_write(key, key_len, val, val_len) -> i32` | In-process map: 0, or -1 on error |
| `host_storage_delete(key, key_len) -> i32` | In-process map: 1 deleted, 0 missing, -1 on error |
| `host_network_connect` / `_listen` / `_broadcast` | **Always -1**: there is no P2P network natively |
| `host_credit_balance` / `_transfer` / `_reserve` / `_release` / `_consume` / `_available` | **Always -1**: there is no credit ledger natively |

Native storage lives only as long as the process. Network and credit calls never
report success.

**Runtime intrinsics (native only, not the host ABI).** The compiler also emits
calls to `vudo_print`, `vudo_println`, `vudo_alloc`, `vudo_free`, `vudo_realloc`,
`vudo_string_concat`, `vudo_i64_to_string`, `vudo_assert`, `vudo_panic` and
`vudo_breakpoint`. These implement DOL built-ins such as `println`, string `+`
and `.to_string()`. They do not exist in the VM. A Spirit that must also run
there should log through `host_log`. The built-ins `log(level, msg)` and `now()`
lower to `host_log` and `host_time_now`.

The pre-0.8.2 `vudo_*` host functions are removed: `vudo_log`, `vudo_error`,
`vudo_now`, `vudo_sleep`, `vudo_monotonic_now`, `vudo_send`, `vudo_recv`,
`vudo_pending`, `vudo_broadcast`, `vudo_free_message`, `vudo_random`,
`vudo_random_bytes`, `vudo_emit_effect` and `vudo_subscribe`. Several of them
reported success without doing anything.

---

## 🦀 Rust LLVM Integration Options

| Option | Crate | Pros | Cons |
|--------|-------|------|------|
| **Inkwell** | `inkwell` | Safe Rust API, well-maintained | Requires LLVM installation |
| **llvm-sys** | `llvm-sys` | Direct bindings, full control | Unsafe, verbose |
| **Cranelift** | `cranelift` | Pure Rust, no LLVM dep | Less optimized output |
| **MLIR** | `melior` | Modern, extensible | Complex, newer |

**Recommendation:** Start with **Inkwell** for safety + LLVM power. Fall back to Cranelift for environments without LLVM.

---

## 🌐 Connection to Univrs

This backend extends:
- **DOL** (`~/repos/univrs-dol`) — Shares parser, AST, HIR
- **VUDO** (`~/repos/univrs-vudo`) — Native runtime implements same ABI
- **Network** (`~/repos/univrs-network`) — Native Spirits can join P2P mesh

---

*From WASM to metal. Spirits run everywhere.* 🔧
