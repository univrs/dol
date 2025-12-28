#!/bin/bash
# Diagnose the 28 remaining errors precisely
# Run from ~/repos/univrs-dol

cd ~/repos/univrs-dol

echo "═══════════════════════════════════════════════════════════════"
echo "          DIAGNOSING 28 REMAINING ERRORS"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# ─────────────────────────────────────────────────────────────────
echo "1. SEMICOLON ISSUE - Check function definitions in generated code"
echo "─────────────────────────────────────────────────────────────────"
echo ""
echo "Generated is_digit function:"
grep -A4 "pub fn is_digit" target/bootstrap/src/lexer.rs
echo ""
echo "DOL source:"
grep -A4 "pub fun is_digit" dol/lexer.dol
echo ""

# ─────────────────────────────────────────────────────────────────
echo "2. MISSING TokenKind VARIANTS - Check what's defined"
echo "─────────────────────────────────────────────────────────────────"
echo ""
echo "Missing variants referenced in generated lexer.rs:"
for v in RIdiom LIdiom Map Caret Bar Assign Ampersand; do
    if grep -q "TokenKind::$v" target/bootstrap/src/lexer.rs; then
        echo "  Used: TokenKind::$v"
    fi
done
echo ""
echo "Check if these are in token.dol:"
for v in RIdiom LIdiom Map Caret Bar Assign Ampersand; do
    if grep -q "$v" dol/token.dol; then
        echo "  Found in token.dol: $v"
    else
        echo "  MISSING from token.dol: $v"
    fi
done
echo ""

# ─────────────────────────────────────────────────────────────────
echo "3. STRING METHODS - char_at and substring usage"
echo "─────────────────────────────────────────────────────────────────"
echo ""
echo "In DOL source (lexer.dol):"
grep -n "char_at\|substring" dol/lexer.dol | head -5
echo ""
echo "These methods don't exist in Rust. Need translation."
echo ""

# ─────────────────────────────────────────────────────────────────
echo "4. CONSTRUCTOR ARITY - Lexer::new issue"
echo "─────────────────────────────────────────────────────────────────"
echo ""
echo "Generated Lexer::new signature:"
grep -A1 "pub fn new.*Lexer" target/bootstrap/src/lexer.rs | head -3
echo ""
echo "How it's called:"
grep "Lexer::new(" target/bootstrap/src/lexer.rs | head -2
echo ""
echo "DOL source - how Lexer.new is called:"
grep "Lexer.new\|Lexer::new\|Lexer {" dol/lexer.dol | head -3
echo ""

# ─────────────────────────────────────────────────────────────────
echo "5. keyword_lookup FUNCTION"
echo "─────────────────────────────────────────────────────────────────"
echo ""
echo "In DOL source:"
grep -n "keyword_lookup\|fun keyword" dol/lexer.dol dol/token.dol | head -5
echo ""

# ─────────────────────────────────────────────────────────────────
echo "6. DOUBLE QUESTION MARK"
echo "─────────────────────────────────────────────────────────────────"
echo ""
echo "In generated code:"
grep -n "??" target/bootstrap/src/lexer.rs | head -3
echo ""

echo "═══════════════════════════════════════════════════════════════"
echo "                    SUMMARY"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "These are CODEGEN issues that need fixing in src/codegen/rust.rs:"
echo ""
echo "1. SEMICOLONS: Don't add ';' after final expression in function body"
echo "2. TOKEN VARIANTS: Add missing variants to token.dol OR map names"
echo "3. STRING METHODS: Translate char_at→chars().nth(), substring→slice"
echo "4. CONSTRUCTORS: Generate proper factory functions"
echo "5. FUNCTIONS: Ensure keyword_lookup is generated/imported"
echo "6. OPERATORS: Don't double ?? operator"
echo ""
