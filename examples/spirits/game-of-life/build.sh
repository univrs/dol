#!/bin/bash
set -e
cd "$(dirname "${BASH_SOURCE[0]}")"

# DOL repository root (for compiler access)
# From examples/spirits/game-of-life/ → ../../.. → univrs-dol/
DOL_ROOT="${DOL_ROOT:-$(cd ../../.. && pwd)}"

echo "══════════════════════════════════════════════════════════════"
echo "  Building Game of Life Spirit"
echo "══════════════════════════════════════════════════════════════"

# Step 1: DOL → Rust (using dol-codegen)
echo ""
echo "Step 1: DOL → Rust"
echo "────────────────────────────────────────────────────────────────"

# Check if dol-codegen is available
if command -v dol-codegen &> /dev/null; then
    DOL_CODEGEN="dol-codegen"
elif [ -f "$DOL_ROOT/target/release/dol-codegen" ]; then
    DOL_CODEGEN="$DOL_ROOT/target/release/dol-codegen"
else
    echo "  Building dol-codegen..."
    (cd "$DOL_ROOT" && cargo build --release --features cli --bin dol-codegen)
    DOL_CODEGEN="$DOL_ROOT/target/release/dol-codegen"
fi

# Ensure output directory exists
mkdir -p codegen/rust/src/generated

echo "  Compiling genes/cell.dol..."
$DOL_CODEGEN --target rust src/genes/cell.dol -o codegen/rust/src/generated/cell.rs

echo "  Compiling genes/grid.dol..."
$DOL_CODEGEN --target rust src/genes/grid.dol -o codegen/rust/src/generated/grid.rs

echo "  Compiling spells/rules.dol..."
$DOL_CODEGEN --target rust src/spells/rules.dol -o codegen/rust/src/generated/rules.rs

echo "  Compiling spells/grid_ops.dol..."
$DOL_CODEGEN --target rust src/spells/grid_ops.dol -o codegen/rust/src/generated/grid_ops.rs

echo "  Compiling spells/patterns.dol..."
$DOL_CODEGEN --target rust src/spells/patterns.dol -o codegen/rust/src/generated/patterns.rs

echo "  Compiling effects/browser.dol..."
$DOL_CODEGEN --target rust src/effects/browser.dol -o codegen/rust/src/generated/browser.rs

echo "  ✓ DOL compilation complete"

# Step 2: Rust → WASM
echo ""
echo "Step 2: Rust → WASM"
echo "────────────────────────────────────────────────────────────────"
cd codegen/rust
cargo build --target wasm32-unknown-unknown --release
echo "  ✓ WASM binary compiled"

# Step 3: JS bindings
echo ""
echo "Step 3: WASM → JS bindings"
echo "────────────────────────────────────────────────────────────────"
wasm-bindgen target/wasm32-unknown-unknown/release/game_of_life.wasm \
    --out-dir ../../web \
    --target web \
    --omit-default-module-path
echo "  ✓ JS bindings generated"

cd ../..
echo ""
echo "══════════════════════════════════════════════════════════════"
echo "  Build complete!"
echo "  Run: cd web && python3 serve.py"
echo "══════════════════════════════════════════════════════════════"
