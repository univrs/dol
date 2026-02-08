# Task t0.3 Quick Reference Guide

**Task:** DOL CRDT Annotation RFC
**Status:** ✅ COMPLETE
**Date:** 2026-02-05

---

## Absolute File Paths

### RFC Documents (rfcs/)

```
/home/ardeshir/repos/univrs-dol/rfcs/RFC-001-dol-crdt-annotations.md
/home/ardeshir/repos/univrs-dol/rfcs/RFC-001-formal-proofs.md
/home/ardeshir/repos/univrs-dol/rfcs/RFC-001-architectural-review.md
```

### Example DOL Files (examples/crdt/)

```
/home/ardeshir/repos/univrs-dol/examples/crdt/README.md
/home/ardeshir/repos/univrs-dol/examples/crdt/chat-message.dol
/home/ardeshir/repos/univrs-dol/examples/crdt/account-credit.dol
/home/ardeshir/repos/univrs-dol/examples/crdt/task-board.dol
/home/ardeshir/repos/univrs-dol/examples/crdt/config-app.dol
```

### Research Documentation (docs/research/)

```
/home/ardeshir/repos/univrs-dol/docs/research/task-t0.3-completion.md
```

---

## Quick Access Commands

### View RFC Main Specification
```bash
cat /home/ardeshir/repos/univrs-dol/rfcs/RFC-001-dol-crdt-annotations.md | less
```

### View Formal Proofs
```bash
cat /home/ardeshir/repos/univrs-dol/rfcs/RFC-001-formal-proofs.md | less
```

### View All Examples
```bash
ls -lh /home/ardeshir/repos/univrs-dol/examples/crdt/
```

### Search RFC for Specific Topics
```bash
# Search for "escrow pattern"
grep -n "escrow" /home/ardeshir/repos/univrs-dol/rfcs/RFC-001-dol-crdt-annotations.md

# Search for "pn_counter"
grep -n "pn_counter" /home/ardeshir/repos/univrs-dol/rfcs/RFC-001-dol-crdt-annotations.md

# Search for theorems
grep -n "Theorem" /home/ardeshir/repos/univrs-dol/rfcs/RFC-001-formal-proofs.md
```

---

## Document Summary

| File | Lines | Purpose |
|------|-------|---------|
| RFC-001-dol-crdt-annotations.md | 1,600+ | Main specification |
| RFC-001-formal-proofs.md | 1,000+ | Convergence proofs |
| RFC-001-architectural-review.md | 800+ | Approval review |
| chat-message.dol | 150+ | Example: 4 strategies |
| account-credit.dol | 250+ | Example: Escrow pattern |
| task-board.dol | 150+ | Example: RGA |
| config-app.dol | 200+ | Example: MV-Register |

**Total:** 4,161 lines of documentation and code

---

## Key Sections by Topic

### Grammar Specification
- **File:** RFC-001-dol-crdt-annotations.md
- **Section:** 2. Grammar Extensions
- **Line:** ~45-100

### Type Compatibility Matrix
- **File:** RFC-001-dol-crdt-annotations.md
- **Section:** 4. Type Compatibility Matrix
- **Line:** ~450-500

### CRDT Strategy Semantics
- **File:** RFC-001-dol-crdt-annotations.md
- **Sections:** 3.2-3.8 (one per strategy)
- **Lines:** ~150-450

### Constraint Framework
- **File:** RFC-001-dol-crdt-annotations.md
- **Section:** 5. Constraint-CRDT Interaction Rules
- **Line:** ~650-850

### Escrow Pattern
- **File:** RFC-001-dol-crdt-annotations.md
- **Section:** 5.3 Category C: Strong-Consistency Constraints
- **Line:** ~750-850
- **Example:** account-credit.dol (complete implementation)

### Formal Proofs
- **File:** RFC-001-formal-proofs.md
- **Sections:** 2-7 (one proof per strategy)
- **Key Theorems:**
  - Theorem 2.1: Immutable convergence
  - Theorem 3.1: LWW convergence
  - Theorem 4.1: OR-Set convergence
  - Theorem 5.1: PN-Counter convergence
  - Theorem 6.1: RGA convergence
  - Theorem 7.1: MV-Register convergence
  - Theorem 9.3: Escrow correctness proof

