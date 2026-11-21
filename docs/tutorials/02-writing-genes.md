# Tutorial 2: Writing Genes

> Applies to `dol` **0.8.2**. Every `dol` code block in this tutorial parses as a complete file with
> `metadol::parse_file_all`.

## What are genes?

In DOL, a **gene** (`gen`) is the atomic unit of your ontology. Genes represent fundamental properties
or capabilities that you don't decompose further:
- A user **exists**
- A container **has** data
- A file **is** readable
- An entity **derives from** an origin

### Gene philosophy

- Each gene captures **one** concept.
- Genes compose into traits ([Tutorial 3](03-composing-traits.md)).
- Changes to a gene are recorded as `evo` declarations ([Tutorial 4](04-system-design.md#evolution-with-evo)).

## Gene syntax

```dol
gen user.account {
    has username: string
    has email: string
    user has identity
    user is persistent
    user derives from registration
    user requires email.verification
}

docs {
    A user account in the system. This text explains what the gene
    represents and why it exists in the domain ontology.
}
```

The body is a list of statements. The trailing `docs { ... }` block documents the gene. It can also
appear inside the braces as the last item.

Things that are **not** part of gene syntax in 0.8.2:
- **No version in the header.** `gen user.account @1.0.0 {` is a parse error. Versions belong to
  `module`, `system` and `evo` declarations.
- **No version on references.** `requires x @1.0.0` and `uses x @1.0.0` do not parse.
  Version requirements (`requires x >= 1.0.0`) exist only inside a `system`.

## Naming conventions

### Qualified identifiers

Dots are part of an identifier, so you can express a domain hierarchy:

```dol
gen user.exists {
    has id: string
}

gen user.authenticated {
    has session_token: string
}

gen user.profile.complete {
    has display_name: string
}
```

Best practices:
- Use the `domain.property` pattern.
- Start with the entity (user, container, file) and end with the property (exists, authenticated).

The validator also accepts PascalCase gene names such as `Container` or `Vec3`, which are common for
data-like genes.

## The `docs` block

DOL's convention is that every declaration carries documentation. In 0.8.2:
- The parser accepts a gene without docs.
- `dol-parse` and the validator warn when docs are shorter than 20 characters.
- `dol-check --require-exegesis` fails when docs are missing.
  `--min-exegesis-length` sets the minimum length (default 20).

### Good docs

```dol
gen container.lifecycle.started {
    has started_at: u64
    has pid: u32
}

docs {
    Represents a container that has been started and is currently running.
    A started container has an active process, consumes resources, and can
    receive signals. This state is entered after successful initialization
    and before the container stops or fails.
}
```

### Docs that are too thin

```dol
gen container.lifecycle.started {
    has started_at: u64
}

docs {
    Started.
}
```

This parses, but `dol-parse` warns that the docs are unusually short.

Good docs answer:
- What does this gene represent?
- Why does it exist in your ontology?
- When does it apply, and what follows from it?

Avoid `{` and `}` inside docs text. The 0.8.2 parser stops at the first closing brace.

## Statements

### 1. Typed fields: `has name: type`

```dol
gen container.configured {
    has image: string
    has memory_limit: u64
    has cpu_shares: u32
    has environment: Map<string, string>
    has volumes: Vec<string>
    has restart_count: u32 = 0
    has label: Option<string>
}

docs {
    A container with runtime configuration settings.
}
```

Types:
- **Integers:** `i8`, `i16`, `i32`, `i64`, `i128`, `u8`, `u16`, `u32`, `u64`, `u128`
- **Other primitives:** `f32`, `f64`, `bool`, `string`, unit `()`
- **Generics:** `Vec<T>`, `Option<T>`, `Map<K, V>`. Any `Name<...>` is accepted syntactically.
- **Tuples and function types:** `(i64, string)`, `(i64) -> bool`
- **Inline enums:** `enum { A, B, C }`
- **Other gene names:** e.g. `has owner: user.account`

A field can have a default (`= expr`) and a constraint (`where expr`):

```dol
gen account.balance {
    has amount: i64 = 0
    has currency: string = "EUR"
    has limit: i64 where limit >= 0
    has status: enum { Open, Frozen, Closed }
}

docs {
    An account balance with a default currency and a non-negative limit.
}
```

`where` constraints are parsed and stored. In 0.8.2 they are not checked at runtime by the parser or
validator.

The `name: type` shorthand without `has` is also accepted (`count: i32 = 0`).

### 2. Untyped properties: `subject has property`

```dol
gen container.exists {
    container has identity
    container has state
    container has boundaries
}

docs {
    A container exists with an identity, a state and resource boundaries.
}
```

### 3. Categories and states: `subject is state`

The subject is required. A bare `is entity` does not parse.

```dol
gen user.account {
    has username: string
    has email: string

    account is entity
    account is persistent
    account is auditable
}

docs {
    A user account in the system, stored persistently and audited.
}
```

### 4. Origin: `subject derives from origin`

```dol
gen user.admin {
    admin derives from user.account
    has access_level: u8
    admin is privileged
}

docs {
    An administrative user with elevated privileges.
}
```

`derives from` records a relationship in the AST. To name a parent gene, use `extends` in the
header. It is stored in the AST's `extends` field:

```dol
gen user.base {
    has id: string
}

docs {
    The base fields shared by every kind of user.
}

gen user.admin extends user.base {
    has access_level: u8
}

docs {
    An administrator: a base user plus an access level.
}
```

### 5. Dependencies: `subject requires dependency`

```dol
gen container.networked {
    container requires container.exists
    has ip_address: string
    has hostname: string
    container is networked
}

docs {
    A container with network connectivity.
}
```

## Complete example

```dol
gen user.authenticated {
    // Core identity
    has user_id: string
    has username: string
    has email: string

    // Session tracking
    has session_token: string
    has session_expiry: u64
    has last_login: u64

    // Dependencies and categories
    session requires user.exists
    session is authenticated
    session is auditable
}

docs {
    Represents an authenticated user session in the system.

    An authenticated user has verified their identity through credentials
    and has been granted a session token. Authentication is distinct from
    authorization: this gene only confirms identity, not permissions.
}
```

## Best practices

### Keep genes atomic

Separate concerns instead of building one large gene:

```dol
gen user.exists {
    has username: string
    has email: string
}

docs {
    A user exists with a username and an email address.
}

gen user.profile {
    profile requires user.exists
    has profile_picture: string
}

docs {
    Optional profile data for an existing user.
}

gen user.social {
    social requires user.exists
    has posts: Vec<string>
    has followers: Vec<string>
}

docs {
    Social graph data for an existing user.
}
```

### Record changes with `evo`

Don't redeclare a gene with a new version number. Add an evolution:

```dol
gen user.profile {
    has name: string
    has email: string
}

docs {
    A user profile.
}

evo user.profile @ 1.1.0 > 1.0.0 {
    adds has avatar: string
    because "users asked for profile pictures"
}

docs {
    Adds an avatar to the user profile, a backward-compatible addition.
}
```

`evo` bodies accept `adds <statement>`, `deprecates <statement>`, `removes <name>`,
`because "<reason>"` and `migrate { ... }`.

## Testing your genes

```bash
cargo run --features cli --bin dol-parse -- my-genes.dol
cargo run --features cli --bin dol-check -- --require-exegesis my-genes.dol
```

Remember that in 0.8.2 both tools only look at the **first declaration** in each file. For a
multi-declaration file, either split it or check it from Rust with `metadol::parse_file_all`.

## Common patterns

### Existence

```dol
gen entity.exists {
    has id: string
    entity is identifiable
}

docs {
    Base existence for any entity: it has a stable identifier.
}
```

### States as an enum field

```dol
gen task.status {
    has state: enum { Pending, Running, Completed, Failed }
}

docs {
    The current lifecycle state of a task.
}
```

### Capabilities

```dol
gen file.capabilities {
    file is readable
    file is writable
    file is executable
}

docs {
    The capabilities a file may have.
}
```

## Not yet supported in 0.8.2

These forms do not parse:
- versioned gene headers (`gen name @1.0.0 {`)
- versioned references (`requires x @1.0.0`, `uses x @1.0.0`)
- bare predicates without a subject (`is entity`, `emits x`)
- `subject uses x` (only a bare `uses x` statement, used in traits)
- `type X is enum { ... }` (use a `has kind: enum { ... }` field)
- `&T`, `[T]`, `let mut`, `+=`, generic functions (`fun id<T>`)

Two forms are accepted but discarded:
- Generic parameters on `gen` and `trait` (`gen Box<T>`) are skipped.
- A `rule` nested inside a `gen` body is reduced to its name; its body is ignored.

## Next steps

- [Tutorial 3: Composing Traits](03-composing-traits.md)
- [Tutorial 4: System Design](04-system-design.md)

## Key takeaways

1. **Genes are atomic:** one concept per gene.
2. **Document every gene** with `docs`. `dol-check --require-exegesis` enforces it.
3. **Use qualified identifiers** (`domain.property`).
4. **Versions are not on gene headers.** Use `evo` to record changes.
5. **Statements:** `has name: type`, `subject has p`, `subject is s`, `subject derives from o`,
   `subject requires d`.
