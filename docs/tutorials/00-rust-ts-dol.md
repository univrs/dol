# Phase 3: Generate Rust and Build

 cd ~/repos/univrs-vudo/spirits/thermodynamic

## Generate Rust from DOL
 dol-build-crate dol/*.dol -o . --name thermodynamic

## Build native binary (CLI)
 cargo build --release

## Build WASM (browser)
 wasm-pack build --target web --release

# Phase 4: Create CLI Binary

 File: src/bin/thermo.rs
```rust
 use clap::{Parser, Subcommand};
 use thermodynamic::{EnergySystem, SmallWorldMetrics};

 #[derive(Parser)]
 #[command(name = "thermo")]
 #[command(about = "Thermodynamic economics calculator")]
 struct Cli {
     #[command(subcommand)]
     command: Commands,
 }

 #[derive(Subcommand)]
 enum Commands {
     /// Calculate EROEI for a solar system
     Solar { nodes: Option<u32> },
     /// Analyze small-world network properties
     Network { nodes: u32, degree: u32, rewire: f64 },
 }

 fn main() {
     let cli = Cli::parse();
     match cli.command {
         Commands::Solar { nodes } => run_solar_analysis(nodes),
         Commands::Network { nodes, degree, rewire } =>
             run_network_analysis(nodes, degree, rewire),
     }
 }
```

# Phase 5: Spirit Manifest

- File: manifest.toml
```toml
 [spirit]
 name = "thermodynamic"
 version = "0.1.0"
 author = "64-char-hex-pubkey-here"
 description = "EROEI and Small-World calculators for thermodynamic economics"

 [capabilities]
 compute = true

 [pricing]
 base_cost = 100
 per_fuel_cost = 1
```

# Phase 6: Update krzy Documentation

 File: ~/repos/krzy/docs/tutorials/DOGFOODING.md

 # Thermodynamic Calculators - Dogfooding DOL

 The Python calculators have been replaced with DOL Spirits.

 ## Installation

 ```bash
 # From source
 cd ~/repos/univrs-vudo/spirits/thermodynamic
 cargo install --path .

 # Use CLI
 thermo solar --nodes 1000
 thermo network --nodes 100 --degree 6 --rewire 0.1

 Browser Usage

 ```javascript
 import init, { EnergySystem } from '@univrs/thermodynamic';
 await init();
 const system = EnergySystem.new("Solar + Hyphal");
 // ...
 ```

 Legacy Python (Reference Only)

 The Python files in this directory are deprecated and kept for reference.

 ---

 ## File Changes Summary

 ### univrs-dol (New Files)
 | File | Purpose |
 |------|---------|
 | `stdlib/math.dol` | Math stdlib (ln, sqrt, pow, sin, cos, etc.) |
 | `src/codegen/math_impl.rs` | libm bindings for extern functions |

 ### univrs-vudo (New Files)
 | File | Purpose |
 |------|---------|
 | `spirits/thermodynamic/dol/eroei.dol` | EROEI types and calculations |
 | `spirits/thermodynamic/dol/small-world.dol` | Graph metrics types |
 | `spirits/thermodynamic/dol/energy-flow.dol` | Move from krzy |
 | `spirits/thermodynamic/src/lib.rs` | Generated from DOL |
 | `spirits/thermodynamic/src/bin/thermo.rs` | CLI binary |
 | `spirits/thermodynamic/Cargo.toml` | Rust/WASM config |
 | `spirits/thermodynamic/manifest.toml` | Spirit metadata |

 ### krzy (Updates)
 | File | Action |
 |------|--------|
 | `docs/tutorials/DOGFOODING.md` | NEW - links to DOL Spirits |
 | `docs/tutorials/eroei_calculator.py` | Mark DEPRECATED |
 | `docs/tutorials/small_world_metrics.py` | Mark DEPRECATED |
 | `docs/tutorials/energy-flow.dol` | MOVE to univrs-vudo |

 ---

 ## Success Criteria

 1. **Math stdlib accepted** - PR merged to univrs-dol with stdlib/math.dol
 2. **DOL compiles** - `dol-build-crate` produces valid Rust from eroei.dol
 3. **Rust compiles** - `cargo build --release` succeeds
 4. **WASM builds** - `wasm-pack build --target web` produces .wasm + .js + .d.ts
 5. **CLI works** - `thermo solar` and `thermo network` produce correct output
 6. **Results match** - Same calculations as Python versions (within f64 precision)
 7. **Spirit runs** - Executes in VUDO VM sandbox with manifest.toml

 ---

 ## Execution Order

 | Step | Repo | Task | Depends On |
 |------|------|------|------------|
 | 1 | univrs-dol | Add stdlib/math.dol | None |
 | 2 | univrs-dol | Add libm bindings to Rust codegen | Step 1 |
 | 3 | univrs-vudo | Create spirits/thermodynamic/ structure | Step 1-2 |
 | 4 | univrs-vudo | Write eroei.dol, small-world.dol | Step 1 |
 | 5 | univrs-vudo | Move energy-flow.dol from krzy | None |
 | 6 | univrs-vudo | Run dol-build-crate, verify Rust output | Step 2-4 |
 | 7 | univrs-vudo | Add CLI binary (thermo) | Step 6 |
 | 8 | univrs-vudo | Configure wasm-pack, build WASM | Step 6 |
 | 9 | univrs-vudo | Create manifest.toml | Step 8 |
 | 10 | krzy | Add DOGFOODING.md, deprecate Python | Step 7-9 |

 ---

 ## Agent Coordination

 This work spans multiple repos and can be parallelized:

 **Agent 1: DOL Math Stdlib** (univrs-dol)
 - Create stdlib/math.dol
 - Update codegen to handle extern functions
 - PR to univrs-dol

 **Agent 2: Spirit Development** (univrs-vudo)
 - Create directory structure
 - Write DOL schemas
 - Build and test

 **Agent 3: Documentation** (krzy)
 - Update docs after Spirits are working
 - Deprecate Python files

 ---

 ## Notes

 - The DOL compiler already supports `extern fun` declarations
 - libm crate provides no_std math for WASM
 - wasm-bindgen handles JS/TS bindings automatically
 - Spirit manifest format is documented in univrs-vudo/docs/