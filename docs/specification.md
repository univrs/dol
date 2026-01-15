# DOL Language Specification

**Version**: 0.0.1
**Status**: Draft
**Last Updated**: December 2025

## 1. Introduction

DOL (Design Ontology Language) is a declarative specification language that serves as the source of truth for ontology-first software development. In the DOL-first paradigm, design contracts are written before tests, and tests are written before code.

```
Traditional:  Code → Tests → Documentation
DOL-First:    Design Ontology → Tests → Code
```

### 1.1 Design Principles

1. **Declarative**: DOL describes *what* things are, not *how* they work
2. **Ontology-First**: All concepts must be declared before implementation
3. **Exegesis Required**: Every declaration must include human-readable explanation
4. **Versioned**: Changes are tracked through explicit evolution declarations
5. **Composable**: Complex behaviors are built from simple, atomic units

### 1.2 File Conventions

- DOL files use the `.dol` extension
- Test files use the `.dol.test` extension
- Each file contains exactly one primary declaration
- Files are UTF-8 encoded

## 2. Lexical Structure

### 2.1 Character Set

DOL source files use the UTF-8 character encoding. The language syntax uses only ASCII characters, but string literals and exegesis text may contain any valid UTF-8.

### 2.2 Whitespace and Comments

Whitespace (spaces, tabs, newlines) is used to separate tokens but is otherwise insignificant. Comments begin with `//` and extend to the end of the line.

```dol
// This is a comment
gene container.exists {  // Inline comment
  container has identity
}
```

### 2.3 Identifiers

Identifiers follow these rules:
- Must begin with a letter (a-z, A-Z)
- May contain letters, digits (0-9), and underscores
- Are case-sensitive
- Convention: use lowercase with underscores for multi-word names

**Qualified Identifiers** use dot notation to create hierarchical names:

```
container.exists
identity.cryptographic
univrs.orchestrator.scheduler
```

### 2.4 Keywords

DOL reserves the following keywords:

| Category | Keywords |
|----------|----------|
| Declarations | `gene`, `trait`, `constraint`, `system`, `evolves`, `exegesis` |
| Predicates | `has`, `is`, `derives`, `from`, `requires`, `uses`, `emits`, `matches`, `never` |
| Evolution | `adds`, `deprecates`, `removes`, `because` |
| Tests | `test`, `given`, `when`, `then`, `always` |
| Quantifiers | `each`, `all`, `no` |
| Instance | `this` |
| Functions (DOL 2.0) | `fun`, `let`, `return` |
| Control Flow (DOL 2.0) | `if`, `else`, `match`, `for`, `while`, `in` |
| Logic (DOL 2.0) | `forall`, `exists`, `law` |

### 2.5 Operators and Delimiters

| Symbol | Meaning |
|--------|---------|
| `{` `}` | Block delimiters |
| `@` | Version marker |
| `>` | Version greater than |
| `>=` | Version greater than or equal |
| `=` | Version exact match |

### 2.6 Literals

**Version Numbers** follow semantic versioning format:
```
0.0.1
1.2.3
10.20.30
```

**String Literals** are enclosed in double quotes:
```
"This is a string"
"With \"escaped\" quotes"
"Line\nbreak"
```

## 3. Declaration Types

Every DOL file contains exactly one declaration followed by an exegesis block.

### 3.1 Gene Declarations

Genes are the atomic units of DOL. They declare fundamental truths that cannot be decomposed further.

**Syntax:**
```ebnf
gene_declaration = "gene" qualified_identifier "{" { gene_statement } "}" exegesis_block
```

**Example:**
```dol
gene container.exists {
  container has identity
  container has state
  container has boundaries
  container has resources
}

exegesis {
  A container is the fundamental unit of workload isolation.
  Every container possesses these four essential properties.
}
```

**Allowed Statements**: `has`, `is`, `derives from`, `requires`

### 3.2 Trait Declarations

Traits compose genes and declare behaviors. They represent composable capabilities.

**Syntax:**
```ebnf
trait_declaration = "trait" qualified_identifier "{" { trait_statement } "}" exegesis_block
```

**Example:**
```dol
trait container.lifecycle {
  uses container.exists
  uses identity.cryptographic

  container is created
  container is started
  container is stopped

  each transition emits event
}

exegesis {
  The container lifecycle defines the state machine that governs
  container execution from creation through termination.
}
```

**Allowed Statements**: `uses`, `has`, `is`, `emits`, quantified statements

### 3.3 Constraint Declarations

Constraints define invariants that must always hold true in the system.

**Syntax:**
```ebnf
constraint_declaration = "constraint" qualified_identifier "{" { constraint_statement } "}" exegesis_block
```

**Example:**
```dol
constraint container.integrity {
  state matches declared
  identity never changes
  boundaries never expand
}

exegesis {
  Container integrity constraints ensure runtime state matches
  the declared ontology. Violations indicate system errors.
}
```

