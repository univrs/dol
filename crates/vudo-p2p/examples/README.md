# Willow Protocol Examples

This directory contains examples demonstrating the Willow Protocol integration in VUDO P2P.

## Examples

### 1. Namespace Mapping (`namespace_mapping.rs`)

Demonstrates how DOL concepts map to Willow's 3D namespace structure.

**Run**:
```bash
cargo run --example namespace_mapping
```

**Shows**:
- DOL System → Willow Namespace ID (BLAKE3 hashing)
- DOL Collection → Willow Subspace ID (BLAKE3 hashing)
- DOL Document ID → Willow Path (hierarchical components)
- Path prefix matching for scoped access

**Example Output**:
```
=== DOL to Willow Namespace Mapping ===

1. DOL System → Willow Namespace ID
   DOL System: myapp.v1
   Willow Namespace ID: 8f4e33f3e0ff01f7

2. DOL Collection → Willow Subspace ID
   DOL Collection: users
   Willow Subspace ID: 3c6e0b8a9c15e000

3. DOL Document ID → Willow Path
   DOL Document ID: alice
   Willow Path: alice
```

### 2. Meadowcap Capabilities (`capabilities.rs`)

Shows the Meadowcap capability system for fine-grained permissions.

**Run**:
```bash
cargo run --example capabilities
```

**Shows**:
- Creating root capabilities with admin access
- Delegating capabilities with reduced permissions
- Permission hierarchy (Admin > Write > Read)
- Path-based access control
- Cryptographic signature verification

**Example Output**:
```
=== Meadowcap Capability Delegation ===

1. Creating Root Capability
   Namespace: 8f4e33f3e0ff01f7
   Permission: Admin (full access)
   Path prefix: <empty> (all paths)

2. Delegating Write Capability for Users Collection
   Subspace: users
   Permission: Write
   Path prefix: <empty> (all users)

3. Delegating Read-Only Capability for Alice
   Subspace: users
   Permission: Read
   Path prefix: alice/* (only Alice's data)
```

### 3. GDPR-Compliant Deletion (`gdpr_delete.rs`)

Demonstrates true deletion with tombstone propagation for GDPR compliance.

**Run**:
```bash
cargo run --example gdpr_delete
```

**Shows**:
- Creating documents with personal data
- Syncing to Willow network
- GDPR deletion request (Article 17)
- Tombstone creation and propagation
- Verification of complete deletion

**Example Output**:
```
=== GDPR-Compliant Deletion Example ===

1. Creating User Document
   Created user document: users/alice
   Name: Alice Johnson
   Email: alice@example.com

2. Syncing to Willow Network
   Willow stats:
     Entries: 1
     Tombstones: 0

3. User Requests GDPR Deletion
   User: Alice Johnson
   Request: Delete all personal data
   Legal basis: GDPR Article 17 (Right to Erasure)

4. Verifying Deletion
   Willow stats:
     Entries: 0
     Tombstones: 1
   Data readable: false
```

### 4. Resource-Aware Sync (`resource_aware_sync.rs`)

Shows how to sync with memory and bandwidth constraints.

**Run**:
```bash
cargo run --example resource_aware_sync
```

**Shows**:
- High-priority sync (user-initiated, generous limits)
- Medium-priority sync (balanced constraints)
- Low-priority sync (tight resource limits)
- Incremental sync respecting constraints
- Sync statistics and performance

**Example Output**:
```
=== Resource-Aware Sync Example ===

1. Creating Sample Documents
   Created 50 user documents

2. High-Priority Sync (User-Initiated)
   Constraints:
     Max memory: 200 KB
     Max bandwidth: 1024 KB/s
     Priority: High
   Results:
     Synced: 45 documents
     Total size: 180 KB
     Errors: 0

3. Low-Priority Background Sync
   Constraints:
     Max memory: 50 KB
     Max bandwidth: 128 KB/s
     Priority: Low
   Results:
     Synced: 10 documents
     Total size: 40 KB
     Errors: 0
```

## Prerequisites

Before running these examples, ensure:

1. **Dependencies installed**:
   ```bash
   cd crates/vudo-p2p
   cargo build --examples
   ```

2. **No compilation errors**: The examples require a clean build of `vudo-p2p` and `vudo-state`.

## Key Concepts

### 3D Namespace Structure

Willow organizes data in a 3D coordinate system:

```
(namespace_id, subspace_id, path)
    ↓              ↓           ↓
  System      Collection   Document ID
```

### Capability Delegation

Capabilities can be delegated with restrictions:

```
Root (Admin)
  └─> Users Admin (Write to users/*)
        └─> Alice (Read to users/alice/*)
```

### Path Prefix Matching

Capabilities grant access to path prefixes:

```
Capability with path ["users", "alice"]
  ✓ Can access: users/alice/profile
  ✓ Can access: users/alice/posts/1
  ✗ Cannot access: users/bob/profile
```

### Permission Hierarchy

```
Admin
  ├─ Can read, write, delegate
  └─ Full namespace access

Write
  ├─ Can read, write
  └─ Cannot delegate

Read
  ├─ Can read only
  └─ Cannot write or delegate
```

## Troubleshooting

### Example won't compile

**Issue**: Compilation errors in `vudo-p2p` or `vudo-state`.

**Solution**: Check that task t2.3 (Iroh P2P integration) has been completed and all compilation errors resolved.

### "Permission denied" errors

**Issue**: Capability doesn't grant required permission.

**Solution**: Check:
1. Capability permission level (Read < Write < Admin)
2. Path prefix matches the document path
3. Subspace ID matches the collection
4. Capability signature is valid

### Sync not finding documents

**Issue**: Documents aren't being synced.

**Solution**: Verify:
1. Documents exist in the state engine
2. Capability has read permission
3. Namespace/collection mapping is correct
4. Resource constraints aren't too restrictive

## Additional Resources

- **Willow Protocol Specification**: https://willowprotocol.org/specs/
- **Meadowcap Specification**: https://willowprotocol.org/specs/meadowcap/
- **Implementation Documentation**: See `WILLOW_IMPLEMENTATION.md` in parent directory
- **Integration Tests**: See `tests/willow_integration_tests.rs`
- **Standalone Tests**: See `tests/willow_standalone_tests.rs`

## Next Steps

After running these examples, explore:

1. **Integration tests**: Run `cargo test --test willow_integration_tests`
2. **Custom capabilities**: Create your own capability hierarchies
3. **Multi-peer sync**: Test synchronization across multiple peers
4. **Performance tuning**: Experiment with resource constraints
5. **GDPR workflows**: Implement full data deletion pipelines
