# DOL Feature: Genes

**Generated:** $(date)
**Status:** PARSING & VALIDATION WORKING | WASM NOT YET SUPPORTED

---

## Overview

Genes are DOL's core data abstraction - similar to structs or classes in other languages, but with built-in ontological semantics. A Gene defines an entity with properties (using `has`) and behaviors.

### Gene Syntax

```dol
gene <domain>.<name> {
    <entity> has <property>
    <entity> has <property>: <type>
}
```

---

## Building DOL

```bash
cargo build --features cli 2>&1 | tail -3
   Compiling dol v0.4.0 (/home/ardeshir/repos/univrs-dol)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.51s
```

## Gene Examples

### Example 1: Hello World Gene

The simplest gene - a message with basic properties:

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

  This demonstrates the basic pattern:
  - gene <domain>.<name> declares a new gene
  - <entity> has <property> declares properties
  - exegesis explains the intent and usage
}
```

**Parse Result:**
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/dol-parse examples/genes/hello.world.dol`
✓ examples/genes/hello.world.dol (hello.world)
    hello.world gene with 3 statements

Summary
  Total:    1
  Success:  1
```

### Example 2: Counter Gene

A stateful gene with typed properties:

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

  Properties:
  - value: Current count (integer)
  - minimum: Lower bound (typically 0)
  - maximum: Upper bound (optional, may be unbounded)

  The "derives from" pattern indicates that the initial state comes from
  some configuration or initialization source.
}
```

**Parse Result:**
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/dol-parse examples/genes/counter.dol`
✓ examples/genes/counter.dol (counter.state)
    counter.state gene with 4 statements

Summary
  Total:    1
  Success:  1
```

### Example 3: Container Exists Gene

A domain-specific gene for container management:

```dol
// DOL v0.0.1 - Metal Primitives
// genes/container.exists.dol

gene container.exists {
  container has identity
  container has status
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

**Parse Result:**
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/dol-parse examples/genes/container.exists.dol`
✓ examples/genes/container.exists.dol (container.exists)
    container.exists gene with 5 statements

Summary
  Total:    1
  Success:  1
```

### Example 4: Cryptographic Identity Gene

```dol
// DOL v0.0.1 - Metal Primitives
// genes/identity.cryptographic.dol

gene identity.cryptographic {
  identity has keypair
  identity has signature
  identity has public_key
  identity has private_key
  identity derives from ed25519 keypair
}

exegesis {
  The identity.cryptographic gene establishes the foundational cryptographic
  properties that enable secure, verifiable identity in the Univrs platform.
  Every entity in the system possesses a cryptographic identity that serves
  as the root of trust for all operations.

  Keypair: The identity is anchored in an Ed25519 elliptic curve keypair,
  chosen for its security properties, performance characteristics, and
  resistance to side-channel attacks. The keypair generation uses
  cryptographically secure randomness and follows RFC 8032.

  Public Key: The public key component is openly shared and serves as the
  canonical identifier for the entity. It enables verification of signatures
  and supports public key infrastructure operations without exposing sensitive
  material.

  Private Key: The private key is maintained in secure storage and never
  leaves its protection boundary. All signing operations occur within the
  security context that controls the private key, ensuring it cannot be
  extracted or compromised.

  Signature: Digital signatures created with the private key provide
  cryptographic proof of authenticity and non-repudiation. Any entity with
  access to the public key can verify that a message was signed by the
  corresponding private key holder.

  This gene establishes the primitive from which all authentication,
  authorization, and trust relationships derive. Without cryptographic
  identity, no secure operations can occur in the Univrs platform.
}
```


---

## Gene Feature Status

| Feature | Status | Notes |
|---------|--------|-------|
| Gene declaration | WORKING | `gene name { }` |
| Entity properties | WORKING | `entity has property` |
| Typed properties | WORKING | `entity has property: Type` |
| Exegesis docs | WORKING | Full documentation support |
| Parse to AST | WORKING | Complete |
| Validate | WORKING | Semantic checks pass |
| WASM compile | NOT YET | Requires struct lowering |

---

## WASM Status

Genes do not currently compile to WASM. The error when attempting:

```
WasmError: Only function declarations can be compiled to WASM
```

**Roadmap:** Gene WASM support requires implementing struct layouts and memory management in the WASM backend.

---

*Generated by DOL Feature Demo Script*
