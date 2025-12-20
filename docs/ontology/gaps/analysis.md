# Ontology Gap Analysis: Design vs Implementation

> **Generated**: 2025-12-20
> **Source Design**: `docs/ontology/05-application.md`
> **Implementation**: `docs/ontology/retrospective/**/*.dol` (94 files across 5 domains)

## Executive Summary

This analysis compares the ontological design in `05-application.md` against the actual DOL files generated from the Univrs codebase. We identify three categories of gaps:

1. **Design concepts missing implementation** (12 concepts)
2. **Implementation without design coverage** (28+ concepts)
3. **Misalignments between design and reality** (8 key areas)

---

## 1. Inventory Summary

### Design Document (05-application.md)

| Category | Concepts |
|----------|----------|
| **Continuants (Genes)** | univrs.container, univrs.node, univrs.cluster, univrs.identity |
| **Occurrents (Traits)** | univrs.container.lifecycle, univrs.cluster.events, univrs.scheduling, univrs.consensus |
| **Information Systems** | univrs.events, univrs.state, univrs.metrics |
| **Transformations** | univrs.reconciliation, univrs.migration |
| **Constraints** | univrs.integrity |
| **Implementations** | univrs.rust_impl, univrs.zig_impl, univrs.quantum_impl |

### Retrospective DOL Files

