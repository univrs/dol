# Hyphal Network Patterns for Distributed Systems

This document explores how fungal hyphal growth patterns from biology can be applied to design resilient, adaptive distributed networks. These patterns form the foundation of the DOL network stdlib.

## Table of Contents

1. [Biological Foundations](#biological-foundations)
2. [Core Concepts](#core-concepts)
3. [Growth Patterns](#growth-patterns)
4. [Network Topology](#network-topology)
5. [Resource Distribution](#resource-distribution)
6. [Failure Handling](#failure-handling)
7. [DOL Implementation](#dol-implementation)
8. [Use Cases](#use-cases)
9. [Rust Implementation](#rust-implementation)

---

## Biological Foundations

### What is a Hyphal Network?

A hyphal network (mycelium) is the vegetative structure of fungi, consisting of branching, thread-like filaments called hyphae. Unlike static networks, mycelium is a **living, adaptive system** that:

- Grows toward resources (chemotropism)
- Branches to explore space efficiently
- Fuses separate hyphae (anastomosis) to form redundant paths
- Transports nutrients bidirectionally
- Self-repairs when damaged

### Key Biological Structures

| Structure | Function | Network Analog |
|-----------|----------|----------------|
| **Hyphal Tip** | Growing edge, senses environment | Discovery agent, edge node |
| **Hyphal Segment** | Transport conduit | Network link, channel |
| **Septum** | Compartment boundary with pore | Firewall, rate limiter |
| **Woronin Body** | Emergency pore plug | Circuit breaker |
| **Branching Point** | Network junction | Router, hub |
| **Anastomosis Site** | Fusion of separate hyphae | Mesh reconnection |

### The DOL Biological Model

From `biology.hyphal @ 1.0.0`:

```dol
/// State of a hyphal tip
pub gene HyphalTip {
  has position: Vec3
  has direction: Vec3
  has age: Float64
  has branching_potential: Float64
  has nutrients_absorbed: Nutrient

  constraint unit_direction {
    this.direction.magnitude() >= 0.99 &&
    this.direction.magnitude() <= 1.01
  }
}
```

The `HyphalTip` represents the growing edge of the network:
- **position**: Current location in space
- **direction**: Unit vector indicating growth heading
- **branching_potential**: Accumulates with nutrients; triggers branching when high
- **nutrients_absorbed**: Resources collected at this tip

---

## Core Concepts

### 1. Chemotropism: Growing Toward Resources

In biology, hyphal tips sense chemical gradients and grow toward nutrient sources. In distributed systems, this translates to:

```
Resource Gradient (Biology)     -->  Metric Gradient (Network)
   Nutrient concentration              CPU availability
   Chemical signals                    Latency measurements
   Moisture levels                     Bandwidth capacity
```

**DOL Pattern from hyphal.dol:**

```dol
fun extend(gradient: Gradient<Nutrient>) -> HyphalTip {
  nutrient_level = gradient.sample_at(this.position)

  new_direction = this.direction
    .combine(gradient.direction, weight: nutrient_level.total_mass())
    .normalize()

  growth_rate = 0.1 + nutrient_level.total_mass() * 0.01
  new_position = Vec3 {
    x: this.position.x + new_direction.x * growth_rate,
    y: this.position.y + new_direction.y * growth_rate,
    z: this.position.z + new_direction.z * growth_rate
  }
  // ... return updated tip
}
```

**Key insight**: Growth rate scales with nutrient availability. Networks should expand faster toward resource-rich regions.

### 2. Anastomosis: Network Fusion

When two hyphal tips meet from compatible mycelia, they fuse to form a continuous network. This creates **redundant paths** and **load balancing** opportunities.

**Properties:**
- Requires compatibility check (prevents hostile takeover)
- Merges resources at fusion point
- Creates mesh topology from tree topology

**DOL Pattern:**

```dol
fun fuse(other: HyphalTip) -> Option<HyphalTip> {
  distance = (this.position.x - other.position.x).abs() +
             (this.position.y - other.position.y).abs() +
             (this.position.z - other.position.z).abs()

  if distance > 0.5 {
    return None  // Too far apart
  }

  return Some(HyphalTip {
    position: midpoint(this.position, other.position),
    direction: this.direction.combine(other.direction, weight: 0.5).normalize(),
    age: max(this.age, other.age),
    branching_potential: this.branching_potential + other.branching_potential,
    nutrients_absorbed: this.nutrients_absorbed.combine(other.nutrients_absorbed)
  })
}
```

**Law from hyphal.dol:**

```dol
law fusion_requires_compatibility {
  forall a: Self, b: Self.
    a.fuse(b).is_some() implies a.is_compatible(b)
}
```

### 3. Nutrient Transport

Mycelium networks transport nutrients bidirectionally through cytoplasmic streaming:
- From sources (areas with excess) to sinks (areas with need)
- Not passive diffusion - active, pressure-driven flow
- Routing decisions made locally based on gradients

**DOL Pattern from transport.dol:**

```dol
pub trait Transport<T> {
  is transport(
    source: TransportNode<T>,
    sink: TransportNode<T>,
    amount: T
  ) -> Result<Flow<T>, TransportError>

  is optimize_flow(nodes: List<TransportNode<T>>) -> List<Flow<T>>

  law mass_conservation {
    forall network: Self.
      network.total_inflow() ==
        network.total_outflow() + network.accumulation()
  }
}
```

### 4. Branching Exploration

Hyphal tips branch when they accumulate enough "potential" (nutrients, signals). This is an exploration strategy:

- **Low branching**: Linear growth, efficient for known paths
- **High branching**: Space-filling exploration, good for resource discovery

**DOL Pattern:**

```dol
fun branch(factor: Float64) -> List<HyphalTip> {
  if this.branching_potential < factor {
    return [this]  // Not enough potential to branch
  }

  branch_angle = 0.5

  left_dir = this.direction.rotate_y(branch_angle)
  right_dir = this.direction.rotate_y(-branch_angle)

  return [
    HyphalTip {
      position: this.position,
      direction: left_dir,
      age: 0.0,
      branching_potential: this.branching_potential / 2.0,
      nutrients_absorbed: Nutrient.zero()
    },
    HyphalTip {
      position: this.position,
      direction: right_dir,
      age: 0.0,
      branching_potential: this.branching_potential / 2.0,
      nutrients_absorbed: Nutrient.zero()
    }
  ]
}
```

**Law from hyphal.dol:**

```dol
law branching_conserves_potential {
  forall self: Self, factor: Float64.
    sum(self.branch(factor).map(|b| b.branching_potential))
      <= self.branching_potential
}
```

---

## Growth Patterns

### Apical Dominance

The leading tip grows fastest, suppressing lateral branching. In networks:

```
Primary connection:    Fast, direct path (main route)
Secondary connections: Slower, backup paths (redundancy)

         [Primary]
Node A =============> Node B
      \              /
       \--[Backup]--/
```

### Space-Filling Growth

Mycelium optimizes coverage of available space. Network equivalent:

1. **Exploration phase**: Many tips, broad coverage
2. **Optimization phase**: Prune unused paths, strengthen active ones
3. **Maintenance phase**: Stable topology with periodic refresh

### Foraging Strategies

| Strategy | Biological Use | Network Use |
|----------|----------------|-------------|
| **Phalanx** | Dense, slow advance | High-availability cluster |
| **Guerrilla** | Scattered, fast scouts | CDN edge discovery |
| **Mixed** | Core density + scout tips | Hybrid topology |

---

## Network Topology

### From Tree to Mesh

Hyphal networks start as trees (branching from spore) but become meshes through anastomosis:

```
Stage 1: Tree (branching)
        Root
       /    \
     /        \
   Tip1      Tip2
   /  \      /  \
 T1a  T1b  T2a  T2b

Stage 2: Mesh (after anastomosis)
        Root
       /    \
     /   X    \      <-- Anastomosis creates cross-links
   Tip1------Tip2
   /  \  X  /  \
 T1a--T1b--T2a--T2b
```

### DOL Network Graph

From `network.hyphal @ 1.0.0`:

```dol
pub gene HyphalNetwork {
  has nodes: Map<NodeId, HyphalTip>
  has edges: List<NetworkEdge>
  has active_tips: Set<NodeId>
  has generation: UInt64

  constraint has_nodes {
    this.nodes.len() > 0
  }
}
```

### Hierarchical Organization

Real mycelium has hierarchy:
- **Thick hyphae**: High-capacity transport (backbone)
- **Thin hyphae**: Low-capacity exploration (edge)

Network equivalent:

```dol
pub gene NetworkEdge {
  has source: NodeId
  has target: NodeId
  has capacity: Float64
  has latency: Float64

  constraint positive_capacity {
    this.capacity > 0.0
  }
}
```

---

## Resource Distribution

### Pressure-Driven Flow

Like turgor pressure in hyphae, resources flow from high to low:

```
High Resource Node               Low Resource Node
  [CPU: 80%]   ─────────────>     [CPU: 20%]
  [Mem: 90%]   Workload flows     [Mem: 40%]

  Pressure = resource availability
  Flow direction = down pressure gradient
```

### Market-Based Allocation

Modern understanding (Kiers et al.) shows fungi operate as **traders, not altruists**:

```
OLD MODEL (Mother Tree):
  "Nodes share freely with those in need"
  → Leads to free-rider problem, unsustainable

NEW MODEL (Biological Market):
  "Nodes trade based on exchange rates"
  → Sustainable, self-organizing, resilient
```

**Implications for network design:**
- Nodes should **track exchange rates** over time
- **Hoard scarce resources** (phosphorus hoarding = resource reservation)
- **Preferential allocation** to best trading partners

### Nutrient Types (Resource Classes)

In biology: C, N, P, water
In networks:

| Biological | Network | Characteristics |
|------------|---------|-----------------|
| Carbon | Compute (CPU/GPU) | Primary energy source |
| Nitrogen | Memory | Required for structure |
| Phosphorus | Bandwidth | Scarce, high-value |
| Water | Storage | Bulk transport |

From `biology.types.dol`:

```dol
pub gene Nutrient {
  has carbon: Float64
  has nitrogen: Float64
  has phosphorus: Float64
  has water: Float64

  constraint stoichiometry {
    // Redfield ratio: C:N:P approximately 106:16:1
    this.nitrogen > 0.0 implies (
      this.carbon / this.nitrogen >= 6.0 &&
      this.carbon / this.nitrogen <= 10.0
    )
  }
}
```

---

## Failure Handling

### Septal Isolation (Circuit Breakers)

When a hypha is damaged, Woronin bodies plug the septal pore:

```
Before damage:
  [A]---[B]---[C]---[D]

After C damaged (septal isolation):
  [A]---[B]---X   X---[D]
              |   |
          isolated compartment

Network [A]-[B] continues operating
Node [D] can reroute through alternative paths
```

### DOL Pattern for Circuit Breakers

```dol
pub gene SeptalGate {
  has segment: NetworkSegment
  has status: HealthStatus
  has failure_threshold: UInt32
  has recovery_timeout: Duration

  fun should_isolate() -> Bool {
    match this.status.type {
      Degraded { failure_count } => failure_count >= this.failure_threshold,
      Failed { .. } => true,
      _ => false
    }
  }

  fun isolate() -> SeptalGate {
    return SeptalGate {
      ...this,
      status: HealthStatus::Isolated { isolated_at: Timestamp.now() }
    }
  }
}
```

### Self-Healing

If damage is repaired, connection can be restored:
1. **Health check** on isolated connection
2. **Gradual reopening** (avoid flood)
3. **Reintegration** into network

---

## DOL Implementation

### Module Structure

```
examples/stdlib/
  biology/
    hyphal.dol          -- Biological primitives
    transport.dol       -- Source-sink flow
    mycelium.dol        -- Complete network simulation
    types.dol           -- Vec3, Nutrient, etc.
  network/
    hyphal_network.dol  -- Network abstractions
```

### Core Traits from biology.hyphal

```dol
pub trait Hyphal {
  /// Extend toward nutrient gradient
  is extend(gradient: Gradient<Nutrient>) -> Self

  /// Branch into multiple hyphae
  is branch(factor: Float64) -> List<Self>

  /// Fuse with another hypha (anastomosis)
  is fuse(other: Self) -> Option<Self>

  /// Absorb nutrients at current position
  is absorb(available: Nutrient) -> Tuple<Self, Nutrient>

  /// Check if ready to fruit
  is can_fruit() -> Bool

  // Laws (Biological Constraints)
  law conservation_of_mass { ... }
  law branching_conserves_potential { ... }
  law fusion_requires_compatibility { ... }
}
```

### Network Trait from network.hyphal

```dol
pub trait NetworkGrowth {
  /// Grow network toward resource gradient
  is grow(gradient: ResourceGradient) -> Self

  /// Prune low-value edges
  is prune(threshold: Float64) -> Self

  /// Find shortest path between nodes
  is find_path(from: NodeId, to: NodeId) -> Option<List<NodeId>>

  /// Route message through network
  is route(message: Bytes, from: NodeId, to: NodeId) -> Result<Void, String>

  law connectivity_preserved {
    forall self: Self, threshold: Float64.
      self.prune(threshold).is_connected() implies self.is_connected()
  }
}
```

---

## Use Cases

### 1. Distributed Computing Cluster

Hyphal patterns for workload distribution:
- **Tips** = Worker discovery agents
- **Segments** = Job queues
- **Branching** = Scaling out
- **Anastomosis** = Load balancing

### 2. Content Delivery Network

- **Chemotropism** = Cache placement near demand
- **Transport** = Content replication
- **Branching** = Edge proliferation
- **Pruning** = Removing underused edges

### 3. Peer-to-Peer File Sharing

- **Tips** = Peer discovery
- **Anastomosis** = Swarm formation
- **Transport** = Chunk distribution
- **Market** = Tit-for-tat incentives

### 4. Microservice Mesh

- **Network** = Service mesh topology
- **Segments** = gRPC channels
- **Septal isolation** = Circuit breakers
- **Health checks** = Readiness probes

### 5. IoT Sensor Networks

- **Tips** = Sensor nodes
- **Segments** = Low-power links
- **Branching** = Network expansion
- **Resource gradients** = Data aggregation paths

---

## Rust Implementation

The hyphal network is implemented in the following Rust modules:

### `src/network/topology.rs`

Core graph representation:
- `NodeId` - Unique node identifier
- `Edge` - Directed edge with capacity/latency
- `Topology` - Network graph structure
- Dijkstra's shortest path algorithm

### `src/network/discovery.rs`

Resource gradient navigation:
- `ResourceType` - Type identifier for resources
- `ResourceGradient` - Concentration field across nodes
- `ResourceExplorer` - Agent that follows gradients

### `src/network/growth.rs`

Adaptive network expansion:
- `GrowthParams` - Branching, pruning thresholds
- `GrowthSimulator` - Simulates hyphal growth cycles
- Branch/extend/fuse operations

### `src/swarm/hyphal_coordinator.rs`

Agent swarm coordination:
- `HyphalAgent` - Agent with role and position
- `AgentRole` - Explorer, Transport, Hub, Dormant
- `HyphalSwarm` - Coordinator managing agents
- Message routing via hyphal topology

### Usage Example

```rust
use metadol::network::{Topology, NodeId, GrowthSimulator, GrowthParams};
use metadol::network::{ResourceGradient, ResourceType};
use metadol::swarm::HyphalSwarm;

// Create a new swarm
let mut swarm = HyphalSwarm::new();

// Spawn explorer agents
let agent1 = swarm.spawn_explorer();
let agent2 = swarm.spawn_explorer();

// Run coordination cycles
for _ in 0..10 {
    swarm.tick();
}

// Get swarm metrics
let metrics = swarm.metrics();
println!("Agents: {}, Generation: {}", metrics.agent_count, metrics.generation);
```

---

## Summary: Key Principles

1. **Grow toward resources**: Connect to where value is
2. **Branch to explore**: Do not over-commit to one path
3. **Fuse for resilience**: Mesh topology beats tree
4. **Transport actively**: Not passive pipes, active routing
5. **Trade, do not gift**: Market incentives beat altruism
6. **Isolate failures**: Contain damage, preserve network
7. **Self-heal**: Reconnect when possible

These patterns, derived from 450+ million years of fungal evolution, provide a proven foundation for building resilient, adaptive distributed networks.

---

## Related DOL Modules

- `biology.hyphal @ 1.0.0` - Biological primitives
- `biology.transport @ 1.0.0` - Source-sink flow
- `biology.mycelium @ 1.0.0` - Complete network simulation
- `network.hyphal @ 1.0.0` - Network abstractions

## References

- Simard, S. (1997). Net transfer of carbon between ectomycorrhizal tree species in the field. *Nature*
- Karst, J. et al. (2023). Positive citation bias in the Wood Wide Web literature. *Nature Ecology & Evolution*
- Kiers, E.T. et al. (2011). Reciprocal rewards stabilize cooperation in the mycorrhizal symbiosis. *Science*
- Adamatzky, A. (2018). On spiking behaviour of oyster fungi. *Scientific Reports*
- [Fungal Mycelium Networks](https://en.wikipedia.org/wiki/Mycelium)
- [Anastomosis in Fungi](https://en.wikipedia.org/wiki/Anastomosis)
- [Chemotropism](https://en.wikipedia.org/wiki/Chemotropism)
