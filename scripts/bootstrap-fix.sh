#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════════
# DOL BOOTSTRAP FIX SCRIPT
# ═══════════════════════════════════════════════════════════════════════════════
#
# This script applies all necessary fixes to generated Rust code after running
# dol-codegen. These fixes compensate for current codegen limitations.
#
# Usage:
#   ./scripts/bootstrap-fix.sh [--regenerate]
#
# Options:
#   --regenerate  First regenerate all files from DOL source, then apply fixes
#
# ═══════════════════════════════════════════════════════════════════════════════

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BOOTSTRAP_DIR="$PROJECT_ROOT/target/bootstrap"
BOOTSTRAP_SRC="$BOOTSTRAP_DIR/src"

cd "$PROJECT_ROOT"

echo "═══════════════════════════════════════════════════════════════════════════════"
echo "                    DOL BOOTSTRAP FIX SCRIPT"
echo "═══════════════════════════════════════════════════════════════════════════════"
echo ""

# ─────────────────────────────────────────────────────────────────────────────────
# STEP 0: Optionally regenerate from DOL source
# ─────────────────────────────────────────────────────────────────────────────────
if [[ "$1" == "--regenerate" ]]; then
    echo "Step 0: Regenerating from DOL source..."
    mkdir -p "$BOOTSTRAP_SRC"
    for module in types token ast lexer; do
        echo "  Generating ${module}.rs..."
        cargo run --release --features cli --bin dol-codegen -- \
            --target rust "dol/${module}.dol" 2>/dev/null \
            > "$BOOTSTRAP_SRC/${module}.rs"
    done
    echo ""
fi

cd "$BOOTSTRAP_SRC"

# ─────────────────────────────────────────────────────────────────────────────────
# STEP 1: Add cross-module imports
# ─────────────────────────────────────────────────────────────────────────────────
echo "Step 1: Adding cross-module imports..."

# ast.rs needs Span from token
if ! grep -q "use crate::token::Span" ast.rs 2>/dev/null; then
    sed -i '4a use crate::token::Span;' ast.rs
    echo "  ✓ Added Span import to ast.rs"
fi

# lexer.rs needs types from ast and token
if ! grep -q "use crate::ast" lexer.rs 2>/dev/null; then
    sed -i '4a use crate::ast::{BinOp, Stmt, WherePredicate, Expr};' lexer.rs
    echo "  ✓ Added ast imports to lexer.rs"
fi
if ! grep -q "use crate::token" lexer.rs 2>/dev/null; then
    sed -i '5a use crate::token::{Token, TokenKind, Span, keyword_lookup};' lexer.rs
    echo "  ✓ Added token imports to lexer.rs"
fi

# token.rs needs types from ast
if ! grep -q "use crate::ast" token.rs 2>/dev/null; then
    sed -i '4a use crate::ast::{BinOp, Stmt, WherePredicate, Expr};' token.rs
    echo "  ✓ Added ast imports to token.rs"
fi

# ─────────────────────────────────────────────────────────────────────────────────
# STEP 2: String method translations
# ─────────────────────────────────────────────────────────────────────────────────
echo "Step 2: Translating string methods..."

sed -i 's/self\.source\.char_at(self\.pos)/self.source.chars().nth(self.pos as usize).unwrap()/g' lexer.rs
sed -i 's/self\.source\.char_at(target)/self.source.chars().nth(target as usize).unwrap()/g' lexer.rs
echo "  ✓ Translated char_at() calls"

sed -i 's/self\.source\.substring(start, self\.pos)/self.source[start as usize..self.pos as usize].to_string()/g' lexer.rs
echo "  ✓ Translated substring() calls"

# ─────────────────────────────────────────────────────────────────────────────────
# STEP 3: Type cast fixes (isize vs usize)
# ─────────────────────────────────────────────────────────────────────────────────
echo "Step 3: Fixing type casts..."

sed -i 's/(self\.pos >= self\.source\.len())/(self.pos as usize >= self.source.len())/g' lexer.rs
sed -i 's/(target >= self\.source\.len())/(target as usize >= self.source.len())/g' lexer.rs
echo "  ✓ Added usize casts for len() comparisons"

# ─────────────────────────────────────────────────────────────────────────────────
# STEP 4: Constructor fixes
# ─────────────────────────────────────────────────────────────────────────────────
echo "Step 4: Fixing constructors..."

sed -i 's/let mut lexer = Lexer::new(source);/let mut lexer = Lexer { source, pos: 0, line: 1, column: 1, tokens: vec![] };/' lexer.rs
echo "  ✓ Fixed Lexer constructor"

# ─────────────────────────────────────────────────────────────────────────────────
# STEP 5: Operator translations
# ─────────────────────────────────────────────────────────────────────────────────
echo "Step 5: Translating operators..."

