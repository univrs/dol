#!/bin/bash
#
# WASM Size Budget Enforcement Script
#
# Ensures WASM modules meet size budgets for production deployment.
# Budget: <100KB gzipped per module (from t4.1 performance requirements)
#
# Usage: ./scripts/wasm-size-budget.sh <input.wasm> [budget_kb]

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default budget: 100KB gzipped (from t4.1)
DEFAULT_BUDGET_KB=100

usage() {
    echo "Usage: $0 <input.wasm> [budget_kb]"
    echo ""
    echo "Optimizes WASM module and enforces size budget."
    echo ""
    echo "Arguments:"
    echo "  input.wasm    Path to WASM module to check"
    echo "  budget_kb     Optional size budget in KB (default: ${DEFAULT_BUDGET_KB}KB)"
    echo ""
    echo "Example:"
    echo "  $0 target/wasm32-unknown-unknown/release/vudo.wasm"
    echo "  $0 target/wasm32-unknown-unknown/release/vudo.wasm 50"
    exit 1
}

if [ $# -lt 1 ]; then
    usage
fi

WASM_FILE="$1"
BUDGET_KB="${2:-$DEFAULT_BUDGET_KB}"

# Validate input
if [ ! -f "$WASM_FILE" ]; then
    echo -e "${RED}❌ Error: WASM file not found: $WASM_FILE${NC}"
    exit 1
fi

# Check for wasm-opt
if ! command -v wasm-opt &> /dev/null; then
    echo -e "${RED}❌ Error: wasm-opt not found${NC}"
    echo "Install binaryen:"
    echo "  wget https://github.com/WebAssembly/binaryen/releases/download/version_116/binaryen-version_116-x86_64-linux.tar.gz"
    echo "  tar xzf binaryen-version_116-x86_64-linux.tar.gz"
    echo "  export PATH=\$PWD/binaryen-version_116/bin:\$PATH"
    exit 1
fi

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}WASM Size Budget Enforcement${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Input:  $WASM_FILE"
echo "Budget: ${BUDGET_KB} KB (gzipped)"
echo ""

# Get original size (cross-platform stat)
get_file_size() {
    stat -c%s "$1" 2>/dev/null || stat -f%z "$1" 2>/dev/null
}

ORIGINAL_SIZE=$(get_file_size "$WASM_FILE")
ORIGINAL_SIZE_KB=$((ORIGINAL_SIZE / 1024))

echo -e "${YELLOW}Original size:${NC} ${ORIGINAL_SIZE_KB} KB"

# Create output filenames
OUTPUT_WASM="${WASM_FILE}.opt"
OUTPUT_WASM_GZ="${WASM_FILE}.opt.gz"

# Step 1: Optimize with wasm-opt
echo ""
echo -e "${YELLOW}Step 1: Optimizing with wasm-opt -Oz...${NC}"

wasm-opt \
    -Oz \
    --strip-debug \
    --strip-producers \
    --strip-dwarf \
    --enable-mutable-globals \
    --enable-bulk-memory \
    --enable-sign-ext \
    --enable-simd \
    --converge \
    "$WASM_FILE" \
    -o "$OUTPUT_WASM"

OPTIMIZED_SIZE=$(get_file_size "$OUTPUT_WASM")
OPTIMIZED_SIZE_KB=$((OPTIMIZED_SIZE / 1024))
REDUCTION_PCT=$(echo "scale=1; 100 - ($OPTIMIZED_SIZE * 100 / $ORIGINAL_SIZE)" | bc 2>/dev/null || echo "0")

echo -e "${GREEN}Optimized size:${NC} ${OPTIMIZED_SIZE_KB} KB (${REDUCTION_PCT}% reduction)"

# Step 2: Gzip compression
echo ""
echo -e "${YELLOW}Step 2: Gzip compression (level 9)...${NC}"

gzip -9 -c "$OUTPUT_WASM" > "$OUTPUT_WASM_GZ"

GZIPPED_SIZE=$(get_file_size "$OUTPUT_WASM_GZ")
GZIPPED_SIZE_KB=$((GZIPPED_SIZE / 1024))
GZIP_REDUCTION_PCT=$(echo "scale=1; 100 - ($GZIPPED_SIZE * 100 / $ORIGINAL_SIZE)" | bc 2>/dev/null || echo "0")

echo -e "${GREEN}Gzipped size:${NC} ${GZIPPED_SIZE_KB} KB (${GZIP_REDUCTION_PCT}% total reduction)"

# Step 3: Verify budget
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Budget Verification${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

if [ "$GZIPPED_SIZE_KB" -lt "$BUDGET_KB" ]; then
    MARGIN=$((BUDGET_KB - GZIPPED_SIZE_KB))
    echo -e "${GREEN}✅ Size budget met!${NC}"
    echo ""
    echo "  Gzipped: ${GZIPPED_SIZE_KB} KB"
    echo "  Budget:  ${BUDGET_KB} KB"
    echo "  Margin:  ${MARGIN} KB under budget"
    echo ""
    echo -e "${GREEN}PASS${NC}"
    EXIT_CODE=0
elif [ "$GZIPPED_SIZE_KB" -eq "$BUDGET_KB" ]; then
    echo -e "${YELLOW}⚠️  Size exactly at budget${NC}"
    echo ""
    echo "  Gzipped: ${GZIPPED_SIZE_KB} KB"
    echo "  Budget:  ${BUDGET_KB} KB"
    echo "  Margin:  0 KB (consider optimizing further)"
    echo ""
    echo -e "${YELLOW}WARN${NC}"
    EXIT_CODE=0
else
    OVERAGE=$((GZIPPED_SIZE_KB - BUDGET_KB))
    echo -e "${RED}❌ Size budget exceeded!${NC}"
    echo ""
    echo "  Gzipped: ${GZIPPED_SIZE_KB} KB"
    echo "  Budget:  ${BUDGET_KB} KB"
    echo "  Overage: ${OVERAGE} KB over budget"
    echo ""
    echo "Suggestions to reduce size:"
    echo "  - Enable LTO (Link Time Optimization)"
    echo "  - Use opt-level = 'z' in Cargo.toml"
    echo "  - Remove unused dependencies"
    echo "  - Enable strip = true in release profile"
    echo "  - Review and minimize included features"
    echo ""
    echo -e "${RED}FAIL${NC}"
    EXIT_CODE=1
fi

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo "Summary:"
echo "  Original:  ${ORIGINAL_SIZE_KB} KB"
echo "  Optimized: ${OPTIMIZED_SIZE_KB} KB (-${REDUCTION_PCT}%)"
echo "  Gzipped:   ${GZIPPED_SIZE_KB} KB (-${GZIP_REDUCTION_PCT}%)"
echo ""
echo "Output files:"
echo "  - ${OUTPUT_WASM}"
echo "  - ${OUTPUT_WASM_GZ}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

exit $EXIT_CODE
