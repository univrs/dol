# Game of Life Spirit: DOL Compiler Benchmark

This document establishes the Game of Life Spirit as the primary benchmark for DOL compiler development, providing a real-world WASM compilation target that exercises the full DOL → Rust → WASM pipeline.

## Overview

The Game of Life Spirit serves as a comprehensive test benchmark for:

1. **DOL Language Features**: Genes, Spells, Effects (Sex), type inference, pattern matching
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
├── codegen/
│   └── rust/
│       ├── Cargo.toml     # Rust project config
│       └── src/
│           ├── lib.rs     # Reference implementation
│           └── generated/ # DOL-generated code
├── web/                   # Browser frontend
│   ├── index.html         # UI
│   ├── game.js            # Game logic
│   └── serve.py           # Development server
└── build.sh               # Build script
```

## DOL Language Features Demonstrated

### 1. Genes (Pure Data Types)

```dol
// genes/cell.dol
gene CellState { Dead Alive }

gene Position { x: i32 y: i32 }

gene Cell {
  pos: Position
  state: CellState
  neighbors: u8
}
```

**Codegen targets:**
- Rust enums with `#[derive(Clone, Copy, PartialEq)]`
- Struct definitions with named fields

### 2. Spells (Pure Functions)

```dol
// spells/rules.dol
spell next_state(cell: Cell) -> CellState {
  match cell.state {
    Alive { match cell.neighbors { 0 | 1 { Dead } 2 | 3 { Alive } _ { Dead } } }
    Dead { if cell.neighbors == 3 { Alive } else { Dead } }
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
./build.sh
cd web
python3 -m http.server 8888
# Open http://localhost:8888
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

1. Open http://localhost:8888
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

When adding new DOL language features:

1. Add test case to Game of Life if applicable
2. Verify build pipeline completes
3. Test in browser
4. Update this benchmark document
5. Run full test suite: `cargo test`

## Related Documents

- [DOL Architecture](../../ARCHITECTURE.md)
- [Spirit Guide](spirit-guide.md)
- [Codegen Reference](../codegen/rust.md)