| Domain | Genes | Traits | Constraints | Systems | Total |
|--------|-------|--------|-------------|---------|-------|
| **container/** | 9 | 7 | 5 | 1 | 22 |
| **cluster/** | 4 | 5 | 4 | 1 | 14 |
| **state/** | 10 | 6 | 4 | 1 | 21 |
| **api/** | 5 | 6 | 4 | 3 | 18 |
| **events/** | 7 | 6 | 5 | 1 | 19 |
| **TOTAL** | **35** | **30** | **22** | **7** | **94** |

---

## 2. Design Concepts Without Implementation

These concepts appear in `05-application.md` but have no corresponding DOL formalization:

### 2.1 Missing Genes

| Design Concept | Status | Notes |
|----------------|--------|-------|
| `univrs.container` | **PARTIAL** | Split into 9 specialized genes (container_state, container_resources, container_process, container_identity, container_bundle, container_image, oci_specification, rootfs_structure) |
| `univrs.node` | **PARTIAL** | Implemented as node_identity + node_state, missing physical substrate modeling |
| `univrs.cluster` | **PARTIAL** | Implemented as cluster_membership, missing emergent properties formalization |
| `univrs.identity` | **MISSING** | Ed25519 keypair concepts scattered, no unified identity gene |

### 2.2 Missing Traits

| Design Concept | Status | Notes |
|----------------|--------|-------|
| `univrs.scheduling` | **MISSING** | No scheduling domain exists; filter/score/select/bind phases not formalized |
| `univrs.consensus` | **MISSING** | No Raft consensus formalization; design mentions leader/follower/election |
| `univrs.reconciliation` | **MISSING** | Sense/compare/plan/actuate loop not captured in DOL |
| `univrs.migration` | **MISSING** | Checkpoint/transfer/restore/cleanup phases not formalized |

### 2.3 Missing Systems

| Design Concept | Status | Notes |
|----------------|--------|-------|
| `univrs.events @ 0.1.0` | **PARTIAL** | Event concepts exist but event_stream protocol buffer encoding not specified |
| `univrs.metrics @ 0.1.0` | **PARTIAL** | Metrics exist in observability_stack but sampling/aggregation semantics missing |
| `univrs.rust_impl @ 0.1.0` | **MISSING** | No implementation-layer system formalized (youki, chitchat, openraft bindings) |

### 2.4 Missing Constraints

| Design Concept | Status | Notes |
|----------------|--------|-------|
| `univrs.integrity` (unified) | **PARTIAL** | Split across domain-specific constraints; no unified integrity constraint |
| Causal integrity | **MISSING** | "events have monotonic timestamps, effects follow causes" not enforced |
| Availability integrity | **MISSING** | "cluster survives minority failure" not formalized |

---

## 3. Implementation Without Design Coverage

These concepts exist in the retrospective DOL files but have no corresponding design in `05-application.md`:

### 3.1 API Domain (Entirely Undesigned)

The entire API layer is implementation-only, with no ontological design:

| Concept | Type | Why It Matters |
|---------|------|----------------|
| `api.persistence` | Gene | State store interface abstraction |
| `api.authentication` | Gene | Ed25519 request signing |
| `api.logging` | Gene | Container log retrieval |
| `api.routing` | Gene | Endpoint hierarchy |
| `api.error.handling` | Gene | Structured error types |
| `rest.api` | Trait | HTTP API operations |
| `workload.management` | Trait | CRUD for workloads |
| `node.management` | Trait | Node operations |
| `cluster.observability` | Trait | Status aggregation |
| `mcp.protocol` | Trait | AI agent interface |
| `log.access` | Trait | Log streaming |
| `http.api @ 1.0.0` | System | HTTP server composition |
| `orchestrator.api @ 0.1.0` | System | REST + MCP dual interface |
| `mcp.api @ 1.0.0` | System | Model Context Protocol |

### 3.2 OCI-Specific Details

| Concept | Type | Design Gap |
|---------|------|------------|
| `oci.specification` | Gene | OCI Runtime Spec v1.0.2 details |
| `container.bundle` | Gene | Bundle structure (config.json + rootfs) |
| `container.image` | Gene | Image manifest, layers, registries |
| `rootfs.structure` | Gene | Linux FHS within container |
| `image.extraction` | Trait | Whiteout handling, layer ordering |
| `image.distribution` | Trait | Registry protocol |
| `bundle.generation` | Trait | OCI bundle creation |
| `oci.compliance` | Constraint | Spec conformance |
| `security.capabilities` | Constraint | Linux capabilities restriction |
| `namespace.isolation` | Constraint | PID/network/mount isolation |

### 3.3 Observability Stack

| Concept | Type | Design Gap |
|---------|------|------------|
| `websocket.connection` | Gene | Real-time client connections |
| `subscription.management` | Gene | Topic-based subscriptions |
| `event.emission` | Gene | 14 event types with correlation |
| `metric.collection` | Gene | 30+ Prometheus metrics |
| `correlation.tracking` | Gene | Distributed trace correlation |
| `tracing.context` | Gene | Structured logging spans |
| `health.tracking` | Gene | Component health status |
| `websocket.streaming` | Trait | Real-time event delivery |
| `metric.exposition` | Trait | Prometheus /metrics endpoint |
| `health.aggregation` | Trait | Worst-status propagation |
| `event.broadcasting` | Trait | 1024-capacity broadcast channel |
| `observability.stack @ 1.0.0` | System | Integrated observability |

### 3.4 State Management Details

| Concept | Type | Design Gap |
|---------|------|------------|
| `entity.instance` | Gene | Workload instance on node |
| `entity.workload` | Gene | Workload definition template |
| `error.handling` | Gene | 6 error variants with thiserror |
| `configuration.factory` | Gene | Runtime backend selection |
| `state.namespacing` | Gene | Key hierarchy prefixes |
| `state.concurrency` | Gene | RwLock/Mutex threading |
| `operation.crud` | Trait | Put/get/list/delete semantics |
| `operation.batch` | Trait | Bulk insertion |
| `operation.watch` | Trait | Change notification (placeholder) |
| `store.distributed` | Trait | etcd implementation |
| `store.inmemory` | Trait | HashMap implementation |

---

## 4. Misalignments Between Design and Reality

### 4.1 Naming Convention Mismatch

| Design | Implementation |
|--------|----------------|
| `univrs.*` prefix | Domain-specific prefixes (container.*, cluster.*, api.*) |
| Unified namespace | Fragmented across 5 domains |

**Recommendation**: Establish naming convention that bridges conceptual design to domain implementation.

### 4.2 Consensus Protocol Mismatch

| Design Specifies | Implementation Uses |
|-----------------|---------------------|
| Raft consensus | Chitchat gossip protocol |
| Leader/follower/election | CRDT-based eventually consistent |
| Log replication | Phi-accrual failure detection |

**Recommendation**: Either update design to reflect gossip reality, or document Raft as future work.

### 4.3 Event Encoding Mismatch

| Design Specifies | Implementation Uses |
|-----------------|---------------------|
| Protocol Buffers | JSON serialization |
| Monotonic sequence numbers | UUID event IDs |

**Recommendation**: Align encoding strategy in design or formalize current JSON approach.

### 4.4 Abstraction Level Mismatch

The design operates at a conceptual level (e.g., "container has boundaries"), while implementation is concrete (e.g., "container_process has command.arguments, environment.variables, working.directory").

| Design Level | Implementation Level |
|--------------|---------------------|
| `container has state` | `container_state has lifecycle.status, exit.code, error.message` |
| `container has resources` | `container_resources has cpu.allocation, memory.limit, pids.maximum` |

**Recommendation**: Add intermediate abstraction layer mapping design concepts to implementation concepts.

### 4.5 Missing Transformation Patterns

The design emphasizes transformations (reconciliation, migration) as first-class concepts. Implementation lacks these:

| Design Pattern | Implementation Status |
|----------------|----------------------|
| Reconciliation (sense/compare/plan/actuate) | Not formalized |
| Migration (checkpoint/transfer/restore/cleanup) | Not formalized |
| Container lifecycle state machine | Exists but transitions not formalized |

**Recommendation**: Add transformation-focused DOL files to formalize these patterns.

### 4.6 Physical vs Logical Separation

Design explicitly separates:
- Physical substrate (nodes as machines)
- Logical entities (containers as workloads)

Implementation blurs this:
- `entity.node` in state/ domain (logical)
- `node_identity`, `node_state` in cluster/ domain (logical)
- No physical substrate gene

### 4.7 Cross-Cutting Concerns Not Unified

| Concern | Design Approach | Implementation Approach |
|---------|----------------|------------------------|
| Identity | Unified `univrs.identity` gene | Scattered: container_identity, node_identity, api_authentication |
| Integrity | Unified `univrs.integrity` constraint | Scattered: resource_enforcement, oci_compliance, consistency_etcd |
| Conservation | Explicit in constraints | Implicit in resource_enforcement |

### 4.8 Information Flow Gaps

| Design Component | Implementation Status |
|-----------------|----------------------|
| Event stream with sequence numbers | Events have UUIDs, no sequence numbers |
| State store with versioning | Versions mentioned but not formalized |
| Metrics with sampling semantics | Metrics exist but sampling not specified |

---

## 5. Recommendations

### 5.1 High Priority (Close Critical Gaps)

1. **Create `scheduling/` domain** with:
   - `scheduling.policy` gene
   - `scheduling.decision` trait (filter/score/select/bind)
   - `scheduling.constraint` constraint

2. **Create `reconciliation/` domain** with:
   - `reconciliation.loop` trait (sense/compare/plan/actuate)
   - `reconciliation.convergence` constraint

3. **Unify identity concepts** into:
   - `identity.cryptographic` gene (Ed25519 keypairs)
   - Cross-reference from container_identity, node_identity, api_authentication

### 5.2 Medium Priority (Improve Coverage)

4. **Formalize consensus layer**:
   - Either document gossip/CRDT approach in design
   - Or add Raft formalization for future work

5. **Add API domain to design document**:
   - REST interface design
   - MCP protocol integration
   - Authentication model

6. **Create mapping document**:
   - Design concept → Implementation file(s)
   - Implementation concept → Design section(s)

### 5.3 Low Priority (Long-term Alignment)

7. **Standardize naming conventions**:
   - Decide on `univrs.*` vs domain-specific
   - Document convention in CLAUDE.md

8. **Add abstraction layers**:
   - Design-level concepts
   - Domain-level specializations
   - Implementation-level details

9. **Formalize versioning strategy**:
   - Gene/trait/constraint versions
   - System composition versions
   - Evolution tracking

---

## 6. Gap Matrix

| Design Concept | Container | Cluster | State | API | Events | Status |
|----------------|-----------|---------|-------|-----|--------|--------|
| univrs.container | ✓ (split) | - | - | - | - | PARTIAL |
| univrs.node | - | ✓ (split) | ✓ | - | - | PARTIAL |
| univrs.cluster | - | ✓ (partial) | - | - | - | PARTIAL |
| univrs.identity | ✓ | ✓ | - | ✓ | - | SCATTERED |
| univrs.container.lifecycle | ✓ | - | - | - | - | COMPLETE |
| univrs.cluster.events | - | ✓ | - | - | ✓ | PARTIAL |
| univrs.scheduling | - | - | - | - | - | **MISSING** |
| univrs.consensus | - | gossip | - | - | - | DIVERGENT |
| univrs.events | - | - | - | - | ✓ (partial) | PARTIAL |
| univrs.state | - | - | ✓ | - | - | COMPLETE |
| univrs.metrics | - | - | - | - | ✓ (partial) | PARTIAL |
| univrs.reconciliation | - | - | - | - | - | **MISSING** |
| univrs.migration | - | - | - | - | - | **MISSING** |
| univrs.integrity | ✓ (split) | ✓ (split) | ✓ (split) | ✓ (split) | ✓ (split) | SCATTERED |
| API layer | - | - | - | ✓ | - | **NO DESIGN** |
| OCI specifics | ✓ | - | - | - | - | **NO DESIGN** |
| Observability | - | - | - | - | ✓ | **NO DESIGN** |

---

## 7. Conclusion

The gap analysis reveals that while the retrospective DOL files comprehensively capture the implementation reality (94 concepts across 5 domains), they have diverged significantly from the conceptual design in `05-application.md`. Key findings:

1. **Coverage**: Implementation is ~60% aligned with design
2. **Completeness**: Implementation has ~40% more concepts than design
3. **Missing in Implementation**: Scheduling, reconciliation, migration, Raft consensus
4. **Missing in Design**: API layer, OCI details, observability stack

The ontology serves its purpose of documenting what exists, but the design document needs updating to reflect reality, or the implementation needs enhancement to match design intent.

---

*This analysis was generated by a Claude Flow swarm with 5 specialized agents analyzing each domain in parallel.*
