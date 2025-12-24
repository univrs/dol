# DOL 2.0: Next Steps for Claude-Flow Development

> **Document Version:** 1.0.0  
> **Created:** December 23, 2025  
> **Purpose:** Actionable development roadmap for advancing DOL toward self-hosting  
> **Target:** Claude-flow orchestrated development

---

## Executive Summary

DOL (Design Ontology Language) has achieved significant progress through Phase 1-3 of Year 1. The compiler toolchain now includes a working lexer, parser, type checker, and Rust code generator with 367 passing tests. The immediate focus must shift to **meta-programming capabilities (Q2)** to unlock DOL's potential as a self-referential, biologically-inspired modeling language.

### Current State Assessment

| Component | Status | Tests | Notes |
|-----------|--------|-------|-------|
| Lexer (logos) | ✅ Complete | 80+ | All DOL 2.0 tokens recognized |
| Parser (Pratt) | ✅ Complete | 150+ | Full expression precedence |
| Type Checker | ✅ Complete | 87 | Bidirectional inference |
| Rust Codegen | ✅ Complete | 8 | Genes → structs, traits → traits |
| **Meta-Programming** | 🚧 **Next** | 0 | Quote, Eval, Macro, Reflect |
| MLIR Backend | ⏳ Q3 | — | WASM compilation target |
| Self-Hosting | ⏳ Q4 | — | DOL compiles DOL |

---

## Part 1: Immediate Priority — Meta-Programming (Q2)

The meta-programming system is the gateway to DOL's self-referential nature. Without it, DOL cannot describe or compile itself.

### 1.1 Quote Operator (`'`)

**Purpose:** Capture code as Abstract Syntax Tree (AST) data without evaluation.

**Syntax:**
```dol
// Quote a simple expression
expr = '(1 + 2 * 3)           // Type: Quoted<Expr>

// Quote a block
block = '{
  x = 10
  y = x * 2
  return y
}                              // Type: Quoted<Block>

// Quasi-quoting with splice ($)
template = '{ result = $inner + 1 }
expanded = template.splice({ inner: '(x * 2) })
// Result: '{ result = (x * 2) + 1 }
```

**Implementation Tasks:**

| Task | File | Description |
|------|------|-------------|
| 1.1.1 | `src/lexer.rs` | Add `Quote` token for `'` operator |
| 1.1.2 | `src/ast.rs` | Add `Expr::Quoted(Box<Expr>)` variant |
| 1.1.3 | `src/parser.rs` | Parse quote as prefix operator |
| 1.1.4 | `src/typechecker.rs` | Infer `Quoted<T>` wrapper type |
| 1.1.5 | `src/codegen/rust.rs` | Generate AST representation struct |
| 1.1.6 | `tests/quote_tests.rs` | Minimum 20 test cases |

**Type Rules:**
```
Γ ⊢ e : T
─────────────────
Γ ⊢ 'e : Quoted<T>
```

---

### 1.2 Eval Operator (`!`)

**Purpose:** Execute a quoted expression in the current context.

**Syntax:**
```dol
// Basic eval
quoted = '(3 + 4)
result = !quoted              // result: Int64 = 7

// Eval with bindings
template = '(a + b)
result = !template where { a = 10, b = 20 }  // result: 30

// Dynamic dispatch
ops = ['add, 'subtract, 'multiply]
for op in ops {
  result = !(op)(x, y)        // Calls each operation
}
```

**Implementation Tasks:**

| Task | File | Description |
|------|------|-------------|
| 1.2.1 | `src/lexer.rs` | Add `Eval` token for `!` prefix |
| 1.2.2 | `src/ast.rs` | Add `Expr::Eval(Box<Expr>, Option<Bindings>)` |
| 1.2.3 | `src/parser.rs` | Parse eval with optional `where` clause |
| 1.2.4 | `src/interpreter.rs` | **New file**: AST interpreter for eval |
| 1.2.5 | `src/typechecker.rs` | Unwrap `Quoted<T>` → `T` |
| 1.2.6 | `tests/eval_tests.rs` | Minimum 25 test cases |

