cat > ./validate-v0.3.0.sh << 'EOF'
#!/bin/bash
set -e

DOL_DIR=~/repos/univrs-dol
DOL_CHECK="cargo run --release --bin dol-check --"

cd $DOL_DIR
git checkout feature/hir-v0.3.0
cargo build --release

echo "=== Validating univrs-orchestration ==="
$DOL_CHECK ~/repos/univrs-orchestration/ontology/ 2>&1 | tee /tmp/orchestration.log

echo "=== Validating univrs-network ==="
$DOL_CHECK ~/repos/univrs-network/**/*.dol 2>&1 | tee /tmp/network.log

echo "=== Validating univrs-state ==="
$DOL_CHECK ~/repos/univrs-state/**/*.dol 2>&1 | tee /tmp/state.log

echo "=== Validating univrs-identity ==="
$DOL_CHECK ~/repos/univrs-identity/**/*.dol 2>&1 | tee /tmp/identity.log

echo "=== Validating univrs-vudo ==="
$DOL_CHECK ~/repos/univrs-vudo/**/*.dol 2>&1 | tee /tmp/vudo.log

echo "=== Summary ==="
grep -c "warning\|error" /tmp/*.log || echo "All clean!"
EOF

chmod +x ./validate-v0.3.0.sh
