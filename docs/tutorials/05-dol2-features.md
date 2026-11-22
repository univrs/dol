# Tutorial 5: Expressions, Functions and Meta-programming

> **About the name "DOL 2.0".** Older docs call these features "DOL 2.0". It is a historical
> name, not a version. It refers to the expression-capable generation of the language:
> functions, control flow, pattern matching, lambdas, pipes and meta-programming, all
> introduced around crate v0.1.0. There is no separate "DOL 2.0" specification. The language
> is versioned by the `dol` crate, currently **0.8.2**. Before that generation, DOL was purely
> declarative: `gen`, `trait`, `rule`, `system` and `evo`.

This tutorial covers the expression side of DOL as the 0.8.2 parser accepts it. Every
```` ```dol ```` block below parses as a whole file with `metadol::parse_file_all`. Blocks
marked *fragment* are statements and expressions. They were checked by wrapping them in a
`fun`. For the full list of syntax, see the [syntax reference](./syntax-reference.md).

### What parses vs. what compiles

These features all **parse** into the AST and lower to HIR. Code generation covers less:

- The **WASM backend** (`dol-build`, the VUDO path) supports functions, arithmetic, `if`,
  `match`, `while`, `loop` and `for` over ranges. It does **not** support lambdas, tuples,
  lists, `sex { }` blocks, `?`, quote/eval/reflect, idiom brackets, `this`, casts, or `for`
  over anything other than a range.
- **No type checking is enforced.** `fun f() -> i64 { return "hello" + true }` parses and
  passes validation.

See [`docs/DOL-STATUS-2026-11.md`](../DOL-STATUS-2026-11.md) §5 for details.

## Functions

Functions are top-level declarations, or members of a `gen` body.

```dol
fun add(a: i64, b: i64) -> i64 {
    return a + b
}

fun clamp(x: i64, lo: i64, hi: i64) -> i64 {
    if x < lo {
        return lo
    } else if x > hi {
        return hi
    }
    return x
}

gen counter {
    has value: i64 = 0

    fun doubled() -> i64 {
        return value * 2
    }
}

docs {
    A counter gen with a member function.
}
```

Function generics (`fun id<T>(x: T)`) do **not** parse in 0.8.2.

## Functional pipelines

### Pipe (`|>`)

`a |> f` passes `a` to `f`. Data flows left to right.

<!-- fragment: checked wrapped in a fun -->
```dol
let result = data |> validate |> transform |> store
```

### Compose (`>>`)

`f >> g` is the function that applies `f`, then `g`.

<!-- fragment: checked wrapped in a fun -->
```dol
let normalize = trim >> lowercase >> validate
let out = input |> (trim >> validate) |> process
```

`|>` binds more loosely than `>>`, so `a |> f >> g` parses as `a |> (f >> g)`.

## Pattern matching

<!-- fragment: checked wrapped in a fun -->
```dol
let v = match value {
    Some(x) => x,
    None => 0
}

let label = match n {
    0 => "zero",
    1 | 2 => "small",
    x if x > 100 => "big",
    _ => "other"
}

let p = match pair {
    (Some(x), Some(y)) => x + y,
    (Some(x), None) => x,
    _ => 0
}
```

Supported patterns: wildcards `_`, literals, identifiers, constructors `Some(x)`, tuples
`(a, b)`, or-patterns `a | b`, and guards `if cond`. An arm can also use a block:
`Some(x) { ... }`.

## Lambdas

<!-- fragment: checked wrapped in a fun -->
```dol
let id = |x| x
let double = |x: i64| x * 2
let add = |x: i64, y: i64| -> i64 { x + y }
let curried = |x: i64| |y: i64| x + y
let doubled = xs.map(|x| x * 2)
let piped = 5 |> double
```

Lambdas parse, but the WASM backend cannot compile them.

## Control flow

<!-- fragment: checked wrapped in a fun -->
```dol
let sign = if n > 0 { 1 } else if n < 0 { -1 } else { 0 }

for i in 0..10 {
    process(i)
}

for item in collection {
    process(item)
}

