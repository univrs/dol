# Iroh P2P POC - Quick Start

## Build

```bash
cd prototypes/iroh-sync-poc
cargo build --release
```

## Run Two Nodes

### Terminal 1: Start Node 1
```bash
./scripts/run-node1.sh
```

Copy the Node ID from the output (looks like: `abcd1234...`)

### Terminal 2: Start Node 2 and Connect
```bash
./scripts/run-node2.sh <paste-node1-id-here>
```

## What You'll See

Both nodes will:
1. Start up and show their Node IDs
2. Establish a P2P connection
3. Each add a sample todo item
4. Sync their todo lists every 5 seconds
5. Display the combined todo list from both nodes

Example output:
```
[node1] Current todos:
  - [ ] Task from node1
  - [ ] Task from node2
```

## Run Test Scenarios

```bash
# Same LAN
cargo run -- test --scenario S1

# Partition healing (reconnection + CRDT convergence)
cargo run -- test --scenario S6
```

## Next Steps

1. Review the connectivity report: `../../docs/research/iroh-connectivity-report.md`
2. Test on real networks (different WiFi, cellular, etc.)
3. Measure actual performance metrics
4. Decide on browser strategy

## Troubleshooting

**Build fails**: Make sure you have Rust 1.70+ installed
**Connection fails**: Check firewall settings, try different ports
**Can't find peer**: Make sure both nodes are on the same network for S1 test

## Key Files

- `src/main.rs` - CLI entry point
- `src/app.rs` - TodoApp with Automerge CRDT
- `src/p2p/node.rs` - Iroh P2P implementation
- `src/tests/mod.rs` - Test scenarios (S1-S6)
- `README.md` - Full documentation
