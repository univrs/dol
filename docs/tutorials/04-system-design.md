# Tutorial 4: System Design

> Applies to `dol` **0.8.2**. Every `dol` code block in this tutorial parses as a complete file with
> `metadol::parse_file_all`.

## What are systems?

**Systems** are the top-level compositions in DOL. A system combines traits and genes into a
deployable unit, and it is the place where **version requirements** live.

- **Genes:** atomic properties (`container.exists`)
- **Traits:** behaviours (`container.lifecycle`)
- **Systems:** complete architectures (`container.orchestration`)
- **Evolutions (`evo`):** versioned changes to any of the above

## System syntax

```dol
system container.orchestration @ 1.0.0 {
    requires container.lifecycle >= 1.0.0
    requires container.networking > 0.9.0
    requires container.storage = 1.0.0

    uses container.monitoring

    has max_containers: u32
    has default_network: string

    orchestrator emits system.started
    orchestrator is distributed

    state running_count: u32 = 0
}

docs {
    Basic container orchestration system.
}
```

A system body accepts:

| Form | Meaning |
|---|---|
| `@ X.Y.Z` after the name | The system's own version. Optional; it defaults to `0.0.0`. |
| `requires name >= X.Y.Z` (also `>` and `=`) | Versioned dependency, stored as a `Requirement` |
| `uses name` | Composition, as in traits |
| `has name: type`, `subject is s`, `subject emits e`, ... | The same statements as genes and traits |
| `state name: type [= default]` | Parsed, but **discarded** in 0.8.2 (not stored in the AST) |

Things to know:
- Inside a system, a statement that begins with `requires` is always parsed as a **version
  requirement**, so it needs an operator and a version. To state a plain dependency, use the subject
  form: `orchestrator requires scheduler`.
- The version operators `>=`, `>` and `=` are **recorded, not resolved**. The 0.8.2 toolchain doesn't
  fetch or check dependencies, and there is no `--check-dependencies` flag. The validator only checks
  that each version is valid semver.
- `@` is not a requirement operator (`uses x @1.0.0` does not parse).

## Version requirements

```dol
system user.management @ 1.0.0 {
    requires user.authentication >= 1.0.0
    requires user.authorization > 1.0.0
    requires user.session = 2.1.0
}

docs {
    User management system with minimum, strictly-greater and exact requirements.
}
```

The intended meaning of each operator:
- `>=` means this version or newer.
- `>` means strictly newer.
- `=` means exactly this version.

## Composing traits into a system

```dol
system container.orchestration.basic @ 1.0.0 {
    // Core capabilities, pinned exactly for stability
    requires container.lifecycle = 1.0.0
    requires container.networking = 1.0.0
    requires container.storage = 1.0.0

    // Security: accept newer versions so security fixes are picked up
    requires container.security >= 1.0.0

    // System-level properties
    has max_containers: u32
    has default_network: string
    has storage_driver: string

    // System-level events
    orchestrator emits system.started
    orchestrator emits system.stopped
    orchestrator emits system.capacity_warning

    // Categories
    orchestrator is distributed
    orchestrator is resilient
}

docs {
    Basic container orchestration system.

    Capabilities:
    - Lifecycle management: create, start, stop, destroy
    - Network configuration and connectivity
    - Volume mounting and storage management
    - Security isolation

    Architecture: single host, local storage, bridge networking.
}
```

## Evolution with `evo`

Changes over time are expressed with a top-level `evo` declaration, not with a statement inside the
system. Its header is `evo <name> @ <new version> > <previous version>`:

```dol
system user.management @ 1.0.0 {
    requires user.authentication >= 1.0.0
    requires user.authorization >= 1.0.0
}

docs {
    Initial user management system.
}

evo user.management @ 2.0.0 > 1.0.0 {
    adds manager requires user.sso
    adds manager requires user.mfa
    deprecates manager is password_only
    removes legacy_tokens
    because "enterprise customers require SSO and MFA"
}

docs {
    Version 2.0.0 adds single sign-on and multi-factor authentication.
}
```

