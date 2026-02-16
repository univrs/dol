# DOL Native Compilation

Compile DOL programs to native machine code for ARM64, RISC-V, and x86-64 via LLVM.

## Prerequisites

- Rust 1.84+
- LLVM 18 development libraries (`llvm-18-dev` on Ubuntu/Debian)
- Clang or GCC (for linking)

### Ubuntu/Debian

```bash
sudo apt install llvm-18-dev libpolly-18-dev libzstd-dev clang
```

### macOS

```bash
brew install llvm@18
```

## Building dol-native

```bash
cd llvm-backend

# Set LLVM prefix (adjust for your system)
export LLVM_SYS_181_PREFIX=/usr/lib/llvm-18   # Linux
# export LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)  # macOS

cargo build --release -p dol-native
```

The binary is at `llvm-backend/target/release/dol-native`.

## Quick Start

```bash
# Emit LLVM IR (inspect what the compiler generates)
dol-native emit-ir examples/native/hello_native.dol

# Compile to object file
dol-native build examples/native/hello_native.dol -o hello.o

# Link into executable (provide VUDO runtime stubs)
clang hello.o -o hello -lvudo_runtime
./hello
```

## Commands

### `dol-native build`

Compile a DOL file to a native object file.

```
dol-native build <input.dol> [OPTIONS]

Options:
  -o, --output <path>     Output file path (default: input with .o extension)
  -t, --target <triple>   Target architecture (default: x86_64-unknown-linux-gnu)
```

Examples:

```bash
# Default target (x86-64 Linux)
dol-native build program.dol

# ARM64 Linux
dol-native build program.dol --target aarch64-unknown-linux-gnu -o program_arm64.o

# ARM64 macOS (Apple Silicon)
dol-native build program.dol --target aarch64-apple-darwin -o program_macos.o

# RISC-V 64-bit
dol-native build program.dol --target riscv64gc-unknown-linux-gnu -o program_riscv.o

# x86-64 Windows
dol-native build program.dol --target x86_64-pc-windows-msvc -o program.obj
```

### `dol-native emit-ir`

Print LLVM IR to stdout. Useful for debugging and understanding what the compiler generates.

```bash
dol-native emit-ir program.dol
```

Output looks like:

```llvm
; ModuleID = 'program'
source_filename = "program"
target triple = "x86_64-unknown-linux-gnu"

declare void @vudo_println(ptr, i64)

define i64 @add(i64 %a, i64 %b) {
entry:
  %a1 = alloca i64, align 8
  store i64 %a, ptr %a1, align 4
  %b2 = alloca i64, align 8
  store i64 %b, ptr %b2, align 4
  %a3 = load i64, ptr %a1, align 4
  %b4 = load i64, ptr %b2, align 4
  %add = add i64 %a3, %b4
  ret i64 %add
}
```

### `dol-native targets`

List all supported compilation targets.

```
$ dol-native targets
Supported targets:

  aarch64-unknown-linux-gnu                ARM64 Linux
  aarch64-apple-darwin                     ARM64 macOS (Apple Silicon)
  riscv64gc-unknown-linux-gnu              RISC-V 64-bit Linux
  x86_64-unknown-linux-gnu                 x86-64 Linux
  x86_64-pc-windows-msvc                   x86-64 Windows
```

## Compilation Pipeline

```
DOL Source (.dol)
    │
    ▼
  Parser (lexer + recursive descent)
    │
    ▼
  AST (Abstract Syntax Tree)
    │
    ▼
  HIR Lowering (type resolution, symbol interning)
    │
    ▼
  HIR (High-level IR)
    │
    ▼
  LLVM IR Generation (hir_lowering.rs)
    │
    ▼
  LLVM IR (.ll)
    │
    ▼
  LLVM Backend (optimization + code generation)
    │
    ▼
  Native Object (.o / .obj)
    │
    ▼
  Linker (clang/ld)
    │
    ▼
  Executable
```

## What Compiles Today

### Functions

```dol
fun add(a: i64, b: i64) -> i64 {
    return a + b
}

fun fib(n: i64) -> i64 {
    if n <= 1 { return n }
    return fib(n - 1) + fib(n - 2)
}
```

- Parameters with types
- Return types (i64, bool, f64, string, ())
- Recursive calls
- Local variables (val bindings)

### Control Flow

```dol
fun abs(x: i64) -> i64 {
    if x < 0 { return -x }
    return x
}
```

- if / else branches
- Nested conditionals
- Boolean conditions with comparisons

### Genes (Struct Types)

```dol
gen Point {
    has x: f64
    has y: f64
}

gen container.exists {
    container has identity
    container has status
}
```

- Typed fields → LLVM struct types
- Untyped `has` fields → heuristic type inference
- Gene methods

### Traits, Rules, Systems

```dol
trait container.managed {
    uses container.exists
    container has supervisor
}

rule container.valid {
    container has identity
}

system container.runtime @ 0.1.0 {
    requires container.exists >= 0.0.1
}
```

These are compile-time constructs — they guide validation but don't generate runtime code.

### Type Mappings

