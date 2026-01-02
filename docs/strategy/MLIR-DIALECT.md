# DOL MLIR Dialect Specification

## Overview

The DOL MLIR (Multi-Level Intermediate Representation) dialect provides a bridge between DOL's high-level ontological specifications and executable code. It leverages the [MLIR infrastructure](https://mlir.llvm.org) to enable progressive lowering from semantic declarations to platform-specific targets including WebAssembly, LLVM IR, and specialized hardware backends.

### Purpose

The DOL MLIR dialect serves three critical functions:

1. **Type Lowering**: Converts DOL's rich type system (including ontological types like Genes and Traits) into MLIR's type representations
2. **Operation Generation**: Maps DOL expressions and statements to MLIR operations using standard dialects (arith, scf, func)
3. **Target Independence**: Provides a common intermediate representation that can target multiple backends

### Architecture

```
DOL Source (.dol)
      ↓
  Lexer & Parser
      ↓
    AST
      ↓
    HIR (High-level Intermediate Representation)
      ↓
   MLIR Dialect (this layer)
      ↓
  MLIR Standard Dialects
      ↓
  Target Code (WASM, LLVM, etc.)
```

The MLIR layer sits between HIR (which preserves DOL semantics) and target-specific code generation, enabling optimization passes and progressive lowering strategies.

## Type System Mapping

The `TypeLowering` module (`src/mlir/types.rs`) handles conversion from DOL types to MLIR types.

### Primitive Types

DOL's primitive types map directly to MLIR built-in types:

| DOL Type    | MLIR Type                          | Description                              |
|-------------|------------------------------------|------------------------------------------|
| `Void`      | `tuple<>` (empty tuple)           | Unit type, no value                      |
| `Bool`      | `i1`                              | 1-bit integer (boolean)                  |
| `Int8`      | `i8`                              | 8-bit signed integer                     |
| `Int16`     | `i16`                             | 16-bit signed integer                    |
| `Int32`     | `i32`                             | 32-bit signed integer                    |
| `Int64`     | `i64`                             | 64-bit signed integer                    |
| `UInt8`     | `i8`                              | 8-bit unsigned (semantics in ops)        |
| `UInt16`    | `i16`                             | 16-bit unsigned (semantics in ops)       |
| `UInt32`    | `i32`                             | 32-bit unsigned (semantics in ops)       |
| `UInt64`    | `i64`                             | 64-bit unsigned (semantics in ops)       |
| `Float32`   | `f32`                             | 32-bit IEEE 754 floating point           |
| `Float64`   | `f64`                             | 64-bit IEEE 754 floating point           |
| `String`    | `index` (placeholder)             | String reference (future: `!llvm.ptr`)   |

**Note**: MLIR does not distinguish between signed and unsigned integer types at the type level. Signedness is determined by the operations used (e.g., `arith.divsi` vs `arith.divui`).

### Compound Types

| DOL Type                  | MLIR Type                         | Description                              |
|---------------------------|-----------------------------------|------------------------------------------|
| `Function { params, ret }`| `function<(params) -> results>`   | Function signature                       |
| `Tuple(T1, T2, ...)`      | `tuple<T1, T2, ...>`              | Fixed-size tuple of heterogeneous types  |

### Generic Types

DOL's generic types are lowered to specialized MLIR representations:

| DOL Type             | MLIR Type                              | Representation                           |
|----------------------|----------------------------------------|------------------------------------------|
| `List<T>`            | `T` (placeholder)                      | Future: `memref<?xT>` (dynamic array)    |
| `Option<T>`          | `tuple<i1, T>`                         | Tuple of (is_present_flag, value)       |
| `Result<T, E>`       | `tuple<i1, T, E>`                      | Tuple of (is_ok_flag, ok_value, err_value) |
| `Quoted<T>`          | `index` (opaque)                       | Metaprogramming AST representation       |
| `TypeInfo`           | `index` (opaque)                       | Runtime type reflection metadata         |

### Unsupported Types

The following DOL types cannot be lowered and will produce errors:

