#!/bin/bash
# SIMPLE FIX - Just remove trailing semicolons from specific lines
# Run from ~/repos/univrs-dol

set -e
cd ~/repos/univrs-dol

echo "═══════════════════════════════════════════════════════════════"
echo "          REGENERATING AND FIXING BOOTSTRAP (SIMPLE)"
echo "═══════════════════════════════════════════════════════════════"

# Step 0: Regenerate fresh files
echo "Step 0: Regenerating from DOL source..."
for module in types token ast lexer; do
  cargo run --release --features cli --bin dol-codegen -- \
    --target rust "dol/${module}.dol" 2>/dev/null \
    > "target/bootstrap/src/${module}.rs"
done

cd target/bootstrap/src

# ═══════════════════════════════════════════════════════════════════
# Step 1: Add imports
# ═══════════════════════════════════════════════════════════════════
echo "Step 1: Adding imports..."

sed -i '4a use crate::token::Span;' ast.rs
sed -i '4a use crate::ast::{BinOp, Stmt, WherePredicate, Expr};' lexer.rs
sed -i '5a use crate::token::{Token, TokenKind, Span, keyword_lookup};' lexer.rs
sed -i '4a use crate::ast::{BinOp, Stmt, WherePredicate, Expr};' token.rs

# ═══════════════════════════════════════════════════════════════════
# Step 2: Fix char_at → chars().nth()
# ═══════════════════════════════════════════════════════════════════
echo "Step 2: Fixing char_at..."
sed -i 's/self\.source\.char_at(self\.pos)/self.source.chars().nth(self.pos as usize).unwrap()/g' lexer.rs
sed -i 's/self\.source\.char_at(target)/self.source.chars().nth(target as usize).unwrap()/g' lexer.rs

# ═══════════════════════════════════════════════════════════════════
# Step 3: Fix substring → slice
# ═══════════════════════════════════════════════════════════════════
echo "Step 3: Fixing substring..."
sed -i 's/self\.source\.substring(start, self\.pos)/self.source[start as usize..self.pos as usize].to_string()/g' lexer.rs

# ═══════════════════════════════════════════════════════════════════
# Step 4: Fix isize/usize comparisons
# ═══════════════════════════════════════════════════════════════════
echo "Step 4: Fixing type casts..."
sed -i 's/(self\.pos >= self\.source\.len())/(self.pos as usize >= self.source.len())/g' lexer.rs
sed -i 's/(target >= self\.source\.len())/(target as usize >= self.source.len())/g' lexer.rs

# ═══════════════════════════════════════════════════════════════════
# Step 5: Fix Lexer constructor
# ═══════════════════════════════════════════════════════════════════
echo "Step 5: Fixing Lexer constructor..."
sed -i 's/let mut lexer = Lexer::new(source);/let mut lexer = Lexer { source, pos: 0, line: 1, column: 1, tokens: vec![] };/' lexer.rs

# ═══════════════════════════════════════════════════════════════════
# Step 6: Fix ?? operator
# ═══════════════════════════════════════════════════════════════════
echo "Step 6: Fixing ?? operator..."
sed -i 's/keyword_lookup(text)??/keyword_lookup(\&text).unwrap_or(TokenKind::Identifier)/g' lexer.rs

# ═══════════════════════════════════════════════════════════════════
# Step 7: Fix trailing semicolons by finding specific lines
# Strategy: Find the function, then find the body line, remove trailing ;
# ═══════════════════════════════════════════════════════════════════
echo "Step 7: Fixing semicolons..."

# is_ident_start - find line number of the expression (line after function sig)
LINE=$(grep -n "pub fn is_ident_start" lexer.rs | cut -d: -f1)
EXPR_LINE=$((LINE + 1))
echo "  is_ident_start expression at line $EXPR_LINE"
# Remove trailing semicolon from that specific line
sed -i "${EXPR_LINE}s/;$//" lexer.rs

# is_ident_continue
LINE=$(grep -n "pub fn is_ident_continue" lexer.rs | cut -d: -f1)
EXPR_LINE=$((LINE + 1))
echo "  is_ident_continue expression at line $EXPR_LINE"
sed -i "${EXPR_LINE}s/;$//" lexer.rs

# is_digit
LINE=$(grep -n "pub fn is_digit" lexer.rs | cut -d: -f1)
EXPR_LINE=$((LINE + 1))
echo "  is_digit expression at line $EXPR_LINE"
sed -i "${EXPR_LINE}s/;$//" lexer.rs

# make_token - the Token::new is 2 lines after the function signature
LINE=$(grep -n "pub fn make_token" lexer.rs | cut -d: -f1)
TOKEN_LINE=$((LINE + 2))
echo "  make_token Token::new at line $TOKEN_LINE"
sed -i "${TOKEN_LINE}s/;$//" lexer.rs

# ═══════════════════════════════════════════════════════════════════
# VERIFY the fixes look right
# ═══════════════════════════════════════════════════════════════════
echo ""
echo "Verifying patterns:"

echo "=== is_ident_start (should end with ))) not )));) ==="
grep -A2 "pub fn is_ident_start" lexer.rs | tail -2

echo "=== is_digit (should end with )) not ));) ==="
grep -A2 "pub fn is_digit" lexer.rs | tail -2

echo "=== make_token (Token::new should end with )) not ));) ==="
grep -A3 "pub fn make_token" lexer.rs | tail -2

# ═══════════════════════════════════════════════════════════════════
# TEST COMPILATION
# ═══════════════════════════════════════════════════════════════════
echo ""
echo "Testing compilation..."
cd ~/repos/univrs-dol/target/bootstrap

# Check for syntax errors first
if cargo check 2>&1 | grep -q "unclosed delimiter\|mismatched closing"; then
    echo "✗ Syntax errors found!"
    cargo check 2>&1 | grep -B2 -A2 "unclosed delimiter\|mismatched closing" | head -20
    exit 1
fi

ERROR_COUNT=$(cargo check 2>&1 | grep "^error\[" | wc -l)
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "          RESULT: $ERROR_COUNT errors"
echo "═══════════════════════════════════════════════════════════════"

if [ "$ERROR_COUNT" -eq 0 ]; then
    echo "✓ Bootstrap compiles! Building release..."
    cargo build --release 2>&1 | tail -3
else
    cargo check 2>&1 | grep "^error\[" | sort | uniq -c | sort -rn
fi