**Allowed Statements**: `matches`, `never`, `has`, `is`

### 3.4 System Declarations

Systems are top-level compositions that bring together traits with version requirements.

**Syntax:**
```ebnf
system_declaration = "system" qualified_identifier "@" version "{" { system_statement } "}" exegesis_block
```

**Example:**
```dol
system univrs.orchestrator @ 0.1.0 {
  requires container.lifecycle >= 0.0.2
  requires node.discovery >= 0.0.1

  all operations is authenticated
  all events is logged
}

exegesis {
  The Univrs orchestrator is the primary system composition
  responsible for container scheduling and coordination.
}
```

**Allowed Statements**: `requires` (with version), `has`, `is`, quantified statements

### 3.5 Evolution Declarations

Evolutions track changes between versions, maintaining accumulative history.

**Syntax:**
```ebnf
evolution_declaration = "evolves" qualified_identifier "@" version ">" version "{" { evolution_statement } "}" exegesis_block
```

**Example:**
```dol
evolves container.lifecycle @ 0.0.2 > 0.0.1 {
  adds container is paused
  adds container is resumed
  deprecates container is suspended

  because "workload migration requires state preservation"
}

exegesis {
  Version 0.0.2 extends the lifecycle with pause and resume
  capabilities to support live migration between nodes.
}
```

**Allowed Statements**: `adds`, `deprecates`, `removes`, `because`

## 4. Statement Types

### 4.1 Has Statement

Declares that a subject possesses a property.

```dol
container has identity
container has state
node has address
```

### 4.2 Is Statement

Declares that a subject exists in a state or exhibits a behavior.

```dol
container is created
container is running
operation is authenticated
```

### 4.3 Derives From Statement

Declares the origin of a subject.

```dol
identity derives from ed25519 keypair
signature derives from private key
hash derives from content
```

### 4.4 Requires Statement

Declares a dependency or requirement.

```dol
identity requires no external authority
operation requires valid token
connection requires encryption
```

### 4.5 Uses Statement

Declares composition by referencing another declaration.

```dol
uses container.exists
uses identity.cryptographic
uses network.core
```

### 4.6 Emits Statement

Declares that an action produces an event.

```dol
transition emits event
operation emits log
error emits alert
```

### 4.7 Matches Statement

Declares an equivalence constraint.

```dol
state matches declared
runtime matches specification
output matches expected
```

### 4.8 Never Statement

Declares a negative constraint.

```dol
identity never changes
boundaries never expand
secrets never logged
```

### 4.9 Quantified Statements

Apply predicates universally.

```dol
each transition emits event
all operations is authenticated
each request is validated
```

## 5. Exegesis

Every declaration MUST include an exegesis block. The exegesis provides human-readable documentation explaining:

- The purpose of the declaration
- Design rationale
- Usage context
- Semantic meaning

**Requirements:**
- Must not be empty
- Should be at least 20 characters (warning if shorter)
- May span multiple lines
- May contain any UTF-8 text

**Example:**
```dol
exegesis {
  The container gene defines the essential properties of a container
  in the Univrs platform. A container is an isolated execution
  environment that encapsulates a workload.

  Identity: Every container has a unique cryptographic identity
  derived from an Ed25519 keypair. This identity is immutable.

  State: Containers exist in discrete states (created, running,
  paused, stopped, archived). State transitions are atomic.
}
```

## 6. Version Requirements

Systems specify version constraints for their dependencies:

| Operator | Meaning | Example |
|----------|---------|---------|
| `>=` | Greater than or equal | `requires dep >= 0.0.2` |
| `>` | Greater than | `requires dep > 0.0.1` |
| `=` | Exact match | `requires dep = 1.0.0` |

Versions follow semantic versioning: `MAJOR.MINOR.PATCH`

## 7. Naming Conventions

### 7.1 Declaration Names

Use qualified identifiers with meaningful hierarchy:

| Type | Convention | Example |
|------|------------|---------|
| Gene | `domain.property` | `container.exists` |
| Trait | `domain.behavior` | `container.lifecycle` |
| Constraint | `domain.invariant` | `container.integrity` |
| System | `product.component` | `univrs.orchestrator` |

### 7.2 General Guidelines

- Use lowercase for all identifiers
- Use dots for namespace hierarchy
- Use underscores for multi-word segments
- Be descriptive but concise

## 8. Semantic Rules

### 8.1 Gene Rules

- Genes cannot use `uses` statements
- Genes represent atomic, indivisible concepts
- Genes should have at least one statement

### 8.2 Trait Rules

- Traits typically include at least one `uses` statement
- Traits should include behavior (`is`) statements
- Dependencies must be valid declaration names

### 8.3 Constraint Rules

- Constraints should include `matches` or `never` statements
- Constraints define invariants, not behaviors

