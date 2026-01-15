# GDL Equivariant Layers Documentation

**Version:** 0.7.1
**Date:** 2026-01-15
**Status:** Implementation Complete, Tests Pending

## Overview

This document describes the equivariant neural network layer implementations in DOL, forming the **Architecture Pillar** of the Geometric Deep Learning (GDL) Blueprint.

## Architecture Hierarchy

```
gdl.symmetry_group (abstract)
    │
    ├── PermutationGroup<N>      ← S_n for graphs
    └── TranslationGroup<D>      ← T(D) for grids

gdl.equivariant_layer (abstract)
    │
    ├── gdl.invariant_layer      ← Trivial output representation
    │
    └── MessagePassingLayer<N,E,H>  ← Concrete GNN implementation
            implements EquivariantLayer<PermutationGroup>
```

## Layer Traits

### EquivariantLayer<G: SymmetryGroup>

**File:** `examples/traits/equivariant_layer.dol`

The foundation trait for all GDL architectures. Defines layers where the output transforms predictably with input transformations.

#### Interface

| Operation | Signature | Description |
|-----------|-----------|-------------|
| `forward` | `(x: X) -> Y` | Forward pass transformation |
| `parameters` | `() -> List<Tensor>` | Get trainable parameters |
| `set_parameters` | `(params: List<Tensor>) -> Void` | Set trainable parameters |

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `input_symmetry` | `SymmetryGroup` | Group acting on input space |
| `output_symmetry` | `SymmetryGroup` | Group acting on output space |

#### The Equivariance Law

```
∀g ∈ G, ∀x ∈ X:
    G.act(g, self.forward(x)) == self.forward(G.act(g, x))
```

**Interpretation:** Transforming the output equals forwarding the transformed input.

#### Common Equivariant Architectures

| Architecture | Group G | Domain |
|--------------|---------|--------|
| CNN | T(2), T(3) | Images, Volumes |
| GNN | S_n | Graphs |
| Spherical CNN | SO(3) | 3D rotations |
| E(n)-GNN | SE(3), E(3) | Point clouds |

---

### InvariantLayer<G: SymmetryGroup>

**File:** `examples/traits/invariant_layer.dol`

A special case of EquivariantLayer where the output has a trivial group action (unchanged by transformations).

#### The Invariance Law

```
∀g ∈ G, ∀x ∈ X:
    self.forward(G.act(g, x)) == self.forward(x)
```

**Interpretation:** Transforming the input does not change the output.

#### Use Cases

1. **Classification:** Class label unchanged by input transformations
2. **Regression:** Scalar properties invariant to pose
3. **Pooling:** Global aggregation eliminates spatial structure

#### Achieving Invariance

Standard pattern:
```
Input → [Equivariant Layers] → [Invariant Pooling] → Output
        ↑ preserves structure    ↑ removes symmetry
```

---

## Layer Implementations

### MessagePassingLayer<NodeDim, EdgeDim, HiddenDim>

**File:** `examples/genes/message_passing_layer.dol`

Implements the Message Passing Neural Network (MPNN) framework for graph-structured data.

#### Type Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `NodeDim` | UInt64 | Input node feature dimension |
| `EdgeDim` | UInt64 | Edge feature dimension (0 if unused) |
| `HiddenDim` | UInt64 | Output/hidden layer dimension |

#### Fields

| Field | Type | Description |
|-------|------|-------------|
| `message_mlp_weights` | `Array<Float64>` | Message function MLP weights |
| `update_mlp_weights` | `Array<Float64>` | Update function MLP weights |
| `aggregation` | `String` | Aggregation type: "sum", "mean", "max" |
| `use_edge_features` | `Bool` | Include edge features in messages |

#### Constraints

```dol
constraint valid_aggregation {
  this.aggregation == "sum" ||
  this.aggregation == "mean" ||
  this.aggregation == "max"
}
```

#### The Permutation Equivariance Law

```dol
law permutation_equivariance {
  forall perm: PermutationGroup<N>. forall graph: Graph<...>.
    let permuted_input = graph.permute_nodes(perm)
    let permuted_output = this.forward(graph).permute_nodes(perm)
    this.forward(permuted_input) == permuted_output
}
```

#### Message-Aggregate-Update Paradigm

```
For each node i:
  1. MESSAGE:    m_ji = MLP_msg(h_j, h_i, e_ji) for each neighbor j
  2. AGGREGATE:  a_i = AGG({m_ji : j ∈ N(i)})
  3. UPDATE:     h'_i = MLP_update(h_i, a_i)
```

#### Mathematical Formulation

Given graph G = (V, E) with node features X ∈ ℝ^{n×d_in}:

```
h_i^{(l+1)} = UPDATE(h_i^{(l)}, AGGREGATE_{j∈N(i)} MESSAGE(h_i^{(l)}, h_j^{(l)}, e_ji))
```

