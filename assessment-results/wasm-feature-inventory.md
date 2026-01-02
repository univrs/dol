# WASM Feature Inventory

## Assessment Date: 2026-01-01
## Auditor: WASM Deep Auditor

---

## Executive Summary

The WASM backend for DOL/VUDO is **operational and passing all tests**. The compiler directly emits WASM bytecode via `wasm-encoder`, bypassing the HIR/MLIR pipeline for faster compilation.

- **Total WASM Tests**: 61 passing (34 unit + 22 integration + 5 helper tests)
- **Inheritance Tests**: 4 passing
- **Test Coverage**: Comprehensive for implemented features

---

## 1. AST Nodes Handled in compiler.rs

The WASM compiler works directly with AST nodes (not HIR). Here is the complete inventory:

### Declarations

| AST Node | Status | File:Line | Notes |
|----------|--------|-----------|-------|
| `Declaration::Function` | IMPLEMENTED | compiler.rs:866 | Top-level functions fully supported |
| `Declaration::Gene` | IMPLEMENTED | compiler.rs:871-920 | Gene methods extracted and compiled |
| `Declaration::Trait` | NOT_IMPLEMENTED | compiler.rs:915-920 | Returns empty vec, no WASM output |
| `Declaration::System` | NOT_IMPLEMENTED | compiler.rs:915-920 | Returns empty vec, no WASM output |
| `Declaration::Constraint` | NOT_IMPLEMENTED | N/A | Not handled |
| `Declaration::Evolution` | NOT_IMPLEMENTED | N/A | Not handled |

### Statements (Stmt)

| AST Node | Status | File:Line | Notes |
|----------|--------|-----------|-------|
| `Stmt::Return(Some(expr))` | IMPLEMENTED | compiler.rs:1159-1163 | Emit expr + return instruction |
| `Stmt::Return(None)` | IMPLEMENTED | compiler.rs:1129-1131 | Void return |
| `Stmt::Expr` | IMPLEMENTED | compiler.rs:1165-1171 | Expression statement with drop |
| `Stmt::Let` | IMPLEMENTED | compiler.rs:1173-1187 | Local variable declaration |
| `Stmt::Assign` | IMPLEMENTED | compiler.rs:1188-1215 | Variable and identifier assignment |
| `Stmt::While` | IMPLEMENTED | compiler.rs:1216-1243 | While loop with block/loop/br |
| `Stmt::For` | IMPLEMENTED | compiler.rs:1244-1317 | For loops with range iteration |
| `Stmt::Loop` | IMPLEMENTED | compiler.rs:1319-1339 | Infinite loop with break support |
| `Stmt::Break` | IMPLEMENTED | compiler.rs:1340-1346 | Break with depth tracking |
| `Stmt::Continue` | IMPLEMENTED | compiler.rs:1347-1353 | Continue with depth tracking |
| `Stmt::Assign(Member)` | NOT_IMPLEMENTED | compiler.rs:1203-1206 | Member assignment error (Phase 3) |

### Expressions (Expr)

