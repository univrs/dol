# Installing LLVM 18 for DOL Native Compilation

The DOL LLVM backend requires **LLVM 18** and the **inkwell 0.8** crate (feature `llvm18-1`). This guide covers installation, dependency resolution, and a gap analysis of what the native compiler supports today.

---

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `inkwell` | 0.8, feature `llvm18-1` | Safe Rust bindings to LLVM C API |
| `llvm-sys` | 181.x | Raw FFI bindings (pulled in by inkwell) |
| `metadol` | 0.8.1 (path dep `..`) | DOL parser, AST, HIR, lowering |
| `thiserror` | 1.0 | Error type derivation |
| `clap` | 4.5, derive | CLI argument parsing |
| `anyhow` | 1.0 | Error context in CLI |
| `tracing` | 0.1 | Structured logging |

### System Libraries Required by llvm-sys 181

| Library | Package (Ubuntu) | Purpose |
|---------|-----------------|---------|
| `libLLVM-18` | `llvm-18-dev` | LLVM core — passes, codegen, targets |
| `libPolly` | `libpolly-18-dev` | Polyhedral loop optimizer (optional) |
| `libzstd` | `libzstd-dev` | Zstandard compression for LLVM bitcode |
| `libz` | `zlib1g-dev` | Zlib compression |
| `libxml2` | `libxml2-dev` | XML parsing (LLVM diagnostics) |
| `libtinfo` | `libtinfo-dev` | Terminal info (LLVM tools) |
| `libffi` | `libffi-dev` | Foreign function interface |

---

## Ubuntu / Debian

### Option 1: APT (Recommended)

```bash
# Add LLVM 18 repository
wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | sudo apt-key add -
sudo add-apt-repository "deb http://apt.llvm.org/$(lsb_release -cs)/ llvm-toolchain-$(lsb_release -cs)-18 main"

# Install LLVM 18 and all required development libraries
sudo apt update
sudo apt install -y \
    llvm-18 \
    llvm-18-dev \
    llvm-18-tools \
    libllvm18 \
    libpolly-18-dev \
    clang-18 \
    lld-18 \
    libzstd-dev \
    zlib1g-dev \
    libxml2-dev \
    libtinfo-dev \
    libffi-dev

# Verify
llvm-config-18 --version   # Should output: 18.x.x

# Set environment variable (add to ~/.bashrc)
export LLVM_SYS_181_PREFIX=/usr/lib/llvm-18
```

### Option 2: Build from Source

```bash
sudo apt install -y build-essential cmake ninja-build python3 git \
    zlib1g-dev libxml2-dev libzstd-dev libffi-dev

cd /tmp
wget https://github.com/llvm/llvm-project/releases/download/llvmorg-18.1.8/llvm-18.1.8.src.tar.xz
wget https://github.com/llvm/llvm-project/releases/download/llvmorg-18.1.8/cmake-18.1.8.src.tar.xz

tar xf llvm-18.1.8.src.tar.xz
tar xf cmake-18.1.8.src.tar.xz

cd llvm-18.1.8.src
mkdir build && cd build

cmake -G Ninja \
    -DCMAKE_BUILD_TYPE=Release \
    -DLLVM_ENABLE_RTTI=ON \
    -DLLVM_ENABLE_ZLIB=ON \
    -DLLVM_ENABLE_ZSTD=ON \
    -DCMAKE_INSTALL_PREFIX=/usr/local \
    ..

ninja -j$(nproc)       # 30-60 minutes
sudo ninja install

llvm-config --version   # 18.1.8
export LLVM_SYS_181_PREFIX=/usr/local
```

---

## macOS

### Homebrew (Recommended)

```bash
brew install llvm@18

# Set environment variable (add to ~/.zshrc)
export LLVM_SYS_181_PREFIX="$(brew --prefix llvm@18)"

# Verify
$(brew --prefix llvm@18)/bin/llvm-config --version
```

---

## Windows (WSL2 Recommended)

### WSL2 + Ubuntu

Follow the Ubuntu instructions above inside your WSL2 distribution.

### Native Windows

```powershell
# Download from: https://github.com/llvm/llvm-project/releases/tag/llvmorg-18.1.8
# Install LLVM-18.1.8-win64.exe
$env:LLVM_SYS_181_PREFIX = "C:\Program Files\LLVM"
```

---

## Arch Linux

```bash
sudo pacman -S llvm clang lld
export LLVM_SYS_181_PREFIX=/usr
```