- `Type::Unknown` - Type inference incomplete
- `Type::Any` - Requires concrete type
- `Type::Error` - Error during type checking
- `Type::Var(id)` - Unresolved type variable

## Operations

The `OpBuilder` module (`src/mlir/ops.rs`) provides methods to construct MLIR operations from DOL expressions using standard MLIR dialects.

### Arithmetic Operations (arith dialect)

#### Integer Arithmetic

| DOL Operator | MLIR Operation     | Description                    | Signature              |
|--------------|--------------------|--------------------------------|------------------------|
| `+`          | `arith.addi`       | Integer addition               | `(i64, i64) -> i64`    |
| `-`          | `arith.subi`       | Integer subtraction            | `(i64, i64) -> i64`    |
| `*`          | `arith.muli`       | Integer multiplication         | `(i64, i64) -> i64`    |
| `/`          | `arith.divsi`      | Signed integer division        | `(i64, i64) -> i64`    |
| `%`          | `arith.remsi`      | Signed integer remainder       | `(i64, i64) -> i64`    |

**Example MLIR**:
```mlir
%0 = arith.constant 10 : i64
%1 = arith.constant 20 : i64
%2 = arith.addi %0, %1 : i64
// %2 = 30
```

#### Floating-Point Arithmetic

| DOL Operator | MLIR Operation     | Description                    | Signature              |
|--------------|--------------------|--------------------------------|------------------------|
| `+`          | `arith.addf`       | Float addition                 | `(f64, f64) -> f64`    |
| `-`          | `arith.subf`       | Float subtraction              | `(f64, f64) -> f64`    |
| `*`          | `arith.mulf`       | Float multiplication           | `(f64, f64) -> f64`    |
| `/`          | `arith.divf`       | Float division                 | `(f64, f64) -> f64`    |
| `%`          | `arith.remf`       | Float remainder                | `(f64, f64) -> f64`    |

**Example MLIR**:
```mlir
%0 = arith.constant 3.14 : f64
%1 = arith.constant 2.0 : f64
%2 = arith.mulf %0, %1 : f64
// %2 = 6.28
```

### Comparison Operations (arith dialect)

#### Integer Comparisons

| DOL Operator | MLIR Operation                        | Description              | Signature           |
|--------------|---------------------------------------|--------------------------|---------------------|
| `==`         | `arith.cmpi(eq, ...)`                | Equal                    | `(i64, i64) -> i1`  |
| `!=`         | `arith.cmpi(ne, ...)`                | Not equal                | `(i64, i64) -> i1`  |
| `<`          | `arith.cmpi(slt, ...)`               | Signed less than         | `(i64, i64) -> i1`  |
| `<=`         | `arith.cmpi(sle, ...)`               | Signed less or equal     | `(i64, i64) -> i1`  |
| `>`          | `arith.cmpi(sgt, ...)`               | Signed greater than      | `(i64, i64) -> i1`  |
| `>=`         | `arith.cmpi(sge, ...)`               | Signed greater or equal  | `(i64, i64) -> i1`  |

**Example MLIR**:
```mlir
%0 = arith.constant 42 : i64
%1 = arith.constant 100 : i64
%2 = arith.cmpi slt, %0, %1 : i64
// %2 = true (1 : i1)
```

#### Floating-Point Comparisons

| DOL Operator | MLIR Operation                        | Description              | Signature           |
|--------------|---------------------------------------|--------------------------|---------------------|
| `==`         | `arith.cmpf(oeq, ...)`               | Ordered equal            | `(f64, f64) -> i1`  |
| `!=`         | `arith.cmpf(one, ...)`               | Ordered not equal        | `(f64, f64) -> i1`  |
| `<`          | `arith.cmpf(olt, ...)`               | Ordered less than        | `(f64, f64) -> i1`  |
| `<=`         | `arith.cmpf(ole, ...)`               | Ordered less or equal    | `(f64, f64) -> i1`  |
| `>`          | `arith.cmpf(ogt, ...)`               | Ordered greater than     | `(f64, f64) -> i1`  |
| `>=`         | `arith.cmpf(oge, ...)`               | Ordered greater or equal | `(f64, f64) -> i1`  |