### Code Generation
- **File:** RFC-001-dol-crdt-annotations.md
- **Section:** 7. Compilation to Rust/WASM
- **Line:** ~900-1100

### Validation Rules
- **File:** RFC-001-dol-crdt-annotations.md
- **Section:** 8. dol-check Validation Rules
- **Line:** ~1100-1250

---

## Example Usage Patterns

### 1. Immutable Identity
```dol
gen message.chat {
  @crdt(immutable)
  id: Uuid

  @crdt(immutable)
  author: Identity
}
```
**File:** /home/ardeshir/repos/univrs-dol/examples/crdt/chat-message.dol

### 2. Last-Write-Wins Metadata
```dol
gen user.profile {
  @crdt(lww)
  display_name: String

  @crdt(lww)
  avatar_url: String
}
```
**File:** /home/ardeshir/repos/univrs-dol/examples/crdt/chat-message.dol

### 3. OR-Set Collections
```dol
gen document.tags {
  @crdt(or_set)
  tags: Set<String>

  @crdt(or_set)
  collaborators: Set<Identity>
}
```
**File:** /home/ardeshir/repos/univrs-dol/examples/crdt/chat-message.dol

### 4. PN-Counter for Numeric Operations
```dol
gen account.balance {
  @crdt(pn_counter, min_value=0)
  confirmed_balance: Int

  @crdt(pn_counter)
  pending_credits: Int
}
```
**File:** /home/ardeshir/repos/univrs-dol/examples/crdt/account-credit.dol

### 5. Peritext Rich Text Editing
```dol
gen document.content {
  @crdt(peritext, formatting="full", max_length=100000)
  content: RichText
}
```
**File:** /home/ardeshir/repos/univrs-dol/examples/crdt/chat-message.dol

### 6. RGA Ordered Sequences
```dol
gen task.board {
  @crdt(rga)
  task_order: List<TaskId>

  @crdt(rga)
  column_order: List<ColumnId>
}
```
**File:** /home/ardeshir/repos/univrs-dol/examples/crdt/task-board.dol

### 7. MV-Register Conflict Detection
```dol
gen config.app {
  @crdt(mv_register)
  theme: Theme

  @crdt(mv_register)
  language: Language
}
```
**File:** /home/ardeshir/repos/univrs-dol/examples/crdt/config-app.dol

---

## Acceptance Criteria Checklist

- [x] Complete grammar specification for @crdt annotations
  - Location: RFC-001-dol-crdt-annotations.md, Section 2

- [x] Type compatibility matrix: DOL type → allowed CRDT strategies
  - Location: RFC-001-dol-crdt-annotations.md, Section 4

- [x] Constraint interaction rules documented
  - Location: RFC-001-dol-crdt-annotations.md, Section 5

- [x] Formal convergence proof for constrained CRDTs
  - Location: RFC-001-formal-proofs.md, Sections 2-13

- [x] Reviewed and approved by arch-wasm-runtime
  - Location: RFC-001-architectural-review.md

---

## Related Project Files

### DOL Grammar (for reference)
```
/home/ardeshir/repos/univrs-dol/docs/grammar.ebnf
```

### Existing DOL Examples (for patterns)
```
/home/ardeshir/repos/univrs-dol/examples/genes/container.exists.dol
/home/ardeshir/repos/univrs-dol/examples/traits/container.lifecycle.dol
```

### Project Planning
```
/home/ardeshir/repos/univrs-dol/univrs-local-first-swarm.yaml
```

---

## Next Phase Files (Phase 1 - HYPHA)

### To Be Created in Phase 1:
```
/home/ardeshir/repos/univrs-dol/crates/dol-parse/src/crdt_annotations.rs
/home/ardeshir/repos/univrs-dol/crates/dol-check/src/crdt_validation.rs
/home/ardeshir/repos/univrs-dol/crates/dol-codegen-rust/src/automerge_backend.rs
/home/ardeshir/repos/univrs-dol/crates/dol-test/src/crdt_properties.rs
```

---

## Contact & Feedback

**Task Owners:**
- arch-dol-crdt (DOL language design)
- researcher-crdt-frontier (CRDT theory)

**Reviewer:**
- arch-wasm-runtime (WASM compilation strategy)

**Feedback:** Submit issues to the project repository

---

**Last Updated:** 2026-02-05
**Status:** ✅ COMPLETE - Ready for Phase 1 Implementation
