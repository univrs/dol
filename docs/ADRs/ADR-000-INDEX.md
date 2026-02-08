# Architectural Decision Records Index

> **Project:** univrs-dol (Design Ontology Language)  
> **Status:** Active  
> **Last Updated:** 2026-02-07

## Overview

This directory contains Architectural Decision Records (ADRs) documenting significant technical decisions made during the development of DOL and the VUDO ecosystem.

## ADR Format

Each ADR follows this structure:
- **Status:** Proposed | Accepted | Deprecated | Superseded
- **Context:** Why we needed to make a decision
- **Decision:** What we decided
- **Consequences:** What results from this decision

## Index

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| [ADR-001](./ADR-001-CRDT-SELECTION.md) | CRDT Library Selection (Automerge) | Accepted | 2025-10 |
| [ADR-002](./ADR-002-P2P-NETWORKING.md) | P2P Networking Stack (Iroh + Willow) | Accepted | 2025-11 |
| [ADR-003](./ADR-003-DOL-SYNTAX-V081.md) | DOL v0.8.1 Syntax Evolution | Accepted | 2026-01 |
| [ADR-004](./ADR-004-SPIRIT-MANIFEST.md) | Spirit.dol Manifest Format | Accepted | 2025-12 |
| [ADR-005](./ADR-005-WASM-RUNTIME.md) | WASM-First Runtime Target | Accepted | 2025-10 |
| [ADR-006](./ADR-006-EFFECT-SYSTEM.md) | SEX Effect System Design | Accepted | 2025-12 |
| [ADR-007](./ADR-007-META-PROGRAMMING.md) | Meta-Programming Operators | Accepted | 2026-01 |
| [ADR-008](./ADR-008-MULTI-TARGET-CODEGEN.md) | Multi-Target Code Generation | Accepted | 2025-11 |

## Decision Lifecycle

```
Proposed → Accepted → [Deprecated | Superseded]
```

- **Proposed:** Under discussion
- **Accepted:** Implemented and in use
- **Deprecated:** No longer recommended
- **Superseded:** Replaced by newer ADR

## Contributing

When adding a new ADR:
1. Copy `ADR-TEMPLATE.md`
2. Use next available number
3. Add to index table above
4. Submit PR for review
