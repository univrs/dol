# Game of Life Spirit: DOL Compiler Benchmark

> **Status (dol 0.8.2).** The DOL sources in `examples/spirits/game-of-life/src/` parse.
> All 10 files pass a whole-file parse (`metadol::parse_file_all`), and the code blocks below
> are excerpts from them. Two caveats:
>
> - **The build is not reproducible from a clean checkout.** Step 2 of `build.sh` runs `cargo build` in
>   `codegen/rust/`, but that crate (its `Cargo.toml` and hand-written `lib.rs`) is not
>   committed. The WASM in `web/` (`game_of_life_bg.wasm`) is a prebuilt artifact.
> - This pipeline goes through Rust (`dol-codegen --target rust`, then `cargo`/`wasm-bindgen`).
>   The direct WASM backend (`dol-build`) is separate, and it fails on all 9 example Spirits in
>   0.8.2.
>
> See [`docs/DOL-STATUS-2026-11.md`](../../DOL-STATUS-2026-11.md).

This document establishes the Game of Life Spirit as the primary benchmark for DOL compiler development, providing a real-world WASM compilation target that exercises the full DOL → Rust → WASM pipeline.

## Overview

The Game of Life Spirit serves as a comprehensive test benchmark for:

1. **DOL Language Features**: gens, pure functions ("spells"), effects (`sex`), pattern matching
2. **Codegen Quality**: Thread-local state, WASM bindings, Rust idioms
3. **Build Pipeline**: Multi-stage compilation (DOL → Rust → WASM → JS)
4. **Runtime Verification**: Interactive browser-based testing

## Architecture

```
game-of-life/
├── src/                    # DOL source files
│   ├── genes/             # Pure data structures
│   │   ├── cell.dol       # Cell state, position
│   │   └── grid.dol       # Grid configuration, grid type
│   ├── spells/            # Pure functions
│   │   ├── rules.dol      # Conway's rules
│   │   ├── grid_ops.dol   # Grid operations (tick, set_cell)
│   │   └── patterns.dol   # Pattern library
│   └── effects/           # Side-effectful operations
│       ├── browser.dol    # WASM exports
│       └── mod.dol        # Module declaration
├── codegen/               # NOT in the repository (see status note)
│   └── rust/
│       ├── Cargo.toml     # Rust project config
│       └── src/
│           ├── lib.rs     # Reference implementation
│           └── generated/ # DOL-generated code (written by build.sh)
├── web/                   # Browser frontend
│   ├── index.html         # UI
│   ├── game.js            # Game logic
│   └── serve.py           # Development server
└── build.sh               # Build script
```

## DOL Language Features Demonstrated

### 1. Genes (Pure Data Types)

```dol
// excerpt from genes/cell.dol
pub gen CellState {
  type: enum { Dead, Alive }
  docs { Binary state of a cell. }
}

pub gen Position {
  has x: i32
  has y: i32
  rule non_negative { this.x >= 0 && this.y >= 0 }
  docs { 2D coordinate on the grid. }
}
```

`pub` is accepted but not recorded in the AST. The body of a `rule` nested in a `gen` is
discarded by the 0.8.2 parser, so `non_negative` is not checked.

**Codegen targets:**
- Rust enums with `#[derive(Clone, Copy, PartialEq)]`
- Struct definitions with named fields

### 2. Spells (Pure Functions)

```dol
// excerpt from spells/rules.dol
pub fun next_state(cell: Cell) -> CellState {
  match cell.state {
    CellState.Alive {
      match cell.neighbors {
        2 { CellState.Alive }
        3 { CellState.Alive }
        _ { CellState.Dead }
      }
    }
    CellState.Dead {
      match cell.neighbors {
        3 { CellState.Alive }
        _ { CellState.Dead }
      }
    }
  }
}
```

**Codegen targets:**
- Pure Rust functions
- Pattern matching with guards
- No side effects

### 3. Effects (Side-Effectful Operations)

