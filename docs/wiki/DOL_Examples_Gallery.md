# DOL Examples Gallery

> **Real-World Examples of Ontology-First Development**

All examples in this gallery are verified from the DOL compiler test suite.

---

## Quick Examples

### Hello World Gene
**Source**: `examples/genes/hello.world.dol`

```dol
gene hello.world {
  message has content
  message has sender
  message has timestamp
}

exegesis {
  The hello.world gene is the simplest possible DOL example. It defines
  a message entity with three essential properties: content (what the
  message says), sender (who sent it), and timestamp (when it was sent).
}
```

### Counter State Gene
**Source**: `examples/genes/counter.dol`

```dol
gene counter.state {
  counter has value
  counter has minimum
  counter has maximum
  counter derives from initialization
}

exegesis {
  The counter.state gene models a bounded counter. A counter has a current
  value, and bounds (minimum and maximum). The counter derives from an
  initialization value when created.
}
```

---

## Functions

### Simple Function
**Source**: `tests/codegen/golden/input/function.dol`

```dol
fun add(a: Int64, b: Int64) -> Int64 {
    return a + b
}
```

### Gene with Methods
**Source**: `tests/corpus/traits/trait_relationships.dol`

```dol
module tests.trait_relationships @ 1.0.0

pub gene SimpleValue {
    has value: String
    has count: Int64 = 0

    fun get_value() -> String {
        return this.value
    }
}

pub gene StringWrapper {
    has value: String

    fun to_string() -> String {
        return this.value
    }
}
```

### Pipe Operators
**Source**: `tests/codegen/golden/input/pipe_operators.dol`

```dol
fun process(x: Int64) -> Int64 {
    return x |> double |> increment
}
```

---

## Generic Types
**Source**: `tests/corpus/genes/nested_generics.dol`

```dol
module tests.nested_generics @ 1.0.0

// Simple generic
pub gene Container<T> {
    has item: T
}

// Nested generic
pub gene Nested<T> {
    has items: List<T>
    has mapping: Map<String, T>
    has optional: Option<T>
    has result: Result<T, String>
}

// Deeply nested
pub gene DeepNest<T> {
    has deep: Map<String, List<Option<T>>>
    has matrix: List<List<T>>
    has complex: Result<Map<String, List<T>>, String>
}

// Multiple type params
pub gene Multi<K, V> {
    has key: K
    has value: V
    has pairs: List<Tuple<K, V>>
}

// Bounded generics
pub gene Bounded<T: Comparable> {
    has items: List<T>

    fun max() -> Option<T> {
        if this.items.is_empty() {
            return None
        }
        return Some(this.items.reduce(|a, b| if a > b { a } else { b }))
    }
}

// Generic with default
pub gene WithDefault<T = Int64> {
    has value: T
}

exegesis {
    Tests for nested and complex generic type parameters.
}
```

---

## Control Flow

### If Expressions
**Source**: `tests/dol2_tests.rs`

```dol
if x { result }
```

```dol
if condition { positive } else { negative }
```

```dol
if x { a } else if y { b } else if z { c } else { d }
```

### For Loops
**Source**: `tests/parser_tests.rs`

```dol
for outer in outers {
    for inner in inners {
        break;
    }
}
```

### For Loop with Range
**Source**: `tests/corpus/sex/nested_sex.dol`

```dol
for i in 0..n {
    sex {
        COUNTER += 1
        LOG.push("iteration " + i.to_string())
    }
}
```

### Pattern Matching
**Source**: `tests/dol2_tests.rs`

```dol
match value {
    Some(x) => x,
    None => default,
    _ => fallback
}
```

```dol
match x {
    value if condition => positive,
    _ => zero
}
```

```dol
match pair {
    (Some(x), Some(y)) => result,
    _ => default
}
```

---

## Traits

### Metal DOL Trait
**Source**: `examples/traits/greetable.dol`

```dol
trait entity.greetable {
  uses entity.identity
  greetable can greet
  greetable can receive.greeting
  greeting is polite
}

exegesis {
  The entity.greetable trait defines behavior for entities that can
  participate in greetings.
}
```

### DOL 2.0 Trait with Laws
**Source**: `tests/corpus/genes/complex_constraints.dol`

```dol
pub trait Ordered {
    is compare(other: Self) -> Int64

    law reflexive {
        forall x: Self. x.compare(x) == 0
    }

    law antisymmetric {
        forall x: Self. forall y: Self.
            x.compare(y) <= 0 && y.compare(x) <= 0 implies
                x.compare(y) == 0
    }

    law transitive {
        forall x: Self. forall y: Self. forall z: Self.
            x.compare(y) <= 0 && y.compare(z) <= 0 implies
                x.compare(z) <= 0
    }
}
```

---

## Constraints

### Metal DOL Constraint
**Source**: `examples/constraints/counter_bounds.dol`

```dol
constraint counter.bounds_valid {
  value never overflows
  value never underflows
  bounds never inverted
}

exegesis {
  The counter.bounds_valid constraint ensures the counter state is
  always valid.
}
```

### DOL 2.0 Constraints with Quantifiers
**Source**: `tests/corpus/genes/complex_constraints.dol`

```dol
pub gene OrderedList {
    has items: List<Int64>

    // Simple constraint
    constraint non_empty {
        this.items.length() > 0
    }

    // Forall constraint
    constraint sorted {
        forall i: UInt64.
            i < this.items.length() - 1 implies
                this.items[i] <= this.items[i + 1]
    }

    // Exists constraint
    constraint has_positive {
        exists x: Int64. x in this.items && x > 0
    }
}
```

---

## SEX (Side Effect System)
**Source**: `tests/corpus/sex/nested_sex.dol`

### Global Mutable State

