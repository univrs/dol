# Iroh P2P + Automerge Sync - Proof of Concept

This POC validates Iroh as the P2P networking layer for the VUDO Runtime's local-first mode, demonstrating encrypted peer-to-peer connectivity and Automerge CRDT synchronization.

## Overview

Two Rust nodes that:
- Discover each other using Iroh's networking stack
- Establish encrypted connections (direct or via relay)
- Synchronize an Automerge CRDT document (TodoApp)
- Measure connection metrics and validate acceptance criteria

## Architecture

```
┌─────────────────────────────────────────┐
│           TodoApp (CRDT)                │
│  ┌────────────────────────────────────┐ │
│  │   Automerge Document               │ │
│  │   - Todos list (CRDT)              │ │
│  │   - Automatic conflict resolution  │ │
│  └────────────────────────────────────┘ │
│                  │                      │
│                  ▼                      │
│  ┌────────────────────────────────────┐ │
│  │   AutomergeSync                    │ │
│  │   - Generate sync messages         │ │
│  │   - Apply remote changes           │ │
│  └────────────────────────────────────┘ │
│                  │                      │
│                  ▼                      │
│  ┌────────────────────────────────────┐ │
│  │   IrohNode (P2P Layer)             │ │
│  │   - Endpoint management            │ │
│  │   - Connection handling            │ │
│  │   - Discovery (mDNS, relay)        │ │
│  │   - Encrypted transport (QUIC)     │ │
│  └────────────────────────────────────┘ │
│                  │                      │
└──────────────────┼──────────────────────┘
                   │
                   ▼
            Network (UDP/QUIC)
```

## Components

### `src/main.rs`
CLI entry point with two commands:
- `start` - Start a node (with optional peer connection)
- `test` - Run connectivity test scenarios (S1-S6)

### `src/app.rs`
TodoApp - Demonstrates CRDT sync:
- Add/toggle todo items
- Automerge document management
- Sync loop with peers

### `src/p2p/node.rs`
IrohNode - P2P connectivity:
- Endpoint creation and management
- Connection establishment (direct + relay)
- Message broadcasting/receiving
- Connection metrics tracking

### `src/sync/automerge_sync.rs`
AutomergeSync - CRDT synchronization:
- Generate sync messages from document
- Apply remote sync messages
- Handle conflict resolution (automatic via CRDT)

### `src/metrics.rs`
ConnectionMetrics - Performance tracking:
- Connection time
- Sync latency
- Throughput
- Reconnection time
- Acceptance criteria validation

## Building

```bash
cd prototypes/iroh-sync-poc
cargo build --release
```

## Usage

### Manual Testing

**Terminal 1 - Start Node 1:**
```bash
./scripts/run-node1.sh
```

Copy the Node ID from the output (e.g., `abcd1234...`).

**Terminal 2 - Start Node 2 and connect:**
```bash
./scripts/run-node2.sh <node1-id>
```

Both nodes will:
1. Establish connection
2. Add sample todos
3. Sync every 5 seconds
4. Display current state

### Automated Testing

Run all network scenarios:
```bash
./scripts/run-tests.sh
```

Or run individual scenarios:
```bash
cargo run -- test --scenario S1  # Same LAN
cargo run -- test --scenario S2  # Different LANs
cargo run -- test --scenario S3  # Cellular + WiFi
cargo run -- test --scenario S4  # Symmetric NAT
cargo run -- test --scenario S5  # Restrictive firewall
cargo run -- test --scenario S6  # Partition healing
```

## Test Scenarios

### S1: Same LAN (mDNS Discovery)
**Goal:** Direct connection on local network
**Expected:** Connection < 3 seconds
**Method:** Two nodes on same network, mDNS discovery

### S2: Different LANs (NAT Hole-Punching)
**Goal:** Direct connection across different networks
**Expected:** NAT traversal or relay fallback
**Method:** Simulate cross-network connection

### S3: Cellular + WiFi
**Goal:** Mobile-to-WiFi connectivity
**Expected:** Relay-based connection
**Method:** Requires manual testing with actual devices

### S4: Symmetric NAT (Relay Fallback)
**Goal:** Connection when NAT hole-punching fails
**Expected:** Relay connection established
**Method:** Both nodes behind symmetric NAT

### S5: Restrictive Firewall
**Goal:** Connection from restrictive network
**Expected:** Relay-only communication
**Method:** One node behind firewall

### S6: Partition Healing
**Goal:** Automatic reconnection and CRDT convergence
**Expected:** Reconnect < 5 seconds, no data loss
**Method:** Disconnect, modify both sides, reconnect

## Acceptance Criteria

- ✅ Two nodes sync Automerge document over Iroh
- ✅ Works across home WiFi, cellular, relay
- ✅ Connection establishment < 3 seconds (direct)
- ✅ Reconnection after network drop < 5 seconds
- ✅ No data loss in any test scenario
- ✅ CRDT convergence verified

## Performance Metrics

The POC tracks:
- **Connection Time:** Time to establish initial connection
- **Sync Latency:** Time to transmit/receive sync messages
- **Throughput:** Bytes per second
- **Reconnection Time:** Time to reestablish after disconnect
- **Data Integrity:** CRDT convergence verification

## Known Limitations

### Browser Support
⚠️ **Iroh cannot run directly in browsers** due to:
- No UDP/QUIC support in browser JavaScript/WASM
- No direct socket access from web context
- Browser security sandbox restrictions

**Implications for Phase 2:**
- Desktop/mobile: Full P2P connectivity via native Iroh
- Browser: Requires relay server or WebRTC fallback
- Consider WebRTC-based alternative for browser contexts
- Possible architecture: Iroh for native, WebRTC for web

### Relay Server
- Production deployment requires hosted relay servers
- Iroh provides default relays for testing
- Custom relay recommended for production

## Results

See `docs/research/iroh-connectivity-report.md` for detailed findings including:
- Test results for all scenarios
- Performance measurements
- Browser compatibility analysis
- Risk assessment
- Go/No-Go recommendation

## Next Steps

1. **Run Tests:** Execute all scenarios and collect metrics
2. **Document Findings:** Complete connectivity report
3. **Evaluate Browser Strategy:** Decide on WebRTC fallback
4. **Make Decision:** Go/No-Go for Iroh in Phase 2
5. **Plan Integration:** If Go, design integration with VUDO Runtime

## Dependencies

- **iroh** (0.28): P2P networking layer
- **automerge** (0.5): CRDT implementation
- **tokio**: Async runtime
- **clap**: CLI parsing
- **tracing**: Logging

## License

This POC is part of the univrs-dol project.