| DOL Type | LLVM Type |
|----------|-----------|
| `i8`     | `i8`      |
| `i16`    | `i16`     |
| `i32`    | `i32`     |
| `i64`    | `i64`     |
| `u8`     | `i8`      |
| `u16`    | `i16`     |
| `u32`    | `i32`     |
| `u64`    | `i64`     |
| `f32`    | `float`   |
| `f64`    | `double`  |
| `bool`   | `i1`      |
| `string` | `ptr`     |
| `()`     | `void`    |

### VUDO ABI

The native compiler automatically declares all 22 VUDO host functions with C calling convention:

| Category   | Functions |
|------------|-----------|
| I/O        | `vudo_print`, `vudo_println`, `vudo_log`, `vudo_error` |
| Memory     | `vudo_alloc`, `vudo_free`, `vudo_realloc` |
| Time       | `vudo_now`, `vudo_sleep`, `vudo_monotonic_now` |
| Messaging  | `vudo_send`, `vudo_recv`, `vudo_pending`, `vudo_broadcast`, `vudo_free_message` |
| Random     | `vudo_random`, `vudo_random_bytes` |
| Effects    | `vudo_emit_effect`, `vudo_subscribe` |
| Debug      | `vudo_breakpoint`, `vudo_assert`, `vudo_panic` |

## What Needs Stdlib / Library Support

These features parse successfully but need runtime library implementations to work end-to-end:

| Feature | Status | What's Needed |
|---------|--------|---------------|
| String concatenation (`+`) | Parse OK, codegen panics | `dol_string_concat(ptr, ptr) -> ptr` in stdlib |
| `.to_string()` method calls | Parse OK, stub codegen | Type-specific formatters in stdlib |
| `Vec<T>` / `List<T>` | Type maps to fat pointer | Allocator + grow/push/pop in stdlib |
| `Map<K,V>` / `Set<T>` | Type maps to pointer | Hash table implementation in stdlib |
| `Option<T>` | Maps to `{i1, T}` struct | `unwrap`, `map`, `is_some` in stdlib |
| `for` / `while` loops | Not yet in HIR lowering | Loop codegen + iterator protocol |
| `match` expressions | Not yet in HIR lowering | Jump table or branch chain codegen |
| `mut` variables | Parser doesn't accept `mut` | Parser update needed |
| Lambda / closures | Maps to function pointer | Closure capture + environment in stdlib |
| CRDT annotations | Parse OK, no codegen | Full CRDT implementations as library |
| `spirit` manifests | Parser doesn't accept | Parser + package manager tooling |
| Method calls (`.method()`) | Stub codegen | Vtable or monomorphized dispatch |
| Field access (`.field`) | Stub codegen | Struct GEP codegen |
| Index access (`[i]`) | Stub codegen | Bounds checking + GEP codegen |
| `+=`, `-=` operators | Parser doesn't accept | Parser + assignment codegen |
| Path syntax (`Foo::Bar`) | Parser doesn't accept | Namespace resolution |
| Reference types (`&T`) | Parser doesn't accept | Borrow semantics |

## Architecture: llvm-backend/

```
llvm-backend/
├── Cargo.toml                          # Workspace root
├── cli/
│   └── dol-native/
│       ├── Cargo.toml
│       └── src/main.rs                 # CLI: build, emit-ir, targets
├── crates/
│   ├── dol-codegen-llvm/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  # LlvmCodegen: context, module, target machine
│   │       ├── abi.rs                  # VUDO ABI: 22 host function declarations
│   │       ├── functions.rs            # Function codegen utilities
│   │       ├── hir_lowering.rs         # HIR → LLVM IR lowering engine
│   │       ├── structs.rs              # Struct type generation
│   │       ├── targets.rs              # Target triple registry
│   │       └── types.rs                # DOL → LLVM type mapping
│   └── vudo-runtime-native/
│       ├── Cargo.toml
│       └── src/lib.rs                  # Runtime stubs (future)
└── .gitignore
```

## Examples

See `examples/native/` for programs that compile through the native pipeline:

| File | Demonstrates |
|------|-------------|
| `hello_native.dol` | Minimal program, print to stdout |
| `arithmetic.dol` | Integer math, boolean ops, multiple functions |
| `control_flow.dol` | if/else branching, nested conditions |
| `fibonacci.dol` | Recursive functions (fib, factorial, gcd, power) |
| `variables.dol` | val bindings, local scope, temperature conversion |
| `gene_structs.dol` | Genes → LLVM struct types |
| `enum_types.dol` | Tagged unions, variant constants |
| `traits_rules.dol` | Trait composition, rule constraints, system versioning |
| `string_ops.dol` | String literals, print/println |
| `vudo_host.dol` | VUDO ABI, message passing patterns |
| `multi_target.dol` | Cross-compilation to all 5 targets |

## Running Tests

```bash
cd llvm-backend
export LLVM_SYS_181_PREFIX=/usr/lib/llvm-18
cargo test --all
```

Current test count: **10 tests** (abi, functions, structs, types, targets, hir_lowering, codegen).