| AST Node | Status | File:Line | Notes |
|----------|--------|-----------|-------|
| `Expr::Literal(Int)` | IMPLEMENTED | compiler.rs:1372-1374 | i64.const |
| `Expr::Literal(Float)` | IMPLEMENTED | compiler.rs:1375-1377 | f64.const |
| `Expr::Literal(Bool)` | IMPLEMENTED | compiler.rs:1378-1380 | i32.const (0/1) |
| `Expr::Literal(String)` | NOT_IMPLEMENTED | compiler.rs:1381-1385 | Error: not supported |
| `Expr::Literal(Char)` | NOT_IMPLEMENTED | compiler.rs:1386-1390 | Error: not supported |
| `Expr::Literal(Null)` | NOT_IMPLEMENTED | compiler.rs:1391-1395 | Error: not supported |
| `Expr::Identifier` | IMPLEMENTED | compiler.rs:1397-1553 | Full support including dotted names |
| `Expr::Binary` | IMPLEMENTED | compiler.rs:1555-1562 | All arithmetic/comparison ops |
| `Expr::Call` | IMPLEMENTED | compiler.rs:1563-1580 | Direct function calls |
| `Expr::If` | IMPLEMENTED | compiler.rs:1581-1613 | if/else with block type |
| `Expr::Block` | IMPLEMENTED | compiler.rs:1614-1629 | Block expressions |
| `Expr::Match` | IMPLEMENTED | compiler.rs:1630-1667 | Pattern matching via nested if-else |
| `Expr::Member` | IMPLEMENTED | compiler.rs:1673-1749 | Field access with type inference |
| `Expr::Unary(Neg)` | IMPLEMENTED | compiler.rs:1754-1759 | 0 - value for negation |
| `Expr::Unary(Not)` | IMPLEMENTED | compiler.rs:1760-1764 | i64.eqz for boolean not |
| `Expr::StructLiteral` | IMPLEMENTED | compiler.rs:1818-1900 | Struct allocation and field init |
| `Expr::Lambda` | NOT_IMPLEMENTED | compiler.rs:1668-1672 | Error: not supported |
| `Expr::List/Tuple` | NOT_IMPLEMENTED | compiler.rs:1773-1777 | Error: not supported |
| `Expr::Forall/Exists` | NOT_IMPLEMENTED | compiler.rs:1778-1782 | Error: quantifiers not supported |
| `Expr::Quote/Unquote/Reflect` | NOT_IMPLEMENTED | compiler.rs:1783-1787 | Error: metaprogramming |
| `Expr::SexBlock` | NOT_IMPLEMENTED | compiler.rs:1788-1792 | Error: not supported |
| `Expr::Cast` | NOT_IMPLEMENTED | compiler.rs:1793-1796 | Error: type casts |
| `Expr::Try` | NOT_IMPLEMENTED | compiler.rs:1797-1801 | Error: try expressions |
| `Expr::QuasiQuote/Eval` | NOT_IMPLEMENTED | compiler.rs:1803-1807 | Error: not supported |
| `Expr::IdiomBracket` | NOT_IMPLEMENTED | compiler.rs:1808-1812 | Error: not supported |
| `Expr::Implies` | NOT_IMPLEMENTED | compiler.rs:1813-1816 | Error: not supported |

### Pattern Matching

| Pattern | Status | File:Line | Notes |
|---------|--------|-----------|-------|
| `Pattern::Literal(Int)` | IMPLEMENTED | compiler.rs:1957-1966 | i64 comparison |
| `Pattern::Literal(Bool)` | IMPLEMENTED | compiler.rs:1967-1970 | 0/1 comparison |
| `Pattern::Wildcard` | IMPLEMENTED | compiler.rs:1936-1952 | Fallback else case |
| `Pattern::Identifier` | IMPLEMENTED | compiler.rs:2017-2021 | Treated like wildcard |
| `Pattern::Literal(other)` | NOT_IMPLEMENTED | compiler.rs:1971-1976 | Error for other literals |
| `Pattern::Struct/Enum/Tuple` | NOT_IMPLEMENTED | compiler.rs:2022-2028 | Error: unsupported |

---

## 2. WASM Opcodes Emitted

### Constants and Locals

| Opcode | Status | Used For |
|--------|--------|----------|
| `i64.const` | EMITTED | Integer literals |
| `f64.const` | EMITTED | Float literals |
| `i32.const` | EMITTED | Boolean literals, alloc params |
| `local.get` | EMITTED | Variable/parameter access |
| `local.set` | EMITTED | Variable assignment |
| `local.tee` | EMITTED | Struct pointer handling |
| `global.get` | EMITTED | Allocator heap_base/heap_end |
| `global.set` | EMITTED | Allocator heap update |

### Arithmetic (i64)

| Opcode | Status | Binary Op |
|--------|--------|-----------|
| `i64.add` | EMITTED | + |
| `i64.sub` | EMITTED | - |
| `i64.mul` | EMITTED | * |
| `i64.div_s` | EMITTED | / |
| `i64.rem_s` | EMITTED | % |

