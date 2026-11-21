# DOL Examples Gallery

> **Examples of ontology-first development in DOL v0.8 syntax**

Every `dol` code block on this page was checked against **dol 0.8.2** as a whole-file parse (`metadol::parse_file_all`). Snippets that are statements or expressions are wrapped in a `fun` so that they parse as files.

"Parses" means the parser accepts the code. It does not mean every backend can compile it. The WASM backend in particular supports only a subset of the language (no lambdas, tuples, lists or `sex {}` blocks; see [DOL-STATUS-2026-11](../DOL-STATUS-2026-11.md) §5.5), and 0.8.2 does not enforce types (§5.2).

Some examples are adapted from files in `examples/` and `tests/`. Several of those source files still use the deprecated pre-0.8 keywords (`gene`, `exegesis`, `constraint`, `evolves`) and PascalCase types; this page uses the current forms.

---

## Quick Examples

### Hello World Gen
**Adapted from**: `examples/genes/hello.world.dol`

```dol
gen hello.world {
  message has content
  message has sender
  message has timestamp
}

docs {
  The hello.world gen is the simplest possible DOL example. It defines
  a message entity with three essential properties: content (what the
  message says), sender (who sent it), and timestamp (when it was sent).
}
```

### Counter State Gen
**Adapted from**: `examples/genes/counter.dol`

```dol
gen counter.state {
  counter has value
  counter has minimum
  counter has maximum
  counter derives from initialization
}

docs {
  The counter.state gen models a bounded counter. A counter has a current
  value, and bounds (minimum and maximum). The counter derives from an
  initialization value when created.
}
```

### Typed Fields

```dol
gen Point {
    has x: f64
    has y: f64
}

gen User {
    has id: u64
    has name: string
    has email: string
    has tags: Vec<string>
    has manager: Option<u64>
    has status: enum { Active, Suspended }
}
```

---

## Functions

### Simple Function

```dol
fun add(a: i64, b: i64) -> i64 {
    return a + b
}
```

### Gen with Methods

```dol
module tests.trait_relationships @ 1.0.0

gen SimpleValue {
    has value: string
    has count: i64 = 0

    fun get_value() -> string {
        return this.value
    }
}

gen StringWrapper {
    has value: string

    fun to_string() -> string {
        return this.value
    }
}
```

`pub` is accepted before `gen`, but 0.8.2 drops it from the AST, so it is omitted here.

### Pipe Operators

```dol
fun process(x: i64) -> i64 {
    return x |> double |> increment
}
```

---

## Generic Types in Fields

Generic *type arguments* in field types parse:

```dol
gen Nested {
    has items: Vec<i64>
    has mapping: Map<string, i64>
    has optional: Option<i64>
    has result: Result<i64, string>
    has deep: Map<string, Vec<Option<i64>>>
    has matrix: Vec<Vec<f64>>
}
```

**Not supported in 0.8.2:**
- *Type parameters* on a declaration (`gen Container<T> { … }`) are parsed and then silently discarded.
- Generic functions (`fun id<T>(x: T) -> T`) do not parse.

---

## Control Flow

### If Expressions

```dol
fun sign(x: i64) -> i64 {
    if x > 0 {
        return 1
    } else if x < 0 {
        return -1
    } else {
        return 0
    }
}
```

### For Loops

```dol
fun nested(outers: Vec<i64>, inners: Vec<i64>) {
    for outer in outers {
        for inner in inners {
            break
        }
    }
}
```

### For Loop with Range

```dol
sex var COUNTER: i64 = 0

sex fun count_to(n: i64) {
    for i in 0..n {
        COUNTER = COUNTER + 1
    }
}
```

Compound assignment (`+=`, `-=`) does not parse in 0.8.2; write `x = x + 1`.

### While and Loop

```dol
fun countdown(n: i64) -> i64 {
    var i: i64 = n
    while i > 0 {
        i = i - 1
    }
    loop {
        break
    }
    return i
}
```

### Pattern Matching

```dol
fun unwrap_or(value: Option<i64>, fallback: i64) -> i64 {
    match value {
        Some(x) if x > 0 => x,
        Some(x) => 0 - x,
        None => fallback,
        _ => fallback
    }
}

fun both(pair: (Option<i64>, Option<i64>)) -> i64 {
    match pair {
        (Some(x), Some(y)) => x + y,
        _ => 0
    }
}
```

---

## Traits

### Declarative Trait
**Adapted from**: `examples/traits/greetable.dol`

```dol
trait entity.greetable {
  uses entity.identity
  greetable is polite
  greeting is acknowledged
  each greeting emits event
}

docs {
  The entity.greetable trait defines behavior for entities that can
  participate in greetings.
}
```

### Trait with Laws

`law name(params) { expression }` parses inside a trait body:

```dol
trait Ordered {
    law reflexive(x: i64) {
        x == x
    }

    law total(x: i64, y: i64) {
        x <= y || y <= x
    }
}

docs {
    A total order expressed as laws over its comparison.
}
```

The `implies` keyword is lexed, but it is rejected as an operator in 0.8.2.

---

## Rules

### Declarative Rule
**Adapted from**: `examples/constraints/counter_bounds.dol`

```dol
rule counter.bounds_valid {
  value never overflows
  value never underflows
  bounds never inverted
}

docs {
  The counter.bounds_valid rule ensures the counter state is
  always valid.
}
```

