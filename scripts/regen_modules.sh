# Regenerate all modules
for module in types token ast lexer; do
  cargo run --release --features cli --bin dol-codegen -- \
    --target rust "dol/${module}.dol" 2>/dev/null \
    > "target/bootstrap/src/${module}.rs"
done

# Add imports
bash ~/repos/univrs-dol/scripts/add_imports.sh

# Test - should drop from 28 to ~21
cargo check 2>&1 | grep "^error\[" | wc -l
