cd ~/repos/univrs-dol

# Step 1: Regenerate CLEAN files
for module in types token ast lexer; do
  cargo run --release --features cli --bin dol-codegen -- \
    --target rust "dol/${module}.dol" 2>/dev/null \
    > "target/bootstrap/src/${module}.rs"
done

# Step 2: Show the EXACT content of the problematic functions BEFORE any fixes
echo "=== is_ident_start ===" 
grep -A3 "pub fn is_ident_start" target/bootstrap/src/lexer.rs

echo "=== is_ident_continue ==="
grep -A3 "pub fn is_ident_continue" target/bootstrap/src/lexer.rs

echo "=== is_digit ==="
grep -A3 "pub fn is_digit" target/bootstrap/src/lexer.rs

echo "=== make_token ==="
grep -A5 "pub fn make_token" target/bootstrap/src/lexer.rs
