# Network Topology Patterns

## Hyphal-Inspired Topologies

### 1. Exploration Mesh

Initial topology with active tips exploring space:

```
      T1        T2
       \      /
        \    /
         \  /
          \/
       [ROOT]
          /\
         /  \
        /    \
       T3    T4
```

**Characteristics:**
- All nodes are active exploration tips
- Low connectivity, high exploration
- Best for initial discovery phase

### 2. Anastomosis Ring

Tips that have fused form ring structures:

```
     N1 ---- N2
    /  \    /  \
   /    \  /    \
  N6    [HUB]   N3
   \    /  \    /
    \  /    \  /
     N5 ---- N4
```

**Characteristics:**
- Fused tips create redundant paths
- Medium connectivity, fault-tolerant
- Self-healing through alternative routes

### 3. Transport Tree

Optimized for nutrient/message transport:

```
              [ROOT]
             /  |  \
            /   |   \
          N1   N2   N3
         / \   |   / \
        L1 L2 L3  L4 L5
```

**Characteristics:**
- Hierarchical structure
- Low redundancy, high efficiency
- Best for bulk transport

### 4. Hybrid Network

Combines multiple topologies:

```
     [HUB1] ===== [HUB2]
       /|\         /|\
      / | \       / | \
     T1 T2 T3   T4 T5 T6
        |           |
     [RING1]    [RING2]
```

**Characteristics:**
- Hubs connected with high-capacity links
- Explorer tips at the periphery
- Ring structures for local resilience

## Topology Properties

| Topology | Redundancy | Latency | Capacity | Use Case |
|----------|------------|---------|----------|----------|
| Mesh     | High       | Low     | Medium   | Exploration |
| Ring     | Medium     | Medium  | Medium   | Fault tolerance |
| Tree     | Low        | Low     | High     | Transport |
| Hybrid   | High       | Low     | High     | Production |

## Adaptive Transitions

Networks evolve based on resource patterns:

### Phase 1: Exploration

```
State: Many active tips, low connectivity
Goal:  Discover resource locations
Topology: Exploration Mesh
```

### Phase 2: Consolidation

```
State: Tips finding resources
Goal:  Connect high-value nodes
Topology: Anastomosis, ring formation
```

### Phase 3: Optimization

```
State: Resources mapped
Goal:  Efficient transport
Topology: Tree structure
```

### Phase 4: Resilience

```
State: Stable network
Goal:  Fault tolerance
Topology: Hybrid with redundant paths
```

## Implementation

### Topology Metrics

```rust
pub struct TopologyMetrics {
    pub node_count: usize,
    pub edge_count: usize,
    pub active_tip_count: usize,
    pub avg_capacity: f64,
    pub density: f64,
}
```

### Key Operations

1. **add_tip(id)**: Add exploration node
2. **connect(a, b, capacity)**: Create edge
3. **fuse_tips(a, b)**: Anastomosis
4. **shortest_path(from, to)**: Dijkstra routing
5. **prune(threshold)**: Remove low-capacity edges

### Topology Evolution Algorithm

```rust
fn evolve_topology(topo: &mut Topology, gradient: &ResourceGradient) {
    // 1. Extend tips toward gradient
    for tip in topo.active_tips() {
        if gradient.intensity(tip) > EXTEND_THRESHOLD {
            topo.extend(tip, gradient.direction(tip));
        }
    }

    // 2. Branch high-potential tips
    for tip in topo.active_tips() {
        if topo.potential(tip) > BRANCH_THRESHOLD {
            topo.branch(tip);
        }
    }

    // 3. Fuse nearby tips (anastomosis)
    for (a, b) in topo.nearby_pairs(FUSION_DISTANCE) {
        topo.fuse(a, b);
    }

    // 4. Prune low-value edges
    topo.prune(PRUNE_THRESHOLD);
}
```

## Swarm Integration

### Agent Roles from Topology

| Topology Role | Agent Role | Behavior |
|---------------|------------|----------|
| Active Tip    | Explorer   | Discover new resources |
| Junction (3+) | Hub        | Route messages, coordinate |
| Segment Node  | Transport  | Forward messages |
| Isolated      | Dormant    | Wait for connections |

### Message Routing

Messages follow shortest path through topology:

```rust
fn route_message(swarm: &HyphalSwarm, from: NodeId, to: NodeId, msg: &[u8]) {
    if let Some(path) = swarm.topology.shortest_path(from, to) {
        for hop in path.windows(2) {
            let edge = swarm.topology.edge(hop[0], hop[1]);
            if msg.len() <= edge.capacity {
                // Forward to next hop
            }
        }
    }
}
```

## Performance Considerations

### Scaling

- **Node count**: O(n) memory
- **Edge count**: O(e) memory, typically O(n) for sparse hyphal networks
- **Shortest path**: O((n + e) log n) with Dijkstra
- **Fusion check**: O(n^2) naive, O(n log n) with spatial indexing

### Optimization Strategies

1. **Spatial hashing**: Fast fusion candidate detection
2. **Path caching**: Cache frequently used routes
3. **Lazy pruning**: Batch low-capacity edge removal
4. **Parallel exploration**: Tips grow independently

## Related Documentation

- [HYPHAL-NETWORK-PATTERNS.md](./HYPHAL-NETWORK-PATTERNS.md) - Core patterns
- [ENR-ARCHITECTURE.md](./ENR-ARCHITECTURE.md) - Exploration & Resource architecture
- [HyphalNetwork.md](./HyphalNetwork.md) - Detailed network design
