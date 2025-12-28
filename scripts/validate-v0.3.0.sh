#!/bin/bash
set -e

DOL_DIR=~/repos/univrs-dol

cd $DOL_DIR
git checkout feature/hir-v0.3.0
cargo build --release --features cli

echo "=== Validating univrs-orchestration ==="
cargo run --release --features cli --bin dol-check -- ~/repos/univrs-orchestration/ontology/ 2>&1 | tee /tmp/orchestration.log

echo "=== Validating univrs-network ==="
find ~/repos/univrs-network -name "*.dol" -exec cargo run --release --features cli --bin dol-check -- {} \; 2>&1 | tee /tmp/network.log

echo "=== Validating univrs-state ==="
find ~/repos/univrs-state -name "*.dol" -exec cargo run --release --features cli --bin dol-check -- {} \; 2>&1 | tee /tmp/state.log

echo "=== Validating univrs-identity ==="
find ~/repos/univrs-identity -name "*.dol" -exec cargo run --release --features cli --bin dol-check -- {} \; 2>&1 | tee /tmp/identity.log

echo "=== Validating univrs-vudo ==="
find ~/repos/univrs-vudo -name "*.dol" -exec cargo run --release --features cli --bin dol-check -- {} \; 2>&1 | tee /tmp/vudo.log

echo "=== Summary ==="
echo "Errors:"
grep -l "error" /tmp/*.log 2>/dev/null || echo "  None!"
echo "Warnings:"
grep -c "warning" /tmp/*.log 2>/dev/null || echo "  None!"

echo "=== Summary ==="
grep -c "warning\|error" /tmp/*.log || echo "All clean!"
