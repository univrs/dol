# GDL Domain Genes Overview

## Introduction

This document describes the Geometric Deep Learning (GDL) domain genes implemented in DOL. These genes form the foundation for ontology-first development of neural network architectures that respect geometric structure.

## GDL Blueprint Integration

The three domain genes correspond to the fundamental domains in the GDL Blueprint's three-pillar ontology:

| Gene | Domain Type | Inherent Symmetry | Use Cases |
|------|-------------|-------------------|-----------|
| `Grid2D<T>` | Regular Grid | T(2) Translation | Images, spectrograms, feature maps |
| `Graph<N,E>` | Graph | S_n Permutation | Molecules, social networks, knowledge graphs |
| `PointCloud<F>` | Point Cloud | SE(3)/E(3) + S_n | LiDAR scans, 3D objects, molecular surfaces |

## Gene Specifications

### Grid2D<T>

Models regular 2-dimensional grids for structured data.

**Fields:**
- `height`, `width`, `channels`: Grid dimensions
- `data`: Row-major flattened tensor of type T
- `stride`: Pre-computed row stride

**Constraints:**
- `valid_shape`: Ensures `data.length == height * width * channels`
- `positive_dimensions`: Ensures all dimensions > 0

**Symmetry:** Translation Group T(2) - architectures should be translation-equivariant (e.g., CNNs with weight sharing).

### Graph<N,E>

Models graph domains where N is node feature type and E is edge feature type.

**Fields:**
- `nodes`: List of node features
- `edges`: List of (source, target, edge_data) tuples
- `adjacency`: Sparse adjacency matrix
- `node_count`, `edge_count`: Derived counts

**Constraints:**
- `valid_edges`: All edge indices within bounds
- `no_self_loops`: No edges from node to itself

**Laws:**
- `undirected_symmetry`: For every (i,j,e), reverse (j,i,e) exists

**Symmetry:** Permutation Group S_n - architectures must be permutation-equivariant (e.g., GNNs, MPNNs).

### PointCloud<F>

Models unordered sets of 3D points with per-point features, where F is feature dimensionality.

**Fields:**
- `points`: List of 3D coordinates (Vec3)
- `features`: Flattened per-point feature array
- `num_points`, `feature_dim`: Derived values
- `bounds_min`, `bounds_max`: Axis-aligned bounding box

**Constraints:**
- `feature_alignment`: `features.length == points.length * F`
- `non_empty`: At least one point exists
- `valid_bounds`: Bounding box is well-formed

**Symmetry:** SE(3) or E(3) combined with S_n - architectures must be rotation/translation equivariant and permutation invariant (e.g., PointNet, DGCNN).

## Test Coverage

Property-based tests in `tests/gdl_domain_tests.rs` verify:

1. **Parsing**: All genes parse correctly
2. **Constraints**: Constraint documentation in exegesis
3. **Properties**: Required fields present
4. **Symmetry**: Symmetry groups documented
5. **GDL Compliance**: Blueprint references included
6. **Structure**: Minimum statement counts

## Related Work

- [GDL Blueprint](https://geometricdeeplearning.com/) - Theoretical foundation
- Grid1D<T>, Grid3D<T> - 1D/3D translation symmetry variants
- HeteroGraph<N,E> - Heterogeneous multi-type graphs
- Mesh<V,F> - Manifold domains with face connectivity

## Future Extensions

- Implement equivariant layer genes
- Add manifold domains (spheres, Lie groups)
- Define signal/feature transformation genes
- Create composition patterns for multi-domain architectures