### Field Constraints with `where`

```dol
gen BoundedValue {
    has value: i64 where value >= 0
    has name: string where name != ""
}
```

A `rule name { … }` nested *inside* a `gen` is accepted, but its body is discarded in 0.8.2. Use top-level `rule` declarations or `where` clauses instead.

---

## SEX (Side Effect System)

### Global Mutable State

```dol
sex var COUNTER: i64 = 0
sex var LOG: Vec<string> = []
```

### SEX Functions

```dol
sex var COUNTER: i64 = 0

sex fun increment() -> i64 {
    COUNTER = COUNTER + 1
    return COUNTER
}

sex fun logged_increment(label: string) -> i64 {
    sex {
        println(label + ": incrementing")
    }

    let result = COUNTER + 1
    COUNTER = result
    return result
}

fun compute_with_logging(x: i64) -> i64 {
    let result = x * 2 + 1

    sex {
        println("computed")
    }

    return result
}
```

### Host Imports (VUDO VM)

A bodiless `sex fun` is a host import. With the WASM backend, `host_*` names import from module `vudo` (dol 0.8.2):

```dol
sex fun host_log(level: i64, msg: string) -> i64

sex fun hello() -> i64 {
    return host_log(2, "hello from DOL")
}
```

A `string` argument to a `host_*` function is lowered to a `(ptr, len)` pair (taken from `tests/wasm_host_abi.rs`). Signatures are checked against `src/wasm/host_abi.rs`. Effect *propagation*, i.e. a pure function calling `sex` code, is not checked in 0.8.2.

---

## Systems
**Adapted from**: `examples/systems/greeting.service.dol`

```dol
system greeting.service @ 0.1.0 {
  requires entity.greetable >= 0.0.1
  requires greeting.protocol >= 0.0.1

  service has greeting.templates
  service has response.timeout
}

docs {
  The greeting.service system composes gens, traits, and rules
  into a complete, versioned component.
}
```

### Bounded Counter System
**Adapted from**: `examples/systems/bounded.counter.dol`

```dol
system bounded.counter @ 0.1.0 {
  requires counter.state >= 0.0.1
  requires counter.countable >= 0.0.1
  requires counter.bounds_valid >= 0.0.1

  counter has persistence.strategy
  counter has overflow.policy
}

docs {
  The bounded.counter system composes gens, traits, and rules
  into a complete, versioned component.
}
```

---

## Evolution and Versioning

An evolution is written `evo Name @ new-version > parent-version`:

```dol
gen Entity {
    has id: u32
    has name: string
}

docs {
    Base entity.
}

evo Entity @ 2.0.0 > 1.0.0 {
    adds created_at: i64 = 0
    adds Entity is timestamped
    deprecates Entity is anonymous
    removes nickname
    because "auditing requires creation timestamps"

    migrate {
        let created_at = 0
    }
}

docs {
    Adds a creation timestamp to Entity.
}
```

**Not supported in 0.8.2:** `evolves A > B` without `@ version`, and the `added`/`changed`/`renamed`/`removed` keywords (the real keywords are `adds`, `deprecates`, `removes`, `because`, `migrate`). `migrate from X { … }` and `...old` spread syntax are also not supported.

---

## Lambda Expressions

```dol
fun lambdas(list: Vec<i64>) -> i64 {
    let ids = map(|x| x, list)
    let first = |x: i32, y: i32, z: i32| -> i32 { x }
    let curried = |x: i64| |y: i64| x + y
    let biggest = list.reduce(|a, b| if a > b { a } else { b })
    return 0
}
```

---

## Pipes and Function Composition

```dol
fun pipelines(data: string) -> string {
    let stored = data |> validate |> transform |> store
    let clean = trim >> lowercase >> validate >> normalize
    let mixed = data |> (trim >> validate) |> process
    let with_lambda = data |> (|x: string| x) |> result
    return stored
}
```

---

## Running the Examples

All binaries need the `cli` feature:

```bash
# Parse and validate (prints the AST)
cargo run --features cli --bin dol-parse -- examples/genes/hello.world.dol

# CI-style check, requiring docs blocks
cargo run --features cli --bin dol-check -- examples/ --require-exegesis

# Generate Rust / TypeScript / JSON Schema
cargo run --features cli --bin dol-codegen -- --target rust examples/genes/counter.dol
```

**Caveat:** `dol-parse` and `dol-check` currently parse only the **first** declaration in each file and ignore the rest (status audit §5.1). `dol-codegen` uses a whole-file parse.

There is no `dol` binary, so `dol check`, `dol compile` and `dol test` do not exist.

---

## More Examples

- `examples/genes/`: gen definitions
- `examples/traits/`: trait definitions
- `examples/constraints/`: rule examples
- `examples/systems/`: system definitions
- `examples/v0.8/`: v0.8 syntax examples (`quickstart.dol` parses as a whole file; the other three do not yet)
- `tests/corpus/`: test corpus
- `dol/`: bootstrap sources for a DOL-in-DOL compiler. They parse, but the compiler is **not** self-hosting; see [DOL_Self_Hosting](./DOL_Self_Hosting.md).

**Official Resources:**
- [GitHub](https://github.com/univrs/dol)
- [Crates.io](https://crates.io/crates/dol)

---

*"Programs must be written for people to read, and only incidentally for machines to execute."* — Harold Abelson