### Comparison (i64)

| Opcode | Status | Binary Op |
|--------|--------|-----------|
| `i64.eq` | EMITTED | == |
| `i64.ne` | EMITTED | != |
| `i64.lt_s` | EMITTED | < |
| `i64.le_s` | EMITTED | <= |
| `i64.gt_s` | EMITTED | > |
| `i64.ge_s` | EMITTED | >= |

### Logical

| Opcode | Status | Used For |
|--------|--------|----------|
| `i64.and` | EMITTED | && |
| `i64.or` | EMITTED | \|\| |
| `i64.eqz` | EMITTED | ! (boolean not) |
| `i32.eqz` | EMITTED | Condition inversion |

### Control Flow

| Opcode | Status | Used For |
|--------|--------|----------|
| `if` | EMITTED | Conditionals |
| `else` | EMITTED | Else branches |
| `block` | EMITTED | Loop break targets |
| `loop` | EMITTED | Loop headers |
| `br` | EMITTED | Break/continue |
| `br_if` | EMITTED | Conditional break |
| `end` | EMITTED | Block/function end |
| `call` | EMITTED | Function calls |
| `return` | EMITTED | Function return |
| `unreachable` | EMITTED | Non-exhaustive match |
| `drop` | EMITTED | Discard expr values |

### Memory

| Opcode | Status | Used For |
|--------|--------|----------|
| `i64.load` | EMITTED | Field access (i64 fields) |
| `i64.store` | EMITTED | Field init (i64 fields) |
| `f64.load` | EMITTED | Field access (f64 fields) |
| `f64.store` | EMITTED | Field init (f64 fields) |
| `i32.load` | EMITTED | Field access (i32/ptr fields) |
| `i32.store` | EMITTED | Field init (i32/ptr fields) |
| `f32.load` | EMITTED | Field access (f32 fields) |
| `f32.store` | EMITTED | Field init (f32 fields) |

### Allocator (i32)

| Opcode | Status | Used For |
|--------|--------|----------|
| `i32.add` | EMITTED | Pointer arithmetic |
| `i32.sub` | EMITTED | Alignment calc |
| `i32.and` | EMITTED | Alignment mask |
| `i32.xor` | EMITTED | Alignment inversion |
| `i32.gt_u` | EMITTED | Heap boundary check |

---

## 3. Features from test-plan.md vs Implementation

| Feature | Test Plan Status | Implementation Status | Notes |
|---------|------------------|----------------------|-------|
| Module declaration | PASS | IMPLEMENTED | |
| Function declaration | PASS | IMPLEMENTED | |
| i64 parameters | PASS | IMPLEMENTED | |
| i64 return type | PASS | IMPLEMENTED | |
| Binary operators | PASS | IMPLEMENTED | +,-,*,/,% |
| Comparison operators | PASS | IMPLEMENTED | >,<,==,!= |
| Return statements | PASS | IMPLEMENTED | |
| Integer literals | PASS | IMPLEMENTED | |
| Float literals | PASS | IMPLEMENTED | |
| Local variables | PASS | IMPLEMENTED | via LocalsTable |
| Variable reassignment | PASS | IMPLEMENTED | |
| If statements | PASS | IMPLEMENTED | |
| If-else | PASS | IMPLEMENTED | |
| While loops | PASS | IMPLEMENTED | |
| For loops | PASS | IMPLEMENTED | Range only |
| Break/continue | PASS | IMPLEMENTED | With LoopContext depth |
| Nested control flow | PASS | IMPLEMENTED | |
| Gene methods | PASS | IMPLEMENTED | |
| Gene field access | PASS | IMPLEMENTED | |
| Implicit self | PASS | IMPLEMENTED | via GeneContext |
| Gene inheritance | PASS | IMPLEMENTED | Topological ordering |
| Pattern matching | PASS | IMPLEMENTED | Literals + wildcard |
| Complex gene layouts | TODO | PARTIAL | Nested genes limited |
| Match expressions (complex) | TODO | PARTIAL | No struct patterns |
| Trait definitions | TODO | NOT_IMPLEMENTED | Parses only |
| Trait implementations | TODO | NOT_IMPLEMENTED | No vtables |
| System declarations | TODO | NOT_IMPLEMENTED | |

