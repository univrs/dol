# First regenerate to get clean files
cd ~/repos/univrs-dol
#for module in types token ast lexer; do
#  cargo run --release --features cli --bin dol-codegen -- \
#    --target rust "dol/${module}.dol" 2>/dev/null \
#    > "target/bootstrap/src/${module}.rs"
#done
# Regenerate clean files
for module in types token ast lexer; do
  cargo run --release --features cli --bin dol-codegen -- \
    --target rust "dol/${module}.dol" 2>/dev/null \
    > "target/bootstrap/src/${module}.rs"
done

# Show the is_digit function before any fixes
grep -A3 "pub fn is_digit" target/bootstrap/src/lexer.rs
# Apply imports
bash ~/repos/univrs-dol/scripts/add_imports.sh

# Apply the 21-error fixes
bash ~/repos/univrs-dol/scripts/fix-21-simple.sh