while running {
    step()
    if done { break }
}

loop {
    if should_stop() {
        break
    }
    continue
}
```

`if` and `match` are expressions. `for`, `while`, `loop`, `break`, `continue` and `return`
are statements. The WASM backend only compiles `for` over a range (`a..b`).

## Block expressions

A `{ ... }` block in expression position evaluates to its last expression.

<!-- fragment: checked wrapped in a fun -->
```dol
let total = {
    let x = 1
    let y = 2
    x + y
}
```

## Types

Use the v0.8 lowercase names: `i8`..`i128`, `u8`..`u128`, `f32`, `f64`, `bool`, `string`,
`()`. Compound types include generics (`Vec<T>`, `Option<T>`, `Result<T, E>`), tuples,
function types and inline enums.

```dol
gen typed.entity {
    has id: u64
    has name: string
    has score: f64
    has active: bool
    has metadata: Option<string>
    has history: Vec<Result<i64, string>>
    has scorer: (i64, string) -> bool
    has kind: enum { User, Admin, Guest }
}

docs {
    An entity with typed fields.
}
```

The old `Int32`/`String`/`Void`/`List<T>` names still parse, but warn "deprecated in v0.8.0".
They are planned to become errors in 0.9.

## Meta-programming

<!-- fragment: checked wrapped in a fun -->
```dol
let code = 'value
let three = !{ 1 + 2 }
let info = ?Point
let applied = [| add a b |]
let s = #stringify(x)
```

| Form | Meaning | Status |
|------|---------|--------|
| `'expr` | Quote: the expression as data | Parses into `Expr::Quote` |
| `!{ expr }` | Eval | Parses into `Expr::Eval` |
| `?Type` | Reflect on a type | Parses. `src/reflect.rs` provides the runtime type registry |
| `[\| f a b \|]` | Idiom brackets (applicative style) | Parses. Desugared by `IdiomDesugar` |
| `#name(args)` | Macro invocation | Parses. Built-in macros live in `src/macros` |

None of these compile to WASM. Quasi-quote with a backtick or `~` does **not** parse.

## Operator precedence

Listed from loosest to tightest, as defined in `src/pratt.rs`:

| Operators | Associativity |
|-----------|---------------|
| `:=` | right |
| `\|>` | left |
| `@` (apply) | left |
| `>>` | right |
| `->` | right |
| `..` | none |
| `\|\|` | left |
| `&&` | left |
| `==` `!=` | none |
| `<` `>` `<=` `>=` | none |
| `+` `-` | left |
| `*` `/` `%` | left |
| `^` | right |
| `as` | left |
| `.` | left |

Prefix operators (`-`, `!`, `'`, `?`) bind tighter than every infix operator except `as` and `.`.

## Complete example

```dol
module stream.processing @ 0.1.0

gen stream.processor {
    has buffer_size: u32 = 64
    has dropped: u64 = 0
    processor is streaming
}

docs {
    A stream processor with a bounded buffer.
}

fun classify(code: i64) -> string {
    return match code {
        0 => "ok",
        c if c < 100 => "warning",
        _ => "error"
    }
}

fun process(items: Vec<i64>) -> i64 {
    let total = items
        |> filter_valid
        |> sum
    if total > 1000 {
        return 1000
    }
    return total
}

sex fun report(msg: string) {
    println(msg)
}
```

## Checking your files

`dol-parse` and `dol-check` only parse the **first** declaration of a file and silently ignore
the rest. To check a whole file, call `metadol::parse_file_all` from Rust. The parser tests
for these features live in `tests/dol2_tests.rs`:

```bash
cargo test --test dol2_tests
```

## Key takeaways

1. "DOL 2.0" is a historical name for the expression features, not a version. The language
   ships as `dol` 0.8.2.
2. Pipes, composition, `match`, lambdas, blocks and meta-programming all parse.
3. Only a subset compiles to WASM, and types are not enforced yet.

## Next steps

- [Syntax reference](./syntax-reference.md)
- [Grammar (EBNF)](../grammar.ebnf)
