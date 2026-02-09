# ADR-001: CRDT Library Selection

| Field | Value |
|-------|-------|
| **Status** | Accepted |
| **Date** | 2025-10-15 |
| **Deciders** | VUDO Core Team |
| **Supersedes** | N/A |
| **Superseded by** | N/A |

## Context

DOL's local-first architecture requires Conflict-free Replicated Data Types (CRDTs) for offline-capable, eventually-consistent data synchronization. We evaluated several CRDT implementations for the Rust ecosystem.

### Requirements

1. **WASM Compatibility** - Must compile to WebAssembly for browser runtime
2. **Rust-Native** - First-class Rust support, not bindings
3. **Rich Data Types** - Support for text, lists, maps, counters, registers
4. **Active Maintenance** - Ongoing development and community support
5. **Performance** - Sub-millisecond merge operations for typical documents
6. **Binary Format** - Efficient serialization for P2P sync

### Options Considered

| Library | WASM | Types | Maintenance | Sync Format |
|---------|------|-------|-------------|-------------|
| **Automerge** | ✅ | Rich | Active (Ink & Switch) | Binary |
| **Yrs (Yjs)** | ✅ | Rich | Active | Binary |
| **crdts** | ✅ | Basic | Minimal | Custom |
| **diamond-types** | ✅ | Text only | Research | Custom |

## Decision

**We chose Automerge as our CRDT foundation.**

### Rationale

1. **Document Model Alignment**
   - Automerge's document-centric model maps naturally to DOL's `gen` definitions
   - Each `gen` instance becomes an Automerge document
   - Field-level CRDT strategies map to Automerge's type system

2. **CRDT Strategy Support**
   - `immutable` → Automerge scalar (first-write-wins)
   - `lww` → Automerge register with timestamp
   - `or_set` → Automerge list/set
   - `pn_counter` → Automerge counter
   - `peritext` → Automerge text with formatting
   - `rga` → Automerge list (Replicated Growable Array)
   - `mv_register` → Automerge multi-value register

3. **Sync Protocol**
   - Built-in sync protocol compatible with Willow's data model
   - Efficient binary encoding reduces bandwidth
   - Incremental sync minimizes data transfer

4. **Ecosystem**
   - JavaScript bindings available for TypeScript host functions
   - Active development by Ink & Switch (local-first pioneers)
   - Growing community and documentation

### Why Not Yrs/Yjs?

While Yrs is excellent, Automerge offered:
- Better alignment with DOL's per-document model (vs Yjs's shared types)
- More intuitive mapping for CRDT strategy annotations
- Closer collaboration potential with Ink & Switch research

## Consequences

### Positive

- **Seamless WASM** - Automerge compiles cleanly to WASM
- **Rich CRDTs** - All 7 DOL CRDT strategies supported
- **Proven** - Battle-tested in production local-first apps
- **Future-Proof** - Active research into performance improvements

### Negative

- **Learning Curve** - Team needed to learn Automerge's document model
- **Memory Overhead** - Automerge documents larger than raw data
- **Change Tracking** - Full history retention increases storage

### Neutral

- **Migration Path** - If needed, CRDT abstraction layer allows swapping implementations
- **Version Lock** - Dependent on Automerge's release cycle

## Implementation Notes

### DOL CRDT Annotations

```dol
gen UserProfile {
    has id: u64 @crdt(immutable)
    has name: string @crdt(lww)
    has bio: string @crdt(peritext)
    has tags: Vec<string> @crdt(or_set)
    has score: i64 @crdt(pn_counter)
    
    docs {
        User profile with field-level CRDT strategies.
    }
}
```

### Generated Automerge Code

```rust
impl UserProfile {
    pub fn apply_crdt(&mut self, doc: &mut AutomergeDoc) {
        // id: immutable - no merge needed
        // name: LWW register
        self.name = doc.get_register("name").resolve_lww();
        // bio: Peritext for rich text
        self.bio = doc.get_text("bio").to_string();
        // tags: OR-Set semantics
        self.tags = doc.get_list("tags").to_or_set();
        // score: PN-Counter
        self.score = doc.get_counter("score").value();
    }
}
```

## References

- [Automerge Documentation](https://automerge.org/docs/)
- [Ink & Switch Local-First Research](https://www.inkandswitch.com/local-first/)
- [DOL CRDT Specification](../specs/dol-crdt-schema-v1.0.md)
- [CRDT Implementation Guide](../specs/dol-crdt-implementation-guide.md)

## Changelog

| Date | Change |
|------|--------|
| 2025-10-15 | Initial decision |
| 2025-12-01 | Added 7th strategy (mv_register) |
| 2026-01-15 | Updated to v0.8.1 syntax examples |
