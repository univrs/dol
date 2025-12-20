// PROSPECTIVE: Unified State Storage
// This spec defines the storage abstraction for all Univrs components
// Implementation: univrs-state crate

system univrs.storage @ 0.1.0 {
  requires persistence.durable >= 0.0.1
  requires concurrency.safe >= 0.0.1
  requires encoding.serde >= 0.0.1
  
  provides key_value_storage
  provides transaction_support
  provides change_notification
  
  default_backend is sqlite
}

exegesis {
  Storage is where state persists across restarts.
  
  Core principles:
  1. ABSTRACTED: Code depends on trait, not implementation
  2. PLUGGABLE: SQLite, etcd, or future backends
  3. TRANSACTIONAL: Atomic multi-key operations
  4. OBSERVABLE: Watch for changes (reactive patterns)
  
  SQLite is the default because:
  - Zero dependencies (embedded)
  - ACID compliant (real transactions)
  - Battle-tested (billions of deployments)
  - Fast for reads (most operations)
  - Portable (single file)
  
  This spec is BACKEND-AGNOSTIC. Same code works with:
  - SQLite (local, embedded)
  - etcd (distributed, consensus)
  - FoundationDB (massive scale)
  - In-memory (testing)
}

// === CORE PRIMITIVES ===

gene storage.key {
  key is string
  key is hierarchical (path-like)
  key uses / as separator
  
  key examples:
    /nodes/{node_id}
    /workloads/{workload_id}
    /workloads/{workload_id}/instances/{instance_id}
    /cluster/config
    /peers/{peer_id}/reputation
  
  key is case_sensitive
  key max_length is 1024 bytes
}

gene storage.value {
  value is bytes
  value is opaque to storage layer
  value is typically JSON or CBOR encoded
  
  value max_size depends on backend
  sqlite: ~1GB practical limit
  etcd: 1.5MB limit
}

gene storage.version {
  version is u64
  version increments on each write
  version enables optimistic concurrency
  version is per_key
  
  version 0 means key does not exist
}

gene storage.entry {
  entry has key
  entry has value
  entry has version
  entry has created_at
  entry has updated_at
}

// === CORE OPERATIONS ===

trait storage.get {
  get takes key
  get returns entry or none
  
  if key exists:
    return entry with current value and version
  else:
    return none
  
  get is read_only
  get is fast (microseconds for sqlite)
  
  is information.sensing
}

trait storage.set {
  set takes key
  set takes value
  set returns new_version or error
  
  if key exists:
    increment version
    update value
    update updated_at
  else:
    create entry with version 1
    set created_at and updated_at
  
  set is durable (survives crash after return)
  
  is information.storage.write
}

trait storage.delete {
  delete takes key
  delete returns success or error
  
  if key exists:
    remove entry
    return success
  else:
    return key_not_found (or success, configurable)
  
  is transformation.destruction
}

trait storage.list {
  list takes prefix
  list returns keys matching prefix
  
  list is paginated (for large result sets)
  list is ordered (lexicographic by key)
  
  example:
    list("/workloads/") returns:
      /workloads/abc-123
      /workloads/def-456
      /workloads/def-456/instances/i-1
      /workloads/def-456/instances/i-2
  
  is information.sensing
}

// === CONDITIONAL OPERATIONS (OPTIMISTIC CONCURRENCY) ===

trait storage.compare_and_set {
  compare_and_set takes key
  compare_and_set takes expected_version
  compare_and_set takes new_value
  compare_and_set returns new_version or conflict_error
  
  if current_version == expected_version:
    set value
    return new_version
  else:
    return version_conflict
  
  // Enables optimistic concurrency control
  // Client reads, computes, writes with version check
  // If conflict, client retries with fresh read
}

trait storage.create_if_not_exists {
  create_if_not_exists takes key
  create_if_not_exists takes value
  create_if_not_exists returns success or already_exists
  
  if key does not exist:
    create entry
    return success
  else:
    return already_exists
  
  // Useful for initialization, leader election
}

// === TRANSACTIONS ===

gene storage.transaction_operation {
  operation is one of:
    set(key, value)
    delete(key)
    check_version(key, expected_version)
  
  operations in transaction are atomic
  all succeed or all fail
}

trait storage.transaction {
  transaction takes operations (list of transaction_operation)
  transaction returns success or error
  
  if all checks pass:
    apply all sets and deletes atomically
    return success
  else:
    return first_failed_check
  
  transaction is ACID:
    atomic: all or nothing
    consistent: constraints maintained
    isolated: appears to execute alone
    durable: survives crash after commit
  
  is transformation.composition
}

exegesis {
  Transactions enable atomic multi-key updates:
  
  Example: Move workload from node A to node B
  
  transaction([
    check_version("/nodes/A/workloads", 5),
    check_version("/nodes/B/workloads", 3),
    set("/nodes/A/workloads", value_without_workload),
    set("/nodes/B/workloads", value_with_workload),
    set("/workloads/W/node", "B"),
  ])
  
  Either all succeed or none do. No partial state.
}

// === WATCH / NOTIFICATIONS ===

gene storage.watch_event {
  event has key
  event has event_type
  event has old_value (if update or delete)
  event has new_value (if create or update)
  event has new_version
  
  event_type is one of:
    created
    updated
    deleted
}

