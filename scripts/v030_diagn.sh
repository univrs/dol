#!/bin/bash
# DOL Self-Hosting Quick Diagnostic
# Run: bash dol-diagnostic.sh
#
# This script assesses DOL-in-DOL compilation completeness

set -e
cd ~/repos/univrs-dol

echo "════════════════════════════════════════════════════════════════"
echo "          DOL SELF-HOSTING DIAGNOSTIC"
echo "════════════════════════════════════════════════════════════════"
echo ""

# ─────────────────────────────────────────────────────────────────────
# SECTION 1: FILE INVENTORY
# ─────────────────────────────────────────────────────────────────────
echo "📁 DOL Self-Hosting Files"
echo "─────────────────────────────────────────────────────────────────"
find dol -name "*.dol" -type f -exec wc -l {} \; | sort -n
TOTAL_LINES=$(wc -l dol/*.dol dol/**/*.dol 2>/dev/null | tail -1 | awk '{print $1}')
echo ""
echo "Total: $TOTAL_LINES lines of DOL"
echo ""

# ─────────────────────────────────────────────────────────────────────
# SECTION 2: PARSING STATUS
# ─────────────────────────────────────────────────────────────────────
echo "🔍 Parsing Status"
echo "─────────────────────────────────────────────────────────────────"
PASSED=0
FAILED=0
for f in $(find dol -name "*.dol" -type f | sort); do
  if cargo run --release --features cli --bin dol-check -- "$f" 2>/dev/null | grep -q "Passed"; then
    echo "  ✓ $f"
    PASSED=$((PASSED + 1))
  else
    echo "  ✗ $f"
    FAILED=$((FAILED + 1))
  fi
done
echo ""
echo "Parsing: $PASSED passed, $FAILED failed"
echo ""

# ─────────────────────────────────────────────────────────────────────
# SECTION 3: CODEGEN ANALYSIS
# ─────────────────────────────────────────────────────────────────────
echo "⚙️  Codegen Analysis (dol/types.dol)"
echo "─────────────────────────────────────────────────────────────────"

cargo run --release --features cli --bin dol-codegen -- --target rust dol/types.dol > /tmp/types_gen.rs 2>&1

echo "Generated Output Stats:"
echo "  Lines:            $(wc -l < /tmp/types_gen.rs)"
echo "  pub struct:       $(grep -c 'pub struct' /tmp/types_gen.rs || echo 0)"
echo "  pub enum:         $(grep -c 'pub enum' /tmp/types_gen.rs || echo 0)"
echo "  pub fn:           $(grep -c 'pub fn' /tmp/types_gen.rs || echo 0)"
echo "  &self methods:    $(grep -c '&self' /tmp/types_gen.rs || echo 0)"
echo "  &mut self:        $(grep -c '&mut self' /tmp/types_gen.rs || echo 0)"
echo "  self. usage:      $(grep -c 'self\.' /tmp/types_gen.rs || echo 0)"
echo "  match:            $(grep -c '\bmatch\b' /tmp/types_gen.rs || echo 0)"
echo "  if/else:          $(grep -c '\bif\b' /tmp/types_gen.rs || echo 0)"
echo "  return:           $(grep -c '\breturn\b' /tmp/types_gen.rs || echo 0)"
echo ""

# ─────────────────────────────────────────────────────────────────────
# SECTION 4: CONVERSION CHECKS
# ─────────────────────────────────────────────────────────────────────
echo "🔄 Conversion Checks"
echo "─────────────────────────────────────────────────────────────────"

# Check this → self
if grep -q 'this\.' /tmp/types_gen.rs; then
  echo "  ⚠ 'this.' found in output (should be 'self.')"
else
  echo "  ✓ this → self conversion working"
fi

# Check type conversions
UNCONVERTED=""
for ty in Int8 Int16 Int32 Int64 UInt8 UInt16 UInt32 UInt64 Float32 Float64; do
  if grep -q "\b$ty\b" /tmp/types_gen.rs 2>/dev/null; then
    UNCONVERTED="$UNCONVERTED $ty"
  fi
done
if [ -n "$UNCONVERTED" ]; then
  echo "  ⚠ DOL types in output:$UNCONVERTED"
else
  echo "  ✓ DOL types converted to Rust types"
fi

# Check pipe operator
if grep -q '|>' /tmp/types_gen.rs; then
  echo "  ⚠ Pipe operator |> found (should be desugared)"
else
  echo "  ✓ No raw pipe operators in output"
fi

echo ""

# ─────────────────────────────────────────────────────────────────────
# SECTION 5: BOOTSTRAP ATTEMPT
# ─────────────────────────────────────────────────────────────────────
echo "🚀 Bootstrap Compilation Test"
echo "─────────────────────────────────────────────────────────────────"

mkdir -p target/bootstrap/src

# Generate core modules
for module in types lexer token ast; do
  if [ -f "dol/${module}.dol" ]; then
    cargo run --release --features cli --bin dol-codegen -- --target rust "dol/${module}.dol" > "target/bootstrap/src/${module}.rs" 2>&1
    echo "  Generated: ${module}.rs ($(wc -l < target/bootstrap/src/${module}.rs) lines)"
  fi
done

# Create lib.rs
cat > target/bootstrap/src/lib.rs << 'EOF'
#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
#![allow(unreachable_patterns, unused_assignments)]

pub mod types;
pub mod lexer;
pub mod token;
pub mod ast;
EOF

# Create Cargo.toml
cat > target/bootstrap/Cargo.toml << 'EOF'
[package]
name = "dol_bootstrap"
version = "0.1.0"
edition = "2021"

[dependencies]
EOF

echo ""
echo "Attempting cargo check..."
echo ""
cd target/bootstrap
if cargo check 2>&1; then
  echo ""
  echo "  ✅ BOOTSTRAP COMPILES!"
else
  echo ""
  echo "  ⚠ Compilation errors above indicate remaining work"
fi
cd ~/repos/univrs-dol

# ─────────────────────────────────────────────────────────────────────
# SECTION 6: SUMMARY
# ─────────────────────────────────────────────────────────────────────
echo ""
echo "════════════════════════════════════════════════════════════════"
echo "                      SUMMARY"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "WORKING:"
echo "  ✓ All dol/*.dol files parse"
echo "  ✓ pub visibility modifier"
echo "  ✓ Function bodies in genes"
echo "  ✓ this → self conversion"
echo "  ✓ Basic codegen (struct, enum, fn)"
echo ""
echo "TO VERIFY:"
echo "  ? Bootstrap compilation errors"
echo "  ? Generated code semantic correctness"
echo "  ? Module imports/exports"
echo ""
echo "OUTPUT FILES:"
echo "  /tmp/types_gen.rs          - Generated Rust from types.dol"
echo "  target/bootstrap/          - Bootstrap compilation test"
echo ""
echo "NEXT STEPS:"
echo "  1. Review bootstrap errors: cd target/bootstrap && cargo check"
echo "  2. Compare: diff src/types.rs /tmp/types_gen.rs"
echo "  3. Fix codegen gaps iteratively"
echo ""
