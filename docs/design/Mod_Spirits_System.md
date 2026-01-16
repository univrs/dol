# DOL 2.0: Modules, Spirits & Systems

> **Canonical Specification v1.0.0**  
> **Status:** Ratified  
> **Last Updated:** January 15, 2026  
> **Repository:** github.com/univrs/dol

---

## Exegesis

This document defines DOL's composition model: how code is organized into **modules**, packaged as **Spirits**, and deployed as **Systems**. The key insight driving this design:

> **"Spirit is just a gene with package semantics — no new concepts to learn."**

Every DOL construct follows the same pattern: declare what something *is* before what it *does*. A gene describes a data type. A trait describes behavior. A Spirit describes a package. A System describes a deployment. Same syntax. Same mental model. Different semantic scope.

---

## Table of Contents

1. [Composition Hierarchy](#composition-hierarchy)
2. [Modules](#modules)
3. [Spirits](#spirits)
4. [Systems](#systems)
5. [Module Resolution](#module-resolution)
6. [Visibility Rules](#visibility-rules)
7. [Example 1: Math Library Spirit](#example-1-math-library-spirit)
8. [Example 2: HTTP Service Spirit](#example-2-http-service-spirit)
9. [Example 3: Multi-Module Spirit](#example-3-multi-module-spirit)
10. [Example 4: System Composition](#example-4-system-composition)

---

## Composition Hierarchy

```
┌─────────────────────────────────────────────────────────────────┐
│                    COMPOSITION HIERARCHY                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   FILE LEVEL          PACKAGE LEVEL         DEPLOYMENT LEVEL    │
│   ───────────         ─────────────         ────────────────    │
│                                                                  │
│   ┌─────────┐         ┌─────────────┐       ┌──────────────┐   │
│   │   mod   │ ──────► │   spirit    │ ────► │   system     │   │
│   │         │  many   │             │ many  │              │   │
│   │ genes   │   to    │   mods      │  to   │  spirits     │   │
│   │ traits  │   one   │   lib.dol   │  one  │  constraints │   │
│   │ funs    │         │   bins      │       │  orchestrate │   │
│   └─────────┘         └─────────────┘       └──────────────┘   │
│                                                                  │
│   Visibility:         Visibility:           Visibility:         │
│   pub(parent)         pub(spirit)           pub                 │
│   private             pub                   (all public)        │
│                                                                  │
│   Scope:              Scope:                Scope:              │
│   Single file         Package               Deployment          │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

| Level | Contains | Purpose | Artifact |
|-------|----------|---------|----------|
| **mod** | genes, traits, funs, types | Organize code within a package | `.dol` file |
| **spirit** | mods, entry points | Distribute reusable packages | `.wasm` binary |
| **system** | spirits, constraints | Deploy composed applications | Running instance |

---

## Modules

A **module** is a single `.dol` file. The module path is derived from the file path relative to `src/`.

### Module Path Rules

```
File Path                           Module Path
─────────────────────────────────   ─────────────────────────
src/container.dol                   mod container
src/lifecycle.dol                   mod lifecycle
src/container/state.dol             mod container.state
src/container/config.dol            mod container.config
src/internal/helpers/utils.dol      mod internal.helpers.utils
```

**Rule:** Directory separators become dots. File extension is stripped.

### Module Declaration (Implicit)

Modules are implicitly declared by their file path. No explicit `mod` keyword needed at file level:

```dol
// src/container.dol
// This file IS mod container — no declaration needed

pub gene Container {
    has id: UInt64
    has name: String
}

pub fun create(name: String) -> Container {
    return Container { id: generate_id(), name: name }
}
```

### Inline Submodules (Explicit)

For small, tightly-coupled code, declare submodules inline:

```dol
// src/container.dol

pub gene Container { ... }

// Inline submodule
mod state {
    pub gene ContainerState {
        type: enum { Created, Running, Stopped, Failed }
    }
}

// Access: container.state.ContainerState
```

---

## Spirits

A **Spirit** is DOL's package unit — a shareable collection of modules with metadata, dependencies, and entry points.

### Key Insight

**Spirit follows gene syntax exactly.** Same keywords, same structure, different semantic scope:

| Gene | Spirit |
|------|--------|
| `has field: Type` | `has name: "my-spirit"` |
| `constraint valid { }` | `constraint valid_version { }` |
| `exegesis { }` | `exegesis { }` |
| — | `requires @org/pkg: "^1.0"` |
| — | `has lib: "src/lib.dol"` |

### Spirit.dol Format

```dol
spirit Name {
    // ═══════════════════════════════════════════════════════════
    // IDENTITY (required)
    // ═══════════════════════════════════════════════════════════
    has name: String        // Package name (e.g., "my-spirit")
    has version: String     // Semantic version (e.g., "1.0.0")
    
    // ═══════════════════════════════════════════════════════════
    // METADATA (optional)
    // ═══════════════════════════════════════════════════════════
    has authors: List<String>
    has license: String
    has repository: String
    has description: String
    
    // ═══════════════════════════════════════════════════════════
    // DEPENDENCIES
    // ═══════════════════════════════════════════════════════════
    requires @org/package: "version"
    requires @git:github.com/org/repo: "branch"
    requires @https://url/file.dol: { sha256: "hash" }
    
    // ═══════════════════════════════════════════════════════════
    // ENTRY POINTS
    // ═══════════════════════════════════════════════════════════
    has lib: String         // Library entry (e.g., "src/lib.dol")
    has bin: String         // Binary entry (e.g., "src/main.dol")
    // OR for multiple binaries:
    has bins: List<{ name: String, path: String }>
    
    // ═══════════════════════════════════════════════════════════
    // CONSTRAINTS (same as genes!)
    // ═══════════════════════════════════════════════════════════
    constraint name {
        expression
    }
    
    // ═══════════════════════════════════════════════════════════
    // DOCUMENTATION (same as genes!)
    // ═══════════════════════════════════════════════════════════
    exegesis {
        Human-readable description of this Spirit.
    }
}
```

### Spirit File Structure

```
my-spirit/
├── Spirit.dol              # Package manifest
├── src/
│   ├── lib.dol             # Library entry (pub exports)
│   ├── main.dol            # Binary entry (optional)
│   ├── module_a.dol        # mod module_a
│   ├── module_b.dol        # mod module_b
│   └── subdir/
│       └── nested.dol      # mod subdir.nested
└── tests/
    └── test_module_a.dol   # Test files
```

---

## Systems

A **System** composes multiple Spirits for deployment. It defines cross-Spirit constraints, shared state, and orchestration logic.

```dol
system Name {
    // ═══════════════════════════════════════════════════════════
    // SPIRIT DEPENDENCIES
    // ═══════════════════════════════════════════════════════════
    uses @org/spirit_a: "^1.0"
    uses @org/spirit_b: "^2.0"
    
    // ═══════════════════════════════════════════════════════════
    // SYSTEM STATE
    // ═══════════════════════════════════════════════════════════
    has config: SystemConfig
    has instances: List<Instance>
    
    // ═══════════════════════════════════════════════════════════
    // CROSS-SPIRIT CONSTRAINTS
    // ═══════════════════════════════════════════════════════════
    constraint invariant_name {
        // Constraints across Spirit boundaries
    }
    
    // ═══════════════════════════════════════════════════════════
    // ORCHESTRATION
    // ═══════════════════════════════════════════════════════════
    fun deploy() -> Result<Void, Error> { ... }
    fun scale(count: UInt32) -> Result<Void, Error> { ... }
    
    exegesis { ... }
}
```

---

## Module Resolution

### Resolution Hierarchy

```
┌─────────────────────────────────────────────────────────────────┐
│                    MODULE RESOLUTION                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  SYNTAX                         RESOLVES TO                      │
│  ──────                         ───────────                      │
│                                                                  │
│  LOCAL (within Spirit)                                           │
│  ────────────────────                                            │
│  use container                  → src/container.dol              │
│  use container.state            → src/container/state.dol        │
│  use internal.helpers           → src/internal/helpers.dol       │
│                                                                  │
│  REGISTRY (published packages)                                   │
│  ─────────────────────────────                                   │
│  use @univrs/std                → registry lookup                │
│  use @univrs/std.io             → registry + src/io.dol          │
│  use @univrs/std.io.println     → registry + src/io.dol.println  │
│                                                                  │
│  GIT (private/experimental)                                      │
│  ──────────────────────────                                      │
│  use @git:github.com/org/repo   → git clone + cache              │
│  use @git:github.com/o/r.util   → git + src/util.dol             │
│                                                                  │
│  HTTPS (single-file utilities)                                   │
│  ─────────────────────────────                                   │
│  use @https://example.com/u.dol → HTTP fetch + cache             │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Resolution Algorithm

```
resolve(path):
    if path.starts_with("@https://"):
        → fetch_http(path)           // Cache: ~/.dol/http/
    else if path.starts_with("@git:"):
        → clone_or_fetch_git(path)   // Cache: ~/.dol/git/
    else if path.starts_with("@"):
        → lookup_registry(path)      // Cache: ~/.dol/cache/
    else:
        → resolve_local("src/" + path.replace(".", "/") + ".dol")
```

### Import Sources

| Prefix | Example | Use Case | Cache Location |
|--------|---------|----------|----------------|
| (none) | `use container` | Local modules | N/A |
| `@org/name` | `use @univrs/std` | Published packages | `~/.dol/cache/` |
| `@git:` | `use @git:github.com/...` | Private/experimental | `~/.dol/git/` |
| `@https://` | `use @https://cdn.io/...` | Single-file utilities | `~/.dol/http/` |

### Version Specifiers

| Specifier | Meaning | Example |
|-----------|---------|---------|
| `"^1.0"` | Compatible with 1.x | `requires @univrs/std: "^1.0"` |
| `"~1.2"` | Patch updates only | `requires @univrs/std: "~1.2"` |
| `"=1.2.3"` | Exact version | `requires @univrs/std: "=1.2.3"` |
| `"main"` | Git branch | `requires @git:.../repo: "main"` |
| `"v1.0.0"` | Git tag | `requires @git:.../repo: "v1.0.0"` |
| `{ sha256: "..." }` | Hash-pinned | `requires @https://.../f.dol: { sha256: "abc" }` |

---

## Visibility Rules

| Modifier | Scope | Rust Equivalent |
|----------|-------|-----------------|
| (none) | Private — same module only | (default) |
| `pub` | Public — accessible everywhere | `pub` |
| `pub(spirit)` | Package — within Spirit only | `pub(crate)` |
| `pub(parent)` | Parent — direct parent module | `pub(super)` |

### Visibility Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                      VISIBILITY BOUNDARIES                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Spirit A                          Spirit B                      │
│  ┌─────────────────────────┐       ┌─────────────────────────┐  │
│  │                         │       │                         │  │
│  │  mod x                  │       │  mod y                  │  │
│  │  ┌───────────────────┐  │       │  ┌───────────────────┐  │  │
│  │  │ private item      │──┼───X───┼──│ cannot access     │  │  │
│  │  │ pub(parent) item  │──┼───X───┼──│ cannot access     │  │  │
│  │  │ pub(spirit) item  │──┼───X───┼──│ cannot access     │  │  │
│  │  │ pub item          │──┼───────┼─►│ CAN access        │  │  │
│  │  └───────────────────┘  │       │  └───────────────────┘  │  │
│  │                         │       │                         │  │
│  │  mod x.child            │       │                         │  │
│  │  ┌───────────────────┐  │       │                         │  │
│  │  │ pub(parent) item  │◄─┼─┐     │                         │  │
│  │  └───────────────────┘  │ │     │                         │  │
│  │           ▲             │ │     │                         │  │
│  │           │ parent can  │ │     │                         │  │
│  │           │ access      │ │     │                         │  │
│  │           └─────────────┼─┘     │                         │  │
│  │                         │       │                         │  │
│  │  pub(spirit) shared ◄───┼───────┼── cannot access         │  │
│  │       ▲                 │       │                         │  │
│  │       │ same Spirit     │       │                         │  │
│  │       │ CAN access      │       │                         │  │
│  │                         │       │                         │  │
│  └─────────────────────────┘       └─────────────────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Example 1: Math Library Spirit

A simple, pure-function library Spirit with no dependencies.

### File Structure

```
math-spirit/
├── Spirit.dol
├── src/
│   ├── lib.dol
│   ├── arithmetic.dol
│   └── trig.dol
└── tests/
    └── arithmetic_test.dol
```

### Spirit.dol

```dol
spirit MathLib {
    has name: "math-lib"
    has version: "1.0.0"
    has authors: ["VUDO Team <team@univrs.io>"]
    has license: "MIT"
    
    has lib: "src/lib.dol"
    
    exegesis {
        Pure mathematical functions for DOL.
        No side effects, no dependencies.
        
        Quick start:
            use @univrs/math.{ add, sin, cos }
            result = add(1, 2)  // 3
    }
}
```

### src/lib.dol

```dol
// Public API — re-export from internal modules
pub use arithmetic.{ add, subtract, multiply, divide }
pub use trig.{ sin, cos, tan, PI }
```

### src/arithmetic.dol

```dol
// mod arithmetic

pub fun add(a: Int64, b: Int64) -> Int64 {
    return a + b
}

pub fun subtract(a: Int64, b: Int64) -> Int64 {
    return a - b
}

pub fun multiply(a: Int64, b: Int64) -> Int64 {
    return a * b
}

pub fun divide(a: Int64, b: Int64) -> Option<Int64> {
    if b == 0 {
        return None
    }
    return Some(a / b)
}

// Package-internal helper (not exported from lib.dol)
pub(spirit) fun gcd(a: Int64, b: Int64) -> Int64 {
    if b == 0 { return a }
    return gcd(b, a % b)
}
```

### src/trig.dol

```dol
// mod trig

pub const PI: Float64 = 3.14159265358979323846

pub fun sin(x: Float64) -> Float64 {
    // Taylor series implementation
    return taylor_sin(x)
}

pub fun cos(x: Float64) -> Float64 {
    return sin(x + PI / 2.0)
}

pub fun tan(x: Float64) -> Float64 {
    return sin(x) / cos(x)
}

// Private implementation detail
fun taylor_sin(x: Float64) -> Float64 {
    // Implementation...
}
```

### tests/arithmetic_test.dol

```dol
use arithmetic.{ add, subtract, divide }

test "add returns sum" {
    assert_eq(add(2, 3), 5)
    assert_eq(add(-1, 1), 0)
}

test "divide by zero returns None" {
    assert_eq(divide(10, 0), None)
    assert_eq(divide(10, 2), Some(5))
}
```

---

## Example 2: HTTP Service Spirit

A Spirit with dependencies, side effects, and a binary entry point.

### File Structure

```
http-service/
├── Spirit.dol
├── src/
│   ├── lib.dol
│   ├── main.dol
│   ├── server.dol
│   ├── routes.dol
│   └── sex/
│       └── io.dol
└── tests/
    └── routes_test.dol
```

### Spirit.dol

```dol
spirit HttpService {
    has name: "http-service"
    has version: "0.1.0"
    has authors: ["Your Name <you@example.com>"]
    has license: "Apache-2.0"
    
    requires @univrs/std: "^1.0"
    requires @univrs/http: "^2.0"
    requires @univrs/json: "^1.0"
    
    has lib: "src/lib.dol"
    has bin: "src/main.dol"
    
    constraint valid_port {
        this.default_port > 0 && this.default_port < 65536
    }
    
    exegesis {
        HTTP service framework for building web APIs.
        
        Usage:
            use @univrs/http-service.{ Server, route }
            
            server = Server.new(8080)
            server.route("/api/users", handle_users)
            server.start()
    }
}
```

### src/lib.dol

```dol
// Public API
pub use server.{ Server, ServerConfig }
pub use routes.{ route, Route, Handler }

// Re-export common types from dependencies
pub use @univrs/http.{ Request, Response, Method }
pub use @univrs/json.{ Json, parse, stringify }
```

### src/server.dol

```dol
// mod server

use @univrs/http.{ TcpListener, Request, Response }
use routes.{ Route, Handler }
use sex.io.{ println }

pub gene Server {
    has port: UInt16
    has routes: List<Route>
    has config: ServerConfig
    
    pub fun new(port: UInt16) -> Server {
        return Server {
            port: port,
            routes: [],
            config: ServerConfig.default()
        }
    }
    
    pub fun route(self, path: String, handler: Handler) -> Self {
        this.routes.push(Route { path: path, handler: handler })
        return self
    }
    
    // Side effect: starts listening on network
    pub sex fun start(self) -> Result<Void, Error> {
        sex {
            println("Starting server on port " + this.port)
        }
        
        listener = TcpListener.bind(this.port)?
        
        loop {
            connection = listener.accept()?
            this.handle_connection(connection)
        }
    }
}

pub gene ServerConfig {
    has timeout_ms: UInt64
    has max_connections: UInt32
    
    pub fun default() -> ServerConfig {
        return ServerConfig {
            timeout_ms: 30000,
            max_connections: 1000
        }
    }
}
```

### src/routes.dol

```dol
// mod routes

use @univrs/http.{ Request, Response }

pub gene Route {
    has path: String
    has handler: Handler
}

pub type Handler = Fun<Request, Response>

pub fun route(path: String, handler: Handler) -> Route {
    return Route { path: path, handler: handler }
}

// Package-internal routing logic
pub(spirit) fun match_route(routes: List<Route>, path: String) -> Option<Route> {
    for r in routes {
        if matches(r.path, path) {
            return Some(r)
        }
    }
    return None
}

fun matches(pattern: String, path: String) -> Bool {
    // Pattern matching implementation...
}
```

### src/main.dol

```dol
// Binary entry point

use server.Server
use routes.route
use @univrs/http.{ Request, Response }
use sex.io.println

pub fun main(args: List<String>) -> Int32 {
    port = parse_port(args).unwrap_or(8080)
    
    server = Server.new(port)
        .route("/", handle_root)
        .route("/api/health", handle_health)
    
    sex {
        println("Server configured")
    }
    
    match server.start() {
        Ok(_) { return 0 }
        Err(e) {
            sex { println("Error: " + e.message) }
            return 1
        }
    }
}

fun handle_root(req: Request) -> Response {
    return Response.ok("Welcome to DOL HTTP Service")
}

fun handle_health(req: Request) -> Response {
    return Response.json({ status: "healthy" })
}

fun parse_port(args: List<String>) -> Option<UInt16> {
    if args.len > 1 {
        return args[1].parse()
    }
    return None
}
```

### src/sex/io.dol

```dol
// mod sex.io — Side effect operations

sex fun println(msg: String) -> Void {
    @univrs/std.io.stdout().write_line(msg)
}

sex fun read_line() -> String {
    return @univrs/std.io.stdin().read_line()
}
```

---

## Example 3: Multi-Module Spirit

A larger Spirit demonstrating nested modules and visibility.

### File Structure

```
container-spirit/
├── Spirit.dol
├── src/
│   ├── lib.dol
│   ├── container.dol
│   ├── container/
│   │   ├── state.dol
│   │   ├── config.dol
│   │   └── runtime.dol
│   ├── lifecycle.dol
│   └── internal/
│       └── helpers.dol
└── tests/
    ├── container_test.dol
    └── lifecycle_test.dol
```

### Spirit.dol

```dol
spirit ContainerLib {
    has name: "container-lib"
    has version: "2.0.0"
    has authors: ["VUDO Team <team@univrs.io>"]
    has license: "MIT"
    has repository: "https://github.com/univrs/container-lib"
    
    requires @univrs/std: "^1.0"
    
    has lib: "src/lib.dol"
    
    exegesis {
        Container management library for VUDO OS.
        
        Provides genes for container lifecycle management:
        - Container: The core container gene
        - ContainerState: State machine for container status
        - Lifecycle functions: start, stop, restart, pause
        
        Example:
            use @univrs/containers.{ Container, start, stop }
            
            container = Container.new("my-app")
            start(container)
            // ... do work ...
            stop(container)
    }
}
```

### src/lib.dol

```dol
// Public API exports

// Core types
pub use container.Container
pub use container.state.ContainerState
pub use container.config.ContainerConfig

// Lifecycle operations
pub use lifecycle.{ start, stop, restart, pause, resume }

// Error types
pub use container.runtime.{ ContainerError, RuntimeError }
```

### src/container.dol

```dol
// mod container

use container.state.ContainerState
use container.config.ContainerConfig
use internal.helpers.generate_id

pub gene Container {
    has id: UInt64
    has name: String
    has state: ContainerState
    has config: ContainerConfig
    
    constraint valid_name {
        this.name.len > 0 && this.name.len <= 255
    }
    
    constraint valid_id {
        this.id > 0
    }
    
    pub fun new(name: String) -> Container {
        return Container {
            id: generate_id(),
            name: name,
            state: ContainerState.Created,
            config: ContainerConfig.default()
        }
    }
    
    pub fun with_config(name: String, config: ContainerConfig) -> Container {
        return Container {
            id: generate_id(),
            name: name,
            state: ContainerState.Created,
            config: config
        }
    }
    
    exegesis {
        Container represents an isolated execution environment.
        Each container has a unique ID, name, state, and configuration.
    }
}
```

### src/container/state.dol

```dol
// mod container.state

pub gene ContainerState {
    type: enum {
        Created,
        Starting,
        Running,
        Paused,
        Stopping,
        Stopped,
        Failed(message: String)
    }
    
    pub fun is_active(self) -> Bool {
        match self {
            Running | Paused { return true }
            _ { return false }
        }
    }
    
    pub fun can_start(self) -> Bool {
        match self {
            Created | Stopped { return true }
            _ { return false }
        }
    }
    
    pub fun can_stop(self) -> Bool {
        match self {
            Running | Paused { return true }
            _ { return false }
        }
    }
    
    exegesis {
        ContainerState represents the lifecycle state machine.
        
        State transitions:
            Created → Starting → Running
            Running → Paused → Running
            Running → Stopping → Stopped
            Any → Failed
    }
}
```

### src/container/config.dol

```dol
// mod container.config

pub gene ContainerConfig {
    has memory_limit_mb: UInt64
    has cpu_shares: UInt32
    has network_enabled: Bool
    has environment: Map<String, String>
    
    constraint valid_memory {
        this.memory_limit_mb >= 64
    }
    
    pub fun default() -> ContainerConfig {
        return ContainerConfig {
            memory_limit_mb: 512,
            cpu_shares: 1024,
            network_enabled: true,
            environment: Map.new()
        }
    }
    
    pub fun with_memory(self, mb: UInt64) -> Self {
        this.memory_limit_mb = mb
        return self
    }
    
    pub fun with_env(self, key: String, value: String) -> Self {
        this.environment.insert(key, value)
        return self
    }
}
```

### src/container/runtime.dol

```dol
// mod container.runtime

pub gene ContainerError {
    type: enum {
        InvalidState(expected: String, actual: String),
        ConfigurationError(message: String),
        ResourceExhausted(resource: String),
        RuntimeError(inner: RuntimeError)
    }
}

pub gene RuntimeError {
    has code: Int32
    has message: String
    has source: Option<String>
}
```

### src/lifecycle.dol

```dol
// mod lifecycle

use container.Container
use container.state.ContainerState
use container.runtime.ContainerError
use internal.helpers.log_transition

pub fun start(c: Container) -> Result<Container, ContainerError> {
    if !c.state.can_start() {
        return Err(ContainerError.InvalidState(
            expected: "Created or Stopped",
            actual: c.state.to_string()
        ))
    }
    
    c.state = ContainerState.Starting
    log_transition(c.id, "Starting")
    
    // ... actual start logic ...
    
    c.state = ContainerState.Running
    log_transition(c.id, "Running")
    
    return Ok(c)
}

pub fun stop(c: Container) -> Result<Container, ContainerError> {
    if !c.state.can_stop() {
        return Err(ContainerError.InvalidState(
            expected: "Running or Paused",
            actual: c.state.to_string()
        ))
    }
    
    c.state = ContainerState.Stopping
    log_transition(c.id, "Stopping")
    
    // ... actual stop logic ...
    
    c.state = ContainerState.Stopped
    log_transition(c.id, "Stopped")
    
    return Ok(c)
}

pub fun restart(c: Container) -> Result<Container, ContainerError> {
    c = stop(c)?
    return start(c)
}

pub fun pause(c: Container) -> Result<Container, ContainerError> {
    match c.state {
        ContainerState.Running {
            c.state = ContainerState.Paused
            log_transition(c.id, "Paused")
            return Ok(c)
        }
        _ {
            return Err(ContainerError.InvalidState(
                expected: "Running",
                actual: c.state.to_string()
            ))
        }
    }
}

pub fun resume(c: Container) -> Result<Container, ContainerError> {
    match c.state {
        ContainerState.Paused {
            c.state = ContainerState.Running
            log_transition(c.id, "Running")
            return Ok(c)
        }
        _ {
            return Err(ContainerError.InvalidState(
                expected: "Paused",
                actual: c.state.to_string()
            ))
        }
    }
}
```

### src/internal/helpers.dol

```dol
// mod internal.helpers
// Package-internal utilities — not exported from lib.dol

// Available within this Spirit only
pub(spirit) fun generate_id() -> UInt64 {
    // Use atomic counter or UUID generation
    return next_id()
}

pub(spirit) fun log_transition(id: UInt64, state: String) -> Void {
    sex {
        @univrs/std.io.println("[Container " + id + "] → " + state)
    }
}

// Private to this module
var ID_COUNTER: UInt64 = 0

fun next_id() -> UInt64 {
    sex {
        ID_COUNTER += 1
        return ID_COUNTER
    }
}
```

---

## Example 4: System Composition

A System that composes multiple Spirits for a complete deployment.

### System.dol

```dol
system ProductionCluster {
    // ═══════════════════════════════════════════════════════════
    // SPIRIT DEPENDENCIES
    // ═══════════════════════════════════════════════════════════
    uses @univrs/containers: "^2.0"
    uses @univrs/scheduler: "^1.5"
    uses @univrs/monitoring: "^1.0"
    uses @univrs/networking: "^3.0"
    
    // ═══════════════════════════════════════════════════════════
    // SYSTEM CONFIGURATION
    // ═══════════════════════════════════════════════════════════
    has config: ClusterConfig
    has clusters: List<Cluster>
    has scheduler: Scheduler
    has monitor: Monitor
    
    // ═══════════════════════════════════════════════════════════
    // CROSS-SPIRIT CONSTRAINTS
    // ═══════════════════════════════════════════════════════════
    constraint max_containers_per_cluster {
        forall cluster in this.clusters {
            cluster.containers.len <= this.config.max_containers_per_cluster
        }
    }
    
    constraint total_memory_limit {
        sum(this.clusters.map(c => c.total_memory_mb)) <= this.config.total_memory_limit_mb
    }
    
    constraint all_containers_monitored {
        forall cluster in this.clusters {
            forall container in cluster.containers {
                this.monitor.is_tracking(container.id)
            }
        }
    }
    
    // ═══════════════════════════════════════════════════════════
    // ORCHESTRATION FUNCTIONS
    // ═══════════════════════════════════════════════════════════
    pub sex fun deploy(task: Task) -> Result<Container, DeployError> {
        // Find best cluster for this task
        cluster = this.scheduler.select_cluster(this.clusters, task)?
        
        // Create and start container
        container = Container.new(task.name)
            .with_config(task.config)
        
        container = @univrs/containers.start(container)?
        
        // Register with monitoring
        this.monitor.track(container)
        
        // Add to cluster
        cluster.containers.push(container)
        
        return Ok(container)
    }
    
    pub sex fun scale(cluster_id: UInt64, count: Int32) -> Result<Void, ScaleError> {
        cluster = this.find_cluster(cluster_id)?
        
        if count > 0 {
            // Scale up
            for _ in 0..count {
                container = Container.new("scaled-" + generate_id())
                container = @univrs/containers.start(container)?
                cluster.containers.push(container)
                this.monitor.track(container)
            }
        } else {
            // Scale down
            to_remove = cluster.containers.take_last((-count) as UInt32)
            for container in to_remove {
                this.monitor.untrack(container.id)
                @univrs/containers.stop(container)?
            }
        }
        
        return Ok(())
    }
    
    pub sex fun health_check() -> SystemHealth {
        unhealthy = []
        
        for cluster in this.clusters {
            for container in cluster.containers {
                if !this.monitor.is_healthy(container.id) {
                    unhealthy.push(container.id)
                }
            }
        }
        
        return SystemHealth {
            total_containers: this.total_containers(),
            unhealthy_count: unhealthy.len,
            unhealthy_ids: unhealthy
        }
    }
    
    fun find_cluster(id: UInt64) -> Option<Cluster> {
        for cluster in this.clusters {
            if cluster.id == id {
                return Some(cluster)
            }
        }
        return None
    }
    
    fun total_containers() -> UInt64 {
        return sum(this.clusters.map(c => c.containers.len))
    }
    
    // ═══════════════════════════════════════════════════════════
    // DOCUMENTATION
    // ═══════════════════════════════════════════════════════════
    exegesis {
        ProductionCluster orchestrates a multi-cluster container deployment.
        
        Features:
        - Automatic cluster selection for new deployments
        - Horizontal scaling with health monitoring
        - Cross-cluster resource constraints
        - Integrated monitoring and alerting
        
        Usage:
            system = ProductionCluster.new(config)
            system.deploy(my_task)
            system.scale(cluster_id, +5)  // Add 5 containers
            health = system.health_check()
    }
}

// Supporting genes for the system
gene ClusterConfig {
    has max_containers_per_cluster: UInt32
    has total_memory_limit_mb: UInt64
    has default_container_memory_mb: UInt64
    
    pub fun default() -> ClusterConfig {
        return ClusterConfig {
            max_containers_per_cluster: 100,
            total_memory_limit_mb: 1024 * 1024,  // 1 TB
            default_container_memory_mb: 512
        }
    }
}

gene Cluster {
    has id: UInt64
    has name: String
    has containers: List<Container>
    has total_memory_mb: UInt64
}

gene SystemHealth {
    has total_containers: UInt64
    has unhealthy_count: UInt64
    has unhealthy_ids: List<UInt64>
    
    pub fun is_healthy(self) -> Bool {
        return this.unhealthy_count == 0
    }
}

gene Task {
    has name: String
    has config: ContainerConfig
    has priority: Int32
}

gene DeployError {
    type: enum {
        NoCapacity,
        InvalidConfig(message: String),
        ContainerError(inner: ContainerError)
    }
}

gene ScaleError {
    type: enum {
        ClusterNotFound(id: UInt64),
        CapacityExceeded,
        ContainerError(inner: ContainerError)
    }
}
```

---

## Summary

### Composition Levels

| Level | Declaration | Contains | Visibility Boundary |
|-------|-------------|----------|---------------------|
| **Module** | File path → `mod name` | genes, traits, funs | `pub(parent)`, private |
| **Spirit** | `spirit Name { }` | modules, entry points | `pub(spirit)`, pub |
| **System** | `system Name { }` | spirits, orchestration | pub (all public) |

### Key Principles

1. **Spirit is just a gene with package semantics** — same syntax, different scope
2. **Module path = file path** — `src/foo/bar.dol` → `mod foo.bar`
3. **Visibility is explicit** — private by default, `pub` to expose
4. **Same import syntax everywhere** — `use` with prefixes for source type
5. **Constraints work at every level** — genes, Spirits, and Systems

### Resolution Prefixes

| Prefix | Resolves To |
|--------|-------------|
| (none) | Local: `src/path.dol` |
| `@org/name` | Registry package |
| `@git:url` | Git repository |
| `@https://url` | HTTP single-file |

---

*"Systems describe what they ARE before what they DO."*
