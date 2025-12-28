cd ~/repos/univrs-dol

# Step 1: CLEAN regenerate
echo "Regenerating..."
for module in types token ast lexer; do
  cargo run --release --features cli --bin dol-codegen -- \
    --target rust "dol/${module}.dol" 2>/dev/null \
    > "target/bootstrap/src/${module}.rs"
done

# Step 2: Verify we have FRESH files (should show ); at the end)
echo "Fresh is_ident_start:"
grep -A1 "pub fn is_ident_start" target/bootstrap/src/lexer.rs

# Step 3: Count the parens to understand the structure
echo ""
echo "Counting parens in is_ident_start expression:"
grep -A1 "pub fn is_ident_start" target/bootstrap/src/lexer.rs | tail -1 | grep -o ")" | wc -l
