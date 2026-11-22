# DOL Syntax Reference (dol 0.8.2)

This page describes the DOL syntax that the parser in the `dol` crate **0.8.2** accepts
(`src/lexer.rs` and `src/parser.rs`). It only lists what actually parses. Things that
appear in older docs but do not parse are listed under
[Not yet supported](#not-yet-supported-in-082). Constructs that parse but are thrown away are
listed under [Accepted but ignored](#accepted-but-ignored).

The language is versioned by the `dol` crate. "DOL 2.0" is the historical name for the
expression-capable generation of the language (functions, control flow, pattern matching,
lambdas, pipes, meta-programming). It is not a version number, and there is no separate
"DOL 2.0" specification. For the grammar in EBNF form, see
[`docs/grammar.ebnf`](../grammar.ebnf) ("DOL language as implemented by dol 0.8.2").

Every ```` ```dol ```` block on this page parses as a whole file with
`metadol::parse_file_all`. Blocks marked *fragment* contain statements or expressions. They
were checked by wrapping them in a `fun`.

> **Checking your own files.** `dol-parse` and `dol-check` only parse the **first**
> declaration in a file and silently ignore the rest. To check a whole file, call
> `metadol::parse_file_all` from Rust.

## Contents

1. [Lexical elements](#lexical-elements)
2. [Keywords](#keywords)
3. [Types](#types)
4. [Declarations](#declarations)
5. [Ontology statements](#ontology-statements)
6. [Functions and statements](#functions-and-statements)
7. [Expressions and operators](#expressions-and-operators)
8. [Side effects (`sex`)](#side-effects-sex)
9. [Modules and imports](#modules-and-imports)
10. [Attributes and annotations](#attributes-and-annotations)
11. [Legacy (deprecated) syntax](#legacy-deprecated-syntax)
12. [Accepted but ignored](#accepted-but-ignored)
13. [Not yet supported in 0.8.2](#not-yet-supported-in-082)

---

## Lexical elements

### Identifiers

An identifier starts with a letter or `_` and continues with letters, digits, `_` and `.`.
The dot is part of the identifier, so `container.exists` and `self.count` are each a
**single token**. This is how qualified names like `domain.property` work.

```dol
gen container.exists {
    container has identity
}
```

A lone `_` is the wildcard token.

### Literals

| Kind | Examples | Notes |
|------|----------|-------|
| Integer | `0`, `42` | Decimal only |
| Float | `3.14`, `0.5` | No exponent form |
| String | `"hello"`, `"a\nb"` | Double-quoted, single line. Escapes: `\n \t \r \" \\` |
| Char | `'a'`, `'\n'` | |
| Bool | `true`, `false` | |
| Null | `null` | |
| Version | `0.1.0`, `1.2.3` | Exactly three numeric parts. Used after `@` and in `requires` |

Number formats that look valid but are **mis-lexed without an error**: `1e9`, `1_000` and `0x10`.
For example, `1e9` becomes the integer `1` followed by an identifier `e9`. Don't use them.

### Comments

Line comments start with `//` or `--`. There are no block comments.

```dol
// a comment
-- also a comment
gen Example {
    has x: i64 // trailing comment
}
```

---

## Keywords

Preferred (v0.8) keywords:

| Group | Keywords |
|-------|----------|
| Declarations | `gen`, `trait`, `rule`, `system`, `evo`, `docs`, `type`, `fun`, `const`, `module` / `mod`, `use` |
| Ontology predicates | `has`, `is`, `derives from`, `requires`, `uses`, `emits`, `matches`, `never`, `extends` |
| Quantifiers | `each`, `all` |
| Evolution | `adds`, `deprecates`, `removes`, `because`, `migrate` |
| Control flow | `if`, `else`, `match`, `for`, `in`, `while`, `loop`, `break`, `continue`, `return`, `where` |
| Bindings | `let`, `val`, `var`, `const` |
| Effects | `sex`, `extern` |
| Visibility | `pub` |
| Traits and systems | `law`, `state` |
| Logic | `forall` |
| Other | `as`, `this`, `true`, `false`, `null` |

These words are reserved by the lexer but **have no grammar rule** in 0.8.2. Using them
causes a parse error: `impl`, `spirit` (except inside `pub(spirit)`), `config`, `mut`, `not`,
`exists`, `implies`, `given`, `when`, `then`, `always`, `test`, `no`.
(`given`, `when` and `then` belong to the separate `.dol.test` format that `dol-test` reads.)

Many keywords can still be used as field names, parameter names and struct-field names, for
example `has type: string` or `fun test()`. `docs`, `gen` and `evo` cannot be used this way.

---

## Types

| Type | Syntax |
|------|--------|
| Signed integers | `i8`, `i16`, `i32`, `i64`, `i128` |
| Unsigned integers | `u8`, `u16`, `u32`, `u64`, `u128` |
| Floats | `f32`, `f64` |
| Boolean | `bool` |
| String | `string` |
| Unit | `()` |
| Never | `!` |
| Generic application | `Vec<T>`, `Option<T>`, `Result<T, E>`, `Map<K, V>`, any `Name<...>` |
| Tuple | `(i64, string)` |
| Function | `(i64, i64) -> bool` |
| Named | any identifier, e.g. `Point` |
| Inline enum | `enum { A, B(i64), C { x: i64 }, D = 3 }` |

```dol
gen TypeShowcase {
    has id: u64
    has name: string
    has ratio: f64
    has active: bool
    has tags: Vec<string>
    has parent: Option<u64>
    has lookup: Map<string, Vec<i64>>
    has pair: (i64, string)
    has predicate: (i64) -> bool
    has status: enum { Pending, Running, Stopped }
}
```

Type names like `Vec` and `Map` are not checked: the parser accepts any `Name<...>`.

---

## Declarations

A file is an optional `module` line, then `use` lines, then any number of declarations.
Top-level declarations: `gen`, `type`, `trait`, `rule`, `system`, `evo`, `fun`, `sex fun`,
`sex var`, `sex extern fun`, `const`, plus `docs { }`, `use`, and `mod` blocks.

### Documentation (`docs`)

A `docs { ... }` block can appear:
- **after** a `gen`/`trait`/`rule`/`system`/`evo` body, where it becomes that declaration's
  documentation;
- as the **last item inside** the body;
- **standalone** at top level. A standalone block is skipped and is not attached to the next
  declaration.

The docs text is free prose. Avoid `{` and `}` inside it, because nested braces end the block
early.

```dol
gen point {
    has x: f64
    has y: f64
}

docs {
    A point in the plane.
}

gen size {
    has width: f64
    has height: f64

    docs {
        Docs can also be the last item inside the body.
    }
}
```

The parser does not require `docs`. `dol-check --require-exegesis` enforces it, but only on
the first declaration of each file.

### `gen` (genes) and `type`

```dol
gen container.exists {
    container has identity
    container has state
    container is persistent
    has id: u64
    has name: string = "unnamed"
    has replicas: u32 = 1 where replicas > 0
}

docs {
    A container is an isolated unit with an identity and a state.
}

gen managed.container extends container.exists {
    has owner: string
}

docs {
    Extends adds fields to an existing gen.
}

type Meters {
    has value: f64
}
```

`type Name { ... }` is an alias for `gen` and produces the same AST. A gen body can also hold
functions (`fun`, `pub fun`, `sex fun`).

```dol
gen counter {
    has value: i64 = 0

    fun get() -> i64 {
        return value
    }

    sex fun increment() {
        value = value + 1
    }
}

docs {
    A counter with a pure getter and a side-effecting increment.
}
```

### `trait`

```dol
trait container.lifecycle {
    uses container.exists
    container is created
    container is started
    container is stopped
    each transition emits event
    law idempotent_stop(x: i64) { x == x }
}

docs {
    Lifecycle states for a container.
}
```

A `law name(params) { expr }` inside a trait is parsed. The law's body must be a single
expression.

### `rule` (constraints)

```dol
rule container.integrity {
    identity never changes
    state matches declared
    has max_restarts: u32 = 5
}

docs {
    Integrity guarantees for containers.
}
```

### `system`

```dol
system univrs.orchestrator @ 0.1.0 {
    requires container.lifecycle >= 0.0.1
    requires node.discovery >= 0.0.1
    state tick: u64 = 0
    all operations is authenticated
}

docs {
    The orchestrator composes lifecycle and discovery.
}
```

The `@ version` after a system name is optional. Inside a system body, a line that starts with
`requires` must be a versioned requirement: `requires name >= | > | = X.Y.Z`. To write a
phrase requirement, start the line with a subject: `x requires y`.

### `evo` (evolutions)

```dol
evo container.lifecycle @ 0.0.2 > 0.0.1 {
    adds container is paused
    adds has pause_reason: string
    deprecates container is zombie
    removes legacy_flag
    because "Operators need to pause containers without stopping them"
    migrate {
        let note = "no data migration needed"
    }
}

docs {
    Adds a paused state.
}
```

The form is `evo Name @ new_version > parent_version { ... }`. Both versions are required.
`adds` and `deprecates` take a statement. `removes` takes a name. `because` takes a string.

### `fun`

```dol
fun add(a: i64, b: i64) -> i64 {
    return a + b
}

pub fun greet(name: string) -> string {
    return "hello, " + name
}

fun noop() {
}
```

The return type is optional. Function generics (`fun id<T>`) are not supported.

### `const`

```dol
const MAX_RETRIES: u32 = 3
const GREETING = "hello"
```

---

## Ontology statements

These statements can appear in `gen`, `trait`, `rule` and `system` bodies, and after
`adds`/`deprecates` in `evo`.

| Statement | Form |
|-----------|------|
| Has (property) | `subject has property` |
| Has (typed field) | `has name: Type [= default] [where expr]` or `subject has name: Type ...` |
| Field (short form) | `name: Type [= default]` |
| Is | `subject is state` |
| Derives | `subject derives from phrase` |
| Requires | `subject requires phrase` |
| Uses | `uses qualified.name` (no subject) |
| Emits | `action emits event` |
| Matches | `subject matches phrase` |
| Never | `subject never action` |
| Quantified | `each ...` / `all ...` followed by a phrase |

Subjects can be multi-word phrases, such as `identity never changes`. A quantified statement
(`each`/`all`) consumes words and predicates until the next `}`, `uses`, `each` or `all`, so
put it last in the body or follow it with another quantifier.

```dol
gen identity.cryptographic {
    identity derives from ed25519 keypair
    identity requires secure randomness
    identity has public_key
    signature matches public_key
    private_key never leaves
    rotation emits key_rotated
    created_at: u64 = 0
    each identity has fingerprint
}

docs {
    A cryptographic identity.
}
```

---

## Functions and statements

Statements inside a function body. Semicolons are optional.

<!-- fragment: checked wrapped in a fun -->
```dol
let a = 1
let b: i64 = 2
let c: i64
val d = 3
var e: i64 = 4
const F: i64 = 5
c = a + b
e = e + 1
for i in 0..10 {
    e = e + i
}
while e > 100 {
    e = e - 1
}
loop {
    break
}
if a > 0 {
    return 1
} else if a < 0 {
    return -1
} else {
    return 0
}
```

| Statement | Notes |
|-----------|-------|
| `let x [: T] [= expr]` | The value can be omitted when a type is given |
| `val x [: T] [= expr]` | Same as `let` |
| `var x [: T] = expr` | Parsed like `let`; mutability is not tracked |
| `const X [: T] = expr` | Inside a function |
| `target = expr` | Assignment to a name, field (`a.b = 1`) or index (`a[i] = 1`) |
| `for x in expr { }` | Any expression parses after `in`. The WASM backend only compiles ranges (`a..b`) |
| `while cond { }`, `loop { }` | |
| `break`, `continue`, `return [expr]` | |
| any expression | `if`, `match`, calls, `sex { }` blocks, ... |

---

## Expressions and operators

### Operators by precedence (loosest first)

| Precedence | Operators | Associativity |
|------------|-----------|---------------|
| 1 | `:=` (bind) | right |
| 2 | `\|>` (pipe) | left |
| 3 | `@` (apply) | left |
| 4 | `>>` (compose) | right |
| 5 | `->` | right |
| 6 | `..` (range) | none |
| 7 | `\|\|` | left |
| 8 | `&&` (also a single `&`) | left |
| 9 | `==`, `!=` | none |
| 10 | `<`, `>`, `<=`, `>=` | none |
| 11 | `+`, `-` | left |
| 12 | `*`, `/`, `%` | left |
| 13 | `^` (power) | right |
| 14 | `as` (cast to a type) | left |
| 15 | `.` (member) | left |

Prefix operators: `-x`, `!x`, `*x` (deref), `'x` (quote), `?Type` (reflect), `!{ expr }` (eval).
Postfix operators: calls `f(a, b)`, indexing `a[i]`, and `expr?` (try).

<!-- fragment: checked wrapped in a fun -->
```dol
let sum = 1 + 2 * 3
let pow = 2 ^ 10
let ok = a >= 0 && b != 0 || !done
let r = 0..10
let n = x as f64
let s = "hello, " + name
let m = point.distance(other)
let v = xs.map(|x| x * 2).filter(|x| x > 1)
let first = xs[0]
let value = parse(input)?
```

### Lambdas, pipes and composition

<!-- fragment: checked wrapped in a fun -->
```dol
let inc = |x: i64| x + 1
let dbl = |x: i64| -> i64 { x * 2 }
let both = inc >> dbl
let out = 5 |> inc |> dbl
```

### `if` and `match`

<!-- fragment: checked wrapped in a fun -->
```dol
let sign = if n > 0 { 1 } else if n < 0 { -1 } else { 0 }

let label = match opt {
    Some(v) if v > 100 => "big",
    Some(v) => "small",
    None => "none"
}

let kind = match code {
    1 | 2 => "low",
    (a, b) => "pair",
    _ => "other"
}
```

Patterns: `_`, literals, identifiers, constructors `Some(x)`, tuples `(a, b)`, or-patterns
`a | b`, and guards `if cond`. A match arm can be `pattern => expr` or `pattern { block }`.

### Collections, tuples and struct literals

<!-- fragment: checked wrapped in a fun -->
```dol
let list = [1, 2, 3]
let pair = (1, "one")
let unit = ()
let p = Point { x: 1.0, y: 2.0 }
let m = Map::new()
```

A struct literal needs a capitalised type name: `Point { x: 1 }`.

### Logic and meta-programming

<!-- fragment: checked wrapped in a fun -->
```dol
let all_equal = forall x: i64. x == x
let q = 'expr
let e = !{ 1 + 2 }
let t = ?Point
let i = [| add a b |]
let m = #stringify(x)
```

These forms parse into the AST. The WASM backend **cannot compile** lambdas, tuples, lists,
`sex { }` blocks, `?`, quote/eval/reflect, idiom brackets or `this`. It also cannot compile
casts, or `for` loops over anything other than a range. See
[`docs/DOL-STATUS-2026-11.md`](../DOL-STATUS-2026-11.md) §5.

> **No type checking.** `fun f() -> i64 { return "hello" + true }` parses and passes
> validation in 0.8.2.

---

## Side effects (`sex`)

`sex` marks side-effecting code.

```dol
sex var REQUEST_COUNT: i64 = 0

sex fun log_line(msg: string) {
    println(msg)
}

sex fun host_log(level: i32, ptr: i32, len: i32)

sex extern fun c_abs(x: i32) -> i32

fun tick() -> i64 {
    sex {
        log_line("tick")
    }
    return 0
}
```

- `sex fun name(...) { ... }`: a side-effecting function.
- `sex fun host_*(...)` **without a body**: in 0.8.2 this is a host import. The WASM backend
  imports it from module `vudo` with the VUDO VM's signature (`src/wasm/host_abi.rs`). A
  bodiless `sex fun` the VM doesn't provide is a compile error.
- `sex extern [ "abi" ] fun name(...) [-> T]`: parses, but the AST stores it only as a
  placeholder.
- `sex var NAME: T = value`: a mutable global.
- `sex { ... }`: a side-effect block inside a function. Its body holds `let`/`var`
  statements and expressions; plain assignment (`x = 1`) is not accepted inside it.

---

## Modules and imports

```dol
module app.main @ 1.0.0

use std.io
use app::{Point, Size}

fun main() -> i32 {
    return 0
}
```

`parse_file_all` skips the `module` line and `use` lines. To keep them in the AST, use
`metadol::parse_dol_file`. The `use` forms are:

- `use a.b`
- `use a::b`
- `use a::{X, Y as Z}`
- `use a::*`
- `use @org/pkg`
- `pub use ...`

---

## Attributes and annotations

```dol
#[wasm_export]
pub fun add(a: i64, b: i64) -> i64 {
    return a + b
}

gen document {
    @crdt(immutable) has id: string
    @crdt(lww) has title: string
    @crdt(peritext) has body: string
    @crdt(or_set) has tags: Vec<string>
    @crdt(pn_counter, min_value=0) has views: i64
    @personal has author_email: string
}

docs {
    A collaborative document.
}
```

The valid `@crdt` strategies are `immutable`, `lww`, `or_set`, `pn_counter`, `peritext`, `rga`
and `mv_register`. Any other name is a parse error. `@crdt` produces a real `CrdtAnnotation` in
the AST.

---

## Legacy (deprecated) syntax

The pre-0.8 keywords and type names still parse. Each use prints a warning of the form
`'gene' keyword is deprecated in v0.8.0, use 'gen' instead`. They are planned to become
errors in 0.9. To convert files, run `dol-migrate 0.7-to-0.8 <path>`.

| Deprecated | Use instead |
|------------|-------------|
| `gene` | `gen` |
| `constraint` | `rule` |
| `evolves` | `evo` |
| `exegesis` | `docs` |
| `Int8`..`Int64`, `UInt8`..`UInt64` | `i8`..`i64`, `u8`..`u64` |
| `Float32`, `Float64` | `f32`, `f64` |
| `Bool`, `String` | `bool`, `string` |
| `Void` | `()` |
| `List<T>`, `Optional<T>` | `Vec<T>`, `Option<T>` |

---

## Accepted but ignored

These constructs parse in 0.8.2 but have no effect, or are dropped from the AST:

- **Visibility on declarations**: `pub`, `pub(spirit)` and `pub(crate)` are accepted, but the
  AST always records `Private`. Inside a gen body, `pub fun` is the exception: it is kept.
- **Generic parameters on `gen`, `type`, `trait`, `rule` and `system`** (`gen Box<T> { }`) are
  skipped.
- **A `rule name { ... }` nested inside a gen body**: the body is discarded. Only the name is
  kept, as a `requires` statement.
- **A `law` inside a gen body** is skipped. A `law` inside a `trait` is parsed.
- **`mod name { ... }` blocks** in the middle of a file are skipped entirely. As the **first**
  item of a file, `mod name { ... }` is a parse error.
- **`#[test]` functions** are skipped.
- **`docs` blocks** written inside a body before other statements, or standalone at top level,
  are skipped.
- **`state` declarations in `system`** and **`migrate` blocks in `evo`** are parsed and then
  dropped.

---

## Not yet supported in 0.8.2

These forms appear in older docs and examples but **do not parse**:

| Construct | Status |
|-----------|--------|
| `let mut x = ...` | Parse error. Use `let x` or `var x`; mutability is not tracked |
| `x += 1` (also `-=`, `*=`, `/=`) | Parse error. Write `x = x + 1` |
| `&T` (reference types) | Parse error |
| `[T]` (array types) | Parse error. Use `Vec<T>` |
| `*T` (pointer types, e.g. `*Void`) | Parse error |
| `fun id<T>(x: T)` | Function generics: parse error |
| `impl X { }` | Parse error |
| top-level `spirit X { }` | Parse error |
| `extern fun ...` / `extern sex fun ...` | Parse error. Use `sex extern fun` or a bodiless `sex fun` |
| `fun f() -> T with IO` | Effect rows: parse error |
| `gen name @ 1.0.0 { }` | Versioned gen headers: parse error. Only `system` takes `@ version` |
| `evo A > B { }` (no `@`), `added ...` | Parse error. Use `evo A @ new > old { adds ... }` |
| `gen A evolves B { }` | Parse error |
| `type X is enum { }` | Parse error. Use `has kind: enum { ... }` |
| `` `x `` and `~x` (quasi-quote) | Parse error |
| `a implies b` | Lexed, then rejected as an invalid binary operator |
| `container uses network` (subject form of `uses`) | Parse error. `uses` takes no subject |
| `provides`, `exists`, `not` | No grammar rule |
| Struct patterns with field shorthand, e.g. `Shape.Circle { r } =>` | Parse error |
| `1e9`, `1_000`, `0x10` | Mis-lexed without an error (see [Literals](#literals)) |

---

## File extensions

| Extension | Purpose |
|-----------|---------|
| `.dol` | DOL source file |
| `.dol.test` | Test spec read by `dol-test` (a separate format) |