trait storage.watch {
  watch takes pattern (key prefix or glob)
  watch returns event_stream
  
  event_stream yields watch_events
  event_stream is async
  event_stream continues until cancelled
  
  // Enables reactive patterns
  // Dashboard receives real-time updates
  // Reconciliation loop triggers on changes
  
  is information.channel
}

exegesis {
  Watch enables reactive architectures:
  
  Instead of polling:
    loop {
      state = store.get("/workloads")
      compare_and_reconcile(state)
      sleep(5 seconds)
    }
  
  Use watch:
    store.watch("/workloads/*").for_each(|event| {
      reconcile_single(event.key, event.new_value)
    })
  
  Benefits:
  - Lower latency (immediate notification)
  - Less load (no polling overhead)
  - More efficient (process only changes)
}

// === BACKEND IMPLEMENTATIONS ===

gene storage.sqlite_backend {
  sqlite is embedded database
  sqlite uses single file
  sqlite supports transactions
  sqlite supports watch via triggers + channels
  
  sqlite is default backend
  sqlite is best for:
    single node
    development
    edge deployment
    simplicity
  
  sqlite limitations:
    single writer (use WAL mode)
    no distributed consensus
    file-based replication only
}

gene storage.etcd_backend {
  etcd is distributed key-value store
  etcd uses raft consensus
  etcd supports transactions
  etcd supports watch natively
  
  etcd is best for:
    multi-node clusters
    strong consistency requirements
    leader election
    distributed coordination
  
  etcd limitations:
    external dependency
    1.5MB value limit
    requires etcd cluster
}

gene storage.memory_backend {
  memory is in_process hashmap
  memory supports transactions (via locks)
  memory supports watch (via channels)
  
  memory is for testing only
  memory is fast
  memory is not durable
}

// === SCHEMA / ENCODING ===

trait storage.encoding {
  encoding converts typed data to bytes
  encoding converts bytes to typed data
  
  default encoding is json
  alternative encoding is cbor (smaller, faster)
  
  encoding is pluggable per key prefix
}

gene storage.schema {
  schema defines expected structure per key pattern
  schema enables validation
  schema enables migration
  
  schema is optional
  schema is enforced at application layer
  
  // Storage layer stores bytes, doesn't interpret
  // Application layer enforces schema
}

// === CONSTRAINTS ===

constraint storage.durability {
  after set returns success:
    value survives process crash
    value survives power failure (with proper HW)
  
  durability is backend-specific:
    sqlite: durable after commit (WAL mode)
    etcd: durable after majority ack
    memory: not durable (by design)
}

constraint storage.consistency {
  reads reflect all prior writes
  
  consistency model is backend-specific:
    sqlite: sequential (single writer)
    etcd: linearizable (strong)
    memory: sequential (single process)
}

constraint storage.isolation {
  concurrent operations are safe
  
  sqlite: serialized writes, concurrent reads
  etcd: serializable transactions
  memory: mutex-protected
}

// === ERROR TYPES ===

trait storage.errors {
  key_not_found means key does not exist
  version_conflict means expected_version mismatch
  already_exists means key exists (for create_if_not_exists)
  transaction_failed means one or more checks failed
  connection_error means cannot reach backend
  storage_full means no space remaining
  
  errors are explicit
  errors are retryable (except storage_full)
}

// === MIGRATIONS ===

trait storage.migration {
  migration transforms schema from version N to N+1
  migration is idempotent (safe to run multiple times)
  migration runs at startup
  
  migration_version stored at /meta/schema_version
  
  // Pattern from mycelial-state:
  // migrations/ directory with numbered SQL files
  // 001_initial.sql, 002_add_index.sql, etc.
}

// === USAGE PATTERNS ===

exegesis {
  Common patterns in Univrs:
  
  ORCHESTRATOR:
  /workloads/{id}              → WorkloadSpec (JSON)
  /workloads/{id}/instances    → Vec<InstanceId> (JSON)
  /nodes/{id}                  → NodeInfo (JSON)
  /nodes/{id}/containers       → Vec<ContainerId> (JSON)
  /cluster/leader              → PublicKey (for leader election)
  
  P2P NETWORK:
  /peers/{peer_id}             → PeerInfo (JSON)
  /peers/{peer_id}/reputation  → ReputationScore (JSON)
  /topics/{topic}/messages     → Append-only log (CBOR)
  
  SHARED:
  /identities/{public_key}     → IdentityMetadata (JSON)
  /meta/schema_version         → u64
  
  The same storage trait serves both systems.
  The key hierarchy keeps them organized.
}

// === INTEGRATION ===

exegesis {
  How storage integrates with other specs:
  
  IDENTITY (identity.dol):
  - Keypairs stored encrypted via identity.save
  - Storage keys may include public_key
  - Signed claims stored as values
  
  RECONCILIATION (reconciliation.dol):
  - Sense reads current state from storage
  - Actuate writes new state to storage
  - Watch triggers reconciliation on changes
  
  SCHEDULING (scheduling.dol):
  - Node capacity read from storage
  - Binding decisions written to storage
  - Reservations are time-limited storage entries
  
  PROTOCOLS:
  - libp2p and Chitchat use storage for persistence
  - Events may be stored for replay
  - Storage is protocol-agnostic
}
