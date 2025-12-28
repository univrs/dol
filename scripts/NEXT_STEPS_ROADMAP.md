# DOL Self-Hosting Roadmap

## Phase 1: Complete ✓

Bootstrap compilation achieved. DOL source compiles to working Rust.

```
dol/types.dol  →  types.rs   ✓
dol/token.dol  →  token.rs   ✓
dol/ast.dol    →  ast.rs     ✓
dol/lexer.dol  →  lexer.rs   ✓

Result: libdol_bootstrap.rlib (823 KB)
Errors: 248 → 0 (with fix script)
```

## Phase 2: Codegen Improvements

**Goal:** Eliminate the fix script.

| Task | Priority | Status |
|------|----------|--------|
| Import generation | P0 | TODO |
| Expression semicolons | P0 | TODO |
| String method translation | P1 | TODO |
| ?? operator | P1 | TODO |
| Match arm types | P1 | TODO |
| Constructors | P2 | TODO |

**Success:** `make regen && cargo check` → 0 errors (no fix script)

## Phase 3: Parser in DOL

**Goal:** Write the DOL parser in DOL.

```
dol/parser.dol  ← NEW
dol/pratt.dol   ← NEW
```

## Phase 4: Full Self-Hosting

**Goal:** DOL compiles itself.

```
Stage 0: Rust Compiler (src/*.rs)
    ↓
Stage 1: DOL Compiler (dol/*.dol → Rust)
    ↓
Stage 2: DOL Compiler (dol/*.dol → Rust)
    ↓
Stage 1 == Stage 2  ✓
```

## Timeline

| Month | Phase | Goal |
|-------|-------|------|
| Dec 2025 | Phase 1 | ✓ Bootstrap compiles |
| Jan 2026 | Phase 2 | No fix script needed |
| Feb 2026 | Phase 3 | Parser in DOL |
| Mar 2026 | Phase 4 | Full self-hosting |

## Quick Commands

```bash
make bootstrap      # Full regenerate + fix + build
make check          # Check DOL source
make errors         # Show error breakdown
make test           # Run tests
```
