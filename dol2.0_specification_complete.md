# DOL 2.0 Complete Language Specification

> **Version:** 2.1.0  
> **Status:** Living Document  
> **Updated:** December 23, 2025

---

## Table of Contents

1. [Language Overview](#language-overview)
2. [Complete Keyword Reference](#complete-keyword-reference)
3. [Operators Reference](#operators-reference)
4. [Type System](#type-system)
5. [SEX System](#sex-system)
6. [Meta-Programming](#meta-programming)
7. [Biological Modeling](#biological-modeling)
8. [Grammar Summary](#grammar-summary)

---

## Language Overview

DOL (Design Ontology Language) is a Turing-complete functional programming language designed for ontology-first development. Systems describe *what they are* before *what they do*.

### Core Philosophy

- **Ontology First**: Specification before implementation
- **Pure by Default**: Side effects require explicit `sex` marking
- **Private by Default**: Visibility requires explicit `pub` marking
- **Self-Documenting**: Every construct includes `exegesis`
- **Multi-Target**: One source compiles to Rust, TypeScript, WASM

---

## Complete Keyword Reference

### Ontological Keywords

| Keyword | Category | Description | Example |
|---------|----------|-------------|---------|
| `gene` | Declaration | Atomic unit of truth | `gene ProcessId { ... }` |
| `trait` | Declaration | Composable behavior | `trait Runnable { ... }` |
| `system` | Declaration | Module composition | `system Scheduler { ... }` |
| `constraint` | Declaration | Validation invariant | `constraint positive { ... }` |
| `evolves` | Declaration | Version migration | `evolves V1 > V2 @ 2.0 { ... }` |
| `exegesis` | Documentation | Self-documentation | `exegesis { Description. }` |
| `law` | Contract | Trait law/invariant | `law identity { ... }` |

### Function Keywords

| Keyword | Category | Description | Example |
|---------|----------|-------------|---------|
| `fun` | Declaration | Function definition | `fun add(a: Int64, b: Int64) -> Int64` |
| `return` | Statement | Return value | `return x + y` |
| `is` | Trait | Required method | `is run() -> Result` |
| `provides` | Trait | Default implementation | `provides default() -> Self` |
| `requires` | Trait | Dependency | `requires Comparable` |

### Control Flow Keywords

| Keyword | Category | Description | Example |
|---------|----------|-------------|---------|
| `if` | Control | Conditional | `if x > 0 { ... }` |
| `else` | Control | Alternative branch | `else { ... }` |
| `match` | Control | Pattern matching | `match x { ... }` |
| `for` | Loop | Iteration | `for item in list { ... }` |
| `while` | Loop | Conditional loop | `while x > 0 { ... }` |
| `loop` | Loop | Infinite loop | `loop { break }` |
| `break` | Control | Exit loop | `break` |
| `continue` | Control | Next iteration | `continue` |
| `where` | Guard | Pattern guard | `n where n > 0` |
| `in` | Iterator | Iteration source | `for x in items` |

### Type Keywords

| Keyword | Category | Description | Example |
|---------|----------|-------------|---------|
| `type` | Declaration | Type alias | `type TaskId is UInt64` |
| `has` | Field | Gene field | `has id: UInt64` |
| `enum` | Type | Enumeration | `type Status is enum { ... }` |
| `impl` | Implementation | Trait impl | `impl Runnable for Task` |

### Module Keywords

| Keyword | Category | Description | Example |
|---------|----------|-------------|---------|
| `module` | Declaration | Module definition | `module scheduler @ 1.0` |
| `use` | Import | Import items | `use std.io.println` |
| `spirit` | Package | Package manifest | `spirit MyApp { ... }` |

### SEX Keywords (Side Effect eXecution)

| Keyword | Category | Description | Example |
|---------|----------|-------------|---------|
| `sex` | Effect | Side effect marker | `sex fun log() { ... }` |
| `pub` | Visibility | Public export | `pub fun process()` |
| `var` | Mutability | Mutable variable | `sex var COUNTER: Int64` |
| `const` | Mutability | Immutable constant | `const MAX: Int64 = 100` |
| `extern` | FFI | Foreign function | `sex extern fun malloc()` |

### Meta-Programming Keywords

| Keyword | Category | Description | Example |
|---------|----------|-------------|---------|
| `macro` | Definition | Macro definition | `macro unless(...) { }` |
| `migrate` | Evolution | Migration function | `migrate from OldType { }` |
| `added` | Evolution | Added field | `added name: String` |
| `removed` | Evolution | Removed field | `removed legacy` |
| `changed` | Evolution | Changed type | `changed id: UInt32 -> UInt64` |

### Boolean Keywords

| Keyword | Category | Description | Example |
|---------|----------|-------------|---------|
| `true` | Literal | Boolean true | `active = true` |
| `false` | Literal | Boolean false | `done = false` |
| `and` | Operator | Logical AND | `a and b` |
| `or` | Operator | Logical OR | `a or b` |
| `not` | Operator | Logical NOT | `not done` |

### Option/Result Keywords

| Keyword | Category | Description | Example |
|---------|----------|-------------|---------|
| `Some` | Constructor | Optional value | `Some(42)` |
| `None` | Constructor | No value | `None` |
| `Ok` | Constructor | Success result | `Ok(value)` |
| `Err` | Constructor | Error result | `Err(message)` |

### Special Keywords

| Keyword | Category | Description | Example |
|---------|----------|-------------|---------|
| `this` | Reference | Self reference | `this.id` |
| `Self` | Type | Self type | `-> Self` |
| `_` | Pattern | Wildcard | `match x { _ { } }` |
| `forall` | Quantifier | Universal quantifier | `forall x: T. ...` |
| `exists` | Quantifier | Existential quantifier | `exists x: T. ...` |
| `implies` | Logic | Implication | `a implies b` |

---

## Operators Reference

### Arithmetic Operators

| Operator | Name | Description | Precedence |
|----------|------|-------------|------------|
| `+` | Plus | Addition | 9 |
| `-` | Minus | Subtraction | 9 |
| `*` | Star | Multiplication | 10 |
| `/` | Slash | Division | 10 |
| `%` | Percent | Modulo | 10 |
| `**` | Power | Exponentiation | 11 (right) |

### Comparison Operators

| Operator | Name | Description | Precedence |
|----------|------|-------------|------------|
| `==` | Equals | Equality | 3 |
| `!=` | NotEquals | Inequality | 3 |
| `<` | Less | Less than | 4 |
| `<=` | LessEq | Less or equal | 4 |
| `>` | Greater | Greater than | 4 |
| `>=` | GreaterEq | Greater or equal | 4 |

### Logical Operators

| Operator | Name | Description | Precedence |
|----------|------|-------------|------------|
| `&&` | And | Logical AND | 2 |
| `\|\|` | Or | Logical OR | 1 |
| `!` | Not | Logical NOT | 12 (prefix) |
| `and` | And | Logical AND (word) | 2 |
| `or` | Or | Logical OR (word) | 1 |
| `not` | Not | Logical NOT (word) | 12 (prefix) |

### Composition Operators

| Operator | Name | Description | Precedence |
|----------|------|-------------|------------|
| `\|>` | Pipe | Forward application | 5 |
| `>>` | Compose | Function composition | 6 |
| `<\|` | BackPipe | Reverse application | 5 |
| `@` | Apply | Applicative apply | 8 |
| `:=` | Bind | Monadic bind/partial | 7 (right) |

### Meta-Programming Operators

| Operator | Name | Description | Precedence |
|----------|------|-------------|------------|
| `'` | Quote | Defer evaluation | 12 (prefix) |
| `!` | Eval | Force evaluation | 12 (prefix) |
| `#` | Macro | Macro invocation | 12 (prefix) |
| `?` | Reflect | Type introspection | 12 (prefix) |
| `[\|` | IdiomOpen | Begin idiom bracket | — |
| `\|]` | IdiomClose | End idiom bracket | — |

### Type Operators

| Operator | Name | Description | Example |
|----------|------|-------------|---------|
| `->` | Arrow | Function type | `Int64 -> Bool` |
| `=>` | FatArrow | Constraint impl | `A => B` |
| `\|` | Bar | Union type | `A \| B` |
| `<$>` | Map | Functor map | `f <$> x` |
| `<*>` | Ap | Applicative apply | `f <*> x` |

### Assignment Operators

| Operator | Name | Description | Example |
|----------|------|-------------|---------|
| `=` | Assign | Assignment | `x = 10` |
| `+=` | AddAssign | Add and assign | `x += 1` |
| `-=` | SubAssign | Subtract and assign | `x -= 1` |
| `*=` | MulAssign | Multiply and assign | `x *= 2` |
| `/=` | DivAssign | Divide and assign | `x /= 2` |

### Delimiters

| Symbol | Name | Description |
|--------|------|-------------|
| `{` `}` | Braces | Block scope |
| `(` `)` | Parens | Grouping, parameters |
| `[` `]` | Brackets | Array access, type params |
| `:` | Colon | Type annotation |
| `,` | Comma | Separator |
| `.` | Dot | Member access |
| `@` | At | Version annotation |
| `$` | Dollar | Splice in quasi-quote |

---

## Type System

### Primitive Types

| Type | Size | Rust | Description |
|------|------|------|-------------|
| `Void` | 0 | `()` | No value |
| `Bool` | 1 | `bool` | Boolean |
| `Int8` | 8 | `i8` | Signed 8-bit |
| `Int16` | 16 | `i16` | Signed 16-bit |
| `Int32` | 32 | `i32` | Signed 32-bit |
| `Int64` | 64 | `i64` | Signed 64-bit (default) |
| `UInt8` | 8 | `u8` | Unsigned 8-bit |
| `UInt16` | 16 | `u16` | Unsigned 16-bit |
| `UInt32` | 32 | `u32` | Unsigned 32-bit |
| `UInt64` | 64 | `u64` | Unsigned 64-bit |
| `Float32` | 32 | `f32` | 32-bit float |
| `Float64` | 64 | `f64` | 64-bit float (default) |
| `String` | var | `String` | UTF-8 string |

### Compound Types

| Type | Rust | Description |
|------|------|-------------|
| `List<T>` | `Vec<T>` | Dynamic array |
| `Map<K, V>` | `HashMap<K, V>` | Key-value map |
| `Option<T>` | `Option<T>` | Optional value |
| `Result<T, E>` | `Result<T, E>` | Result type |
| `Tuple<A, B>` | `(A, B)` | Product type |
| `Fun<A, B>` | `fn(A) -> B` | Function type |
| `Sex<T>` | — | Effectful type |
| `Quoted<T>` | — | Quoted expression |

### Pointer Types (SEX only)

| Type | Rust | Description |
|------|------|-------------|
| `Ptr<T>` | `*mut T` | Raw pointer |
| `Ref<T>` | `&T` | Immutable reference |
| `MutRef<T>` | `&mut T` | Mutable reference |

---

## SEX System

### Safety Hierarchy

```
PURE (default)
├── No side effects
├── Referentially transparent
├── Private by default
└── Safe to parallelize

PUB (public)
├── Exported from module
├── Still pure unless sex
└── API boundary

SEX (side effects)
├── Can mutate global state
├── Can perform I/O
├── Can call FFI
├── Must be explicitly marked
└── Compiler tracks propagation
```

### Syntax

```dol
// Sex function
sex fun log(msg: String) -> Void {
    println(msg)
}

// Sex variable (mutable global)
sex var COUNTER: Int64 = 0

// Sex block (inline)
fun mostly_pure(x: Int64) -> Int64 {
    result = x * 2
    sex {
        println("Debug: " + result)
    }
    return result
}

// Sex extern (FFI)
sex extern fun malloc(size: UInt64) -> Ptr<Void>

sex extern "C" {
    fun getpid() -> Int32
    fun fork() -> Int32
}

// Constant (allowed anywhere)
const MAX_SIZE: Int64 = 100

// Visibility
pub fun public_pure() -> Int64 { }
pub sex fun public_effectful() -> Void { }
pub(spirit) fun spirit_only() -> Void { }
```

### File Conventions

| Pattern | Meaning |
|---------|---------|
| `*.sex.dol` | File is sex context |
| `sex/` | Directory is sex context |
| `sex { }` | Inline sex block |

---

## Meta-Programming

### Quote (`'`)

```dol
// Quote expression
expr = '(1 + 2 * 3)           // Quoted<Expr>

// Quote block
block = '{
    x = 10
    return x * 2
}

// Quasi-quote with splice
template = '(result = $inner + 1)
```

### Eval (`!`)

```dol
// Evaluate quoted expression
result = !expr                // Executes, returns value

// Eval with bindings
result = !template where { inner = '(x * 2) }
```

### Macro (`#`)

```dol
// Macro invocation
#stringify(x + y)             // "x + y"
#concat("hello", "_", "world") // "hello_world"
#env("HOME")                  // Environment variable
#cfg(target.wasm) { ... }     // Conditional compilation
#derive(Debug, Clone)         // Trait derivation
#assert(x > 0)                // Runtime assertion
#dbg(expr)                    // Debug print

// Macro definition
macro unless(cond: Expr, body: Block) -> Expr {
    return '{ if not !cond { !body } }
}
```

### Reflect (`?`)

```dol
// Get type info
info = ?Container
// info.name == "Container"
// info.fields == [...]

// Check trait
if ?Task.implements(Runnable) { ... }

// Match on type
match ?value {
    Int64 { handle_int() }
    String { handle_string() }
}
```

### Idiom Brackets (`[| |]`)

```dol
// Applicative style
result = [| add mx my |]
// Desugars to: add <$> mx <*> my
```

---

## Biological Modeling

DOL includes primitives for modeling biological systems, inspired by mycelium networks and ecological dynamics.

### Core Types

```dol
use biology.types.{ Vec3, Gradient, Nutrient, Energy, GeoTime }

// 3D vector
vec = Vec3 { x: 1.0, y: 2.0, z: 3.0 }

// Nutrient (Redfield ratio constrained)
nutrient = Nutrient {
    carbon: 106.0,
    nitrogen: 16.0,
    phosphorus: 1.0,
    water: 50.0
}
```

### Hyphal Trait

```dol
use biology.hyphal.{ Hyphal, HyphalTip }

trait Hyphal {
    is extend(gradient: Gradient<Nutrient>) -> Self
    is branch(factor: Float64) -> List<Self>
    is fuse(other: Self) -> Option<Self>
    is absorb(available: Nutrient) -> Tuple<Self, Nutrient>
    
    law conservation_of_mass {
        self.absorbed == self.transported + self.stored
    }
}
```

### Transport Trait

```dol
use biology.transport.{ Transport, Flow }

trait Transport<T> {
    is transport(source: Node, sink: Node, amount: T) -> Result<Flow<T>, Error>
    is optimize_flow(nodes: List<Node>) -> List<Flow<T>>
    
    law mass_conservation {
        network.inflow == network.outflow + network.accumulation
    }
}
```

### Ecosystem System

```dol
use biology.ecosystem.{ Ecosystem, Species, TrophicRole }

system Ecosystem {
    state species: Map<UInt64, Species>
    state interactions: List<Interaction>
    
    constraint trophic_efficiency {
        // 10% energy transfer between levels
        energy_at(level + 1) <= energy_at(level) * 0.15
    }
    
    constraint carrying_capacity {
        for sp in species.values() {
            sp.population <= sp.carrying_capacity
        }
    }
}
```

### Evolution

```dol
use biology.evolution.{ Evolvable, Lineage }

evolves Organism > Prokaryote @ 3.5Gya {
    added cell_wall: CellWall
    constraint no_nucleus { not this.has_nucleus }
    
    migrate from Organism {
        return Prokaryote {
            ...old,
            cell_wall: CellWall.default()
        }
    }
}

evolves Prokaryote > Eukaryote @ 2.0Gya {
    added nucleus: Nucleus
    added mitochondria: List<Mitochondrion>
}
```

---

## Grammar Summary

### EBNF Grammar (Simplified)

```ebnf
(* Top-level *)
program        = declaration* ;
declaration    = visibility? (gene | trait | system | function | evolves | const | sex_decl) ;

(* Visibility *)
visibility     = "pub" ("(" ("spirit" | "parent") ")")? ;

(* Sex declarations *)
sex_decl       = "sex" (sex_fun | sex_var | sex_extern) ;
sex_fun        = "fun" IDENT params "->" type block ;
sex_var        = "var" IDENT ":" type ("=" expr)? ;
sex_extern     = "extern" STRING? ("{" extern_fn* "}" | extern_fn) ;
extern_fn      = "fun" IDENT params "->" type ;

(* Core declarations *)
gene           = "gene" IDENT "{" gene_body "}" ;
gene_body      = (field | constraint | function | exegesis)* ;
field          = "has" IDENT ":" type ("=" expr)? ;

trait          = "trait" IDENT "{" trait_body "}" ;
trait_body     = (requires | provides | law | exegesis)* ;
requires       = "is" IDENT params "->" type ;
provides       = "provides" IDENT params "->" type block ;
law            = "law" IDENT block ;

system         = "system" IDENT "{" system_body "}" ;
system_body    = (uses | state | constraint | function | exegesis)* ;
uses           = "uses" type ;
state          = "state" IDENT ":" type ;

function       = "fun" IDENT params ("->" type)? block? ;
params         = "(" (param ("," param)*)? ")" ;
param          = IDENT ":" type ;

evolves        = "evolves" type ">" IDENT "@" version "{" evolves_body "}" ;
evolves_body   = (added | removed | changed | constraint | migrate | exegesis)* ;
added          = "added" IDENT ":" type ("=" expr)? ;
removed        = "removed" IDENT ;
changed        = "changed" IDENT ":" type "->" type ;
migrate        = "migrate" "from" IDENT block ;

const          = "const" IDENT ":" type "=" expr ;
exegesis       = "exegesis" "{" TEXT "}" ;

(* Expressions *)
expr           = quote_expr | eval_expr | macro_expr | reflect_expr
               | idiom_expr | lambda | if_expr | match_expr | binary ;

quote_expr     = "'" expr ;
eval_expr      = "!" expr ("where" bindings)? ;
macro_expr     = "#" IDENT ("(" args ")" | block)? ;
reflect_expr   = "?" type ;
idiom_expr     = "[|" expr+ "|]" ;

lambda         = "|" params "|" ("->" type)? block ;
if_expr        = "if" expr block ("else" (if_expr | block))? ;
match_expr     = "match" expr "{" match_arm* "}" ;
match_arm      = pattern ("where" expr)? block ;

binary         = unary (binop unary)* ;
unary          = ("!" | "-" | "'" | "?")? postfix ;
postfix        = primary ("." IDENT | "(" args ")" | "[" expr "]")* ;
primary        = literal | IDENT | "(" expr ")" | sex_block | block ;
sex_block      = "sex" block ;
block          = "{" statement* expr? "}" ;

(* Statements *)
statement      = let_stmt | return_stmt | expr_stmt | for_stmt | while_stmt ;
let_stmt       = IDENT (":" type)? "=" expr ;
return_stmt    = "return" expr? ;
for_stmt       = "for" IDENT "in" expr block ;
while_stmt     = "while" expr block ;

(* Types *)
type           = primitive | compound | generic | function_type ;
primitive      = "Void" | "Bool" | "Int8" | ... | "String" ;
compound       = IDENT "<" type ("," type)* ">" ;
generic        = IDENT ;
function_type  = type "->" type ;

(* Literals *)
literal        = INT | FLOAT | STRING | "true" | "false" | "None" ;

(* Identifiers *)
IDENT          = [a-zA-Z_][a-zA-Z0-9_]* ;
version        = INT "." INT ("." INT)? | FLOAT IDENT? ;
```

---

## Reserved Keywords (Complete List)

```
// Ontological
gene trait system constraint evolves exegesis law

// Functions
fun return is provides requires

// Control Flow
if else match for while loop break continue where in

// Types
type has enum impl

// Modules
module use spirit

// SEX
sex pub var const extern

// Meta
macro migrate added removed changed

// Boolean
true false and or not

// Option/Result
Some None Ok Err

// Special
this Self _ forall exists implies

// Future Reserved
async await yield spawn try catch finally
```

---

*"The system that knows what it is, becomes what it knows."*