## Fedora / RHEL

```bash
sudo dnf install -y llvm18-devel clang18 lld18 libzstd-devel
export LLVM_SYS_181_PREFIX=/usr/lib64/llvm18
```

---

## Building the Project

```bash
cd llvm-backend

# Required environment
export LLVM_SYS_181_PREFIX=/usr/lib/llvm-18    # Adjust for your system

# Build
cargo build --release -p dol-native

# Run tests (10 tests)
cargo test --all

# Try it
cargo run --bin dol-native -- emit-ir ../examples/native/fibonacci.dol
cargo run --bin dol-native -- build ../examples/native/fibonacci.dol -o fib.o
cargo run --bin dol-native -- targets
```

---

## Workarounds for Missing Libraries

If you don't have `sudo` access or a library is missing, these workarounds unblock the build:

### Missing libPolly (no libpolly-18-dev)

```bash
cd llvm-backend
ar cr libPolly.a
ar cr libPollyISL.a
export RUSTFLAGS="-L $(pwd)"
```

These empty archives satisfy the linker. Polly is an optional optimization pass — the compiler works without it.

### Missing libzstd.so (only .so.1 installed)

```bash
cd llvm-backend
ln -sf /usr/lib/x86_64-linux-gnu/libzstd.so.1 libzstd.so
export RUSTFLAGS="-L $(pwd)"
```

### Combined build command with workarounds

```bash
cd llvm-backend
LLVM_SYS_181_PREFIX=/usr/lib/llvm-18 RUSTFLAGS="-L $(pwd)" cargo build
```

---

## Troubleshooting

### "No suitable version of LLVM was found"

```bash
# Find where llvm-config lives
find /usr -name "llvm-config*" 2>/dev/null

# Set prefix to the parent directory
# e.g., /usr/lib/llvm-18/bin/llvm-config → prefix is /usr/lib/llvm-18
export LLVM_SYS_181_PREFIX=/usr/lib/llvm-18
```

### "could not find native static library Polly"

See the libPolly workaround above. This happens when `libpolly-18-dev` is not installed.

### "cannot find -lzstd"

See the libzstd workaround above. This happens when only the runtime `.so.1` is installed, not the development symlink.

### "LLVM version mismatch"

The inkwell 0.8 crate with feature `llvm18-1` requires exactly LLVM 18.x. The environment variable name encodes the version: `LLVM_SYS_181_PREFIX` (the `181` corresponds to llvm-sys 181.x for LLVM 18.1).

```bash
llvm-config-18 --version    # Must be 18.x.x
```

### "libLLVM-18.so not found" at runtime

```bash
export LD_LIBRARY_PATH=/usr/lib/llvm-18/lib:$LD_LIBRARY_PATH
```

### Inkwell lifetime errors in new code

When holding a reference to `Module<'ctx>`, use a separate lifetime:

```rust
// WRONG: causes "does not live long enough" errors
struct Foo<'ctx> { module: &'ctx Module<'ctx> }

// CORRECT: separate borrow lifetime from LLVM context lifetime
struct Foo<'a, 'ctx> { module: &'a Module<'ctx> }
```

---

## Environment Variables Reference

Add to `~/.bashrc` or `~/.zshrc`:

```bash
# LLVM 18 prefix (required)
export LLVM_SYS_181_PREFIX=/usr/lib/llvm-18

# Library search path for stubs (only if using workarounds)
# export RUSTFLAGS="-L /path/to/llvm-backend"

# LLVM tools on PATH (optional)
export PATH=$LLVM_SYS_181_PREFIX/bin:$PATH

# Shared library path (Linux, if needed at runtime)
export LD_LIBRARY_PATH=$LLVM_SYS_181_PREFIX/lib:$LD_LIBRARY_PATH

# Shared library path (macOS, if needed at runtime)
# export DYLD_LIBRARY_PATH=$LLVM_SYS_181_PREFIX/lib:$DYLD_LIBRARY_PATH
```

---

## Gap Analysis: Native Compilation Status

### What compiles today (93 of 155 .dol files)

