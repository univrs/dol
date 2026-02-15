# DOL Native Compilation

Compile DOL programs to native machine code. No virtual machine, no interpreter — your DOL code runs directly on hardware.

## What You Need

| Tool | Version | What it does |
|------|---------|-------------|
| **Rust** | 1.84+ | Builds the compiler and runtime |
| **LLVM** | 18 | Generates machine code from DOL |
| **cc** (gcc/clang) | any | Links the final executable |

## Install

### 1. Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. LLVM 18

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y llvm-18 llvm-18-dev clang-18
```

**macOS:**
```bash
brew install llvm@18
```

**Verify:**
```bash
llvm-config-18 --version
# Should print: 18.x.x
```

### 3. Build the DOL Native Compiler

From the repository root:

```bash
cd llvm-backend

# Set LLVM path (Ubuntu/Debian)
export LLVM_SYS_181_PREFIX=/usr/lib/llvm-18

# Fix missing libraries (one-time setup, if needed)
ar cr libPolly.a        # empty stub, only if build complains about Polly
ar cr libPollyISL.a     # empty stub, only if build complains about PollyISL
ln -sf /usr/lib/x86_64-linux-gnu/libzstd.so.1 libzstd.so  # if build complains about zstd

# Build everything
RUSTFLAGS="-L $(pwd)" cargo build --release

# Build the runtime library (needed for linking)
RUSTFLAGS="-L $(pwd)" cargo build --release -p vudo-runtime-native
```

**Verify the build:**
```bash
RUSTFLAGS="-L $(pwd)" cargo test
# Should show: 15 passed; 0 failed
```

## Compile Your First Program

### Step 1: Write a DOL program

```dol
// hello.dol
fun main() {
    println("Hello from native DOL!")
}
```

### Step 2: Compile to an object file

```bash
cd llvm-backend

LLVM_SYS_181_PREFIX=/usr/lib/llvm-18 \
RUSTFLAGS="-L $(pwd)" \
cargo run --bin dol-native -- build ../examples/native/hello_native.dol -o hello.o
```

This produces `hello.o` — a standard ELF object file containing your compiled DOL code.

### Step 3: Link into an executable

```bash
cc hello.o \
  -L target/release \
  -Wl,-Bstatic -lvudo_runtime_native -Wl,-Bdynamic \
  -lpthread -ldl -lm \
  -o hello
```

This links your compiled code with the VUDO runtime library, which provides `println`, memory allocation, and other host functions.

### Step 4: Run it

```bash
./hello
# Output: Hello from native DOL!
```

## The Three Commands

The entire compilation pipeline is three commands:

```bash
# 1. Compile:  DOL source → object file
dol-native build input.dol -o output.o

# 2. Link:     object file + runtime → executable
cc output.o -Ltarget/release -Wl,-Bstatic -lvudo_runtime_native -Wl,-Bdynamic -lpthread -ldl -lm -o program

# 3. Run:      execute the native binary
./program
```

## CLI Reference

### `dol-native build <input.dol> -o <output.o>`

Compiles a DOL source file to a native object file.

| Flag | Default | Description |
|------|---------|-------------|
| `-o <file>` | `output.o` | Output object file path |
| `--target <triple>` | host platform | Target architecture (see below) |

### `dol-native emit-ir <input.dol>`

Prints the generated LLVM IR to stdout. Useful for debugging and understanding what the compiler produces.

```bash
dol-native emit-ir examples/native/hello_native.dol
```

### `dol-native targets`

Lists all supported compilation targets.

## Supported Targets

| Target Triple | Platform |
|---------------|----------|
| `x86_64-unknown-linux-gnu` | x86-64 Linux (default on Linux) |
| `aarch64-unknown-linux-gnu` | ARM64 Linux (Raspberry Pi, AWS Graviton) |
| `aarch64-apple-darwin` | Apple Silicon (M1/M2/M3 Mac) |
| `riscv64gc-unknown-linux-gnu` | RISC-V 64-bit Linux |
| `x86_64-pc-windows-msvc` | x86-64 Windows |

Cross-compile with `--target`:
```bash
dol-native build hello.dol -o hello_arm.o --target aarch64-unknown-linux-gnu
```

## What DOL Features Work in Native

### Fully Working

| Feature | Example |
|---------|---------|
| Functions with parameters and return types | `fun add(a: i64, b: i64) -> i64` |
| Integer arithmetic (`+`, `-`, `*`, `/`, `%`) | `return a + b` |
| Comparison operators (`==`, `!=`, `<`, `>`, `<=`, `>=`) | `if x > 0` |
| Boolean logic (`and`, `or`, `not`) | `if a and b` |
| Unary operators (`-`, `not`) | `return -x` |
| If/else conditionals | `if n % 2 == 0 { ... }` |
| Recursive function calls | `return fib(n - 1) + fib(n - 2)` |
| Local variables (`let`) | `let sum = add(10, 20)` |
| String literals | `"Hello, World!"` |
| String concatenation | `"prefix " + "suffix"` |
| String + integer concat | `"value = " + 42` |
| `println()` and `print()` | `println("Hello!")` |
| String-returning functions | `fun grade(n: i64) -> string` |
| Gen (struct) type declarations | `gen Point { has x: f64 }` |
| Enum type declarations | `enum Color { Red, Green, Blue }` |
| Trait/rule/system declarations | Compile-time, validated but no runtime code |
| Multiple functions per file | All functions compiled and callable |
| `main()` entry point | Required for executables |

### Not Yet Supported

| Feature | Status |
|---------|--------|
| Method calls (`obj.method()`) | Parser limitation |
| Field access (`point.x`) | Codegen stub |
| `while` / `for` loops | Codegen not implemented |
| `match` expressions | Codegen stub |
| Mutable variables (`var`) | Parser limitation |
| Compound assignment (`+=`, `-=`) | Parser limitation |
| Path syntax (`module::item`) | Parser limitation |
| Reference types (`&T`) | Parser limitation |
| Lambdas / closures | Codegen stub |
| Standard library (collections, etc.) | Not yet available |

## Examples

This directory contains 12 working examples that all compile and run:

| File | What it demonstrates |
|------|---------------------|
| `hello_native.dol` | Minimal program — println with string literals |
| `arithmetic.dol` | Integer functions: add, subtract, multiply, divide, abs, max, min, clamp |
| `fibonacci.dol` | Recursive functions: fib, factorial, gcd, power |
| `control_flow.dol` | If/else chains: fizzbuzz, sign detection, grading |
| `variables.dol` | Local variable bindings with let |
| `string_ops.dol` | String concatenation, print vs println, log levels |
| `enum_types.dol` | Enum type declarations compiled to tagged unions |
| `gene_structs.dol` | Gen declarations compiled to LLVM struct types |
| `traits_rules.dol` | Trait/rule/system declarations (compile-time) |
| `multi_target.dol` | Cross-compilation target demonstration |
| `vudo_host.dol` | VUDO runtime host function interface |
| `program.dol` | **Comprehensive showcase** of all working features |

### Run all examples

```bash
cd llvm-backend

