# Tutorial 5: DOL 2.0 Features

## Introduction to DOL 2.0

DOL 2.0 extends Metal DOL with functional programming features, pattern matching, advanced control flow, and meta-programming capabilities. These features enable more expressive ontology definitions while maintaining backward compatibility with DOL 1.x.

### What's New in DOL 2.0

- **Functional pipelines** - `|>` and `>>` operators for data flow
- **Pattern matching** - `match` expressions with guards
- **Lambdas** - First-class anonymous functions
- **Advanced control flow** - `if/else`, `for`, `while`, `loop`
- **Block expressions** - Scoped expressions with local bindings
- **Rich type system** - Built-in types, generics, function types
- **Meta-programming** - Quote, eval, and reflection

## Functional Pipelines

DOL 2.0 introduces functional composition operators for expressing data transformations.

### The Pipe Operator (`|>`)

The pipe operator passes the left operand as input to the right operand:

```dol
// Data flows left to right
data |> validate |> transform |> store

// Equivalent to: store(transform(validate(data)))
```

Use pipe when you want to emphasize the data flowing through transformations:

```dol
gene data.pipeline @1.0.0 {
    // Define a processing pipeline
    has input: String
    has output: String
    is transformable
}

exegesis {
    A gene representing data flowing through a transformation pipeline.
    The pipe operator (|>) passes data from left to right through each stage.
}
```

### The Compose Operator (`>>`)

The compose operator creates a new function by combining functions:

```dol
// Function composition: (f >> g)(x) = g(f(x))
trim >> lowercase >> validate >> normalize

// Creates a new function that applies all four in sequence
```

Use compose when building reusable transformation chains:

```dol
trait transform.chain @1.0.0 {
    // Composable transformers
    uses transform.trim @1.0.0
    uses transform.normalize @1.0.0

    is composable
    is functional
}

exegesis {
    Trait for composable transformation chains.
    Use the compose operator (>>) to build pipelines.
}
```

### Combining Pipe and Compose

You can mix both operators for expressive data flows:

```dol
// Apply composed functions to data
data |> (trim >> validate) |> process

// The parenthesized composition runs first, then pipes
```

## Pattern Matching

DOL 2.0 adds powerful pattern matching with the `match` expression.

### Basic Match

```dol
match value {
    Some(x) => x,
    None => default,
    _ => fallback
}
```

Pattern types supported:
- **Identifier** - Matches and binds: `x => ...`
- **Wildcard** - Matches anything: `_ => ...`
- **Constructor** - Matches variants: `Some(x) => ...`
- **Tuple** - Matches tuples: `(a, b) => ...`
- **Literal** - Matches values: `42 => ...`

### Match with Guards

Add conditions to patterns with `if`:

```dol
match x {
    value if condition => positive,
    value if other_condition => alternative,
    _ => default
}
```

### Nested Patterns

Patterns can be nested for complex matching:

```dol
match pair {
    (Some(x), Some(y)) => result,
    (Some(x), None) => partial,
    (None, _) => empty,
    _ => default
}
```

### Using Match in Ontology

```dol
gene state.machine @1.0.0 {
    has current_state: String
    is stateful
}

exegesis {
    State machine gene that uses pattern matching for state transitions.

    Example transition logic:
    match current_state {
        "idle" => start_processing,
        "running" if complete => finish,
        "error" => recover,
        _ => maintain
    }
}
```

## Lambda Expressions

DOL 2.0 supports anonymous functions (lambdas) with optional type annotations.

### Basic Lambda

```dol
// Simple lambda
|x| x

// Lambda with body expression
|x| x * 2
```

### Typed Lambda

Add type annotations for clarity:

```dol
// Single typed parameter
|x: Int32| x * 2

// Multiple typed parameters with return type
|x: Int32, y: Int32| -> Int32 { x + y }
```

### Lambda in Pipelines

Lambdas work naturally with pipe operators:

```dol
data |> (|x| x * 2) |> result

// Or as function arguments
map(|x| x * 2, list)
```

### Nested Lambdas (Currying)

```dol
// Curried function
|x| |y| x + y

// Equivalent to a function that returns a function
```

## Control Flow

DOL 2.0 adds structured control flow expressions.

### If-Else Expressions

```dol
if condition {
    then_value
} else {
    else_value
}

// Chained conditions
if x { a } else if y { b } else if z { c } else { d }
```

### For Loops

Iterate over collections:

```dol
for item in collection {
    process(item);
}

// Nested loops
for outer in outers {
    for inner in inners {
        combine(outer, inner);
    }
}
```

### While Loops

Conditional iteration:

```dol
while condition {
    step();
    if done { break; }
    continue;
}
```

### Infinite Loops

Use `loop` for infinite iteration with explicit exit:

```dol
loop {
    process();
    if should_stop {
        break;
    }
}
```

### Break and Continue

- `break` - Exit the current loop
- `continue` - Skip to the next iteration

## Block Expressions

Blocks create scoped expressions with local bindings.

### Basic Blocks

```dol
{
    let x = 1;
    let y = 2;
    x + y  // Final expression is the block's value
}
```

### Nested Blocks

```dol
{
    let outer = 1;
    {
        let inner = 2;
        inner + outer  // Can access outer scope
    }
}
```