sed -i 's/keyword_lookup(text)??/keyword_lookup(text.clone()).unwrap_or(TokenKind::Identifier)/g' lexer.rs
echo "  ✓ Translated ?? operator"

# ─────────────────────────────────────────────────────────────────────────────────
# STEP 6: Fix trailing semicolons on expression-bodied functions
# ─────────────────────────────────────────────────────────────────────────────────
echo "Step 6: Fixing trailing semicolons..."

fix_expr_semicolon() {
    local func_name="$1"
    local offset="${2:-1}"
    
    local line=$(grep -n "pub fn $func_name" lexer.rs | head -1 | cut -d: -f1)
    if [ -n "$line" ]; then
        local target=$((line + offset))
        sed -i "${target}s/;$//" lexer.rs
        echo "  ✓ Fixed $func_name (line $target)"
    fi
}

fix_expr_semicolon "is_ident_start"
fix_expr_semicolon "is_ident_continue"
fix_expr_semicolon "is_digit"
fix_expr_semicolon "make_token" 2

# ─────────────────────────────────────────────────────────────────────────────────
# STEP 7: Fix Token::new argument order
# ─────────────────────────────────────────────────────────────────────────────────
echo "Step 7: Fixing Token::new argument order..."

LINE=$(grep -n "pub fn make_token" lexer.rs | head -1 | cut -d: -f1)
if [ -n "$LINE" ]; then
    TOKEN_LINE=$((LINE + 2))
    CURRENT=$(sed -n "${TOKEN_LINE}p" lexer.rs)
    if echo "$CURRENT" | grep -q 'Token::new(kind, text, Span'; then
        sed -i "${TOKEN_LINE}c\\        Token::new(kind, Span::new(start, self.pos, self.line, (self.column - (self.pos - start) as u32)), text)" lexer.rs
        echo "  ✓ Fixed Token::new in make_token"
    fi
fi

# ─────────────────────────────────────────────────────────────────────────────────
# STEP 8: Fix more trailing semicolons
# ─────────────────────────────────────────────────────────────────────────────────
echo "Step 8: Fixing additional semicolons..."

LINE=$(grep -n "self\.make_token(kind, start);" lexer.rs | head -1 | cut -d: -f1)
if [ -n "$LINE" ]; then
    sed -i "${LINE}s/;$//" lexer.rs
    echo "  ✓ Fixed scan_identifier return"
fi

LINE=$(grep -n "self\.make_token(TokenKind::StringLit, start);" lexer.rs | head -1 | cut -d: -f1)
if [ -n "$LINE" ]; then
    sed -i "${LINE}s/;$//" lexer.rs
    echo "  ✓ Fixed scan_string return"
fi

LINE=$(grep -n "self\.make_token(kind, start);" lexer.rs | tail -1 | cut -d: -f1)
if [ -n "$LINE" ]; then
    sed -i "${LINE}s/;$//" lexer.rs
    echo "  ✓ Fixed scan_number return"
fi

sed -i 's/^};$/}/' lexer.rs
echo "  ✓ Fixed match block semicolons"

# ─────────────────────────────────────────────────────────────────────────────────
# STEP 9: Fix match arm type mismatches
# ─────────────────────────────────────────────────────────────────────────────────
echo "Step 9: Fixing match arm types..."

sed -i 's/self\.advance()$/self.advance();/' lexer.rs
echo "  ✓ Added semicolons to advance() calls in match arms"

# ─────────────────────────────────────────────────────────────────────────────────
# VERIFICATION
# ─────────────────────────────────────────────────────────────────────────────────
echo ""
echo "═══════════════════════════════════════════════════════════════════════════════"
echo "                         VERIFICATION"
echo "═══════════════════════════════════════════════════════════════════════════════"

cd "$BOOTSTRAP_DIR"

if cargo check 2>&1 | grep -q "unclosed delimiter\|mismatched closing"; then
    echo ""
    echo "✗ SYNTAX ERRORS DETECTED"
    cargo check 2>&1 | grep -B2 "unclosed delimiter\|mismatched closing" | head -15
    exit 1
fi

ERROR_COUNT=$(cargo check 2>&1 | grep "^error\[" | wc -l)

echo ""
if [ "$ERROR_COUNT" -eq 0 ]; then
    echo "✓ Bootstrap compiles successfully!"
    echo ""
    echo "Building release..."
    cargo build --release 2>&1 | tail -3
    echo ""
    echo "Output: $BOOTSTRAP_DIR/target/release/libdol_bootstrap.rlib"
else
    echo "✗ $ERROR_COUNT errors remaining"
    echo ""
    echo "Error breakdown:"
    cargo check 2>&1 | grep "^error\[" | sort | uniq -c | sort -rn
    exit 1
fi