for f in ../examples/native/*.dol; do
    base=$(basename "$f" .dol)
    LLVM_SYS_181_PREFIX=/usr/lib/llvm-18 RUSTFLAGS="-L $(pwd)" \
      cargo run --release --bin dol-native -- build "$f" -o "/tmp/${base}.o" && \
    cc "/tmp/${base}.o" \
      -Ltarget/release -Wl,-Bstatic -lvudo_runtime_native -Wl,-Bdynamic \
      -lpthread -ldl -lm -o "/tmp/${base}" && \
    echo "=== $base ===" && "/tmp/${base}"
done
```

## How It Works

```
 hello.dol               DOL source code
    |
    v
 DOL Parser              Tokenizes and parses into AST
    |
    v
 HIR Lowering            Converts AST to High-level IR
    |
    v
 LLVM Codegen            Translates HIR to LLVM IR
    |                     - Maps DOL types to LLVM types
    |                     - Generates function bodies
    |                     - Declares VUDO host functions
    v
 LLVM Backend            Optimizes and emits machine code
    |                     for the target architecture
    v
 hello.o                 Standard object file (ELF/Mach-O/COFF)
    |
    v
 System Linker (cc)      Links with VUDO runtime + system libs
    |
    v
 hello                   Native executable — runs on bare metal
```

### The VUDO Runtime

DOL programs call host functions like `println()`, `alloc()`, `now()`, etc. These are provided by `libvudo_runtime_native.a`, a static library written in Rust that implements 24 host functions:

| Category | Functions |
|----------|-----------|
| **I/O** | `print`, `println`, `log`, `error` |
| **Memory** | `alloc`, `free`, `realloc` |
| **Time** | `now`, `sleep`, `monotonic_now` |
| **Strings** | `string_concat`, `i64_to_string` |
| **Messaging** | `send`, `recv`, `pending`, `broadcast`, `free_message` |
| **Effects** | `emit_effect`, `subscribe` |
| **Random** | `random`, `random_bytes` |
| **Debug** | `breakpoint`, `assert`, `panic` |

When you write `println("Hello!")` in DOL, the compiler generates a call to `vudo_println(ptr, len)` which the runtime implements using Rust's standard I/O.

## Troubleshooting

### "error: could not find native static library `Polly`"

Create empty stub libraries:
```bash
cd llvm-backend
ar cr libPolly.a
ar cr libPollyISL.a
```

### "error: unable to find library -lzstd"

Create a symlink:
```bash
cd llvm-backend
ln -sf /usr/lib/x86_64-linux-gnu/libzstd.so.1 libzstd.so
```

### "undefined reference to `main`"

Your DOL file needs a `fun main()` function to produce an executable.

### Build is slow

LLVM linking takes time on first build. Subsequent builds are fast (under 1 second). Use `--release` for the runtime but `debug` for the compiler during development:
```bash
cargo build --release -p vudo-runtime-native   # build runtime once
cargo build                                      # iterate on compiler fast
```

### "LLVM_SYS_181_PREFIX" not set

Always set the LLVM path before building:
```bash
export LLVM_SYS_181_PREFIX=/usr/lib/llvm-18
```

Add it to your shell profile (`~/.bashrc` or `~/.zshrc`) to make it permanent.