**Type Rules:**
```
Γ ⊢ e : Quoted<T>
─────────────────
Γ ⊢ !e : T
```

---

### 1.3 Macro System (`#`)

**Purpose:** Compile-time code generation and transformation.

**Syntax:**
```dol
// Macro definition
macro unless(condition: Expr, body: Block) -> Expr {
  return '{ if not !condition { !body } }
}

// Macro invocation
#unless(list.is_empty()) {
  process(list)
}

// Built-in macros
#stringify(expr)              // → String literal
#concat("hello", "_", "world") // → "hello_world"
#cfg(target.wasm) { ... }     // Conditional compilation
#derive(Debug, Clone)         // Trait generation
#assert(x > 0)                // Runtime assertion
#format("Hello, {}!", name)   // String formatting
```

**20 Built-in Macros to Implement:**

| Macro | Category | Description |
|-------|----------|-------------|
| `#stringify(expr)` | Conversion | Expression to string literal |
| `#concat(a, b, ...)` | String | Concatenate string literals |
| `#env("VAR")` | Environment | Compile-time env variable |
| `#option_env("VAR")` | Environment | Optional env variable |
| `#cfg(condition)` | Conditional | Conditional compilation |
| `#derive(Trait, ...)` | Generation | Auto-implement traits |
| `#assert(cond)` | Debug | Runtime assertion |
| `#assert_eq(a, b)` | Debug | Assert equality |
| `#assert_ne(a, b)` | Debug | Assert inequality |
| `#format(fmt, ...)` | String | String formatting |
| `#dbg(expr)` | Debug | Debug print (returns value) |
| `#todo(msg)` | Placeholder | Mark unimplemented |
| `#unreachable()` | Control | Mark unreachable code |
| `#compile_error(msg)` | Error | Compile-time error |
| `#vec(a, b, c)` | Collection | Create vector |
| `#file()` | Location | Current file name |
| `#line()` | Location | Current line number |
| `#column()` | Location | Current column |
| `#module_path()` | Location | Current module path |
| `#include(path)` | File | Include file contents |

**Implementation Tasks:**

| Task | File | Description |
|------|------|-------------|
| 1.3.1 | `src/lexer.rs` | Add `MacroInvoke` token for `#` |
| 1.3.2 | `src/ast.rs` | Add `Decl::Macro` and `Expr::MacroCall` |
| 1.3.3 | `src/parser.rs` | Parse macro definitions and invocations |
| 1.3.4 | `src/macros/mod.rs` | **New module**: Macro expansion engine |
| 1.3.5 | `src/macros/builtin.rs` | 20 built-in macro implementations |
| 1.3.6 | `src/macros/hygiene.rs` | Hygienic macro scoping |
| 1.3.7 | `tests/macro_tests.rs` | Minimum 50 test cases |

---

### 1.4 Reflect Operator (`?`)

**Purpose:** Runtime type introspection and discovery.

**Syntax:**
```dol
// Get type information
info = ?Container
// info.name == "Container"
// info.fields == [{ name: "id", type: "UInt64" }, ...]
// info.constraints == ["valid_id"]

// Check trait implementation
if ?Task.implements(Schedulable) {
  schedule(task)
}

// Runtime type matching
match ?value {
  Int64 { handle_int(value) }
  String { handle_string(value) }
  Gene<T> { handle_gene(value) }
  _ { handle_unknown(value) }
}

// Field enumeration
for field in ?MyGene.fields {
  println(field.name + ": " + field.type_name)
}
```

**Implementation Tasks:**

| Task | File | Description |
|------|------|-------------|
| 1.4.1 | `src/lexer.rs` | Add `Reflect` token for `?` |
| 1.4.2 | `src/ast.rs` | Add `Expr::Reflect(Box<Expr>)` |
| 1.4.3 | `src/runtime/typeinfo.rs` | **New**: TypeInfo struct |
| 1.4.4 | `src/typechecker.rs` | Generate TypeInfo at compile time |
| 1.4.5 | `src/codegen/rust.rs` | Emit TypeInfo structs |
| 1.4.6 | `tests/reflect_tests.rs` | Minimum 30 test cases |

