#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════════
# DOL FLYWHEEL - Local Development Runner
# ═══════════════════════════════════════════════════════════════════════════════
#
# Runs the complete flywheel pipeline locally and tracks metrics.
#
# Usage:
#   ./scripts/flywheel.sh [--quick] [--verbose]
#
# Options:
#   --quick    Skip full test suite, just check compilation
#   --verbose  Show all output
#
# ═══════════════════════════════════════════════════════════════════════════════

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
METRICS_FILE="$PROJECT_ROOT/.metrics/flywheel.local.csv"

QUICK=false
VERBOSE=false

for arg in "$@"; do
    case $arg in
        --quick) QUICK=true ;;
        --verbose) VERBOSE=true ;;
    esac
done

cd "$PROJECT_ROOT"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() { echo -e "${BLUE}[FLYWHEEL]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
warn() { echo -e "${YELLOW}⚠${NC} $1"; }
fail() { echo -e "${RED}✗${NC} $1"; }

echo ""
echo "═══════════════════════════════════════════════════════════════════════════════"
echo "                         🔄 DOL FLYWHEEL                                       "
echo "═══════════════════════════════════════════════════════════════════════════════"
echo ""

START_TIME=$(date +%s)

# ─────────────────────────────────────────────────────────────────────────────────
# STAGE 1: PARSE DOL
# ─────────────────────────────────────────────────────────────────────────────────
log "Stage 1: Parsing DOL source..."

DOL_FILES=$(find dol -name "*.dol" | wc -l)
DOL_LINES=$(wc -l dol/*.dol 2>/dev/null | tail -1 | awk '{print $1}')

PARSE_PASSED=0
PARSE_FAILED=0

for f in dol/*.dol; do
    if $VERBOSE; then
        cargo run --release --features cli --bin dol-check -- "$f"
    else
        cargo run --release --features cli --bin dol-check -- "$f" 2>/dev/null
    fi
    
    if [ $? -eq 0 ]; then
        ((PARSE_PASSED++))
    else
        ((PARSE_FAILED++))
        fail "Parse failed: $f"
    fi
done

if [ "$PARSE_FAILED" -eq 0 ]; then
    success "Parsed $PARSE_PASSED DOL files ($DOL_LINES lines)"
else
    fail "Parse: $PARSE_PASSED passed, $PARSE_FAILED failed"
    exit 1
fi

# ─────────────────────────────────────────────────────────────────────────────────
# STAGE 2: GENERATE RUST
# ─────────────────────────────────────────────────────────────────────────────────
log "Stage 2: Generating Rust from DOL..."

mkdir -p target/bootstrap/src

for module in types token ast lexer; do
    if $VERBOSE; then
        cargo run --release --features cli --bin dol-codegen -- \
            --target rust "dol/${module}.dol" > "target/bootstrap/src/${module}.rs"
    else
        cargo run --release --features cli --bin dol-codegen -- \
            --target rust "dol/${module}.dol" 2>/dev/null > "target/bootstrap/src/${module}.rs"
    fi
done

# Create lib.rs if needed
cat > target/bootstrap/src/lib.rs << 'EOF'
pub mod types;
pub mod token;
pub mod ast;
pub mod lexer;
EOF

# Create Cargo.toml if needed
if [ ! -f target/bootstrap/Cargo.toml ]; then
    cat > target/bootstrap/Cargo.toml << 'EOF'
[package]
name = "dol-bootstrap"
version = "0.3.1"
edition = "2021"

[lib]
path = "src/lib.rs"
EOF
fi

# Count raw errors
cd target/bootstrap
RAW_ERRORS=$(cargo check 2>&1 | grep "^error\[" | wc -l || echo "0")
cd "$PROJECT_ROOT"

success "Generated Rust code (raw errors: $RAW_ERRORS)"

# ─────────────────────────────────────────────────────────────────────────────────
# STAGE 3: APPLY FIXES AND COMPILE
# ─────────────────────────────────────────────────────────────────────────────────
log "Stage 3: Applying fixes and compiling..."

if [ -x scripts/bootstrap-fix.sh ]; then
    if $VERBOSE; then
        ./scripts/bootstrap-fix.sh
    else
        ./scripts/bootstrap-fix.sh > /dev/null 2>&1
    fi
fi

cd target/bootstrap
FIXED_ERRORS=$(cargo check 2>&1 | grep "^error\[" | wc -l || echo "0")

if [ "$FIXED_ERRORS" -eq 0 ]; then
    if $VERBOSE; then
        cargo build --release
    else
        cargo build --release 2>/dev/null
    fi
    
    if [ $? -eq 0 ]; then
        success "Bootstrap compiled successfully"
        BUILD_SUCCESS=true
    else
        fail "Bootstrap build failed"
        BUILD_SUCCESS=false
    fi
else
    fail "Bootstrap has $FIXED_ERRORS errors after fixes"
    BUILD_SUCCESS=false
fi

cd "$PROJECT_ROOT"

# ─────────────────────────────────────────────────────────────────────────────────
# STAGE 4: RUN TESTS
# ─────────────────────────────────────────────────────────────────────────────────
if ! $QUICK; then
    log "Stage 4: Running tests..."
    
    if $VERBOSE; then
        TEST_OUTPUT=$(cargo test --lib 2>&1)
        echo "$TEST_OUTPUT"
    else
        TEST_OUTPUT=$(cargo test --lib 2>&1)
    fi
    
    TEST_PASSED=$(echo "$TEST_OUTPUT" | grep -oP '\d+ passed' | grep -oP '\d+' || echo "0")
    TEST_FAILED=$(echo "$TEST_OUTPUT" | grep -oP '\d+ failed' | grep -oP '\d+' || echo "0")
    TEST_TOTAL=$((TEST_PASSED + TEST_FAILED))
    
    if [ "$TEST_FAILED" -eq 0 ]; then
        success "Tests: $TEST_PASSED/$TEST_TOTAL passed"
    else
        fail "Tests: $TEST_PASSED passed, $TEST_FAILED failed"
    fi
else
    TEST_PASSED="skipped"
    TEST_TOTAL="skipped"
    log "Stage 4: Skipped (--quick mode)"
fi

# ─────────────────────────────────────────────────────────────────────────────────
# STAGE 5: METRICS
# ─────────────────────────────────────────────────────────────────────────────────
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo ""
echo "═══════════════════════════════════════════════════════════════════════════════"
echo "                         📊 FLYWHEEL REPORT                                    "
echo "═══════════════════════════════════════════════════════════════════════════════"
echo ""
echo "  DOL Source:     $DOL_FILES files, $DOL_LINES lines"
echo "  Parse:          $PARSE_PASSED passed"
echo "  Raw Errors:     $RAW_ERRORS"
echo "  Fixed Errors:   $FIXED_ERRORS"
echo "  Build:          $($BUILD_SUCCESS && echo 'SUCCESS' || echo 'FAILED')"
echo "  Tests:          $TEST_PASSED/$TEST_TOTAL"
echo "  Duration:       ${DURATION}s"
echo ""

# Calculate progress
if [ "$RAW_ERRORS" -gt 0 ] && [ "$FIXED_ERRORS" -eq 0 ]; then
    REDUCTION="100%"
    echo "  Fix Script:     REQUIRED (reduces $RAW_ERRORS → 0)"
elif [ "$RAW_ERRORS" -eq 0 ]; then
    REDUCTION="N/A"
    echo "  Fix Script:     NOT NEEDED 🎉"
else
    REDUCTION="$((100 - (FIXED_ERRORS * 100 / RAW_ERRORS)))%"
    echo "  Fix Script:     PARTIAL ($REDUCTION reduction)"
fi

echo ""
echo "═══════════════════════════════════════════════════════════════════════════════"

# Record metrics
mkdir -p .metrics
echo "$(date -Iseconds),local,$DOL_FILES,$DOL_LINES,$RAW_ERRORS,$FIXED_ERRORS,$TEST_PASSED,$TEST_TOTAL,$DURATION" >> "$METRICS_FILE"

# Exit status
if $BUILD_SUCCESS && [ "$FIXED_ERRORS" -eq 0 ]; then
    exit 0
else
    exit 1
fi
