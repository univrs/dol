cd ~/repos/univrs-dol

# Fix: redirect stderr to /dev/null when capturing output
#for module in types lexer token ast; do
#  cargo run --release --features cli --bin dol-codegen -- --target rust "dol/${module}.dol" 2>/dev/null > "target/bootstrap/src/${module}.rs"
#done

# Or strip the first 2 lines (the cargo output)
for f in target/bootstrap/src/*.rs; do
  tail -n +3 "$f" > "$f.tmp" && mv "$f.tmp" "$f"

done

# Now try compilation again
cd target/bootstrap
cargo check