**TypeInfo Structure:**
```dol
gene TypeInfo {
  has name: String
  has module_path: String
  has fields: List<FieldInfo>
  has methods: List<MethodInfo>
  has constraints: List<String>
  has traits: List<String>
  has exegesis: String
}

gene FieldInfo {
  has name: String
  has type_name: String
  has offset: UInt64
  has is_public: Bool
}
```

---

## Part 2: Biological Design Patterns

DOL's vision is to model all Systems of Life. This requires embedding biological patterns into the language's core abstractions.

### 2.1 Mycelial Network Primitives

DOL should natively support patterns found in mycelium networks:

**Hyphal Growth Pattern:**
```dol
trait Hyphal {
  // Extend toward resources
  is extend(direction: Vec3, gradient: Nutrient) -> Self
  
  // Anastomosis: fusion of hyphae
  is fuse(other: Self) -> Option<Self>
  
  // Branching at decision points
  is branch(factor: Float64) -> List<Self>
  
  law conservation_of_mass {
    // Total biomass is conserved during branching
    sum(branches.map(|b| b.mass)) == self.mass
  }
  
  exegesis {
    Hyphal growth follows nutrient gradients while
    maintaining mass conservation. Fusion (anastomosis)
    enables network formation.
  }
}
```

**Nutrient Transport Pattern:**
```dol
gene Nutrient {
  has carbon: Float64
  has nitrogen: Float64
  has phosphorus: Float64
  has water: Float64
  
  constraint stoichiometry {
    // Redfield ratio for biological systems
    this.carbon / this.nitrogen >= 6.0 &&
    this.carbon / this.nitrogen <= 10.0
  }
}

trait Transport<T> {
  is source_to_sink(
    source: Node,
    sink: Node,
    amount: T
  ) -> Result<Flow<T>, TransportError>
  
  law mass_balance {
    // What leaves source arrives at sink
    source.outflow == sink.inflow
  }
}
```

### 2.2 Evolution as First-Class Concept

DOL's `evolves` keyword should capture biological evolution patterns:

```dol
// Speciation through divergence
evolves Organism > Prokaryote @ 3.5Gya {
  removed nucleus
  constraint circular_genome
}

evolves Organism > Eukaryote @ 2.0Gya {
  added nucleus: Organelle
  added mitochondria: Organelle  // Endosymbiosis
  
  migrate from Prokaryote {
    return Eukaryote {
      genes: old.genes,
      nucleus: Nucleus.from_genes(old.genes),
      mitochondria: Mitochondria.default()
    }
  }
  
  exegesis {
    Eukaryotes emerged through endosymbiosis,
    incorporating mitochondria as organelles.
    This evolution is irreversible in practice.
  }
}
```

### 2.3 Constraint Systems as Ecological Laws

```dol
system Ecosystem {
  uses Predator, Prey, Producer
  
  state populations: Map<Species, UInt64>
  state carrying_capacity: UInt64
  
  // Lotka-Volterra dynamics
  constraint predator_prey_balance {
    for species in populations.keys() {
      match species.role {
        Predator { 
          // Predator growth depends on prey availability
          species.growth_rate <= prey_abundance * hunt_efficiency
        }
        Prey {
          // Prey decline depends on predator pressure
          species.decline_rate <= predator_abundance * vulnerability
        }
      }
    }
  }
  
  // Carrying capacity limit
  constraint population_limit {
    sum(populations.values()) <= carrying_capacity
  }
  
  // Energy flow (10% rule)
  constraint trophic_efficiency {
    for level in 1..max_trophic_level {
      energy_at(level + 1) <= energy_at(level) * 0.10
    }
  }
  
  exegesis {
    Ecosystem constraints encode fundamental ecological laws:
    - Predator-prey dynamics (Lotka-Volterra)
    - Carrying capacity limits
    - Trophic energy transfer (~10% efficiency)
  }
}
```