### Blocks in Pipelines

```dol
data |> {
    let validated = validate(data);
    let transformed = transform(validated);
    transformed
} |> store
```

## Type System

DOL 2.0 includes a rich type system with built-in types, generics, and function types.

### Built-in Types

Integer types:
- `Int8`, `Int16`, `Int32`, `Int64` - Signed integers
- `UInt8`, `UInt16`, `UInt32`, `UInt64` - Unsigned integers

Floating-point types:
- `Float32`, `Float64`

Other types:
- `Bool` - Boolean
- `String` - Text
- `Void` - No value

### Generic Types

```dol
// Option type
Option<Int32>

// Result type
Result<Int32, String>

// Nested generics
Result<Option<Int32>, String>
```

### Function Types

```dol
// Simple function type
(Int32) -> Int32

// Multiple parameters
(Int32, String, Bool) -> Result<Int32, String>

// Higher-order function type
((Int32) -> Int32, List<Int32>) -> List<Int32>
```

### Using Types in Ontology

```dol
gene typed.entity @1.0.0 {
    has id: UInt64
    has name: String
    has score: Float64
    has active: Bool
    has metadata: Option<String>

    is typed
}

exegesis {
    Entity with explicit DOL 2.0 type annotations.
    Types enable static analysis and code generation.
}
```

## Meta-programming

DOL 2.0 includes meta-programming primitives for code manipulation.

### Quote (`'`)

Quote prevents evaluation, treating code as data:

```dol
'{ value }

// The expression inside is not evaluated
// It becomes a data structure representing the code
```

### Eval (`!`)

Eval evaluates quoted code:

```dol
!{ code }

// Executes the code inside the braces
```

### Reflect (`?`)

Reflect inspects type information:

```dol
?TypeName

// Returns type metadata about TypeName
```

### Meta-programming Use Cases

```dol
gene meta.aware @1.0.0 {
    has type_info: String
    is reflective
}

exegesis {
    Gene that can inspect its own type at runtime.

    Use reflection for:
    - Dynamic dispatch
    - Serialization/deserialization
    - Type-safe code generation
}
```

## Operator Precedence

DOL 2.0 operators follow standard precedence rules:

| Precedence | Operators | Associativity |
|------------|-----------|---------------|
| Highest    | `()`, `[]`, `.` | Left |
| | Unary: `!`, `-`, `?`, `'` | Right |
| | `*`, `/`, `%` | Left |
| | `+`, `-` | Left |
| | `>>` (compose) | Left |
| | `|>` (pipe) | Left |
| | `<`, `>`, `<=`, `>=` | Left |
| | `==`, `!=` | Left |
| | `&&` | Left |
| Lowest | `\|\|` | Left |

Example:

```dol
// a + b * c parses as a + (b * c)
a + b * c

// a |> f >> g >> h parses as a |> (f >> g >> h)
a |> f >> g >> h

// a == b && c != d parses as (a == b) && (c != d)
a == b && c != d
```

## Backward Compatibility

DOL 2.0 is fully backward compatible with DOL 1.x:

```dol
// DOL 1.x syntax still works
gene container.exists @1.0.0 {
    has identifier: string
    is entity
    is persistent
}

exegesis {
    Standard DOL 1.x gene works unchanged in DOL 2.0.
}

// DOL 1.x trait syntax
trait container.lifecycle @1.0.0 {
    uses container.exists @1.0.0
    is lifecycle
}

exegesis {
    Standard DOL 1.x trait works unchanged in DOL 2.0.
}
```

## Complete Example

Here's a complete example combining DOL 2.0 features:

```dol
gene stream.processor @1.0.0 {
    has input_type: String
    has output_type: String
    has buffer_size: UInt32

    is functional
    is streaming
    is typed
}

exegesis {
    Stream processor gene with DOL 2.0 functional features.

    Processing model:
    - Input flows through pipeline stages
    - Each stage can transform, filter, or aggregate
    - Output is produced incrementally

    Example pipeline:
        input
        |> (parse >> validate)
        |> transform
        |> {
            let filtered = filter(|x| x.valid, data);
            aggregate(filtered)
        }
        |> output

    Pattern matching for error handling:
        match result {
            Ok(data) => emit(data),
            Err(e) if recoverable(e) => retry(),
            Err(e) => fail(e)
        }
}
```

## Testing DOL 2.0 Syntax

Parse your DOL 2.0 files:

```bash
cargo run --bin dol-parse -- my-file.dol
```

Run the DOL 2.0 test suite:

```bash
cargo test dol2
```

## Key Takeaways

1. **Pipe operator (`|>`)** - Data flows left to right
2. **Compose operator (`>>`)** - Build function chains
3. **Pattern matching** - Destructure and match with guards
4. **Lambdas** - Anonymous functions with optional types
5. **Control flow** - `if`, `for`, `while`, `loop` expressions
6. **Block expressions** - Scoped local bindings
7. **Rich types** - Built-in types, generics, function types
8. **Meta-programming** - Quote, eval, reflect primitives
9. **Backward compatible** - All DOL 1.x code works unchanged

## Next Steps

- Explore the [Type System Reference](../reference/types.md)
- Learn about [Code Generation](../reference/codegen.md)
- See [Advanced Patterns](../advanced/patterns.md)