### 8.4 System Rules

- Systems must have a version
- Version requirements must reference valid declarations
- Systems compose traits, not genes directly

### 8.5 Evolution Rules

- The new version must be greater than the parent version
- Evolutions must include at least one change (adds/deprecates/removes)
- The `because` rationale is recommended but optional

## 9. DOL 2.0 Features (v0.7.1+)

DOL 2.0 extends the declarative ontology language with computational capabilities, enabling genes to contain executable methods while preserving the ontology-first philosophy.

### 9.1 The `this` Keyword

The `this` keyword provides instance self-reference within genes and spirits. It refers to the current instance, enabling access to instance fields and methods.

**Semantics:**
- `this` is only valid within gene method bodies
- `this.field` accesses an instance field
- `this.method()` calls an instance method
- `this` cannot be reassigned

**Example:**
```dol
gene Counter {
    has value: Int

    fun get() -> Int {
        return this.value
    }

    fun increment() {
        this.value = this.value + 1
    }

    fun add(n: Int) -> Int {
        this.value = this.value + n
        return this.value
    }
}
```

**Design Rationale:**
DOL uses `this` rather than `self` to:
1. Differentiate DOL's semantics from Rust's `self` conventions
2. Provide broader familiarity (JavaScript, Java, C++, C#, TypeScript)
3. Emphasize that DOL genes are domain objects, not implementation structs

### 9.2 Functions and Methods

Genes can define methods using the `fun` keyword:

```dol
fun method_name(param: Type) -> ReturnType {
    // method body
}
```

**Components:**
- `fun` - Function declaration keyword
- Parameters with type annotations
- Optional return type (void if omitted)
- Body with statements and expressions

### 9.3 Variables and Bindings

The `let` keyword introduces local variable bindings:

```dol
fun example() {
    let x = 42
    let name = "DOL"
    let result = this.compute(x)
}
```

### 9.4 Control Flow

DOL 2.0 supports structured control flow:

**Conditionals:**
```dol
if condition {
    // then branch
} else {
    // else branch
}
```

**Pattern Matching:**
```dol
match value {
    Pattern1 => result1,
    Pattern2 => result2,
    _ => default
}
```

**Loops:**
```dol
for item in collection {
    // loop body
}

while condition {
    // loop body
}
```

### 9.5 Logic Keywords

For formal specification and verification:

- `forall` - Universal quantification
- `exists` - Existential quantification
- `law` - Invariant declarations

```dol
law equivariance {
    forall g in G, forall x in X:
        G.act(g, this.forward(x)) == this.forward(G.act(g, x))
}
```

## 10. Error Handling

### 10.1 Lexical Errors

- Unexpected character in source
- Unterminated string literal
- Invalid version number format
- Invalid escape sequence

### 10.2 Parse Errors

- Missing required exegesis block
- Unexpected token
- Invalid statement structure
- Missing delimiters

### 10.3 Validation Errors

- Empty exegesis
- Invalid identifier format
- Duplicate `uses` statements
- Invalid version numbers

### 10.4 Validation Warnings

- Exegesis too short (< 20 chars)
- Non-standard naming convention
- Missing recommended statements

## 11. Grammar Summary

See `docs/grammar.ebnf` for the complete formal grammar in EBNF notation.

## Appendix A: Complete Example

```dol
// genes/container.exists.dol
gene container.exists {
  container has identity
  container has state
  container has boundaries
  container has resources
  container has image
}

exegesis {
  The container gene defines the essential properties of a container in
  the Univrs platform. A container is an isolated execution environment
  that encapsulates a workload.

  Identity: Every container has a unique cryptographic identity derived
  from an Ed25519 keypair. This identity is immutable for the container's
  lifetime and serves as the basis for all authentication.

  State: Containers exist in discrete states (created, running, paused,
  stopped, archived). State transitions are atomic and authenticated.

  Boundaries: Resource isolation is enforced through Linux namespaces and
  cgroups. A container cannot escape its boundaries.

  Resources: CPU, memory, network, and storage allocations are declared
  and enforced. Resource limits are constraints, not suggestions.

  Image: The container's filesystem derives from an OCI-compliant image.
  The image is immutable; runtime changes use copy-on-write layers.
}
```

## Appendix B: Glossary

| Term | Definition |
|------|------------|
| **Gene** | The atomic unit of DOL; declares fundamental truths |
| **Trait** | Composable behavior built from genes |
| **Constraint** | Invariant that must always hold |
| **System** | Top-level composition with version requirements |
| **Evolution** | Version change record |
| **Exegesis** | Required documentation block |
| **Predicate** | Statement type (has, is, uses, etc.) |
| **Qualified Identifier** | Dot-separated hierarchical name |
| **this** | Instance self-reference keyword (DOL 2.0) |
| **Spirit** | Reactive computation unit with versioning (DOL 2.0) |
