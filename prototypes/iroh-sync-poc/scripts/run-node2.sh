#!/bin/bash
# Start Node 2 and connect to Node 1

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/.."

# Get Node 1 ID from user
if [ -z "$1" ]; then
    echo "Usage: $0 <node1-id>"
    echo "Get Node 1 ID from the first terminal where Node 1 is running"
    exit 1
fi

NODE1_ID=$1

echo "Starting Node 2 and connecting to Node 1..."
cargo run -- start --name node2 --port 9002 --connect "$NODE1_ID"
