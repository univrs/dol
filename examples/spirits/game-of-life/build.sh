#!/bin/bash
set -e
cd "$(dirname "${BASH_SOURCE[0]}")"
echo "Building Game of Life Spirit..."
echo "Step 1: DOL → Rust (using pre-generated)"
echo "Step 2: Rust → WASM..."
cd codegen/rust
cargo build --target wasm32-unknown-unknown --release
echo "Step 3: JS bindings..."
wasm-bindgen target/wasm32-unknown-unknown/release/game_of_life.wasm --out-dir ../../web --target web --omit-default-module-path
cd ../..
echo "Build complete! Run: cd web && python3 serve.py"
