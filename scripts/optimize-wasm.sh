#!/bin/bash
# WASM Optimization Script
#
# Optimizes WASM modules for production deployment with aggressive size reduction.
# Target: < 100KB gzipped per Gen module

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

WASM_SIZE_BUDGET_KB=100

usage() {
    echo "Usage: $0 <input.wasm> [output.wasm]"
    echo ""
    echo "Optimizes WASM module for production deployment."
    echo "If output is not specified, overwrites input file."
    exit 1
}

if [ $# -lt 1 ]; then
    usage
fi

INPUT_WASM="$1"
OUTPUT_WASM="${2:-$INPUT_WASM}"

if [ ! -f "$INPUT_WASM" ]; then
    echo -e "${RED}Error: Input file '$INPUT_WASM' not found${NC}"
    exit 1
fi

# Check if wasm-opt is installed
if ! command -v wasm-opt &> /dev/null; then
    echo -e "${RED}Error: wasm-opt not found${NC}"
    echo "Install with: cargo install wasm-opt"
    exit 1
fi

echo -e "${YELLOW}Optimizing WASM module...${NC}"
echo "Input: $INPUT_WASM"
echo "Output: $OUTPUT_WASM"
echo ""

# Get original size
ORIGINAL_SIZE=$(stat -f%z "$INPUT_WASM" 2>/dev/null || stat -c%s "$INPUT_WASM")
ORIGINAL_SIZE_KB=$((ORIGINAL_SIZE / 1024))

echo "Original size: ${ORIGINAL_SIZE_KB} KB"

# Create temporary file for intermediate steps
TEMP_WASM=$(mktemp)
trap "rm -f $TEMP_WASM" EXIT

# Step 1: Run wasm-opt with aggressive size optimization
echo -e "${YELLOW}Step 1: Running wasm-opt -Oz...${NC}"
wasm-opt \
    -Oz \
    --strip-debug \
    --strip-producers \
    --strip-dwarf \
    --enable-mutable-globals \
    --enable-bulk-memory \
    --enable-sign-ext \
    --enable-simd \
    --enable-threads \
    --converge \
    "$INPUT_WASM" \
    -o "$TEMP_WASM"

OPTIMIZED_SIZE=$(stat -f%z "$TEMP_WASM" 2>/dev/null || stat -c%s "$TEMP_WASM")
OPTIMIZED_SIZE_KB=$((OPTIMIZED_SIZE / 1024))
echo "Optimized size: ${OPTIMIZED_SIZE_KB} KB ($(echo "scale=2; 100 - ($OPTIMIZED_SIZE * 100 / $ORIGINAL_SIZE)" | bc)% reduction)"

# Step 2: Gzip compression
echo -e "${YELLOW}Step 2: Gzip compression...${NC}"
GZIPPED_SIZE=$(gzip -c "$TEMP_WASM" | wc -c)
GZIPPED_SIZE_KB=$((GZIPPED_SIZE / 1024))
echo "Gzipped size: ${GZIPPED_SIZE_KB} KB"

# Step 3: Verify budget
echo ""
if [ $GZIPPED_SIZE_KB -lt $WASM_SIZE_BUDGET_KB ]; then
    echo -e "${GREEN}✅ Size budget met: ${GZIPPED_SIZE_KB} KB < ${WASM_SIZE_BUDGET_KB} KB${NC}"
else
    echo -e "${RED}❌ Size budget exceeded: ${GZIPPED_SIZE_KB} KB >= ${WASM_SIZE_BUDGET_KB} KB${NC}"
    exit 1
fi

# Copy optimized file to output
cp "$TEMP_WASM" "$OUTPUT_WASM"

echo ""
echo -e "${GREEN}Optimization complete!${NC}"
echo "Summary:"
echo "  Original: ${ORIGINAL_SIZE_KB} KB"
echo "  Optimized: ${OPTIMIZED_SIZE_KB} KB ($(echo "scale=2; 100 - ($OPTIMIZED_SIZE * 100 / $ORIGINAL_SIZE)" | bc)% reduction)"
echo "  Gzipped: ${GZIPPED_SIZE_KB} KB ($(echo "scale=2; 100 - ($GZIPPED_SIZE * 100 / $ORIGINAL_SIZE)" | bc)% reduction)"
echo ""
echo "Output: $OUTPUT_WASM"
