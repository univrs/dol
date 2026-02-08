#!/bin/bash
# Start Node 1

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/.."

echo "Starting Node 1..."
cargo run -- start --name node1 --port 9001
