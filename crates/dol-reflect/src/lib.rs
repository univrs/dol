//! # DOL Reflect - Runtime Schema Reflection API
//!
//! This crate provides comprehensive runtime reflection capabilities for DOL schemas,
//! including dynamic schema loading, hot-reload support, and CRDT introspection.
//!
//! # Features
//!
//! - **Schema Reflection API**: Query Gen structures, fields, constraints at runtime
//! - **Dynamic Loading**: Load .dol files at runtime with hot-reload support
//! - **CRDT Introspection**: Analyze CRDT strategies and compatibility
//! - **Type-safe API**: No stringly-typed operations
//! - **High Performance**: < 1ms reflection queries
//!
//! # Modules
//!
//! - [`schema_api`]: Core reflection API for querying schema structure
//! - [`dynamic_load`]: Dynamic schema loading with hot-reload
//! - [`crdt_introspection`]: CRDT-specific reflection and validation
//!
//! # Quick Start
//!
//! ## Basic Reflection
//!
//! ```rust
//! use dol_reflect::schema_api::SchemaRegistry;
//!
//! let mut registry = SchemaRegistry::new();
//! registry.load_schema(r#"
//! gen user.profile {
//!   user has name: String
//!   user has age: Int32
//! }
//!
//! exegesis { User profile schema }
//! "#).unwrap();
//!
//! // Query the schema
//! let gen = registry.get_gen("user.profile").unwrap();
//! assert_eq!(gen.field_count(), 2);
//! ```
//!
//! ## Dynamic Loading
//!
//! ```rust,no_run
//! use dol_reflect::dynamic_load::SchemaLoader;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut loader = SchemaLoader::new();
//! loader.load_file(Path::new("schema.dol")).await?;
//!
//! let registry = loader.registry();
//! let registry_guard = registry.read().await;
//! // Use registry...
//! # Ok(())
//! # }
//! ```
//!
//! ## CRDT Introspection
//!
//! ```rust
//! use dol_reflect::crdt_introspection::{CrdtIntrospector, MergeSemantics};
//! use metadol::CrdtStrategy;
//!
//! let mut introspector = CrdtIntrospector::new();
//!
//! // Check merge semantics
//! let semantics = MergeSemantics::for_strategy(CrdtStrategy::Lww);
//! assert!(semantics.is_commutative());
//! assert!(semantics.is_sec()); // Strong Eventual Consistency
//!
//! // Get recommended strategy for a type
//! let strategy = introspector.recommend_strategy("Set<String>");
//! assert_eq!(strategy, Some(CrdtStrategy::OrSet));
//! ```
//!
//! # Architecture
//!
//! The reflection system is built on three core components:
//!
//! 1. **SchemaRegistry** - Central registry for parsed schemas
//! 2. **SchemaLoader** - Dynamic loading with file watching
//! 3. **CrdtIntrospector** - CRDT analysis and validation
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │         Application Code                │
//! └─────────────────────────────────────────┘
//!              │         │         │
//!              ▼         ▼         ▼
//!    ┌─────────────┬──────────┬────────────┐
//!    │   Schema    │ Dynamic  │   CRDT     │
//!    │  Registry   │  Loader  │Introspector│
//!    └─────────────┴──────────┴────────────┘
//!              │         │         │
//!              └─────────┼─────────┘
//!                        ▼
//!              ┌──────────────────┐
//!              │   DOL Parser     │
//!              │   (metadol)      │
//!              └──────────────────┘
//! ```
//!
//! # Performance
//!
//! All reflection queries are designed to complete in under 1 millisecond
//! for typical schemas (< 1000 declarations). Dynamic loading time depends
//! on file size and parsing complexity.
//!
//! # Examples
//!
//! See the `examples/` directory for complete working examples:
//!
//! - `basic_reflection.rs` - Basic schema reflection
//! - `hot_reload.rs` - Dynamic loading with hot-reload
//! - `crdt_analysis.rs` - CRDT introspection and validation

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod crdt_introspection;
pub mod dynamic_load;
pub mod schema_api;

// Re-export commonly used types
pub use crdt_introspection::{
    ConflictResolution, CrdtError, CrdtFieldAnalysis, CrdtIntrospector, CrdtResult,
    MergeSemantics, TypeCompatibility,
};
pub use dynamic_load::{
    LoadError, LoadOptions, LoadResult, SchemaEvent, SchemaLoader, SchemaVersion,
};
pub use schema_api::{
    EvoReflection, FieldReflection, GenReflection, ReflectionError, ReflectionResult,
    SchemaRegistry, SystemReflection, TraitReflection,
};

// Re-export DOL types for convenience
pub use metadol::ast::{CrdtAnnotation, CrdtStrategy, Declaration, Visibility};

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_reflection_workflow() {
        let source = r#"
gen chat.message {
  @crdt(immutable)
  message has id: String

  @crdt(peritext)
  message has content: String

  @crdt(or_set)
  message has reactions: Set<String>
}

exegesis { Chat message with CRDT annotations }

trait message.editable {
  uses chat.message
  message is editable
  message is versionable
}

exegesis { Editable message trait }
"#;

        // Load schema
        let mut registry = SchemaRegistry::new();
        assert!(registry.load_schema(source).is_ok());

        // Query Gen
        let gen = registry.get_gen("chat.message").unwrap();
        assert_eq!(gen.field_count(), 3);

        // Query CRDT fields
        let crdt_fields = gen.crdt_fields();
        assert_eq!(crdt_fields.len(), 3);

        // Query Trait
        let trait_refl = registry.get_trait("message.editable").unwrap();
        assert_eq!(trait_refl.dependencies().len(), 1);

        // CRDT introspection
        let mut introspector = CrdtIntrospector::new();
        let analyses = introspector.analyze_gen(gen);
        assert_eq!(analyses.len(), 3);

        // Check merge semantics
        let id_field = gen.get_field("id").unwrap();
        let analysis = introspector.analyze_field(id_field).unwrap();
        assert!(analysis.semantics.is_sec());
        assert_eq!(
            analysis.semantics.conflict_resolution(),
            ConflictResolution::NoConflicts
        );
    }

    #[test]
    fn test_crdt_compatibility_validation() {
        let source = r#"
gen counter {
  @crdt(pn_counter)
  counter has value: Int32
}

exegesis { Counter }
"#;

        let mut registry = SchemaRegistry::new();
        registry.load_schema(source).unwrap();

        let gen = registry.get_gen("counter").unwrap();
        let field = gen.get_field("value").unwrap();

        let mut introspector = CrdtIntrospector::new();
        let analysis = introspector.analyze_field(field).unwrap();

        assert!(analysis.compatible);
        assert_eq!(analysis.strategy, CrdtStrategy::PnCounter);
    }

    #[test]
    fn test_personal_data_query() {
        let source = r#"
gen user.profile {
  user has id: String

  @personal
  user has email: String

  @personal
  user has phone: String
}

exegesis { User profile }
"#;

        let mut registry = SchemaRegistry::new();
        registry.load_schema(source).unwrap();

        let gen = registry.get_gen("user.profile").unwrap();
        let personal_fields = gen.personal_fields();

        assert_eq!(personal_fields.len(), 2);
        assert!(personal_fields.iter().any(|f| f.name() == "email"));
        assert!(personal_fields.iter().any(|f| f.name() == "phone"));
    }
}
