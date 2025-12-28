#!/bin/bash
# Fix 21 bootstrap errors - SURGICAL VERSION
# This version targets specific lines, not patterns
# Run from ~/repos/univrs-dol

set -e
cd ~/repos/univrs-dol

echo "═══════════════════════════════════════════════════════════════"
echo "          REGENERATING AND FIXING BOOTSTRAP (SURGICAL)"
echo "═══════════════════════════════════════════════════════════════"

# Step 0: Regenerate fresh files
echo "Step 0: Regenerating from DOL source..."
for module in types token ast lexer; do
  cargo run --release --features cli --bin dol-codegen -- \
    --target rust "dol/${module}.dol" 2>/dev/null \
    > "target/bootstrap/src/${module}.rs"
done

cd target/bootstrap/src

# Step 1: Add imports
echo "Step 1: Adding imports..."

# ast.rs
sed -i '4a use crate::token::Span;' ast.rs

# lexer.rs  
sed -i '4a use crate::ast::{BinOp, Stmt, WherePredicate, Expr};' lexer.rs
sed -i '5a use crate::token::{Token, TokenKind, Span, keyword_lookup};' lexer.rs

# token.rs
sed -i '4a use crate::ast::{BinOp, Stmt, WherePredicate, Expr};' token.rs

# Step 2: Fix char_at method calls
echo "Step 2: Fixing char_at..."
sed -i 's/self\.source\.char_at(self\.pos)/self.source.chars().nth(self.pos as usize).unwrap()/g' lexer.rs
sed -i 's/self\.source\.char_at(target)/self.source.chars().nth(target as usize).unwrap()/g' lexer.rs

# Step 3: Fix substring method calls
echo "Step 3: Fixing substring..."
sed -i 's/self\.source\.substring(start, self\.pos)/self.source[start as usize..self.pos as usize].to_string()/g' lexer.rs

# Step 4: Fix isize/usize comparisons
echo "Step 4: Fixing type casts..."
sed -i 's/(self\.pos >= self\.source\.len())/(self.pos as usize >= self.source.len())/g' lexer.rs
sed -i 's/(target >= self\.source\.len())/(target as usize >= self.source.len())/g' lexer.rs

# Step 5: Fix Lexer constructor
echo "Step 5: Fixing Lexer constructor..."
sed -i 's/let mut lexer = Lexer::new(source);/let mut lexer = Lexer { source, pos: 0, line: 1, column: 1, tokens: vec![] };/' lexer.rs

# Step 6: Fix ?? operator
echo "Step 6: Fixing ?? operator..."
sed -i 's/keyword_lookup(text)??/keyword_lookup(\&text).unwrap_or(TokenKind::Identifier)/g' lexer.rs

# Step 7: Fix ONLY the three boolean helper functions' trailing semicolons
echo "Step 7: Fixing expression semicolons (surgical)..."

# Find line numbers for the three helper functions
# These functions have a single expression as their body, which incorrectly has a semicolon

# is_ident_start: ((c >= 'a') && (c <= 'z')) || ((c >= 'A') && (c <= 'Z')) || (c == '_')
# is_ident_continue: is_ident_start(c) || is_digit(c)  
# is_digit: (c >= '0') && (c <= '9')

# Strategy: Find lines that match the pattern "fn is_X(...) -> bool" and fix the NEXT line

# For is_ident_start
LINE=$(grep -n "pub fn is_ident_start.*-> bool" lexer.rs | head -1 | cut -d: -f1)
if [ -n "$LINE" ]; then
    NEXT=$((LINE + 1))
    # Remove trailing semicolon from that line only
    sed -i "${NEXT}s/;$//" lexer.rs
    echo "  Fixed is_ident_start at line $NEXT"
fi

# For is_ident_continue
LINE=$(grep -n "pub fn is_ident_continue.*-> bool" lexer.rs | head -1 | cut -d: -f1)
if [ -n "$LINE" ]; then
    NEXT=$((LINE + 1))
    sed -i "${NEXT}s/;$//" lexer.rs
    echo "  Fixed is_ident_continue at line $NEXT"
fi

# For is_digit
LINE=$(grep -n "pub fn is_digit.*-> bool" lexer.rs | head -1 | cut -d: -f1)
if [ -n "$LINE" ]; then
    NEXT=$((LINE + 1))
    sed -i "${NEXT}s/;$//" lexer.rs
    echo "  Fixed is_digit at line $NEXT"
fi

# Step 8: Fix other return type mismatches
echo "Step 8: Checking for other return issues..."

# Functions returning Option<char>, Token, etc. that might have semicolon issues
# Let's find them by looking at the error messages after a check

# ═══════════════════════════════════════════════════════════════════
# VERIFY
# ═══════════════════════════════════════════════════════════════════
echo ""
echo "Testing syntax..."
cd ~/repos/univrs-dol/target/bootstrap

# Check for syntax errors first (unclosed delimiters etc)
if ! cargo check 2>&1 | grep -q "unclosed delimiter\|mismatched closing"; then
    echo "✓ No syntax errors"
else
    echo "✗ Syntax errors found:"
    cargo check 2>&1 | grep -B2 "unclosed delimiter\|mismatched closing" | head -10
    exit 1
fi

echo ""
echo "Counting type errors..."
ERROR_COUNT=$(cargo check 2>&1 | grep "^error\[" | wc -l)
echo "Errors remaining: $ERROR_COUNT"

if [ "$ERROR_COUNT" -gt 0 ]; then
    echo ""
    echo "Error breakdown:"
    cargo check 2>&1 | grep "^error\[" | sort | uniq -c | sort -rn
    
    echo ""
    echo "First few errors:"
    cargo check 2>&1 | grep -A2 "^error\[" | head -20
fi
