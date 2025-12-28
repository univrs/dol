#!/bin/bash
# PRECISE FIX SCRIPT - Based on exact output from fresh regeneration
# Run from ~/repos/univrs-dol

set -e
cd ~/repos/univrs-dol

#!/bin/bash
# PRECISE FIX SCRIPT - Based on exact output from fresh regeneration
# Run from ~/repos/univrs-dol

set -e
cd ~/repos/univrs-dol

echo "═══════════════════════════════════════════════════════════════"
echo "          REGENERATING AND FIXING BOOTSTRAP (PRECISE)"
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
# Step 7: Fix trailing semicolons - PRECISE PATTERNS
# These patterns match EXACTLY what was shown in the fresh output
# ═══════════════════════════════════════════════════════════════════
echo "Step 7: Fixing semicolons (precise patterns)..."

# is_ident_start: (c == '_')));  →  (c == '_')))
sed -i "s/(c == '_')));/(c == '_')))/g" lexer.rs

# is_ident_continue: (c <= '9')));  →  (c <= '9')))
sed -i "s/(c <= '9')));/(c <= '9')))/g" lexer.rs

# is_digit: (c <= '9'));  →  (c <= '9'))
# Be careful: this is different from the above - only 2 closing parens + semicolon
sed -i "s/((c >= '0') \&\& (c <= '9'));/((c >= '0') \&\& (c <= '9'))/g" lexer.rs

# make_token: The Token::new(...) line ends with );  →  )
# Pattern: u32)));  at end of line (before the closing brace)
sed -i 's/as u32)));$/as u32))/' lexer.rs

# ═══════════════════════════════════════════════════════════════════
# VERIFY
# ═══════════════════════════════════════════════════════════════════
echo ""
echo "Verifying..."

echo "=== is_ident_start ==="
grep -A2 "pub fn is_ident_start" lexer.rs

echo "=== is_digit ==="
grep -A2 "pub fn is_digit" lexer.rs

echo "=== make_token ==="
grep -A4 "pub fn make_token" lexer.rs

echo ""
echo "Testing compilation..."
cd ~/repos/univrs-dol/target/bootstrap

# Check for syntax errors
if cargo check 2>&1 | grep -q "unclosed delimiter\|mismatched closing"; then
    echo "✗ Syntax errors found!"
    cargo check 2>&1 | grep -B2 "unclosed delimiter\|mismatched closing" | head -15
    exit 1
fi

ERROR_COUNT=$(cargo check 2>&1 | grep "^error\[" | wc -l)
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "          RESULT: $ERROR_COUNT errors"
echo "═══════════════════════════════════════════════════════════════"

if [ "$ERROR_COUNT" -eq 0 ]; then
    echo "✓ Bootstrap compiles successfully!"
    echo ""
    echo "Building release..."
    cargo build --release 2>&1 | tail -3
else
    echo ""
    cargo check 2>&1 | grep "^error\[" | sort | uniq -c | sort -rn
fi