| Feature | LLVM Codegen | Status |
|---------|-------------|--------|
| Functions (params, return, recursion) | Full IR generation | Working |
| Integer arithmetic (14 binary ops) | `add`, `sub`, `mul`, `sdiv`, `srem`, `icmp` | Working |
| Unary ops (negate, not) | `neg`, `not` | Working |
| Boolean logic (and, or, comparisons) | `and`, `or`, `icmp` | Working |
| if/else branching | `br`, `phi` nodes | Working |
| val/var bindings | `alloca`, `store`, `load` | Working |
| Gene → struct types | `opaque_struct_type`, `set_body` | Working |
| Enum → tagged unions | `{i32 tag, i64 payload}` + global constants | Working |
| Traits, rules, systems | Compile-time only (no IR emitted) | Working |
| String literals | Global string constants | Working |
| print/println | Calls to `vudo_println` ABI | Working |
| VUDO ABI (22 host functions) | `declare` with C calling convention | Working |
| Cross-compilation (5 targets) | Target triple + machine selection | Working |

### What needs codegen work (in hir_lowering.rs)

| Feature | Current State | What's Needed |
|---------|--------------|---------------|
| String concatenation (`"a" + "b"`) | Panics (binary op assumes integers) | Type-aware dispatch: detect pointer operands, call `dol_string_concat` |
| `.to_string()` method calls | Returns stub `i64(0)` | Monomorphized dispatch or vtable lookup |
| `.field` access | Returns stub `i64(0)` | `build_struct_gep` on struct pointer |
| `[index]` access | Returns stub `i64(0)` | `build_gep` with bounds check |
| `match` expressions | Returns stub `i64(0)` | Branch chain: compare tag, jump to arm, phi merge |
| `for` / `while` loops | Not in HIR lowering | Loop header/body/exit blocks, `br` back-edge |
| Lambda / closures | Returns null pointer | Closure environment capture, function pointer pair |

### What needs parser work

| Feature | Error | What's Needed |
|---------|-------|---------------|
| `mut` variables | `expected identifier, found 'mut'` | Accept `mut` keyword before binding name |
| `+=`, `-=` operators | `expected expression, found '+='` | Compound assignment token + desugar |
| `::` path syntax | `expected '=>' or '{', found '::'` | Namespace path resolution |
| `&T` reference types | `expected type, found '&'` | Reference type parsing |
| `spirit` manifests | `invalid declaration type 'spirit'` | New top-level declaration form |
| `fun` inside gene body | `expected predicate, found '{'` | Allow function decls inside gene blocks |
| `docs` as identifier | `expected expression, found 'docs'` | Contextual keyword handling |
| `from` as identifier | `expected expression, found 'from'` | `from` is reserved for `derives from` |

### What needs stdlib Spirit libraries

| Library | Functions Needed |
|---------|-----------------|
| `dol-stdlib` | `string_concat`, `string_length`, `to_string` (i64, f64, bool), `abs`, `min`, `max`, `clamp` |
| `dol-math` | `sin`, `cos`, `tan`, `sqrt`, `pow`, `exp`, `log`, `floor`, `ceil`, `round`, `PI`, `E` |
| `dol-collections` | `Vec<T>` (new, push, pop, get, len), `Map<K,V>` (insert, get, contains), `Set<T>`, iterators |
| `dol-io` | `read_file`, `write_file`, `stdin_line`, `format` (sprintf-style) |
| `dol-json` | `json_encode`, `json_decode`, `json_get` |
| `dol-crdt` | Peritext, RGA, OR-Set, PN-Counter, LWW-Register merge implementations |
| `dol-time` | `now`, `duration`, `format_time`, `parse_time` (beyond raw `vudo_now`) |

### Priority Roadmap

**Phase 1 — Core codegen (unblocks most examples)**
1. String concat + `.to_string()` in hir_lowering.rs
2. Field access via struct GEP
3. `for`/`while` loop codegen
4. `match` branch codegen

**Phase 2 — Parser extensions (unblocks Spirit examples)**
1. `mut` variable declarations
2. `fun` declarations inside gene bodies
3. `spirit` manifest parsing
4. `::` path syntax

**Phase 3 — Stdlib Spirit libraries (unblocks real applications)**
1. `dol-stdlib` — strings, math, formatters
2. `dol-collections` — Vec, Map, Set with VUDO memory ABI
3. `dol-crdt` — collaborative data types for Spirit networking

---

## Additional Resources

- [LLVM 18 Release Notes](https://releases.llvm.org/18.1.0/docs/ReleaseNotes.html)
- [Inkwell Documentation](https://thedan64.github.io/inkwell/)
- [llvm-sys Crate](https://crates.io/crates/llvm-sys)
- [DOL Native Compilation Guide](../docs/native-compilation.md)