### Logical Operations (arith dialect)

| DOL Operator | MLIR Operation     | Description              | Signature           |
|--------------|--------------------|--------------------------|---------------------|
| `&`          | `arith.andi`       | Bitwise/logical AND      | `(i1, i1) -> i1`    |
| `\|\|`       | `arith.ori`        | Bitwise/logical OR       | `(i1, i1) -> i1`    |
| `!`          | `arith.xori(..., true)` | Logical NOT (XOR with 1) | `(i1) -> i1`   |

**Example MLIR**:
```mlir
%0 = arith.constant true
%1 = arith.constant false
%2 = arith.andi %0, %1 : i1
// %2 = false

%3 = arith.constant true
%4 = arith.xori %3, true : i1
// %4 = false (NOT operation)
```

### Unary Operations

| DOL Operator | Implementation                        | Description              | Signature           |
|--------------|---------------------------------------|--------------------------|---------------------|
| `-x`         | `arith.subi(0, x)`                   | Negation (0 - x)         | `(i64) -> i64`      |
| `!x`         | `arith.xori(x, true)`                | Logical NOT (XOR with 1) | `(i1) -> i1`        |

### Constant Operations (arith dialect)

| Operation                  | MLIR Operation            | Description                    |
|----------------------------|---------------------------|--------------------------------|
| `build_constant_i64(val)`  | `arith.constant`          | 64-bit integer constant        |
| `build_constant_i1(val)`   | `arith.constant`          | Boolean (i1) constant          |

**Example MLIR**:
```mlir
%0 = arith.constant 42 : i64
%1 = arith.constant true
%2 = arith.constant 3.14159 : f64
```

### Control Flow Operations (scf dialect)

#### If-Then-Else

The `scf.if` operation provides conditional execution with optional else branch.

**OpBuilder Method**: `build_if(condition, result_types, then_region, else_region, location)`

**MLIR Syntax**:
```mlir
%result = scf.if %condition -> (i64) {
  // then branch
  %0 = arith.constant 10 : i64
  scf.yield %0 : i64
} else {
  // else branch
  %1 = arith.constant 20 : i64
  scf.yield %1 : i64
}
```

**DOL Example**:
```dol
val x = if (a > b) { a } else { b }
```

#### For Loop

The `scf.for` operation provides bounded iteration with loop-carried variables.

**OpBuilder Method**: `build_for(lower_bound, upper_bound, step, init_args, body_region, location)`

**MLIR Syntax**:
```mlir
%sum = scf.for %i = %lb to %ub step %step iter_args(%arg = %init) -> (i64) {
  %next = arith.addi %arg, %i : i64
  scf.yield %next : i64
}
```

**DOL Example**:
```dol
var sum = 0
for (i in 0..10) {
  sum = sum + i
}
```

#### While Loop

The `scf.while` operation provides conditional iteration with before/after regions.

**OpBuilder Method**: `build_while(init_args, before_region, after_region, location)`

**MLIR Syntax**:
```mlir
%result = scf.while (%arg = %init) : (i64) -> i64 {
  // before region (condition)
  %cond = arith.cmpi slt, %arg, %limit : i64
  scf.condition(%cond) %arg : i64
} do {
  ^bb0(%arg: i64):
  // after region (body)
  %next = arith.addi %arg, %step : i64
  scf.yield %next : i64
}
```

### Function Operations (func dialect)

#### Function Declaration

The `func.func` operation declares and defines functions.

**OpBuilder Method**: `build_func(name, function_type, body_region, location)`

**MLIR Syntax**:
```mlir
func.func @add(%arg0: i64, %arg1: i64) -> i64 {
  %0 = arith.addi %arg0, %arg1 : i64
  func.return %0 : i64
}
```

**DOL Example**:
```dol
fun add(a: Int64, b: Int64) -> Int64 {
  return a + b
}
```

#### Function Call

The `func.call` operation invokes a function by name.

