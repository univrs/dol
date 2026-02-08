# VUDO AI - Local-First AI Integration

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

Privacy-preserving AI capabilities for the VUDO Runtime, enabling on-device intelligence without compromising user data.

## Features

- **🔒 Privacy-First**: All inference happens locally on-device by default
- **🌐 WASM-Compatible**: Runs in browsers and on native platforms
- **🧠 Local Embeddings**: Generate semantic embeddings for search without external APIs
- **⚡ Conflict Resolution**: AI-assisted suggestions for CRDT merge conflicts
- **🌍 Optional P2P**: Distributed inference via PlanetServe with S-IDA fragmentation
- **📦 Small Models**: Optimized for models < 50MB that run efficiently on-device

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     AIService                           │
├─────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │   Model      │  │  Embedding   │  │  Conflict    │ │
│  │   Manager    │  │   Service    │  │  Resolver    │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
│  ┌──────────────┐  ┌──────────────────────────────────┐│
│  │  Inference   │  │  PlanetServe Integration (opt)  ││
│  │   Engine     │  │  • S-IDA Fragmentation           ││
│  └──────────────┘  │  • Onion Routing                 ││
│                    │  • P2P Model Inference           ││
│                    └──────────────────────────────────┘│
└─────────────────────────────────────────────────────────┘
         ↓                          ↓
   ┌─────────┐              ┌──────────────┐
   │  Tract  │              │ PlanetServe  │
   │  ONNX   │              │   Network    │
   └─────────┘              └──────────────┘
```

## Usage

### Basic Embedding Service

```rust
use vudo_ai::{
    AIService,
    model_manager::{ModelId, ModelMetadata, ModelType},
};

// Initialize AI service
let service = AIService::new();

// Register a model
let metadata = ModelMetadata {
    id: ModelId::new("my-embedding-model"),
    name: "Embedding Model".to_string(),
    description: "Local semantic embedding model".to_string(),
    version: "1.0.0".to_string(),
    input_dims: vec![1, 512],
    output_dims: vec![1, 384],
    size_bytes: 45_000_000,  // 45MB
    model_type: ModelType::Embedding,
    wasm_compatible: true,
};
service.model_manager.register(metadata)?;

// Load model bytes (from file, network, etc.)
let model_bytes = std::fs::read("path/to/model.onnx")?;
service.model_manager.load(&ModelId::new("my-embedding-model"), model_bytes)?;

// Enable embedding service
let service = service.with_embedding_service(ModelId::new("my-embedding-model"));

// Generate embeddings
let embedding = service.embedding_service.as_ref().unwrap()
    .embed("Hello, world!")?;

// Index documents for search
let embedding_service = service.embedding_service.as_ref().unwrap();
embedding_service.index_document("doc1".to_string(), "AI and machine learning")?;
embedding_service.index_document("doc2".to_string(), "Cooking recipes")?;

// Semantic search
let results = embedding_service.search("artificial intelligence", 5)?;
for result in results {
    println!("Document: {}, Score: {}", result.id, result.score);
}
```

### Conflict Resolution

```rust
use vudo_ai::{
    AIService,
    conflict_resolution::{Conflict, ConflictValue},
    model_manager::{ModelId, ModelMetadata, ModelType},
};

// Setup (register model and enable embedding service first)
let service = AIService::new()
    .with_embedding_service(ModelId::new("my-model"))
    .with_conflict_resolver()?;

// Create a conflict
let conflict = Conflict {
    document_id: "user-profile".to_string(),
    object_id: "root".to_string(),
    key: "bio".to_string(),
    local_value: ConflictValue::String("Software engineer".to_string()),
    remote_value: ConflictValue::String("Senior software engineer".to_string()),
    context: vec![],
};

// Get AI-powered resolution suggestion
let resolver = service.conflict_resolver.as_ref().unwrap();
let suggestion = resolver.suggest_resolution(&conflict)?;

println!("Strategy: {:?}", suggestion.strategy);
println!("Confidence: {}", suggestion.confidence);
println!("Explanation: {}", suggestion.explanation);