---

## 4. Missing Features (Based on test-plan.md)

| Feature | Priority | Difficulty | Notes |
|---------|----------|------------|-------|
| Nested gene layouts | MEDIUM | MEDIUM | Requires recursive layout |
| Trait vtables | LOW | HIGH | Dynamic dispatch |
| String literals | MEDIUM | MEDIUM | Memory allocation |
| Char literals | LOW | LOW | Just i32 conversion |
| Lambda/closures | LOW | HIGH | Capture handling |
| Complex match patterns | LOW | MEDIUM | Struct/enum patterns |
| Member assignment | MEDIUM | MEDIUM | Store to field offset |
| Type casts | LOW | LOW | Widening/narrowing |

---

## 5. File Locations

| Component | File | Lines |
|-----------|------|-------|
| WASM Module | `/home/ardeshir/repos/univrs-dol/src/wasm/mod.rs` | 1-157 |
| WASM Compiler | `/home/ardeshir/repos/univrs-dol/src/wasm/compiler.rs` | 1-2308 |
| Gene Layout | `/home/ardeshir/repos/univrs-dol/src/wasm/layout.rs` | 1-1090 |
| Bump Allocator | `/home/ardeshir/repos/univrs-dol/src/wasm/alloc.rs` | 1-507 |
| WASM Runtime | `/home/ardeshir/repos/univrs-dol/src/wasm/runtime.rs` | 1-268 |
| Test Plan | `/home/ardeshir/repos/univrs-dol/test-plan.md` | 1-339 |
| WASM Tests | `/home/ardeshir/repos/univrs-dol/tests/wasm_execution.rs` | - |
| Test Helpers | `/home/ardeshir/repos/univrs-dol/tests/wasm_test_helpers.rs` | - |

---

## 6. Test Summary

### WASM Unit Tests (34 passing)
- `wasm::alloc::tests::*` - 10 tests (allocator functionality)
- `wasm::compiler::tests::*` - 8 tests (compiler configuration)
- `wasm::layout::tests::*` - 13 tests (gene layout computation)
- `wasm::runtime::tests::*` - 3 tests (runtime loading)

### WASM Integration Tests (22 passing)
- `compiler_e2e::*` - 2 tests (end-to-end compilation)
- `wasm_debug::*` - 2 tests (control flow and match)
- `wasm_execution::*` - 18 tests (execution and validation)
- `wasm_test_helpers::*` - 5 tests (helper functions)

### Inheritance Tests (4 passing)
- `reflect::tests::test_type_info_inheritance`
- `wasm::layout::tests::test_gene_inheritance_layout`
- `wasm::layout::tests::test_gene_inheritance_unknown_parent`
- `wasm_execution::test_compile_gene_inheritance_*` - 3 tests

---

## 7. Architecture Notes

### Compilation Pipeline
```
DOL Source -> Parser -> AST -> WasmCompiler -> WASM Bytecode -> Wasmtime
                                   |
                                   v
                          GeneLayoutRegistry (field offsets)
                                   |
                                   v
                          BumpAllocator (heap management)
```

### Key Design Decisions
1. **Direct WASM emission**: Bypasses HIR/MLIR for simplicity
2. **Bump allocator**: No GC, suitable for short-lived computations
3. **C-like struct layout**: Fields aligned, struct padded to alignment
4. **Implicit self**: Gene methods receive self as first parameter
5. **Topological ordering**: Parent genes registered before children

---

## 8. Recommendations

### High Priority
1. Implement member assignment for mutable struct fields
2. Add string literal support (with memory allocation)

### Medium Priority
3. Support nested gene layouts fully
4. Add more complex match patterns (structs, enums)

### Low Priority
5. Implement trait vtables for polymorphism
6. Add system declaration compilation
7. Lambda/closure support

---

*Generated by WASM Deep Auditor - 2026-01-01*
