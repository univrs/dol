# DOL 2.0 Syntax Reference

Complete language reference for the Domain Ontology Language (DOL) version 2.0.

## Table of Contents

1. [Lexical Elements](#lexical-elements)
2. [Keywords](#keywords)
3. [Operators](#operators)
4. [Types](#types)
5. [Declarations](#declarations)
6. [Statements](#statements)
7. [Expressions](#expressions)
8. [SEX System](#sex-system)
9. [Modules](#modules)

---

## Lexical Elements

### Identifiers

```
identifier        = letter { letter | digit | '_' }
qualified_id      = identifier { '.' identifier }
```

Examples:
```dol
container           // simple identifier
container.exists    // qualified identifier
_private            // underscore prefix
my_var123           // mixed
```

### Literals

```dol
// Strings (double-quoted, escape sequences supported)
"hello world"
"line1\nline2"
"path\\to\\file"

// Booleans
true
false

// Null
null

// Versions (semantic versioning)
0.0.1
1.2.3
```

### Comments

```dol
// Single-line comment (only style supported)
gene Example {
  // This is a comment
  container has identity
}
```

---

## Keywords

### Ontological Keywords (DOL 1.x)

| Keyword | Purpose | Example |
|---------|---------|---------|
| `gene` | Atomic ontological unit | `gene container.exists { }` |
| `trait` | Behavioral composition | `trait container.lifecycle { }` |
| `constraint` | Invariant definition | `constraint identity.unique { }` |
| `system` | Versioned composition | `system cluster @ 1.0.0 { }` |
| `evolves` | Version migration | `evolves foo @ 1.0.0 > 0.9.0 { }` |
| `exegesis` | Mandatory documentation | `exegesis { ... }` |

### Predicate Keywords

| Keyword | Purpose | Example |
|---------|---------|---------|
| `has` | Property possession | `container has identity` |
| `is` | State/behavior | `container is running` |
| `derives` | Origin relationship | `container derives from image` |
| `from` | Used with derives | `derives from base` |
| `requires` | Dependency | `service requires database` |
| `uses` | Composition | `uses container.exists` |
| `emits` | Event production | `transition emits event` |
| `matches` | Equivalence | `input matches output` |
| `never` | Negative constraint | `container never orphaned` |

### Evolution Keywords

| Keyword | Purpose | Example |
|---------|---------|---------|
| `adds` | Add capability | `adds container is paused` |
| `deprecates` | Mark deprecated | `deprecates old_field` |
| `removes` | Remove capability | `removes legacy_api` |
| `because` | Reason for change | `because "migration required"` |
| `migrate` | Migration logic | `migrate { ... }` |

### Test Keywords

| Keyword | Purpose | Example |
|---------|---------|---------|
| `test` | Test declaration | `test my_test { }` |
| `given` | Precondition | `given container exists` |
| `when` | Action | `when container starts` |
| `then` | Postcondition | `then container is running` |
| `always` | Invariant check | `always` |

### Quantifiers

| Keyword | Purpose | Example |
|---------|---------|---------|
| `each` | Universal (for each) | `each container has id` |
| `all` | Universal (all) | `all services respond` |
| `no` | Negation | `no container orphaned` |
| `forall` | Logical universal | `forall x: T, P(x)` |
| `exists` | Logical existential | `exists x: T, P(x)` |

### Control Flow Keywords (DOL 2.0)

| Keyword | Purpose | Example |
|---------|---------|---------|
| `let` | Variable binding | `let x = 42` |
| `if` | Conditional | `if cond { } else { }` |
| `else` | Alternative branch | `else { }` |
| `match` | Pattern matching | `match x { ... }` |
| `for` | Iteration | `for x in items { }` |
| `while` | While loop | `while cond { }` |
| `loop` | Infinite loop | `loop { }` |
| `break` | Exit loop | `break` |
| `continue` | Next iteration | `continue` |
| `return` | Return value | `return x` |
| `in` | Membership | `for x in list` |
| `where` | Constraint clause | `where T: Trait` |

### Module Keywords (DOL 2.0)

| Keyword | Purpose | Example |
|---------|---------|---------|
| `module` | Module declaration | `module mylib @ 1.0.0` |
| `pub` | Public visibility | `pub gene Foo { }` |
| `use` | Import | `use std::io` |
| `spirit` | Spirit pattern | `spirit SharedState { }` |

### SEX Keywords (DOL 2.0)

| Keyword | Purpose | Example |
|---------|---------|---------|
| `sex` | Side effect marker | `sex fun write() { }` |
| `var` | Mutable variable | `sex var counter = 0` |
| `const` | Constant | `const PI = 3.14159` |
| `extern` | FFI declaration | `extern fun malloc()` |

### Logic Keywords (DOL 2.0)

| Keyword | Purpose | Example |
|---------|---------|---------|
| `implies` | Logical implication | `A implies B` |
| `not` | Logical negation | `not condition` |
| `impl` | Trait implementation | `impl Trait for Type` |
| `as` | Type cast | `x as Int32` |
| `state` | System state | `state Running { }` |
| `law` | Trait law | `law associativity { }` |
| `mut` | Mutable parameter | `fun modify(mut x: T)` |

---

## Operators

### Arithmetic Operators

| Operator | Name | Example |
|----------|------|---------|
| `+` | Addition | `a + b` |
| `-` | Subtraction | `a - b` |
| `*` | Multiplication | `a * b` |
| `/` | Division | `a / b` |
| `%` | Modulo | `a % b` |
| `^` | Power | `a ^ b` |

### Comparison Operators

| Operator | Name | Example |
|----------|------|---------|
| `==` | Equal | `a == b` |
| `!=` | Not equal | `a != b` |
| `<` | Less than | `a < b` |
| `<=` | Less or equal | `a <= b` |
| `>` | Greater than | `a > b` |
| `>=` | Greater or equal | `a >= b` |

### Logical Operators

| Operator | Name | Example |
|----------|------|---------|
| `&&` | Logical AND | `a && b` |
| `\|\|` | Logical OR | `a \|\| b` |
| `!` | Logical NOT | `!a` |
| `&` | Bitwise AND | `a & b` |
| `\|` | Bitwise OR | `a \| b` |

### Assignment Operators

| Operator | Name | Example |
|----------|------|---------|
| `=` | Assignment | `x = 5` |
| `:=` | Bind (monadic) | `result := computation` |
| `+=` | Add-assign | `x += 1` |
| `-=` | Sub-assign | `x -= 1` |
| `*=` | Mul-assign | `x *= 2` |
| `/=` | Div-assign | `x /= 2` |

### Composition Operators (DOL 2.0)

| Operator | Name | Example |
|----------|------|---------|
| `\|>` | Pipe forward | `x \|> f \|> g` |
| `<\|` | Pipe backward | `f <\| x` |
| `>>` | Compose | `f >> g` |

### Meta-Programming Operators (DOL 2.0)

| Operator | Name | Example |
|----------|------|---------|
| `'` | Quote (AST capture) | `'(1 + 2)` |
| `!` | Eval/Unquote | `!(quoted_expr)` |
| `#` | Macro invocation | `#[derive(Debug)]` |
| `?` | Type reflection | `?Type` |
| `[\|` `\|]` | Idiom brackets | `[\| f x y \|]` |

### Other Operators

| Operator | Name | Example |
|----------|------|---------|
| `@` | Version annotation | `system foo @ 1.0.0` |
| `->` | Return type / lambda | `fun f() -> Int` |
| `=>` | Match arm / closure | `x => x + 1` |
| `::` | Path separator | `std::io::read` |
| `.` | Member access | `obj.field` |
| `...` | Spread | `[...arr, x]` |
| `_` | Wildcard pattern | `match x { _ => }` |

### Delimiters

| Symbol | Name |
|--------|------|
| `{` `}` | Braces (blocks) |
| `(` `)` | Parentheses (grouping, calls) |
| `[` `]` | Brackets (arrays, indexing) |
| `,` | Comma (separator) |
| `:` | Colon (type annotation) |
| `;` | Semicolon (statement end) |

---

## Types

### Primitive Types

| Type | Description | Size |
|------|-------------|------|
| `Int8` | Signed 8-bit integer | 1 byte |
| `Int16` | Signed 16-bit integer | 2 bytes |
| `Int32` | Signed 32-bit integer | 4 bytes |
| `Int64` | Signed 64-bit integer | 8 bytes |
| `UInt8` | Unsigned 8-bit integer | 1 byte |
| `UInt16` | Unsigned 16-bit integer | 2 bytes |
| `UInt32` | Unsigned 32-bit integer | 4 bytes |
| `UInt64` | Unsigned 64-bit integer | 8 bytes |
| `Float32` | 32-bit floating point | 4 bytes |
| `Float64` | 64-bit floating point | 8 bytes |
| `Bool` | Boolean | 1 byte |
| `String` | UTF-8 string | variable |
| `Void` | No value | 0 bytes |

### Compound Types

```dol
// Arrays
[Int32]           // array of Int32
[[String]]        // nested array

// Tuples
(Int32, String)   // pair
(A, B, C)         // triple

// Functions
(Int32) -> Bool   // function type
() -> Void        // no args, no return

// Generics
List<T>           // generic type
Map<K, V>         // multiple type params
Option<T>         // optional value
Result<T, E>      // result with error
```

---

## Declarations

### Gene Declaration

Genes are atomic ontological units that cannot be decomposed.

```dol
gene container.exists {
  container has identity
  container has boundaries
  container is isolated

  exegesis {
    A container is the fundamental unit of workload isolation.
  }
}
```

### Trait Declaration

Traits compose genes and declare behaviors.

```dol
trait container.lifecycle {
  uses container.exists

  container is created
  container is running
  container is stopped
  each transition emits event

  exegesis {
    The lifecycle defines state transitions for containers.
  }
}
```

### Constraint Declaration

Constraints define invariants that must hold.

```dol
constraint identity.unique {
  no two containers share identity
  identity matches cryptographic_hash
  identity never changes

  exegesis {
    Container identities must be globally unique and immutable.
  }
}
```

### System Declaration

Systems are versioned top-level compositions.

```dol
system kubernetes.cluster @ 1.0.0 {
  requires container.runtime >= 1.0.0
  requires network.cni >= 0.4.0

  cluster has control_plane
  cluster has worker_nodes

  exegesis {
    A Kubernetes cluster orchestrates containerized workloads.
  }
}
```

### Evolution Declaration

Evolutions track changes between versions.

```dol
evolves container.lifecycle @ 0.0.2 > 0.0.1 {
  adds container is paused
  adds container is resumed
  deprecates container is suspended
  because "workload migration requires state preservation"

  migrate {
    // Migration logic
  }

  exegesis {
    Version 0.0.2 adds pause/resume for live migration.
  }
}
```

### Function Declaration (DOL 2.0)

```dol
// Pure function
fun add(a: Int32, b: Int32) -> Int32 {
  return a + b
}

// Public function
pub fun greet(name: String) -> String {
  return "Hello, " + name
}

// Generic function
fun identity<T>(x: T) -> T {
  return x
}

// Function with constraints
fun compare<T>(a: T, b: T) -> Bool where T: Ord {
  return a < b
}
```

---

## Statements

### Has Statement

Declares property possession.

```dol
container has identity
container has state
container has boundaries
```

### Is Statement

Declares state or behavior.

```dol
container is running
container is isolated
service is healthy
```

### Derives Statement

Declares origin relationship.

```dol
container derives from image
service derives from template
```

### Requires Statement

Declares dependency.

```dol
service requires database
cluster requires network >= 1.0.0
```

### Uses Statement

Declares composition.

```dol
uses container.exists
uses network.interface
```

### Let Statement (DOL 2.0)

Variable binding.

```dol
let x = 42
let name: String = "DOL"
let (a, b) = get_pair()
```

### Control Flow (DOL 2.0)

```dol
// If-else
if condition {
  do_something()
} else {
  do_other()
}

// Match
match value {
  0 => "zero",
  1 => "one",
  _ => "many"
}

// For loop
for item in collection {
  process(item)
}

// While loop
while condition {
  iterate()
}
```

---

## Expressions

### Arithmetic

```dol
1 + 2 * 3       // = 7
(1 + 2) * 3     // = 9
10 / 3          // = 3
10 % 3          // = 1
2 ^ 10          // = 1024
```

### Comparison

```dol
a == b
a != b
a < b && b < c
x >= 0 || x <= 100
```

### Function Calls

```dol
foo()
bar(1, 2, 3)
obj.method(arg)
```

### Pipe Expressions (DOL 2.0)

```dol
// Forward pipe
data |> parse |> validate |> process

// Equivalent to:
process(validate(parse(data)))

// Function composition
let pipeline = parse >> validate >> process
```

### Lambda Expressions (DOL 2.0)

```dol
// Short form
|x| x + 1

// With types
|x: Int32| -> Int32 { x + 1 }

// Multiple params
|a, b| a + b
```

### Quote/Eval (DOL 2.0)

```dol
// Quote captures AST
let expr = '(1 + 2)

// Eval executes quoted expression
let result = !expr  // = 3

// Quasi-quotation
let x = 5
let expr = '(x + !(get_value()))
```

---

## SEX System

The Side Effect eXecution (SEX) system explicitly marks impure code.

### SEX Functions

```dol
// Impure function (has side effects)
sex fun write_file(path: String, data: String) -> Result<(), Error> {
  // Can perform I/O
}

// Pure function (no sex keyword)
fun add(a: Int32, b: Int32) -> Int32 {
  return a + b
}
```

### SEX Variables

```dol
// Mutable global state
sex var counter: Int32 = 0

// Mutable local (inside sex function)
sex fun increment() {
  counter += 1
}
```

### SEX Blocks

```dol
fun mostly_pure() -> Int32 {
  let x = compute()

  sex {
    // Side effects isolated here
    log("computed: " + x)
  }

  return x
}
```

### Extern (FFI)

```dol
// Declare external function
extern sex fun malloc(size: UInt64) -> *Void
extern sex fun free(ptr: *Void) -> Void

// Use in sex context
sex fun allocate() {
  let ptr = malloc(1024)
  // ...
  free(ptr)
}
```

---

## Modules

### Module Declaration

```dol
module mylib @ 1.0.0

pub gene PublicGene { }
gene PrivateGene { }  // module-private
```

### Imports

```dol
use std::io
use std::collections::HashMap
use mylib::{Foo, Bar}
use other::* // glob import
```

### Visibility

```dol
pub gene Public { }       // accessible outside module
gene Private { }          // module-private (default)
pub(crate) gene Crate { } // crate-visible only
```

---

## Grammar Summary (EBNF)

```ebnf
dol_file = { declaration } ;

declaration = gene_decl | trait_decl | constraint_decl
            | system_decl | evolution_decl | function_decl ;

gene_decl = [ 'pub' ] 'gene' qualified_id '{' { statement } exegesis '}' ;

trait_decl = [ 'pub' ] 'trait' qualified_id '{' { statement } exegesis '}' ;

statement = has_stmt | is_stmt | derives_stmt | requires_stmt
          | uses_stmt | emits_stmt | let_stmt | if_stmt
          | for_stmt | while_stmt | return_stmt | expr_stmt ;

has_stmt = subject 'has' property ;
is_stmt = subject 'is' state ;

expr = literal | identifier | binary_expr | unary_expr
     | call_expr | lambda_expr | quote_expr ;

type = primitive_type | generic_type | function_type | tuple_type ;
```

---

## File Extensions

| Extension | Purpose |
|-----------|---------|
| `.dol` | DOL source file |
| `.dol.test` | DOL test file |

---

## Version

This reference covers DOL 2.0 as implemented in `dol v0.1.0`.

For the formal grammar, see [grammar.ebnf](./grammar.ebnf).
For the language specification, see [specification.md](./specification.md).