**OpBuilder Method**: `build_call(callee, arguments, result_types, location)`

**MLIR Syntax**:
```mlir
%result = func.call @add(%x, %y) : (i64, i64) -> i64
```

**DOL Example**:
```dol
val result = add(10, 20)
```

#### Return Statement

The `func.return` operation returns values from a function.

**OpBuilder Method**: `build_return(operands, location)`

**MLIR Syntax**:
```mlir
func.return %value : i64
func.return  // void return
```

## Unsupported Operations

The following DOL operations are not currently supported in MLIR lowering:

| DOL Operator      | Reason                                          |
|-------------------|-------------------------------------------------|
| `^` (power)       | Requires math dialect or custom implementation  |
| `\|>` (pipe)      | Functional operator, requires custom dialect    |
| `>>` (compose)    | Functional operator, requires custom dialect    |
| `$` (apply)       | Functional operator, requires custom dialect    |
| `>>=` (bind)      | Monadic operator, requires custom dialect       |
| `.` (member)      | Requires struct/object lowering                 |
| `<$>` (map)       | Functor operator, requires custom dialect       |
| `<*>` (ap)        | Applicative operator, requires custom dialect   |
| `'` (quote)       | Metaprogramming, not supported in MLIR lowering |
| `?` (reflect)     | Metaprogramming, not supported in MLIR lowering |

## Complete Example

### DOL Source

```dol
module math @ 1.0

fun factorial(n: Int64) -> Int64 {
  if (n <= 1) {
    return 1
  } else {
    return n * factorial(n - 1)
  }
}

exegesis {
  Computes the factorial of a number recursively.
}
```

### Generated MLIR (Conceptual)

```mlir
module @math {
  func.func @factorial(%arg0: i64) -> i64 {
    %c1 = arith.constant 1 : i64
    %cond = arith.cmpi sle, %arg0, %c1 : i64

    %result = scf.if %cond -> (i64) {
      scf.yield %c1 : i64
    } else {
      %n_minus_1 = arith.subi %arg0, %c1 : i64
      %rec_result = func.call @factorial(%n_minus_1) : (i64) -> i64
      %product = arith.muli %arg0, %rec_result : i64
      scf.yield %product : i64
    }

    func.return %result : i64
  }
}
```

### WASM Output

The MLIR is further lowered to WebAssembly:

```wasm
(module
  (func $factorial (param $n i64) (result i64)
    (if (result i64)
      (i64.le_s (local.get $n) (i64.const 1))
      (then (i64.const 1))
      (else
        (i64.mul
          (local.get $n)
          (call $factorial
            (i64.sub (local.get $n) (i64.const 1))))))
  (export "factorial" (func $factorial)))
```

## Compilation Pipeline

The complete compilation pipeline from DOL source to executable code:

```
┌─────────────────────────────────────────────────────────────────┐
│ DOL Source (.dol file)                                          │
│ ──────────────────────────────────────────                      │
│ fun add(a: Int64, b: Int64) -> Int64 { return a + b }          │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│ Lexer (src/lexer.rs)                                            │
│ ──────────────────────────────────────────                      │
│ Tokens: [Fun, Ident("add"), LParen, Ident("a"), ...]           │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│ Parser (src/parser.rs)                                          │
│ ──────────────────────────────────────────                      │
│ AST: FunctionDecl { name: "add", params: [...], body: ... }    │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│ HIR Lowering (src/hir/)                                         │
│ ──────────────────────────────────────────                      │
│ HIR: Type-checked, semantically validated IR                    │
│ - Type inference complete                                       │
│ - Name resolution complete                                      │
│ - Semantic validation passed                                    │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│ MLIR Lowering (src/mlir/types.rs, src/mlir/ops.rs) ← YOU ARE HERE
│ ──────────────────────────────────────────                      │
│ MLIR Operations:                                                │
│   func.func @add(%arg0: i64, %arg1: i64) -> i64 {              │
│     %0 = arith.addi %arg0, %arg1 : i64                          │
│     func.return %0 : i64                                        │
│   }                                                             │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│ MLIR Optimization Passes (melior crate)                         │
│ ──────────────────────────────────────────                      │
│ - Dead code elimination                                         │
│ - Constant folding                                              │
│ - Loop optimization                                             │
│ - Inlining                                                      │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│ Target Lowering (MLIR → WASM/LLVM)                              │
│ ──────────────────────────────────────────                      │
│ - Standard dialect → Target dialect                             │
│ - Platform-specific optimizations                               │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│ Code Generation (WebAssembly)                                   │
│ ──────────────────────────────────────────                      │
│ Binary WASM module (.wasm)                                      │
│ - Executable in VUDO VM                                         │
│ - Portable across platforms                                     │
│ - Optimized machine code                                        │
└─────────────────────────────────────────────────────────────────┘
```