```dol
// effects/browser.dol
sex var GRID: Option<Grid> = None
sex var CONFIG: GridConfig = GridConfig { width: 100, height: 100, wrap_edges: true }

#[wasm_export]
pub sex fun init(width: u32, height: u32, wrap: bool) {
  CONFIG = GridConfig { width: width, height: height, wrap_edges: wrap }
  GRID = Some(create_grid(CONFIG))
}
```

**Codegen targets:**
- `thread_local!` with `RefCell` for global state
- `#[wasm_bindgen]` attribute translation
- Access pattern transformation:
  - Reads: `VAR.with(|v| v.borrow().clone())`
  - Writes: `VAR.with(|v| *v.borrow_mut() = x)`
  - Field access: `VAR.with(|v| v.borrow().field)`

## Build Pipeline

### Stage 1: DOL → Rust

```bash
dol-codegen --target rust src/genes/cell.dol -o codegen/rust/src/generated/cell.rs
dol-codegen --target rust src/effects/browser.dol -o codegen/rust/src/generated/browser.rs
# ... more files
```

### Stage 2: Rust → WASM

```bash
cd codegen/rust
cargo build --target wasm32-unknown-unknown --release
```

### Stage 3: WASM → JS Bindings

```bash
wasm-bindgen target/wasm32-unknown-unknown/release/game_of_life.wasm \
    --out-dir ../../web \
    --target web
```

### Running Locally

```bash
cd examples/spirits/game-of-life
./build.sh            # step 2 fails unless you supply codegen/rust/Cargo.toml
cd web
python3 serve.py      # serves the prebuilt WASM in web/ on port 8080
# Open http://localhost:8080
```

## Codegen Verification Checklist

When making changes to the DOL compiler, verify:

### Thread-local Pattern
- [ ] `sex var` declarations grouped in `thread_local!` block
- [ ] Types wrapped with `RefCell<T>`
- [ ] `use std::cell::RefCell;` import added

### Access Patterns
- [ ] Sex var reads use `.with(|v| v.borrow().clone())`
- [ ] Sex var writes use `.with(|v| *v.borrow_mut() = x)`
- [ ] Field access uses `.with(|v| v.borrow().field)`
- [ ] Local variables (lowercase) NOT wrapped
- [ ] Sex vars (UPPERCASE) ARE wrapped

### Attribute Translation
- [ ] `#[wasm_export]` → `#[wasm_bindgen]`
- [ ] Attributes appear before function definition

### Module Paths
- [ ] `.` separator → `::` for Rust module paths
- [ ] `spells.grid_ops.func` → `spells::grid_ops::func`

### Integer Literals
- [ ] No `_i64` suffix on integer literals
- [ ] Type inference handles type context

## Testing Methodology

### Unit Tests

```bash
cargo test                    # All tests
cargo test codegen            # Codegen tests only
cargo test thread_local       # Thread-local specific tests
```

### Integration Test

```bash
cd examples/spirits/game-of-life
./build.sh                    # Must complete without errors
```

### Browser Test

1. Open http://localhost:8080
2. Click "Start" - animation should run
3. Click cells to toggle state
4. Load patterns (glider, gun, etc.)
5. Randomize grid at various densities

### Expected Behavior

| Action | Expected Result |
|--------|----------------|
| Start | Cells evolve according to Conway's rules |
| Step | Single generation advance |
| Clear | All cells dead, generation reset to 0 |
| Toggle cell | Cell state inverts |
| Load pattern | Pattern appears at center |
| Randomize | Random cells become alive |

## Version History

| Version | Changes |
|---------|---------|
| v0.8.0 | Initial Game of Life Spirit |
| v0.8.1 | Thread-local pattern, attribute translation, module paths |

## Contributing

This project is not accepting external contributions or pull requests.

## Related Documents

- [DOL Architecture](../../ARCHITECTURE.md)
- [WASM compilation guide](wasm-guide.md)
- [DOL status audit](../../DOL-STATUS-2026-11.md)