---

## Part 3: Self-Hosting Architecture (Q4 Target)

For DOL to compile itself, it needs these components implemented in DOL:

### 3.1 DOL Compiler in DOL

**Directory Structure:**
```
dol/
├── Spirit.dol              # Package manifest
├── src/
│   ├── lib.dol            # Re-exports
│   ├── lexer.dol          # Token definitions
│   ├── parser.dol         # Recursive descent + Pratt
│   ├── ast.dol            # AST node definitions
│   ├── types.dol          # Type system
│   ├── typechecker.dol    # Semantic analysis
│   ├── ir.dol             # HIR/MLIR emission
│   └── emit/
│       ├── rust.dol       # Rust codegen
│       ├── wasm.dol       # WASM codegen
│       └── typescript.dol # TypeScript codegen
└── tests/
    └── bootstrap.dol      # Self-compilation tests
```

### 3.2 Bootstrap Sequence

```bash
# Stage 0: Rust compiler (existing)
cargo build --release
./target/release/dol compile dol/src/lib.dol -o stage1.wasm

# Stage 1: DOL compiler compiled by Rust
wasmtime stage1.wasm -- dol/src/lib.dol -o stage2.wasm

# Stage 2: DOL compiler compiled by Stage 1
wasmtime stage2.wasm -- dol/src/lib.dol -o stage3.wasm

# Verification: Stage 2 and Stage 3 must be identical
diff stage2.wasm stage3.wasm  # Empty = success!
```

### 3.3 Minimum Viable Self-Host

The DOL-in-DOL compiler needs at minimum:

| Module | Lines (Est.) | Dependencies |
|--------|--------------|--------------|
| `lexer.dol` | 400 | Standard library |
| `ast.dol` | 600 | Type definitions |
| `parser.dol` | 1200 | Lexer, AST |
| `typechecker.dol` | 800 | AST, Types |
| `emit/wasm.dol` | 1000 | TypedAST |
| **Total** | ~4000 | |

---

## Part 4: WASM Backend (Q3)

### 4.1 MLIR Integration

DOL should lower to MLIR before WASM for optimization opportunities:

```
DOL Source → DOL-IR (HIR) → MLIR Dialects → WASM
                 ↓
         dol.gene, dol.trait, dol.constraint
                 ↓
         mlir.func, mlir.arith, mlir.memref
                 ↓
         wasm.module, wasm.func, wasm.memory
```

### 4.2 DOL-Specific MLIR Dialect

```mlir
// DOL Gene → MLIR
dol.gene @ProcessId : !dol.gene<i64> {
  dol.constraint @positive {
    %value = dol.extract_value %self : i64
    %zero = arith.constant 0 : i64
    %valid = arith.cmpi sgt, %value, %zero : i64
    dol.yield %valid : i1
  }
}

// DOL Trait → MLIR
dol.trait @Schedulable {
  dol.method @priority : (!dol.self) -> i32
  dol.method @duration : (!dol.self) -> i64
}
```

---

## Part 5: Implementation Order for Claude-Flow

### Phase A: Meta-Programming Foundation (Weeks 1-6)

```
Week 1-2: Quote Operator
├── Lexer: Add Quote token
├── Parser: Parse quoted expressions
├── AST: Quoted<T> wrapper type
└── Tests: 20 cases

Week 3-4: Eval Operator
├── Interpreter module (new)
├── Parser: Eval with bindings
├── Type unwrapping logic
└── Tests: 25 cases

Week 5-6: Integration
├── Quote/Eval interop
├── Quasi-quoting with splice
├── Error handling
└── Tests: Combined scenarios
```

### Phase B: Macro System (Weeks 7-12)

```
Week 7-8: Macro Infrastructure
├── Macro definition parsing
├── Expansion engine
├── Hygiene system
└── Tests: Core macro mechanics

Week 9-10: Built-in Macros (Set 1)
├── #stringify, #concat, #format
├── #assert, #assert_eq, #dbg
├── #file, #line, #column
└── Tests: 10 macros

Week 11-12: Built-in Macros (Set 2)
├── #cfg, #derive
├── #env, #option_env
├── #todo, #unreachable
├── #compile_error, #vec, #include
└── Tests: 10 macros
```

