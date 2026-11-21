# ADR-007: Meta-Programming Operators

| Field | Value |
|-------|-------|
| **Status** | Accepted. Partially implemented: operators parse and have library support; several designed features below are proposals only (see Implementation Status) |
| **Date** | 2026-01-20 |
| **Deciders** | VUDO Core Team |
| **Supersedes** | N/A |
| **Superseded by** | N/A |

## Implementation Status (dol 0.8.2, verified 2026-11)

| Feature | Status |
|---------|--------|
| Quote `'expr`, quasi-quote `''expr` with unquote `,expr` | **Parses** (`Expr::Quote`, `Expr::QuasiQuote`, `Expr::Unquote`) |
| Eval | **Parses** as `!{ expr }` (`Expr::Eval`). Note `!expr` without braces is logical *not*, not eval |
| Macro invocation `#name(args)` in expressions | **Parses**; 20 built-in macros implemented as a library in `src/macros/builtin.rs` |
| Attribute form `#[name(...)]` before a declaration | **Parses**; the attribute name is recorded on functions, otherwise ignored |
| Reflect `?Type` | **Parses** (`Expr::Reflect`); runtime type registry in `src/reflect.rs` |
| Idiom brackets `[| f a b |]` | **Parses** (`Expr::IdiomBracket`); desugaring pass `IdiomDesugar` in `src/transform/desugar_idiom.rs` |
| Bare `#derive(...)` / `#cfg(...) { }` before a declaration, `macro name(...)` definitions, `$hole` quasi-quote holes, `.bind(...)`, `!expr where { ... }`, backtick or `~` quasi-quote | **Not implemented** — do not parse |
| `Quoted<T>` typing as described below | **Not enforced** (no type enforcement in 0.8.2; see the status audit §5.2) |
| Meta-programming in the WASM backend | **Not supported** (quote/meta/idioms are rejected by the WASM compiler) |

Examples below are labelled. Blocks fenced as `dol` parse with dol 0.8.2 (statement examples are wrapped in a `fun` so they parse as whole files); blocks fenced as `text` are proposals.

## Context

The expression-capable generation of DOL (historically called "DOL 2.0" — a generation label, not a version number) requires meta-programming capabilities for:
- Compile-time code generation (reducing boilerplate)
- Runtime type introspection (serialization, debugging)
- AST manipulation (DSL embedding, macros)
- Code quoting (templates, transformations)

### Requirements

1. **Intuitive Syntax** - Single-character operators for common operations
2. **Type-Safe** - Meta-operations should be type-checked
3. **Composable** - Operators should work together naturally
4. **Staged Execution** - Clear compile-time vs runtime boundaries
5. **Debuggable** - Generated code should be traceable

### Options Considered

| Approach | Syntax | Type-Safe | Compile-Time | Runtime |
|----------|--------|-----------|--------------|---------|
| **Operator-based** | `' ! # ?` | ✅ | ✅ | ✅ |
| **Keyword-based** | `quote, eval, macro` | ✅ | ✅ | ✅ |
| **Attribute-based** | `#[derive]` | ⚠️ | ✅ | ❌ |
| **Template strings** | `` `code` `` | ❌ | ⚠️ | ⚠️ |

## Decision

**We chose four single-character operators for meta-programming.**

### Operator Summary

| Operator | Name | Phase | Purpose |
|----------|------|-------|---------|
| `'` | Quote | Compile | Capture expression as AST data |
| `!` | Eval | Runtime | Execute quoted expression |
| `#` | Macro | Compile | Compile-time code generation |
| `?` | Reflect | Runtime | Type introspection |

### Additionally: Idiom Brackets

| Syntax | Name | Purpose |
|--------|------|---------|
| `[| |]` | Idiom Brackets | Applicative functor sugar |

## Operator Specifications

### Quote (`'`) — Capture as AST

The quote operator captures an expression as AST data without evaluating it.

```dol
fun quote_examples() {
    // Quote a simple expression
    val expr = '(1 + 2 * 3)

    // Quote a block
    val block = '{
        val x = 10
        x * 2
    }

    // Quasi-quote with an unquoted (spliced) sub-expression
    val template = ''(base + ,offset)
}
```

Proposed — not implemented (`$` holes and `.bind` do not exist):

```text
val template = '(base + $offset)
val filled = template.bind(offset = 5)
```

**Type:** `'expr` has type `Quoted<T>` where `T` is the type of `expr`.

