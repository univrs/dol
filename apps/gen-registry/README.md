# Gen Registry - Community DOL Gen Module Registry

**A P2P registry for sharing DOL Gen definitions with CRDT annotations**

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-79%20passing-brightgreen)](#testing)

---

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Architecture](#architecture)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Usage](#usage)
- [Web Interface](#web-interface)
- [P2P Synchronization](#p2p-synchronization)
- [Testing](#testing)
- [Documentation](#documentation)
- [Contributing](#contributing)

---

## Overview

Gen Registry is a **local-first, P2P community registry** for discovering, publishing, and sharing DOL Gen module definitions. Built on the VUDO platform, it demonstrates:

- **Offline-First**: Full functionality without internet connection
- **P2P Distribution**: Modules propagate via Iroh networking
- **CRDT Synchronization**: Conflict-free replication with Automerge
- **Willow Protocol**: Structured namespace for module metadata
- **Full-Text Search**: Tantivy-powered search engine
- **Version Management**: Semantic versioning with dependency resolution
- **Community Ratings**: CRDT-backed review system

This is a **reference implementation** showcasing local-first application development with DOL 2.0.

---

## Features

### Publishing
- Upload Gen definition + compiled WASM module
- Cryptographic signing for authenticity
- Version history with changelogs
- Dependency declaration

### Discovery
- Full-text search by name, description, tags
- Filter by capabilities (functions, types, traits)
- Sort by popularity, rating, or recency
- Tag-based browsing

### Version Management
- Semantic versioning (semver)
- Dependency resolution (DAG)
- Auto-update support
- Rollback capability

### P2P Sync
- Iroh networking (NAT traversal, relay fallback)
- Willow Protocol namespaces
- Meadowcap capabilities for permissions
- Bandwidth-aware synchronization
- Offline operation with eventual consistency

### Community
- Star ratings (1-5)
- Text reviews
- Download statistics
- Module recommendations

---

## Architecture

```
┌────────────────────────────────────────────────────────┐
│                   Gen Registry                         │
├────────────────────────────────────────────────────────┤
│  CLI Layer      │  gen-registry publish/search/install │
├────────────────────────────────────────────────────────┤
│  Web UI         │  React/HTML5 + WASM bindings         │
├────────────────────────────────────────────────────────┤
│  Search Layer   │  Tantivy full-text search            │
├────────────────────────────────────────────────────────┤
│  Registry Layer │  Module metadata + version resolver  │
├────────────────────────────────────────────────────────┤
│  CRDT Layer     │  Automerge (OR-Set, RGA, PN-Counter) │
├────────────────────────────────────────────────────────┤
│  Storage Layer  │  VUDO Storage (IndexedDB/SQLite)     │
├────────────────────────────────────────────────────────┤
│  P2P Layer      │  Iroh + Willow Protocol              │
└────────────────────────────────────────────────────────┘
```

### Data Flow

```
User Action (CLI/Web UI)
       ↓
Registry API (Rust)
       ↓
CRDT Update (Automerge)
       ↓
Local Storage (IndexedDB/SQLite)
       ↓
P2P Sync (Iroh) → Other Peers
       ↓
CRDT Merge → Eventual Consistency
```

---

## Quick Start

### Prerequisites

- Rust 1.81+ ([install from rustup.rs](https://rustup.rs))
- Node.js 18+ (for web UI)
- Git

### Build from Source

```bash
# Clone repository
git clone https://github.com/univrs/dol
cd dol/apps/gen-registry

# Build
cargo build --release

# Run tests
cargo test

# Install CLI
cargo install --path . --bin gen-registry
```

### Publish Your First Module

```bash
# Set your DID
export GEN_REGISTRY_DID="did:key:your-key-here"

# Publish module
gen-registry publish \
  --id io.example.hello \
  --name "Hello World" \
  --description "A simple greeting module" \
  --version 1.0.0 \
  --wasm target/wasm32-unknown-unknown/release/hello.wasm \
  --changelog "Initial release" \
  --tags "example,hello"
```

### Search for Modules

```bash
# Search by keyword
gen-registry search authentication

# Search with limit
gen-registry search database -l 50
```

### Install a Module

```bash
# Install latest version
gen-registry install io.univrs.user

# Install specific version
gen-registry install io.univrs.database --version 3.0.1

# Install with auto-update
gen-registry install io.univrs.http --auto-update
```

---

## Installation

### From Source

```bash
cargo build --release
cargo install --path . --bin gen-registry
```

### Pre-built Binaries

_(Coming soon)_

---

## Usage

### CLI Commands

#### Publish a Module

```bash
gen-registry publish \
  --id io.univrs.auth \
  --name "Authentication" \
  --description "User authentication with JWT" \
  --license MIT \
  --version 1.2.0 \
  --wasm auth.wasm \
  --changelog "Added OAuth support" \
  --tags "auth,security,jwt"
```

#### Search

```bash
# Basic search
gen-registry search "database"

# With limit
gen-registry search "http" --limit 10
```

#### Install

```bash
# Latest version
gen-registry install io.univrs.database

# Specific version
gen-registry install io.univrs.crypto --version 2.1.0

# With auto-update
gen-registry install io.univrs.logging --auto-update
```

#### List Installed

```bash
gen-registry list
```

#### Module Info

```bash
gen-registry info io.univrs.user
```

Output:
```
Authentication
  ID: io.univrs.user
  Description: User authentication and profile management
  Author: did:key:alice...
  License: MIT
  Latest: v2.1.0
  Downloads: 15,234
  Tags: authentication, security, jwt
  Rating: 4.8/5.0
```

#### Rate a Module

```bash
gen-registry rate io.univrs.database \
  --stars 5 \
  --review "Excellent database module!"
```

#### P2P Sync

```bash
# Start sync daemon
gen-registry daemon --port 8080

# Sync specific module
gen-registry sync io.univrs.user

# Check status
gen-registry status
```

---

## Web Interface

### Running Locally

```bash
# Start registry server
gen-registry daemon --port 8080

# Open browser
open http://localhost:8080
```

### Features

- **Search**: Full-text search with filters
- **Browse**: Grid view of trending modules
- **Details**: Version history, dependencies, capabilities
- **Install**: One-click installation
- **Ratings**: Community reviews

### Screenshots

![Gen Registry UI](docs/images/ui-screenshot.png)

---

## P2P Synchronization

### Willow Protocol Structure

```
Namespace: registry.gen.community
   ↓
Subspace: io.univrs.user (module_id)
   ↓
Paths:
   - metadata.json          (module metadata)
   - versions/1.0.0.json    (version info)
   - wasm/1.0.0.wasm       (WASM binary)
   - ratings/*.json         (user ratings)
```

### Iroh Networking

- **mDNS Discovery**: Local network peers
- **DHT**: Internet-wide peer discovery
- **QUIC Transport**: Encrypted connections
- **Relay Servers**: NAT traversal fallback
- **Gossip Protocol**: Presence announcements

### Sync Flow

1. User publishes module locally
2. Registry generates CRDT operations
3. Iroh broadcasts to connected peers
4. Peers validate and merge changes
5. All peers converge to same state

---

## Testing

### Run All Tests

```bash
cargo test
```

### Test Suites

| Suite | Tests | Coverage |
|-------|-------|----------|
| Registry | 13 | Core registry operations |
| Version Resolution | 15 | Dependency management |
| WASM Validation | 15 | Binary validation |
| Search Engine | 9 | Full-text search |
| Data Models | 18 | CRDT models |
| P2P Sync | 9 | Synchronization |
| **Total** | **79** | **Comprehensive** |

### Integration Tests

```bash
cargo test --test registry_tests
cargo test --test version_tests
cargo test --test wasm_tests
cargo test --test search_tests
cargo test --test models_tests
cargo test --test sync_tests
```

### Benchmarks

```bash
cargo bench
```

---

## Documentation

### User Guides

- [Getting Started](docs/getting-started.md)
- [Publishing Modules](docs/publishing.md)
- [Installing Modules](docs/installing.md)
- [P2P Networking](docs/p2p-networking.md)

### Developer Guides

- [API Documentation](docs/api.md)
- [CRDT Schema Design](docs/crdt-schema.md)
- [Architecture Overview](docs/architecture.md)
- [Contributing Guide](docs/contributing.md)

### Examples

- [Publishing Example](examples/publish.rs)
- [Search Example](examples/search.rs)
- [P2P Sync Example](examples/p2p_sync.rs)

---

## Contributing

Contributions are welcome! Please follow these guidelines:

1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Ensure `cargo test` passes
5. Run `cargo fmt` and `cargo clippy`
6. Submit a pull request

---

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

---

## Acknowledgments

Gen Registry is part of the [Univrs](https://github.com/univrs) ecosystem, built with:

- **DOL 2.0**: Ontology-first language design
- **VUDO Platform**: Local-first runtime
- **Iroh**: P2P networking
- **Willow Protocol**: Structured data sync
- **Automerge**: CRDT implementation
- **Tantivy**: Full-text search

---

**Built with Rust. Powered by CRDTs. Distributed with P2P.** 🍄