### Phase C: Reflection (Weeks 13-16)

```
Week 13-14: TypeInfo Generation
├── Reflect operator parsing
├── TypeInfo struct definition
├── Compile-time info generation
└── Tests: Basic reflection

Week 15-16: Runtime Reflection
├── Dynamic type matching
├── Field enumeration
├── Trait checking
└── Tests: Advanced reflection
```

---

## Part 6: Test Coverage Requirements

Each component requires minimum test coverage:

| Component | Min Tests | Coverage Target |
|-----------|-----------|-----------------|
| Quote | 20 | 90% |
| Eval | 25 | 90% |
| Macros (each) | 5 | 85% |
| Reflect | 30 | 90% |
| Integration | 50 | 80% |
| **Total New** | 175+ | 85% overall |

### Test Categories

1. **Happy Path**: Normal usage
2. **Error Cases**: Invalid syntax, type mismatches
3. **Edge Cases**: Empty inputs, nested structures
4. **Integration**: Multi-feature combinations
5. **Regression**: Previously fixed bugs

---

## Part 7: Success Criteria

### Q2 Completion Checklist

- [ ] Quote operator parses and type-checks
- [ ] Eval operator executes quoted expressions
- [ ] All 20 built-in macros implemented
- [ ] Reflect operator returns TypeInfo
- [ ] 175+ new tests passing
- [ ] Documentation updated
- [ ] Integration tests cover meta-programming

### Q3 Completion Checklist

- [ ] DOL-IR defined and implemented
- [ ] MLIR lowering works
- [ ] WASM output runs in wasmtime
- [ ] MCP server exposes compile tools
- [ ] End-to-end: .dol → .wasm → execution

### Q4 Completion Checklist

- [ ] DOL compiler written in DOL
- [ ] Bootstrap succeeds (Stage 2 = Stage 3)
- [ ] Self-compilation under 60 seconds
- [ ] All 367+ original tests pass
- [ ] Documentation self-hosted

---

## Part 8: Biological Integration Roadmap

### Immediate (Q2)

- Add `evolves` chain visualization
- Implement `Hyphal` trait in stdlib
- Add `Nutrient` transport primitives

### Near-term (Q3-Q4)

- Mycelium network simulation DSL
- Ecosystem constraint solver
- Evolutionary algorithm integration

### Long-term (Year 2+)

- Living systems modeling toolkit
- Regenerative ecology patterns library
- Cross-species ontology mapping

---

## Appendix A: File Templates

### New Test File Template

```rust
// tests/{feature}_tests.rs

use metadol::{parse, typecheck, codegen};

#[test]
fn test_{feature}_basic() {
    let source = r#"
        // DOL source here
    "#;
    let ast = parse(source).expect("parse failed");
    let typed = typecheck(&ast).expect("typecheck failed");
    // Assertions
}

#[test]
fn test_{feature}_error_case() {
    let source = r#"..."#;
    let result = parse(source);
    assert!(result.is_err());
    // Check error message
}
```

### New Module Template

```rust
// src/{module}/mod.rs

//! {Module} for DOL 2.0
//! 
//! This module handles {description}.

mod types;
mod implementation;

pub use types::*;
pub use implementation::*;

#[cfg(test)]
mod tests;
```

---

## Appendix B: Dependencies

### Required Crates

```toml
[dependencies]
# Existing
logos = "0.13"
chumsky = "0.9"

# For Meta-Programming
proc-macro2 = "1.0"  # AST representation
syn = "2.0"          # Rust AST parsing (for derive)
quote = "1.0"        # Code generation

# For WASM
wasmtime = "15.0"    # WASM runtime
wasm-encoder = "0.38" # WASM emission

# For MLIR (Q3)
melior = "0.15"      # MLIR bindings
```

---

*"From ontology to self-awareness, from specification to life."*

— DOL Development Team