#### Aggregation Functions

| Function | Formula | Expressiveness | Architecture |
|----------|---------|----------------|--------------|
| sum | Σ m_ji | Highest (1-WL) | GIN |
| mean | (1/\|N(i)\|) Σ m_ji | Medium | GCN |
| max | max_j m_ji | Lowest | GraphSAGE |

**Expressiveness Hierarchy:** sum > mean > max

The sum aggregation matches the 1-Weisfeiler-Lehman graph isomorphism test, making it maximally expressive for distinguishing non-isomorphic graphs.

---

## Performance Characteristics

### Time Complexity

| Function | Complexity | Notes |
|----------|------------|-------|
| `forward` | O(E·D_h·D_in + N·D_h²) | Per forward pass |
| `message` | O(D_h·D_in) | Per edge |
| `aggregate` | O(deg(i)·D_h) | Per node |
| `update` | O(D_h²) | Per node |

### Memory Usage

| Component | Size | Notes |
|-----------|------|-------|
| Parameters | O(D_h·D_in + D_h²) | Two MLPs |
| Intermediate messages | O(E·D_h) | Peak during forward |
| Output features | O(N·D_h) | Result |

### Scalability

| Graph Size | Nodes | Edges | Peak Memory |
|------------|-------|-------|-------------|
| Small | 1K | 10K | ~20MB |
| Medium | 100K | 1M | ~2GB |
| Large | 1M | 10M | ~20GB |

### Identified Optimizations

1. **Streaming Aggregation:** 50-90% memory reduction
2. **Batched BLAS:** 10-50x compute speedup
3. **In-place Operations:** 3-5x fewer allocations
4. **Neighbor Sampling:** Handles power-law hub nodes

---

## Usage Examples

### Basic GNN Layer

```dol
let layer = MessagePassingLayer<128, 0, 256> {
  message_mlp_weights: random_weights(256 * 257),
  update_mlp_weights: random_weights(256 * 385),
  aggregation: "sum",
  use_edge_features: false
}

let output_graph = layer.forward(input_graph)
```

### With Edge Features

```dol
let layer = MessagePassingLayer<64, 16, 128> {
  message_mlp_weights: random_weights(128 * 145),
  update_mlp_weights: random_weights(128 * 193),
  aggregation: "mean",
  use_edge_features: true
}
```

### Stacking Layers

```dol
// 3-hop neighborhood via 3 layers
let layer1 = MessagePassingLayer<NodeDim, EdgeDim, 256> { ... }
let layer2 = MessagePassingLayer<256, EdgeDim, 256> { ... }
let layer3 = MessagePassingLayer<256, EdgeDim, HiddenDim> { ... }

let h1 = layer1.forward(graph)
let h2 = layer2.forward(h1)
let output = layer3.forward(h2)
```

---

## Related Work

### GDL Blueprint References

- **Domain Pillar:** `Graph<N, E>` gene with S_n symmetry
- **Symmetry Pillar:** `PermutationGroup<N>` implementing group laws
- **Architecture Pillar:** This document (EquivariantLayer, MessagePassingLayer)

### Academic References

1. Gilmer et al., "Neural Message Passing for Quantum Chemistry" (ICML 2017)
2. Xu et al., "How Powerful are Graph Neural Networks?" (ICLR 2019)
3. Bronstein et al., "Geometric Deep Learning: Grids, Groups, Graphs, Geodesics, and Gauges" (2021)
4. Morris et al., "Weisfeiler and Leman Go Neural" (AAAI 2019)
5. Kipf & Welling, "Semi-Supervised Classification with Graph Convolutional Networks" (ICLR 2017)

### Notable GNN Architectures

| Architecture | Year | Key Innovation |
|--------------|------|----------------|
| GCN | 2017 | Spectral convolution, mean aggregation |
| GraphSAGE | 2017 | Inductive learning, sampling |
| GAT | 2018 | Attention-weighted aggregation |
| GIN | 2019 | Sum aggregation, 1-WL expressiveness |
| MPNN | 2017 | General message passing framework |

---

## File Inventory

| File | Type | Statements | Description |
|------|------|------------|-------------|
| `equivariant_layer.dol` | trait | 14 behaviors | Foundation layer interface |
| `invariant_layer.dol` | trait | 14 behaviors | Trivial output specialization |
| `message_passing_layer.dol` | gene | 19 statements | GNN implementation |

---

## Next Steps

1. **Unit Tests:** Test each layer function in isolation
2. **Integration Tests:** End-to-end GNN pipeline with Graph gene
3. **Additional Layers:**
   - `AttentionLayer<NodeDim, HiddenDim>` - GAT-style attention
   - `Conv2DLayer<InChannels, OutChannels, KernelSize>` - Grid convolution
   - `PoolingLayer<G>` - Invariant pooling operations
4. **Performance Optimization:** Implement streaming aggregation