// Apply resolution to Automerge document
let mut doc = automerge::AutoCommit::new();
resolver.apply_resolution(&mut doc, &automerge::ROOT, &suggestion)?;
```

### P2P Distributed Inference (Optional)

```rust
use vudo_ai::{
    AIService,
    planetserve_integration::{P2PInferenceConfig, P2PInferenceRequest},
    inference::InferenceTensor,
    model_manager::ModelId,
};
use vudo_planetserve::{FragmentManager, SidaConfig, PrivacyConfig};

// Setup PlanetServe integration
let fragment_manager = Arc::new(FragmentManager::new(
    SidaConfig::default(),
    PrivacyConfig::default(),
));

let config = P2PInferenceConfig::default();
let service = AIService::new()
    .with_planetserve(fragment_manager, config);

// Run inference across P2P network
let request = P2PInferenceRequest {
    model_id: ModelId::new("large-model"),
    inputs: vec![InferenceTensor::float32(vec![1, 512], vec![0.5; 512])?],
    timestamp: 0,
};

let response = service.planetserve_ai.as_ref().unwrap()
    .infer_p2p(request).await?;

println!("Inference completed in {} ms", response.latency_ms);
println!("Used {} peers", response.peer_count);
```

## Privacy Guarantees

### Local-First by Default

- All models run **entirely on-device**
- No data sent to external servers
- No telemetry or tracking
- WASM sandboxing for additional security

### Optional P2P with Privacy

When using PlanetServe integration:

- **S-IDA Fragmentation**: Data split into encrypted fragments
- **Onion Routing**: Multi-hop routing with layer encryption
- **Zero-Knowledge**: Peers cannot reconstruct original data
- **Metadata Obfuscation**: Timing and size patterns hidden

## Performance

### Benchmarks

Run benchmarks with:

```bash
cargo bench --features full
```

Typical performance (on modern laptop):

- **Embedding Generation**: ~5-20ms per text (384-dim)
- **Similarity Search**: ~1ms for 1000 documents
- **Conflict Resolution**: ~10-50ms per conflict
- **Model Loading**: ~50-200ms (50MB model)

### Memory Usage

- **Model Cache**: Configurable (default: 200MB max)
- **Embedding Index**: ~1.5KB per document (384-dim)
- **LRU Eviction**: Automatic memory management

## Testing

Run the test suite:

```bash
# All tests
cargo test

# Integration tests only
cargo test --test integration_tests

# With output
cargo test -- --nocapture

# Specific test
cargo test test_embedding_workflow
```

Test coverage:

- **Unit Tests**: 50+ tests across all modules
- **Integration Tests**: 15+ end-to-end scenarios
- **Benchmarks**: Performance regression detection

## WASM Support

Build for WASM:

```bash
# Add WASM target
rustup target add wasm32-unknown-unknown

# Build for WASM
cargo build --target wasm32-unknown-unknown --release

# Optimize with wasm-opt
wasm-opt -Oz -o output.wasm target/wasm32-unknown-unknown/release/vudo_ai.wasm
```

## Dependencies

- `tract-onnx`: ONNX runtime (WASM-compatible)
- `vudo-state`: CRDT document management
- `vudo-planetserve`: P2P networking with S-IDA
- `automerge`: CRDT implementation

## Roadmap

- [ ] Pre-trained embedding models (< 50MB)
- [ ] Quantized model support (INT8, INT4)
- [ ] Streaming inference for large inputs
- [ ] Model hot-swapping without cache flush
- [ ] WebGPU acceleration
- [ ] More sophisticated conflict resolution strategies

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../../LICENSE-MIT))

at your option.

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass: `cargo test`
2. Code is formatted: `cargo fmt`
3. No clippy warnings: `cargo clippy`
4. Documentation is updated

## Acknowledgments

- [Tract](https://github.com/sonos/tract) for WASM-compatible ONNX runtime
- [Automerge](https://automerge.org/) for CRDT support
- VUDO Runtime team for the local-first architecture