### Eval (`!`) — Execute Quoted Expression

The eval operator executes a quoted expression in the current context. As implemented, eval is written `!{ expr }`; `!expr` without braces parses as logical *not*.

```dol
fun eval_example() {
    val expr = '(1 + 2)
    val result = !{ expr }
}
```

Proposed — not implemented:

```text
// Eval with bindings
val template = '(x + y)
val computed = !template where { x = 10, y = 20 }
// computed: i64 = 30

// Dynamic dispatch
fun apply_op(op: Quoted<fn(i64, i64) -> i64>, a: i64, b: i64) -> i64 {
    return !op(a, b)
}
```

**Type:** `!quoted` has type `T` where `quoted: Quoted<T>`.

### Macro (`#`) — Compile-Time Generation

The macro operator invokes compile-time code generation.

```dol
fun macro_examples() {
    val msg = #format("Hello, {}!", name)
    val hash = #env("BUILD_HASH")
    #assert(count > 0, "count must be positive")
    val s = #stringify(a + b)
}
```

Attribute form (parses; the attribute name is recorded but no derive code is generated by the parser):

```dol
#[derive(Debug, Clone)]
gen User {
    has id: u64
    has name: string
}
```

Proposed — not implemented (bare `#cfg` blocks and `macro` definitions do not parse):

```text
// Conditional compilation
#cfg(target = "wasm") {
    fun platform_init() { /* WASM setup */ }
}
#cfg(target = "native") {
    fun platform_init() { /* Native setup */ }
}

// Custom macro definition
macro repeat(n: i64, body: Quoted<()>) -> Quoted<()> {
    var stmts = Vec::new()
    for i in 0..n {
        stmts.push(body.clone())
    }
    return quote_block(stmts)
}

// Usage
#repeat(3, '{
    print("hello")
})
// Expands to: print("hello"); print("hello"); print("hello")
```

**20 Built-in Macros** (as implemented in `src/macros/builtin.rs`):

| Macro | Description |
|-------|-------------|
| `#stringify(expr)` | Convert expression to string literal |
| `#concat(a, b, ...)` | Concatenate string literals |
| `#env("VAR")` | Read environment variable at compile time |
| `#option_env("VAR")` | Optional environment variable |
| `#cfg(condition)` | Conditional compilation |
| `#derive(Trait, ...)` | Generate trait implementations |
| `#assert(cond)` | Runtime assertion |
| `#assert_eq(a, b)` | Assert equality |
| `#assert_ne(a, b)` | Assert inequality |
| `#format(fmt, ...)` | String formatting |
| `#dbg(expr)` | Debug print (returns value) |
| `#todo(msg)` | Mark unimplemented |
| `#unreachable()` | Mark unreachable code |
| `#compile_error(msg)` | Emit compile-time error |
| `#vec(a, b, c)` | Create vector literal |
| `#file()` | Current file name |
| `#line()` | Current line number |
| `#column()` | Current column number |
| `#include_str(path)` | Include file contents as a string |
| `#debug_assert(cond)` | Debug-only assertion |

### Reflect (`?`) — Type Introspection

The reflect operator provides runtime type information. `?Type` parses (`Expr::Reflect`); the runtime registry lives in `src/reflect.rs`. The API shown below (`implements`, `get_field`, `match ?value`) is a design sketch, not a verified API.

```text
// Get type info
val info = ?User
// info: TypeInfo

// Access type metadata
print(info.name)        // "User"
print(info.module)      // "app.models"
print(info.size)        // 24 (bytes)

// Iterate fields
for field in info.fields {
    print(field.name + ": " + field.type_name)
}
// Output:
// id: u64
// name: string

// Check trait implementation
if ?User.implements(Serialize) {
    serialize(user)
}

// Dynamic field access
val user = User { id: 1, name: "Alice" }
val id_value = (?user).get_field("id")
// id_value: Any = 1

// Pattern match on type
fun describe(value: Any) -> string {
    match ?value {
        i64 => "an integer"
        string => "a string"
        User => "a user"
        _ => "unknown"
    }
}
```

**TypeInfo Structure (design sketch):** the implemented Rust `TypeInfo` in `src/reflect.rs` has `name`, `kind`, `fields`, `methods`, `type_params` and more; `FieldInfo` has `name`, `type_name`, `optional`, `mutable`, `doc`. The original design is below.

