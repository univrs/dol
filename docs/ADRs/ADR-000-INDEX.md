# Architectural Decision Records Index

> **Project:** univrs-dol (Design Ontology Language)
> **Status:** Active
> **Last Updated:** 2026-11-21
> **Total ADRs:** 13 (8 Core + 5 Phase 0)

## Overview

This directory contains Architectural Decision Records (ADRs) documenting significant technical decisions made during the development of DOL and the VUDO ecosystem.

## ADR Formats

This repository uses two ADR formats:

### Core ADRs (ADR-001 to ADR-008)
Comprehensive format with:
- **Status:** Proposed | Accepted | Deprecated | Superseded
- **Context:** Why we needed to make a decision
- **Decision:** What we decided with detailed rationale
- **Consequences:** Positive, Negative, and Neutral impacts
- **Implementation Notes:** Code examples and integration points
- **References:** Related documents and resources
- **Changelog:** Decision evolution history

### Phase 0 ADRs (ADR-009 to ADR-013)
SPORE (Short-form Pragmatic Operational Record) format:
- Concise, decision-focused documentation
- Rapid iteration and approval workflow
- Validation checklists
- All dated 2026-02-05 from initial implementation phase

## Index

### Core ADRs (Comprehensive Format)

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| [ADR-001](./ADR-001-CRDT-SELECTION.md) | CRDT Library Selection (Automerge) | Accepted, partially implemented | 2025-10 |
| [ADR-002](./ADR-002-P2P-NETWORKING.md) | P2P Networking Stack (Iroh + Willow) | Accepted, library only (not in CI) | 2025-11 |
| [ADR-003](./ADR-003-DOL-SYNTAX-V081.md) | DOL v0.8 "Clarity" Syntax Evolution (shipped in 0.8.0) | Accepted, implemented | 2026-01 |
| [ADR-004](./ADR-004-SPIRIT-MANIFEST.md) | Spirit.dol Manifest Format | Accepted; implemented format differs | 2025-12 |
| [ADR-005](./ADR-005-WASM-RUNTIME.md) | WASM-First Runtime Target | Accepted, implemented (subset) | 2025-10 |
| [ADR-006](./ADR-006-EFFECT-SYSTEM.md) | SEX Effect System Design | Accepted; syntax implemented, enforcement not wired in | 2025-12 |
| [ADR-007](./ADR-007-META-PROGRAMMING.md) | Meta-Programming Operators | Accepted, partially implemented | 2026-01 |
| [ADR-008](./ADR-008-MULTI-TARGET-CODEGEN.md) | Multi-Target Code Generation | Accepted, partially implemented | 2025-11 |

### Phase 0 ADRs (SPORE Format)

The Validation checkboxes in these ADRs were not reproduced in the 2026-11 audit ([DOL-STATUS-2026-11](../DOL-STATUS-2026-11.md)); read them as targets.

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| [ADR-009](./ADR-009-crdt-library.md) | CRDT Library Implementation | Accepted | 2026-02-05 |
| [ADR-010](./ADR-010-p2p-stack.md) | P2P Stack Implementation | Accepted | 2026-02-05 |
| [ADR-011](./ADR-011-storage-engine.md) | Storage Engine Selection | Accepted | 2026-02-05 |
| [ADR-012](./ADR-012-identity-system.md) | Identity System Design | Accepted | 2026-02-05 |
| [ADR-013](./ADR-013-wasm-compilation.md) | WASM Compilation Strategy | Accepted; not the implemented path | 2026-02-05 |

### Supporting Documents

| Document | Description |
|----------|-------------|
| [ADR-TEMPLATE](./ADR-TEMPLATE.md) | Template for new ADRs |

## Decision Lifecycle

```
Proposed → Accepted → [Deprecated | Superseded]
```

- **Proposed:** Under discussion
- **Accepted:** Decision agreed. This does not by itself mean the decision is implemented; each ADR states its implementation status, and the Status column above summarises it.
- **Deprecated:** No longer recommended
- **Superseded:** Replaced by newer ADR

## Contributing

When adding a new ADR:
1. Copy `ADR-TEMPLATE.md`
2. Use next available number
3. Add to index table above
4. Submit PR for review
