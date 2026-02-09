#!/bin/bash
# Build WASM package from generated Rust code
# Target: < 200KB compressed per module

set -e

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "Error: wasm-pack is not installed"
    echo "Install with: cargo install wasm-pack"
    exit 1
fi

# Check if wasm-opt is installed
if ! command -v wasm-opt &> /dev/null; then
    echo "Warning: wasm-opt is not installed. Install binaryen for optimization."
    echo "On macOS: brew install binaryen"
    echo "On Linux: apt-get install binaryen"
    SKIP_OPT=1
fi

# Build the WASM package
echo "Building WASM package..."
wasm-pack build --target web --release

# Optimize the WASM binary
if [ -z "$SKIP_OPT" ]; then
    echo "Optimizing WASM binary..."
    wasm-opt -Oz -o pkg/dol_codegen_rust_bg.wasm pkg/dol_codegen_rust_bg.wasm

    # Check size
    SIZE=$(wc -c < pkg/dol_codegen_rust_bg.wasm)
    SIZE_KB=$((SIZE / 1024))
    echo "WASM binary size: ${SIZE_KB}KB"

    if [ $SIZE_KB -gt 200 ]; then
        echo "Warning: WASM binary is larger than 200KB target"
    else
        echo "✓ WASM binary is under 200KB target"
    fi
fi

echo "Build complete! Output in pkg/"