```dol
// Global mutable state
sex var COUNTER: Int64 = 0
sex var LOG: List<String> = []
sex var CACHE: Map<String, Int64> = Map.new()
```

### SEX Functions

```dol
// Simple sex function
sex fun increment() -> Int64 {
    COUNTER += 1
    return COUNTER
}

// Sex function with sex block
sex fun logged_increment(label: String) -> Int64 {
    sex {
        LOG.push(label + ": incrementing")
    }

    result = COUNTER + 1
    COUNTER = result

    sex {
        LOG.push(label + ": now " + result.to_string())
    }

    return result
}

// Pure function with sex block
fun compute_with_logging(x: Int64) -> Int64 {
    result = x * 2 + 1

    sex {
        LOG.push("computed: " + result.to_string())
    }

    return result
}
```

### Conditional Effects

```dol
sex fun conditional_effects(cond: Bool) -> Int64 {
    if cond {
        sex {
            COUNTER += 10
        }
    } else {
        sex {
            COUNTER -= 10
        }
    }
    return COUNTER
}
```

---

## Systems
**Source**: `examples/systems/greeting.service.dol`

```dol
system greeting.service @0.1.0 {
  requires entity.greetable >= 0.0.1
  requires greeting.protocol >= 0.0.1

  uses hello.world
  service has greeting.templates
  service has response.timeout
}

exegesis {
  The greeting.service system composes genes, traits, and constraints
  into a complete, versioned component.
}
```

### Bounded Counter System
**Source**: `examples/systems/bounded.counter.dol`

```dol
system bounded.counter @0.1.0 {
  requires counter.state >= 0.0.1
  requires counter.countable >= 0.0.1
  requires counter.bounds_valid >= 0.0.1

  counter has persistence.strategy
  counter has overflow.policy
}

exegesis {
  The bounded.counter system composes genes, traits, and constraints
  into a complete, versioned component.
}
```

---

## Evolution and Versioning
**Source**: `tests/corpus/genes/evolution_chain.dol`

```dol
module tests.evolution_chain @ 1.0.0

// Base type
pub gene EntityV1 {
    has id: UInt32
    has name: String
}

// First evolution - add fields
evolves EntityV1 > EntityV2 @ 2.0.0 {
    added created_at: Int64 = 0
    added updated_at: Int64 = 0

    migrate from EntityV1 {
        return EntityV2 {
            ...old,
            created_at: 0,
            updated_at: 0
        }
    }
}

// Second evolution - change types
evolves EntityV2 > EntityV3 @ 3.0.0 {
    changed id: UInt32 -> UInt64
    added metadata: Map<String, String>
    removed updated_at

    migrate from EntityV2 {
        return EntityV3 {
            id: old.id as UInt64,
            name: old.name,
            created_at: old.created_at,
            metadata: Map.new()
        }
    }
}

// Third evolution - rename fields
evolves EntityV3 > EntityV4 @ 4.0.0 {
    renamed name -> display_name
    added tags: List<String> = []

    migrate from EntityV3 {
        return EntityV4 {
            id: old.id,
            display_name: old.name,
            created_at: old.created_at,
            metadata: old.metadata,
            tags: []
        }
    }
}
```

---

## Lambda Expressions
**Source**: `tests/dol2_tests.rs`

### Basic Lambdas

```dol
map(|x| x, list)
```

```dol
|x: Int32, y: Int32, z: Int32| -> Int32 { x }
```

### Curried Lambda

```dol
|x| |y| x
```

### Lambda in Pipeline

```dol
data |> (|x| x) |> result
```

### Lambda with Reduce
**Source**: `tests/corpus/genes/nested_generics.dol`

```dol
this.items.reduce(|a, b| if a > b { a } else { b })
```

---

## Pipes and Function Composition
**Source**: `tests/dol2_tests.rs`

### Forward Pipe

```dol
data |> validate |> transform |> store
```

### Function Composition

```dol
trim >> lowercase >> validate >> normalize
```

### Mixed Pipe and Composition

```dol
data |> (trim >> validate) |> process
```

---

## Example Index by Concept

| Concept | Example Count | Sources |
|---------|---------------|---------|
| Gene Declaration | 8 | examples/genes/, tests/corpus/ |
| Functions | 10 | tests/codegen/, tests/corpus/ |
| Control Flow | 8 | tests/dol2_tests.rs, tests/parser_tests.rs |
| Types & Generics | 6 | tests/corpus/genes/nested_generics.dol |
| Pattern Matching | 5 | tests/dol2_tests.rs |
| Traits | 5 | examples/traits/, tests/corpus/ |
| Constraints | 4 | examples/constraints/, tests/corpus/ |
| Systems | 2 | examples/systems/ |
| Evolution | 3 | tests/corpus/genes/evolution_chain.dol |
| Lambdas | 5 | tests/dol2_tests.rs |
| SEX (Side Effects) | 6 | tests/corpus/sex/nested_sex.dol |

---

## Running the Examples

```bash
# Parse and validate
dol check examples/

# Generate Rust code
dol compile examples/container.dol --output generated/

# Run tests
dol test examples/
```

---

## More Examples

Find more examples in the repository:

- `examples/genes/` - Gene definitions
- `examples/traits/` - Trait definitions
- `examples/constraints/` - Constraint examples
- `examples/systems/` - System definitions
- `tests/corpus/` - Comprehensive test corpus
- `dol/` - Self-hosted compiler (DOL in DOL!)

**Official Resources:**
- [GitHub](https://github.com/univrs/dol/releases/tag/v0.3.0)
- [Crates.io](https://crates.io/crates/dol/0.3.0)

---

*"Programs must be written for people to read, and only incidentally for machines to execute."* — Harold Abelson
