# Local-First Developer Guide

Welcome to the comprehensive developer guide for building local-first applications with the VUDO stack. This guide teaches you how to create offline-first, peer-to-peer collaborative applications using DOL (Distributed Ontology Language), CRDTs (Conflict-free Replicated Data Types), and the VUDO runtime.

## What is Local-First Software?

Local-first software prioritizes:
- **Offline-first operation**: Applications work without network connectivity
- **Peer-to-peer synchronization**: No central coordination required
- **Data ownership**: Users control their own data
- **Immediate responsiveness**: No network round-trips for local operations
- **Privacy by design**: Data stays on your devices until you choose to sync

## Why Choose VUDO?

The VUDO (Verified Universal Distributed Objects) stack provides:

1. **Ontology-First Development**: Define your data model in DOL, compile to WASM
2. **Automatic Conflict Resolution**: CRDT annotations handle concurrent edits
3. **Type-Safe CRDTs**: Compiler enforces CRDT-type compatibility
4. **P2P Networking**: Built-in Iroh integration for peer discovery and sync
5. **Mutual Credit System**: Decentralized value exchange without tokens
6. **Privacy-Preserving**: GDPR-compliant cryptographic deletion
7. **Multi-Platform**: Deploy to web (PWA), desktop (Tauri), and mobile

## Quick Start

```bash
# Install the DOL toolchain
curl --proto '=https' --tlsv1.2 -sSf https://dol.univrs.io/install.sh | sh

# Create a new project
dol new my-app --template local-first

# Define your schema
cat > schemas/document.dol << 'EOF'
gen document {
  @crdt(immutable)
  has id: String

  @crdt(peritext)
  has content: String
}
EOF

# Compile to Rust
dol-codegen-rust schemas/document.dol --output src/generated/

# Build WASM module
cargo build --target wasm32-unknown-unknown --release

# Run your app
npm run dev
```

Your collaborative document editor is now running offline-first with automatic conflict resolution!

## Documentation Structure

### Getting Started
- [Introduction to Local-First](./getting-started/00-intro.md) - Core concepts and philosophy
- [Installation](./getting-started/01-installation.md) - Set up the DOL toolchain
- [Your First App](./getting-started/02-first-app.md) - Build a collaborative editor in 30 minutes
- [Core Concepts](./getting-started/03-core-concepts.md) - DOL, CRDTs, and P2P basics

### CRDT Guide
Learn how to use each CRDT strategy with real-world examples:
- [Overview](./crdt-guide/00-overview.md) - CRDT fundamentals
- [Immutable](./crdt-guide/01-immutable.md) - One-time set fields (IDs, creation timestamps)
- [Last-Write-Wins (LWW)](./crdt-guide/02-lww.md) - Simple conflict resolution for metadata
- [OR-Set](./crdt-guide/03-or-set.md) - Add-wins sets for tags and collections
- [PN-Counter](./crdt-guide/04-pn-counter.md) - Monotonic counters for metrics
- [Peritext](./crdt-guide/05-peritext.md) - Rich text collaborative editing
- [RGA](./crdt-guide/06-rga.md) - Ordered lists with causal ordering
- [MV-Register](./crdt-guide/07-mv-register.md) - Multi-value registers for conflict detection
- [Choosing a Strategy](./crdt-guide/08-choosing-strategy.md) - Decision tree and best practices

### VUDO Runtime
Deep dive into the runtime architecture:
- [Architecture Overview](./vudo-runtime/00-architecture.md) - System components and data flow
- [State Engine](./vudo-runtime/01-state-engine.md) - CRDT state management
- [Storage Adapters](./vudo-runtime/02-storage-adapters.md) - IndexedDB, SQLite, and custom adapters
- [Schema Evolution](./vudo-runtime/03-schema-evolution.md) - Versioning and migrations
- [Performance Optimization](./vudo-runtime/04-performance.md) - Tuning for production

### P2P Networking
Building decentralized sync:
- [Overview](./p2p-networking/00-overview.md) - P2P architecture and patterns
- [Iroh Setup](./p2p-networking/01-iroh-setup.md) - Configure peer discovery and NAT traversal
- [Sync Protocol](./p2p-networking/02-sync-protocol.md) - Automerge sync over Iroh
- [Willow Protocol](./p2p-networking/03-willow-protocol.md) - Structured sync for large datasets
- [PlanetServe Privacy](./p2p-networking/04-planetserve-privacy.md) - Privacy-preserving sync

### Mutual Credit
Decentralized value exchange:
- [Overview](./mutual-credit/00-overview.md) - Mutual credit fundamentals
- [Escrow Pattern](./mutual-credit/01-escrow-pattern.md) - Offline spending with local escrow
- [BFT Reconciliation](./mutual-credit/02-bft-reconciliation.md) - Byzantine fault tolerant consensus
- [Integration Guide](./mutual-credit/03-integration.md) - Adding credit to your app

### Migration Guide
Adopting local-first in existing apps:
- [Overview](./migration-guide/00-overview.md) - Migration strategies
- [Planning](./migration-guide/01-planning.md) - Assess your app for local-first
- [Gradual Migration](./migration-guide/02-gradual-migration.md) - Incremental adoption
- [Data Migration](./migration-guide/03-data-migration.md) - Migrating existing data to CRDTs

### API Reference
- [DOL Syntax Reference](./api-reference/dol-syntax.md) - Complete DOL language specification
- [Rust API](./api-reference/rust-api.md) - Generated Rust API documentation
- [WIT Interfaces](./api-reference/wit-interfaces.md) - WASM Component Model interfaces

### Troubleshooting
- [Common Issues](./troubleshooting/common-issues.md) - FAQ and solutions
- [Sync Problems](./troubleshooting/sync-problems.md) - Debugging P2P sync
- [Performance Issues](./troubleshooting/performance-issues.md) - Profiling and optimization

## Reference Application

See the [Workspace Reference Application](/apps/workspace/README.md) for a complete example demonstrating:
- Collaborative document editing with Peritext
- Kanban task boards with RGA
- User profiles with mutual credit
- P2P synchronization with Iroh
- Permission management via UCANs
- Offline-first architecture

## Community Resources

- **Documentation**: https://docs.univrs.io
- **GitHub**: https://github.com/univrs/dol
- **Discord**: https://discord.gg/univrs
- **Forum**: https://forum.univrs.io
- **Examples**: `/examples` directory in this repository

## Contributing

Found an issue or want to improve the docs? See [CONTRIBUTING.md](/CONTRIBUTING.md) for guidelines.

## License

This documentation is licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).

The VUDO stack is dual-licensed under MIT/Apache-2.0. See [LICENSE-MIT](/LICENSE-MIT) and [LICENSE-APACHE](/LICENSE-APACHE).

---

**Let's build the local-first web together!**

*Last updated: February 5, 2026*
