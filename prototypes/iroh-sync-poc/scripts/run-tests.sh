#!/bin/bash
# Run all connectivity test scenarios

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/.."

echo "==================================="
echo "Iroh P2P Connectivity Test Suite"
echo "==================================="
echo ""

# Run each scenario
for scenario in S1 S2 S3 S4 S5 S6; do
    echo "Running Scenario $scenario..."
    cargo run -- test --scenario "$scenario"
    echo ""
    echo "Press Enter to continue to next scenario..."
    read
done

echo "All tests completed!"
