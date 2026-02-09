#!/bin/bash
# Build all example schemas for all targets

set -e

EXAMPLES_DIR="$(cd "$(dirname "$0")" && pwd)"
OUTPUT_DIR="$EXAMPLES_DIR/generated"

echo "🔨 Building All DOL Examples"
echo "============================"

# Clean output
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"/{rust,typescript,python,wit,json-schema}

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Count files
DOL_FILES=(*.dol)
TOTAL=${#DOL_FILES[@]}
CURRENT=0

echo -e "${BLUE}Found $TOTAL DOL files${NC}\n"

# Build each example
for dol_file in *.dol; do
    CURRENT=$((CURRENT + 1))
    basename="${dol_file%.dol}"

    echo -e "${YELLOW}[$CURRENT/$TOTAL]${NC} Building $dol_file..."

    # Rust
    if dol-codegen --target rust "$dol_file" > "$OUTPUT_DIR/rust/${basename}.rs" 2>/dev/null; then
        echo -e "  ${GREEN}✓${NC} Rust"
    else
        echo -e "  ✗ Rust (failed)"
    fi

    # TypeScript
    if dol-codegen --target typescript "$dol_file" > "$OUTPUT_DIR/typescript/${basename}.ts" 2>/dev/null; then
        echo -e "  ${GREEN}✓${NC} TypeScript"
    else
        echo -e "  ✗ TypeScript (failed)"
    fi

    # Python
    if dol-codegen --target python "$dol_file" > "$OUTPUT_DIR/python/${basename}.py" 2>/dev/null; then
        echo -e "  ${GREEN}✓${NC} Python"
    else
        echo -e "  ✗ Python (failed)"
    fi

    # WIT
    if dol-codegen --target wit "$dol_file" > "$OUTPUT_DIR/wit/${basename}.wit" 2>/dev/null; then
        echo -e "  ${GREEN}✓${NC} WIT"
    else
        echo -e "  ✗ WIT (failed)"
    fi

    # JSON Schema
    if dol-codegen --target json-schema "$dol_file" > "$OUTPUT_DIR/json-schema/${basename}.json" 2>/dev/null; then
        echo -e "  ${GREEN}✓${NC} JSON Schema"
    else
        echo -e "  ✗ JSON Schema (failed)"
    fi

    echo
done

# Summary
echo "📊 Build Summary"
echo "================"
echo "Rust files:        $(ls -1 "$OUTPUT_DIR/rust" | wc -l)"
echo "TypeScript files:  $(ls -1 "$OUTPUT_DIR/typescript" | wc -l)"
echo "Python files:      $(ls -1 "$OUTPUT_DIR/python" | wc -l)"
echo "WIT files:         $(ls -1 "$OUTPUT_DIR/wit" | wc -l)"
echo "JSON Schemas:      $(ls -1 "$OUTPUT_DIR/json-schema" | wc -l)"

# Total lines
echo ""
echo "Total lines generated:"
find "$OUTPUT_DIR" -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.py" -o -name "*.wit" -o -name "*.json" \) -exec wc -l {} + | tail -1

echo ""
echo -e "${GREEN}✅ Build complete!${NC}"
echo "Output directory: $OUTPUT_DIR"
