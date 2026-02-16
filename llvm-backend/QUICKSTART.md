# DOL LLVM Backend — Quick Start Guide

Get up and running with the DOL LLVM Native backend in under 30 minutes.

---

## Prerequisites

- Rust 1.84+ ([install from rustup.rs](https://rustup.rs))
- LLVM 17 (see [INSTALL_LLVM.md](./INSTALL_LLVM.md))
- Linux, macOS, or WSL2

---

## 1. Install LLVM 17 (5-10 minutes)

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y llvm-17 llvm-17-dev libllvm17 clang-17
export LLVM_SYS_170_PREFIX=/usr/lib/llvm-17
```

**macOS:**
```bash
brew install llvm@17
export LLVM_SYS_170_PREFIX="$(brew --prefix llvm@17)"
```

Verify:
```bash
llvm-config-17 --version  # Should show 17.x.x
```

---

## 2. Build the Project (2-5 minutes)

```bash
cd ~/univrs-Sepah/llvm-backend

# Clean build
cargo clean

# Build in release mode
cargo build --release

# Expected output:
#   Compiling dol-codegen-llvm v0.1.0
#   Compiling vudo-runtime-native v0.1.0
#   Finished `release` profile [optimized] target(s) in 2m 34s
```

---

## 3. Run Tests (1 minute)

```bash
cargo test

# Expected output:
#   running 15 tests
#   test dol_codegen_llvm::tests::test_create_codegen ... ok
#   test dol_codegen_llvm::types::tests::test_primitive_mapping ... ok
#   test dol_codegen_llvm::functions::tests::test_generate_add ... ok
#   ...
#   test result: ok. 15 passed; 0 failed
```

---

## 4. Try Example: Compile a Simple Function (Manual)

Create a test program:

```rust
// examples/test_add.rs
use inkwell::context::Context;
use dol_codegen_llvm::{LlvmCodegen, functions::FunctionCodegen};

fn main() {
    let context = Context::create();
    let codegen = LlvmCodegen::new(&context, "test", "x86_64-unknown-linux-gnu").unwrap();

    let func_gen = FunctionCodegen::new(codegen.context(), codegen.module());
    func_gen.generate_add_function().unwrap();

    println!("{}", codegen.emit_ir());

    // Emit object file
    codegen.emit_object(std::path::Path::new("add.o")).unwrap();
    println!("Object file written to: add.o");
}
```

Run:
```bash
cargo run --example test_add --release
```

Expected output:
```llvm
; ModuleID = 'test'
source_filename = "test"

define i64 @add(i64 %0, i64 %1) {
entry:
  %sum = add i64 %0, %1
  ret i64 %sum
}
```

---

## 5. Current Status

**What Works:**
- ✅ LLVM IR generation for functions and structs
- ✅ Type mapping (primitives, structs, strings)
- ✅ ABI declarations for all 22 VUDO host functions
- ✅ Native runtime (I/O, memory, time)
- ✅ Target definitions (ARM64, RISC-V, x86-64, Windows)

**What's Next:**
- ⏳ CLI tool (`dol-native build input.dol -o output`)
- ⏳ DOL HIR → LLVM IR lowering (full language support)
- ⏳ End-to-end compilation pipeline
- ⏳ Integration tests

---

## 6. Next Steps

### Read the Documentation

1. [README.md](./README.md) - Architecture overview
2. [PROGRESS.md](./PROGRESS.md) - Current status and roadmap
3. [STATUS_REPORT.md](./STATUS_REPORT.md) - Detailed progress metrics
4. [CONTRACTS.md](./CONTRACTS.md) - Interface contracts
5. [INVARIANTS.md](./INVARIANTS.md) - Global invariants

### Explore the Code

```bash
# Core LLVM codegen
cat crates/dol-codegen-llvm/src/lib.rs
cat crates/dol-codegen-llvm/src/types.rs
cat crates/dol-codegen-llvm/src/functions.rs

# Native runtime
cat crates/vudo-runtime-native/src/lib.rs
cat crates/vudo-runtime-native/src/io.rs
```

### Run Unit Tests Individually

```bash
# Test type mapping
cargo test --package dol-codegen-llvm types::tests

# Test function codegen
cargo test --package dol-codegen-llvm functions::tests

# Test ABI declarations
cargo test --package dol-codegen-llvm abi::tests
```

---

## 7. Troubleshooting

**Problem:** `error: No suitable version of LLVM was found`

**Solution:**
```bash
export LLVM_SYS_170_PREFIX=/usr/lib/llvm-17  # Adjust path
cargo clean && cargo build
```

**Problem:** Build is slow

**Solution:**
```bash
# Use fewer parallel jobs
cargo build -j 4

# Or build in debug mode (faster compile, slower runtime)
cargo build
```

**Problem:** Tests fail with "LLVM error"

**Solution:**
```bash
# Ensure LLVM is properly installed
llvm-config-17 --version
llvm-config-17 --libdir

# Verify library path
export LD_LIBRARY_PATH=/usr/lib/llvm-17/lib:$LD_LIBRARY_PATH
cargo test
```

---

## 8. Architecture at a Glance

```
DOL Source (.dol)
      ↓
  Parser (from univrs-dol)
      ↓
    HIR (High-level IR)
      ↓
  LLVM IR Generator (dol-codegen-llvm)
      ↓
  LLVM Backend (ARM64, RISC-V, x86-64)
      ↓
  Object Code (.o)
      ↓
  System Linker + vudo-runtime-native.a
      ↓
  Native Executable
```

---

## 9. Contribute

The project is 80% complete and ready for contributions:

**High Priority:**
1. CLI tool implementation
2. DOL HIR integration
3. Integration tests
4. Messaging system implementation
5. Effects system implementation

**Medium Priority:**
1. Optimization passes
2. Cross-compilation testing
3. Benchmarking
4. Documentation improvements

See [PROGRESS.md](./PROGRESS.md) for detailed task list.

---

## 10. Support

- **Documentation:** Read all `.md` files in this directory
- **Issues:** Check [FAILED_APPROACHES.md](./FAILED_APPROACHES.md) for known issues
- **Architecture:** See [README.md](./README.md) for system design
- **Status:** Check [PROGRESS.md](./PROGRESS.md) for latest updates

---

**Ready to compile DOL to native code? Start with [INSTALL_LLVM.md](./INSTALL_LLVM.md)!**

*Last updated: 2026-02-09*
