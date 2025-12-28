# Re-run the full bootstrap test
cd ~/repos/univrs-dol

# Clean and regenerate
for module in types lexer token ast; do
  cargo run --release --features cli --bin dol-codegen -- \
    --target rust "dol/${module}.dol" 2>/dev/null \
    > "target/bootstrap/src/${module}.rs"
done

# Check error count
cd target/bootstrap
cargo check 2>&1 | grep "^error\[" | wc -l

# Show first 30 errors to see what's left
cargo check 2>&1 | head -60
