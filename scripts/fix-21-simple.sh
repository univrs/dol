#!/bin/bash
# Fix 21 bootstrap errors - Step by step
# Run from ~/repos/univrs-dol/target/bootstrap/src

cd ~/repos/univrs-dol/target/bootstrap/src

echo "Fixing 21 bootstrap errors..."

# ─────────────────────────────────────────────────────────────────
# STEP 1: Add keyword_lookup to imports
# ─────────────────────────────────────────────────────────────────
echo "Step 1: keyword_lookup import"

# Current: use crate::token::{Token, TokenKind, Span};
# Needed:  use crate::token::{Token, TokenKind, Span, keyword_lookup};

sed -i 's/use crate::token::{Token, TokenKind, Span}/use crate::token::{Token, TokenKind, Span, keyword_lookup}/' lexer.rs

# ─────────────────────────────────────────────────────────────────
# STEP 2: Fix char_at method calls (2 occurrences)
# ─────────────────────────────────────────────────────────────────
echo "Step 2: char_at → chars().nth()"

# self.source.char_at(self.pos) → self.source.chars().nth(self.pos as usize).unwrap()
sed -i 's/self\.source\.char_at(self\.pos)/self.source.chars().nth(self.pos as usize).unwrap()/g' lexer.rs
sed -i 's/self\.source\.char_at(target)/self.source.chars().nth(target as usize).unwrap()/g' lexer.rs

# ─────────────────────────────────────────────────────────────────
# STEP 3: Fix substring method calls (2 occurrences)
# ─────────────────────────────────────────────────────────────────
echo "Step 3: substring → slice"

# self.source.substring(start, self.pos) → self.source[start as usize..self.pos as usize].to_string()
sed -i 's/self\.source\.substring(start, self\.pos)/self.source[start as usize..self.pos as usize].to_string()/g' lexer.rs

# ─────────────────────────────────────────────────────────────────
# STEP 4: Fix isize/usize comparisons
# ─────────────────────────────────────────────────────────────────
echo "Step 4: Type casts for len() comparisons"

sed -i 's/(self\.pos >= self\.source\.len())/(self.pos as usize >= self.source.len())/g' lexer.rs
sed -i 's/(target >= self\.source\.len())/(target as usize >= self.source.len())/g' lexer.rs

# ─────────────────────────────────────────────────────────────────
# STEP 5: Fix trailing semicolons on expression functions
# ─────────────────────────────────────────────────────────────────
echo "Step 5: Removing trailing semicolons"

# These are the three helper functions that return bool
# is_ident_start, is_ident_continue, is_digit

# Pattern: The function body has a single expression with a trailing semicolon
# We need to remove that semicolon

# Use a more targeted approach - replace the specific patterns
# Line ~55: ((c >= 'a') && ...)); → ((c >= 'a') && ...))
sed -i "s/)));$/))/" lexer.rs

# ─────────────────────────────────────────────────────────────────
# STEP 6: Fix Lexer constructor call
# ─────────────────────────────────────────────────────────────────
echo "Step 6: Lexer constructor"

# Lexer::new(source) → Lexer { source, pos: 0, line: 1, column: 1, tokens: vec![] }
sed -i 's/let mut lexer = Lexer::new(source);/let mut lexer = Lexer { source, pos: 0, line: 1, column: 1, tokens: vec![] };/' lexer.rs

# ─────────────────────────────────────────────────────────────────
# STEP 7: Fix ?? operator (nullish coalescing)
# ─────────────────────────────────────────────────────────────────
echo "Step 7: ?? operator translation"

# keyword_lookup(text)?? → keyword_lookup(&text).unwrap_or(TokenKind::Identifier)
# Note: The DOL had "keyword_lookup(text) ?? TokenKind.Identifier"
sed -i 's/keyword_lookup(text)??/keyword_lookup(\&text).unwrap_or(TokenKind::Identifier)/g' lexer.rs

# ─────────────────────────────────────────────────────────────────
# Verify
# ─────────────────────────────────────────────────────────────────
echo ""
echo "Testing..."
cd ~/repos/univrs-dol/target/bootstrap
cargo check 2>&1 | grep "^error\[" | wc -l