An `evo` body accepts:
- `adds <statement>` and `deprecates <statement>` (any gene or trait statement)
- `removes <name>`
- `because "<reason>"`
- `migrate { <statements> }`

The `migrate` block is parsed and then discarded in 0.8.2.

```dol
evo account.balance @ 1.1.0 > 1.0.0 {
    adds has currency: string = "EUR"
    because "multi-currency support"
    migrate {
        let default_currency = "EUR"
    }
}

docs {
    Adds a currency field with a default, so existing balances stay valid.
}
```

The validator checks that both versions are valid semver, warns if the new version is not greater than
the previous one, and warns if the evolution has no `adds`, `deprecates` or `removes`.

These older forms do **not** parse:
- `evolves from x @1.0.0` inside a system
- `evo A > B` without versions
- `added`/`removed` instead of `adds`/`removes`

## A larger example

```dol
system ecommerce.platform @ 1.0.0 {
    // User subsystem
    requires user.authentication = 2.0.0
    requires user.authorization >= 1.0.0

    // Catalog and shopping
    requires product.catalog >= 3.0.0
    requires product.inventory = 1.0.0
    requires shopping.cart = 1.0.0
    requires shopping.checkout >= 2.0.0

    // Payments and orders
    requires payment.processing >= 3.0.0
    requires payment.fraud_detection > 1.0.0
    requires order.management >= 2.0.0

    // Platform properties
    has platform_name: string
    has supported_currencies: Vec<string>
    has max_cart_items: u32
    has session_timeout: u32

    // Platform events
    platform emits platform.started
    platform emits order.placed
    platform emits payment.completed

    platform is event_driven
}

docs {
    Complete e-commerce platform architecture.

    User management, a product catalog with inventory, a shopping cart and
    checkout, payment processing with fraud detection, and order management.
    The platform is event-driven: subsystems communicate via the events
    listed in the system body.
}
```

## Best practices

1. **Document architecture decisions in `docs`:**

```dol
system payment.gateway @ 1.0.0 {
    requires payment.processing >= 3.0.0
    gateway is event_sourced
}

docs {
    Payment gateway system.

    Decision: event sourcing. Payment transactions are event-sourced to keep
    a complete audit trail.

    Decision: circuit breakers. All calls to external payment providers go
    through circuit breakers to prevent cascading failures.
}
```

2. **Choose operators deliberately.** Use `>=` for components where you want fixes, and `=` for
   components whose API you depend on exactly.
3. **Split large systems into subsystems:**

```dol
system platform.auth @ 1.0.0 {
    requires user.authentication >= 1.0.0
    requires session.management >= 1.0.0
}

docs {
    Authentication subsystem.
}

system platform.data @ 1.0.0 {
    requires database.connection >= 1.0.0
    requires cache.management >= 1.0.0
}

docs {
    Data subsystem.
}

system platform.complete @ 1.0.0 {
    requires platform.auth = 1.0.0
    requires platform.data = 1.0.0
}

docs {
    The complete platform, composed of the auth and data subsystems.
}
```

## Checking your systems

```bash
cargo run --features cli --bin dol-parse -- my-system.dol
cargo run --features cli --bin dol-check -- --require-exegesis my-system.dol
```

Both tools parse **only the first declaration** in each file in 0.8.2. Output formats are
`-f pretty|json|compact|debug`. There is no Markdown output.

## Key takeaways

1. **Systems are top-level compositions**, and the only place for `requires name OP version`.
2. **Version operators** are `>=`, `>` and `=`. They are recorded but not resolved by the 0.8.2
   toolchain.
3. **Evolution** is a separate declaration: `evo name @ new > old { adds ... because "..." }`.
4. **Document architecture** in `docs`.