### Pipeline Phases

1. **Lexing**: Text → Tokens (logos crate)
2. **Parsing**: Tokens → AST (recursive descent parser)
3. **HIR Lowering**: AST → HIR (type checking, validation)
4. **MLIR Lowering**: HIR → MLIR (type lowering, operation generation) **← Current Layer**
5. **Optimization**: MLIR → Optimized MLIR (MLIR passes)
6. **Target Generation**: MLIR → Target Code (WASM, LLVM, etc.)

## Implementation Details

### Module Structure

```
src/mlir/
├── mod.rs          # Module exports, MlirError type
├── types.rs        # TypeLowering - DOL types → MLIR types
├── ops.rs          # OpBuilder - DOL operations → MLIR operations
├── context.rs      # MLIR context wrapper
├── lowering.rs     # HIR → MLIR lowering pass (future)
└── codegen.rs      # MLIR → target code generation (future)
```

### Feature Flags

The MLIR functionality requires the `mlir` feature flag:

```toml
[dependencies]
dol = { version = "0.4.0", features = ["mlir"] }
```

This ensures the melior crate (MLIR bindings) is only compiled when needed.

### Dependencies

- **melior**: Rust bindings to MLIR C API
  - Provides: Context, Operation, Type, Location, Dialect
  - Dialects used: `arith`, `scf`, `func`

### Error Handling

MLIR operations return `Result<Operation, MlirError>` for operations that may fail:

```rust
pub struct MlirError {
    pub message: String,
    pub span: Option<Span>,
}
```

Common error scenarios:
- Unsupported type (Type::Unknown, Type::Error)
- Unresolved type variable (Type::Var)
- Unsupported operation (power, functional operators)
- Missing MLIR feature flag

## Future Enhancements

### Planned Features

1. **Custom DOL Dialect**
   - Gene operations: `dol.gene`, `dol.field_get`, `dol.field_set`
   - Trait operations: `dol.trait`, `dol.method_call`
   - Constraint operations: `dol.constraint`, `dol.validate`

2. **Advanced Type Lowering**
   - List<T> → `memref<?xT>` (dynamic memory reference)
   - String → `!llvm.ptr` (LLVM dialect pointer)
   - Gene types → struct types with metadata

3. **Optimization Passes**
   - DOL-specific optimizations
   - Constraint folding
   - Gene inlining

4. **Source Maps**
   - Preserve DOL source locations through MLIR
   - Enable debugger integration
   - Improve error messages

5. **Multiple Targets**
   - WebAssembly (current focus)
   - LLVM IR (native code)
   - GPU kernels (CUDA, ROCm via MLIR)
   - Specialized hardware (MLIR backends)

## References

- [MLIR Documentation](https://mlir.llvm.org/)
- [MLIR Dialects](https://mlir.llvm.org/docs/Dialects/)
- [melior Crate](https://docs.rs/melior/)
- [DOL HIR Specification](./HIR-SPECIFICATION.md)
- [DOL Language Specification](./specification.md)

## Version History

- **v0.4.0** (Current): Initial MLIR integration
  - TypeLowering for primitive and compound types
  - OpBuilder for arithmetic, comparison, logical operations
  - Control flow (if, for, while)
  - Function operations (func, call, return)

## License

This specification is part of the DOL project and follows the same license terms.
