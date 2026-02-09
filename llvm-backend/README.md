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
│           ├── io.rs         # vudo_print, vudo_log, etc.
│           ├── memory.rs     # vudo_alloc, vudo_free
│           ├── time.rs       # vudo_now, vudo_sleep
│           ├── messaging.rs  # vudo_send, vudo_recv
│           └── effects.rs    # vudo_emit_effect, vudo_subscribe
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
