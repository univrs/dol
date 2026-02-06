# Gen Registry API Documentation

## Rust API

### Registry

#### Creating a Registry

```rust
use gen_registry::{Registry, RegistryConfig};

// Default configuration
let registry = Registry::new("did:key:your-did").await?;

// Custom configuration
let mut config = RegistryConfig::default();
config.owner_did = "did:key:your-did".to_string();
config.data_dir = "/path/to/data".to_string();
config.enable_p2p = true;
config.enable_search = true;

let registry = Registry::with_config(config).await?;
```

#### Publishing a Module

```rust
use gen_registry::{GenModule, ModuleVersion};
use std::path::Path;

let mut module = GenModule::new(
    "io.example.auth",
    "Authentication",
    "User authentication module",
    "did:key:author",
    "MIT",
);

module.add_tag("authentication");
module.add_tag("security");

let wasm_path = Path::new("target/wasm32-unknown-unknown/release/auth.wasm");

registry.publish(
    module,
    "1.0.0",
    wasm_path,
    "Initial release",
).await?;
```

#### Searching for Modules

```rust
use gen_registry::SearchQuery;

// Basic search
let results = registry.search("authentication").await?;

for result in results {
    println!("{} v{}", result.name, result.version);
}

// Search by tags
let results = registry.search_by_tags(&["database", "sql"]).await?;
```

#### Installing Modules

```rust
// Install latest version
registry.install("io.univrs.database", None).await?;

// Install specific version
registry.install("io.univrs.crypto", Some("2.1.0")).await?;

// Install with auto-update
registry.install_with_auto_update("io.univrs.http", "^4.0.0").await?;
```

#### Rating Modules

```rust
// Submit rating
registry.rate("io.univrs.database", 5, Some("Excellent!")).await?;

// Get average rating
if let Some(avg) = registry.get_average_rating("io.univrs.database") {
    println!("Average rating: {:.1}/5.0", avg);
}
```

#### P2P Synchronization

```rust
// Start P2P sync
registry.start_sync().await?;

// Discover peers
let peers = registry.discover_peers().await?;
println!("Connected to {} peers", peers.len());

// Sync specific module
registry.sync_module("io.univrs.user").await?;
```

### Data Models

#### GenModule

```rust
pub struct GenModule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author_did: String,
    pub license: String,
    pub tags: HashSet<string>,
    pub versions: Vec<ModuleVersion>,
    pub latest_version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub download_count: i64,
    pub dependencies: Vec<Dependency>,
}
```

#### ModuleVersion

```rust
pub struct ModuleVersion {
    pub version: String,
    pub published_at: DateTime<Utc>,
    pub wasm_hash: String,
    pub wasm_size: u64,
    pub changelog: String,
    pub signature: String,
    pub capabilities: Vec<Capability>,
    pub deprecated: bool,
    pub yanked: bool,
}
```

#### Dependency

```rust
pub struct Dependency {
    pub module_id: String,
    pub version_requirement: String,
    pub optional: bool,
}

// Create dependency
let dep = Dependency::new("io.univrs.crypto", "^1.0.0");
let optional_dep = Dependency::new("io.univrs.logging", "^0.5.0").optional();
```

#### Rating

```rust
pub struct Rating {
    pub module_id: String,
    pub user_did: String,
    pub stars: u8,  // 1-5
    pub review: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Create rating
let rating = Rating::new("io.univrs.test", "did:key:alice", 5)
    .with_review("Great module!");
```

### Search

#### SearchEngine

```rust
use gen_registry::search::{SearchEngine, SearchQuery};

// Create search engine
let engine = SearchEngine::new("/path/to/data").await?;

// Index module
let index = SearchIndex::new(&module);
engine.index_module(&index).await?;

// Search
let query = SearchQuery::new("authentication").with_limit(20);
let results = engine.search(&query).await?;
```

### Version Resolution

#### VersionResolver

```rust
use gen_registry::version::VersionResolver;

let resolver = VersionResolver::new();

// Add dependencies
resolver.add_dependency("A", "B");
resolver.add_dependency("B", "C");

// Topological sort (install order)
let order = resolver.topological_sort(&["A", "B", "C"])?;
// Returns: ["C", "B", "A"]

// Cycle detection
if resolver.has_cycle("A", "B")? {
    println!("Cycle detected!");
}
```

### WASM Validation

#### WasmModule

```rust
use gen_registry::wasm::WasmModule;

// Load from file
let module = WasmModule::from_file(Path::new("module.wasm")).await?;

// Get hash
let hash = module.hash();  // SHA-256

// Get size
let size = module.size();  // Bytes

// Extract capabilities
let caps = module.extract_capabilities()?;
```

#### WasmValidator

```rust
use gen_registry::wasm::WasmValidator;

let validator = WasmValidator::new();

// Validate WASM
validator.validate(&module)?;

// Verify hash
validator.verify_hash(&module, "expected_hash")?;
```

## REST API (Planned)

### Endpoints

#### GET /api/modules

Search for modules.

**Query Parameters**:
- `q`: Search query
- `limit`: Max results (default: 20)
- `offset`: Pagination offset

**Response**:
```json
{
  "modules": [
    {
      "id": "io.univrs.user",
      "name": "User Management",
      "version": "2.1.0",
      "description": "...",
      "rating": 4.8,
      "downloads": 15200
    }
  ],
  "total": 142
}
```

#### GET /api/modules/:id

Get module details.

**Response**:
```json
{
  "id": "io.univrs.user",
  "name": "User Management",
  "description": "...",
  "author_did": "did:key:...",
  "license": "MIT",
  "tags": ["authentication", "security"],
  "latest_version": "2.1.0",
  "versions": [...],
  "dependencies": [...]
}
```

#### POST /api/modules

Publish a new module.

**Request**:
```json
{
  "id": "io.example.auth",
  "name": "Authentication",
  "description": "...",
  "license": "MIT",
  "version": "1.0.0",
  "wasm": "base64-encoded-wasm",
  "changelog": "Initial release",
  "tags": ["auth", "security"]
}
```

#### POST /api/modules/:id/ratings

Submit a rating.

**Request**:
```json
{
  "stars": 5,
  "review": "Excellent module!"
}
```

#### GET /api/sync/status

Get P2P sync status.

**Response**:
```json
{
  "peers": 5,
  "modules_synced": 142,
  "last_sync": "2026-02-05T12:34:56Z"
}
```

## Error Handling

All functions return `Result<T, Error>`:

```rust
pub enum Error {
    ModuleNotFound(String),
    VersionNotFound { module: String, version: String },
    InvalidModuleId(String),
    InvalidSemver(semver::Error),
    VersionConflict(String),
    DependencyCycle(String),
    WasmValidationFailed(String),
    HashMismatch { expected: String, actual: String },
    PermissionDenied(String),
    SearchIndexError(String),
    SyncError(String),
    StorageError(String),
    NetworkError(String),
}
```

**Example**:
```rust
match registry.install("io.univrs.nonexistent", None).await {
    Ok(()) => println!("Installed"),
    Err(Error::ModuleNotFound(id)) => {
        println!("Module {} not found", id);
    }
    Err(e) => println!("Error: {}", e),
}
```

## Examples

See [examples/](../examples/) directory for complete examples:

- `publish.rs`: Publishing a module
- `search.rs`: Searching for modules
- `p2p_sync.rs`: P2P synchronization

---

**Next**: [User Guide](USER_GUIDE.md) for CLI usage