```text
gen TypeInfo {
    has name: string
    has module: string
    has size: u64
    has alignment: u64
    has fields: Vec<FieldInfo>
    has methods: Vec<MethodInfo>
    has traits: Vec<string>

    fun implements(trait_name: string) -> bool
    fun get_field(name: string) -> Option<FieldInfo>
}

gen FieldInfo {
    has name: string
    has type_name: string
    has type_info: TypeInfo
    has offset: u64
    has is_public: bool
}
```

### Idiom Brackets (`[| |]`) — Applicative Sugar

Idiom brackets provide syntactic sugar for applicative functor style.

```dol
fun idiom_examples() {
    val result = [| add mx my |]

    // Desugars to: Config <$> host <*> port <*> timeout
    val parsed = [| Config host port timeout |]
}
```

The `<$>` and `<*>` operators shown in the desugaring are notation only; they are not DOL operators.

**Desugaring Rules:**
- `[| f |]` → `pure(f)`
- `[| f x |]` → `f <$> x`
- `[| f x y |]` → `f <$> x <*> y`
- `[| f x y z |]` → `f <$> x <*> y <*> z`

## Consequences

### Positive

- **Expressive** - Complex meta-programming in concise syntax
- **Familiar** - Similar to Lisp quote/eval, Rust macros
- **Type-Safe** - Quoted expressions carry type information
- **Composable** - Operators work together naturally
- **Efficient** - Compile-time expansion, no runtime overhead for macros

### Negative

- **Learning Curve** - Four new operators to learn
- **Debugging** - Macro expansion can be hard to trace
- **Complexity** - Powerful features enable complex (confusing) code

### Neutral

- **Syntax Choice** - Single characters are terse but cryptic
- **Error Messages** - Need excellent errors for meta-programming failures

## Implementation Notes

### AST Nodes (original design; the implemented nodes in `src/ast.rs` are `Quote(Box<Expr>)`, `QuasiQuote(Box<Expr>)`, `Unquote(Box<Expr>)`, `Eval(Box<Expr>)`, `Reflect(Box<TypeExpr>)`, `IdiomBracket { func, args }`; macro calls are encoded as `Call` with a `#name` callee)

```rust
pub enum Expr {
    // ... existing variants ...
    
    /// Quote: 'expr
    Quote {
        inner: Box<Expr>,
        span: Span,
    },
    
    /// Eval: !expr
    Eval {
        quoted: Box<Expr>,
        bindings: Option<Vec<(String, Expr)>>,
        span: Span,
    },
    
    /// Macro invocation: #name(args)
    MacroInvoke {
        name: String,
        args: Vec<Expr>,
        span: Span,
    },
    
    /// Reflect: ?expr or ?Type
    Reflect {
        target: Box<Expr>,
        span: Span,
    },
    
    /// Idiom bracket: [| f x y |]
    IdiomBracket {
        func: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
}
```

### Type Checking (proposed — not enforced in 0.8.2)

```rust
fn check_quote(expr: &Expr, env: &TypeEnv) -> Type {
    let inner_type = check_expr(expr, env)?;
    Type::Quoted(Box::new(inner_type))
}

fn check_eval(quoted: &Expr, bindings: &[(String, Expr)], env: &TypeEnv) -> Type {
    let quoted_type = check_expr(quoted, env)?;
    match quoted_type {
        Type::Quoted(inner) => *inner,
        _ => error!("eval requires Quoted<T>, got {}", quoted_type),
    }
}

fn check_reflect(target: &Expr, env: &TypeEnv) -> Type {
    // Reflect always returns TypeInfo
    Type::TypeInfo
}
```

## References

- [Lisp Quote/Eval](https://www.gnu.org/software/emacs/manual/html_node/elisp/Quoting.html)
- [Rust Macros](https://doc.rust-lang.org/book/ch19-06-macros.html)
- [Template Haskell](https://wiki.haskell.org/Template_Haskell)
- [Idiom Brackets Paper](https://personal.cis.strath.ac.uk/conor.mcbride/Idiom.pdf)

## Changelog

| Date | Change |
|------|--------|
| 2026-01-20 | Initial meta-programming design |
| 2026-01-25 | Added 20 built-in macros |
| 2026-02-05 | Added idiom brackets |
| 2026-11-21 | Added implementation status; corrected eval syntax (`!{ }`) and built-in macro list; marked unimplemented forms as proposed |
