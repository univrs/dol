//! Schema Reflection API for DOL
//!
//! This module provides runtime reflection capabilities for DOL schemas,
//! allowing programs to query Gen structures, fields, constraints, Traits,
//! Systems, and Evolutions at runtime.
//!
//! # Features
//!
//! - Type-safe reflection API (no stringly-typed operations)
//! - Query Gen structure and fields
//! - Enumerate Traits, Systems, Evolutions
//! - Access CRDT annotations programmatically
//! - Performance target: < 1ms reflection queries
//!
//! # Example
//!
//! ```rust
//! use dol_reflect::schema_api::{SchemaRegistry, GenReflection};
//!
//! // Create a registry and load a schema
//! let mut registry = SchemaRegistry::new();
//!
//! // Query a Gen structure
//! if let Some(gen) = registry.get_gen("container.exists") {
//!     println!("Gen: {}", gen.name());
//!     for field in gen.fields() {
//!         println!("  Field: {} : {}", field.name(), field.type_name());
//!     }
//! }
//! ```

use metadol::{
    ast::{
        CrdtAnnotation, CrdtStrategy, Declaration, Evo, Gen, HasField, Rule, Statement, System,
        Trait, TypeExpr, Visibility,
    },
    parse_file, parse_file_all, DolFile, ParseError,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// Error type for schema reflection operations.
#[derive(Debug, thiserror::Error)]
pub enum ReflectionError {
    /// Schema not found in registry
    #[error("Schema '{0}' not found")]
    SchemaNotFound(String),

    /// Field not found in Gen
    #[error("Field '{field}' not found in Gen '{gen}'")]
    FieldNotFound { gen: String, field: String },

    /// Invalid operation on schema type
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Parse error when loading schema
    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),
}

/// Result type for reflection operations.
pub type ReflectionResult<T> = Result<T, ReflectionError>;

/// Reflected field information from a Gen declaration.
///
/// Provides type-safe access to field metadata including name, type,
/// default value, constraints, and CRDT annotations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldReflection {
    name: String,
    type_name: String,
    default_value: Option<String>,
    constraint: Option<String>,
    crdt_annotation: Option<CrdtAnnotation>,
    is_personal: bool,
}

impl FieldReflection {
    /// Creates a new field reflection from an AST HasField node.
    pub fn from_ast(field: &HasField) -> Self {
        Self {
            name: field.name.clone(),
            type_name: Self::type_expr_to_string(&field.type_),
            default_value: field.default.as_ref().map(|expr| format!("{:?}", expr)),
            constraint: field.constraint.as_ref().map(|expr| format!("{:?}", expr)),
            crdt_annotation: field.crdt_annotation.clone(),
            is_personal: field.personal,
        }
    }

    /// Returns the field name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the field type name.
    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    /// Returns the default value if present.
    pub fn default_value(&self) -> Option<&str> {
        self.default_value.as_deref()
    }

    /// Returns the constraint expression if present.
    pub fn constraint(&self) -> Option<&str> {
        self.constraint.as_deref()
    }

    /// Returns the CRDT annotation if present.
    pub fn crdt_annotation(&self) -> Option<&CrdtAnnotation> {
        self.crdt_annotation.as_ref()
    }

    /// Returns the CRDT strategy if annotated.
    pub fn crdt_strategy(&self) -> Option<CrdtStrategy> {
        self.crdt_annotation.as_ref().map(|a| a.strategy)
    }

    /// Returns whether the field is marked as personal data.
    pub fn is_personal(&self) -> bool {
        self.is_personal
    }

    /// Converts a TypeExpr to a string representation.
    fn type_expr_to_string(type_expr: &TypeExpr) -> String {
        match type_expr {
            TypeExpr::Named(name) => name.clone(),
            TypeExpr::Generic { name, args } => {
                let args_str = args
                    .iter()
                    .map(Self::type_expr_to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}<{}>", name, args_str)
            }
            TypeExpr::Function { params, return_type } => {
                let params_str = params
                    .iter()
                    .map(Self::type_expr_to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({}) -> {}", params_str, Self::type_expr_to_string(return_type))
            }
            TypeExpr::Tuple(types) => {
                let types_str = types
                    .iter()
                    .map(Self::type_expr_to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({})", types_str)
            }
            TypeExpr::Never => "!".to_string(),
            TypeExpr::Enum { .. } => "enum".to_string(),
        }
    }
}

/// Reflected Gen (gene) declaration with full metadata.
///
/// Provides type-safe access to Gen structure including fields,
/// statements, and documentation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenReflection {
    name: String,
    visibility: Visibility,
    extends: Option<String>,
    fields: Vec<FieldReflection>,
    statements: Vec<String>,
    exegesis: String,
}

impl GenReflection {
    /// Creates a new Gen reflection from an AST Gen node.
    pub fn from_ast(gen: &Gen) -> Self {
        let fields = gen
            .statements
            .iter()
            .filter_map(|stmt| {
                if let Statement::HasField(field) = stmt {
                    Some(FieldReflection::from_ast(field))
                } else {
                    None
                }
            })
            .collect();

        let statements = gen
            .statements
            .iter()
            .map(|stmt| format!("{:?}", stmt))
            .collect();

        Self {
            name: gen.name.clone(),
            visibility: gen.visibility,
            extends: gen.extends.clone(),
            fields,
            statements,
            exegesis: gen.exegesis.clone(),
        }
    }

    /// Returns the Gen name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the visibility modifier.
    pub fn visibility(&self) -> Visibility {
        self.visibility
    }

    /// Returns the parent Gen name if this Gen extends another.
    pub fn extends(&self) -> Option<&str> {
        self.extends.as_deref()
    }

    /// Returns all fields in this Gen.
    pub fn fields(&self) -> &[FieldReflection] {
        &self.fields
    }

    /// Looks up a field by name.
    pub fn get_field(&self, name: &str) -> Option<&FieldReflection> {
        self.fields.iter().find(|f| f.name() == name)
    }

    /// Returns the number of fields.
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    /// Returns all statements as strings.
    pub fn statements(&self) -> &[String] {
        &self.statements
    }

    /// Returns the exegesis documentation.
    pub fn exegesis(&self) -> &str {
        &self.exegesis
    }

    /// Returns all fields with CRDT annotations.
    pub fn crdt_fields(&self) -> Vec<&FieldReflection> {
        self.fields
            .iter()
            .filter(|f| f.crdt_annotation().is_some())
            .collect()
    }

    /// Returns all personal data fields.
    pub fn personal_fields(&self) -> Vec<&FieldReflection> {
        self.fields.iter().filter(|f| f.is_personal()).collect()
    }
}

/// Reflected Trait declaration with dependencies and statements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitReflection {
    name: String,
    visibility: Visibility,
    dependencies: Vec<String>,
    statements: Vec<String>,
    exegesis: String,
}

impl TraitReflection {
    /// Creates a new Trait reflection from an AST Trait node.
    pub fn from_ast(trait_decl: &Trait) -> Self {
        let dependencies = trait_decl
            .statements
            .iter()
            .filter_map(|stmt| {
                if let Statement::Uses { reference, .. } = stmt {
                    Some(reference.clone())
                } else {
                    None
                }
            })
            .collect();

        let statements = trait_decl
            .statements
            .iter()
            .map(|stmt| format!("{:?}", stmt))
            .collect();

        Self {
            name: trait_decl.name.clone(),
            visibility: trait_decl.visibility,
            dependencies,
            statements,
            exegesis: trait_decl.exegesis.clone(),
        }
    }

    /// Returns the Trait name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the visibility modifier.
    pub fn visibility(&self) -> Visibility {
        self.visibility
    }

    /// Returns all dependencies (uses statements).
    pub fn dependencies(&self) -> &[String] {
        &self.dependencies
    }

    /// Returns all statements.
    pub fn statements(&self) -> &[String] {
        &self.statements
    }

    /// Returns the exegesis documentation.
    pub fn exegesis(&self) -> &str {
        &self.exegesis
    }
}

/// Reflected System declaration with version and requirements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemReflection {
    name: String,
    visibility: Visibility,
    version: String,
    requirements: Vec<(String, String, String)>, // (name, constraint, version)
    statements: Vec<String>,
    exegesis: String,
}

impl SystemReflection {
    /// Creates a new System reflection from an AST System node.
    pub fn from_ast(system: &System) -> Self {
        let requirements = system
            .requirements
            .iter()
            .map(|req| (req.name.clone(), req.constraint.clone(), req.version.clone()))
            .collect();

        let statements = system
            .statements
            .iter()
            .map(|stmt| format!("{:?}", stmt))
            .collect();

        Self {
            name: system.name.clone(),
            visibility: system.visibility,
            version: system.version.clone(),
            requirements,
            statements,
            exegesis: system.exegesis.clone(),
        }
    }

    /// Returns the System name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the visibility modifier.
    pub fn visibility(&self) -> Visibility {
        self.visibility
    }

    /// Returns the System version.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Returns all requirements (dependency constraints).
    pub fn requirements(&self) -> &[(String, String, String)] {
        &self.requirements
    }

    /// Returns all statements.
    pub fn statements(&self) -> &[String] {
        &self.statements
    }

    /// Returns the exegesis documentation.
    pub fn exegesis(&self) -> &str {
        &self.exegesis
    }
}

/// Reflected Evolution (Evo) declaration tracking schema changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvoReflection {
    name: String,
    version: String,
    parent_version: String,
    additions: Vec<String>,
    deprecations: Vec<String>,
    removals: Vec<String>,
    rationale: Option<String>,
    exegesis: String,
}

impl EvoReflection {
    /// Creates a new Evo reflection from an AST Evo node.
    pub fn from_ast(evo: &Evo) -> Self {
        let additions = evo
            .additions
            .iter()
            .map(|stmt| format!("{:?}", stmt))
            .collect();

        let deprecations = evo
            .deprecations
            .iter()
            .map(|stmt| format!("{:?}", stmt))
            .collect();

        Self {
            name: evo.name.clone(),
            version: evo.version.clone(),
            parent_version: evo.parent_version.clone(),
            additions,
            deprecations,
            removals: evo.removals.clone(),
            rationale: evo.rationale.clone(),
            exegesis: evo.exegesis.clone(),
        }
    }

    /// Returns the declaration name being evolved.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the new version.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Returns the parent version.
    pub fn parent_version(&self) -> &str {
        &self.parent_version
    }

    /// Returns additions in this evolution.
    pub fn additions(&self) -> &[String] {
        &self.additions
    }

    /// Returns deprecations in this evolution.
    pub fn deprecations(&self) -> &[String] {
        &self.deprecations
    }

    /// Returns removals in this evolution.
    pub fn removals(&self) -> &[String] {
        &self.removals
    }

    /// Returns the rationale for this evolution.
    pub fn rationale(&self) -> Option<&str> {
        self.rationale.as_deref()
    }

    /// Returns the exegesis documentation.
    pub fn exegesis(&self) -> &str {
        &self.exegesis
    }
}

/// Central registry for schema reflection.
///
/// The schema registry maintains all parsed DOL declarations and provides
/// efficient lookup and querying capabilities.
///
/// # Performance
///
/// All query operations are designed to complete in < 1ms for typical schemas.
#[derive(Debug, Clone, Default)]
pub struct SchemaRegistry {
    gens: HashMap<String, GenReflection>,
    traits: HashMap<String, TraitReflection>,
    systems: HashMap<String, SystemReflection>,
    evos: HashMap<String, EvoReflection>,
}

impl SchemaRegistry {
    /// Creates a new empty schema registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads a schema from DOL source text.
    ///
    /// # Performance
    ///
    /// Parsing time depends on schema size, but reflection indexing
    /// completes in < 1ms after parsing.
    pub fn load_schema(&mut self, source: &str) -> ReflectionResult<()> {
        let start = Instant::now();

        // Parse all declarations
        let declarations = parse_file_all(source)?;

        // Index all declarations
        for decl in declarations {
            match decl {
                Declaration::Gene(gen) => {
                    let reflection = GenReflection::from_ast(&gen);
                    self.gens.insert(gen.name.clone(), reflection);
                }
                Declaration::Trait(trait_decl) => {
                    let reflection = TraitReflection::from_ast(&trait_decl);
                    self.traits.insert(trait_decl.name.clone(), reflection);
                }
                Declaration::System(system) => {
                    let reflection = SystemReflection::from_ast(&system);
                    self.systems.insert(system.name.clone(), reflection);
                }
                Declaration::Evolution(evo) => {
                    let reflection = EvoReflection::from_ast(&evo);
                    self.evos
                        .insert(format!("{}@{}", evo.name, evo.version), reflection);
                }
                Declaration::Constraint(rule) => {
                    // Rules are converted to Gen for reflection
                    // (they share similar structure)
                    let gen = Gen {
                        visibility: rule.visibility,
                        name: rule.name.clone(),
                        extends: None,
                        statements: rule.statements.clone(),
                        exegesis: rule.exegesis.clone(),
                        span: rule.span,
                    };
                    let reflection = GenReflection::from_ast(&gen);
                    self.gens.insert(rule.name, reflection);
                }
                _ => {
                    // Skip function and const declarations for now
                }
            }
        }

        let elapsed = start.elapsed();
        if elapsed.as_millis() > 1 {
            eprintln!("Warning: Reflection indexing took {}ms", elapsed.as_millis());
        }

        Ok(())
    }

    /// Looks up a Gen by name.
    pub fn get_gen(&self, name: &str) -> Option<&GenReflection> {
        self.gens.get(name)
    }

    /// Looks up a Trait by name.
    pub fn get_trait(&self, name: &str) -> Option<&TraitReflection> {
        self.traits.get(name)
    }

    /// Looks up a System by name.
    pub fn get_system(&self, name: &str) -> Option<&SystemReflection> {
        self.systems.get(name)
    }

    /// Looks up an Evolution by name and version.
    pub fn get_evo(&self, name: &str, version: &str) -> Option<&EvoReflection> {
        self.evos.get(&format!("{}@{}", name, version))
    }

    /// Returns all Gen names.
    pub fn gen_names(&self) -> Vec<&str> {
        self.gens.keys().map(|s| s.as_str()).collect()
    }

    /// Returns all Trait names.
    pub fn trait_names(&self) -> Vec<&str> {
        self.traits.keys().map(|s| s.as_str()).collect()
    }

    /// Returns all System names.
    pub fn system_names(&self) -> Vec<&str> {
        self.systems.keys().map(|s| s.as_str()).collect()
    }

    /// Returns all Evo identifiers (name@version).
    pub fn evo_names(&self) -> Vec<&str> {
        self.evos.keys().map(|s| s.as_str()).collect()
    }

    /// Returns all Gens.
    pub fn gens(&self) -> impl Iterator<Item = &GenReflection> {
        self.gens.values()
    }

    /// Returns all Traits.
    pub fn traits(&self) -> impl Iterator<Item = &TraitReflection> {
        self.traits.values()
    }

    /// Returns all Systems.
    pub fn systems(&self) -> impl Iterator<Item = &SystemReflection> {
        self.systems.values()
    }

    /// Returns all Evolutions.
    pub fn evos(&self) -> impl Iterator<Item = &EvoReflection> {
        self.evos.values()
    }

    /// Returns the total number of registered declarations.
    pub fn total_count(&self) -> usize {
        self.gens.len() + self.traits.len() + self.systems.len() + self.evos.len()
    }

    /// Clears all registered schemas.
    pub fn clear(&mut self) {
        self.gens.clear();
        self.traits.clear();
        self.systems.clear();
        self.evos.clear();
    }

    /// Queries all Gens that have CRDT annotations.
    pub fn gens_with_crdt(&self) -> Vec<&GenReflection> {
        self.gens
            .values()
            .filter(|g| !g.crdt_fields().is_empty())
            .collect()
    }

    /// Queries all Gens with personal data fields.
    pub fn gens_with_personal_data(&self) -> Vec<&GenReflection> {
        self.gens
            .values()
            .filter(|g| !g.personal_fields().is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_simple_gen() {
        let source = r#"
gen container.exists {
  container has identity: String
  container has status: String
}

exegesis {
  A container is the fundamental unit.
}
"#;

        let mut registry = SchemaRegistry::new();
        assert!(registry.load_schema(source).is_ok());

        let gen = registry.get_gen("container.exists").unwrap();
        assert_eq!(gen.name(), "container.exists");
        assert_eq!(gen.field_count(), 2);
    }

    #[test]
    fn test_gen_field_lookup() {
        let source = r#"
gen user.profile {
  user has name: String
  user has age: Int32 = 0
}

exegesis { User profile }
"#;

        let mut registry = SchemaRegistry::new();
        registry.load_schema(source).unwrap();

        let gen = registry.get_gen("user.profile").unwrap();
        let name_field = gen.get_field("name");
        assert!(name_field.is_some());
        assert_eq!(name_field.unwrap().type_name(), "String");

        let age_field = gen.get_field("age");
        assert!(age_field.is_some());
        assert!(age_field.unwrap().default_value().is_some());
    }

    #[test]
    fn test_crdt_field_query() {
        let source = r#"
gen chat.message {
  @crdt(immutable)
  message has id: String

  @crdt(peritext)
  message has content: String
}

exegesis { Chat message }
"#;

        let mut registry = SchemaRegistry::new();
        registry.load_schema(source).unwrap();

        let gen = registry.get_gen("chat.message").unwrap();
        let crdt_fields = gen.crdt_fields();
        assert_eq!(crdt_fields.len(), 2);

        let id_field = gen.get_field("id").unwrap();
        assert_eq!(
            id_field.crdt_strategy(),
            Some(CrdtStrategy::Immutable)
        );
    }

    #[test]
    fn test_trait_reflection() {
        let source = r#"
trait container.lifecycle {
  uses container.exists
  container is created
  container is started
}

exegesis { Container lifecycle }
"#;

        let mut registry = SchemaRegistry::new();
        registry.load_schema(source).unwrap();

        let trait_refl = registry.get_trait("container.lifecycle").unwrap();
        assert_eq!(trait_refl.name(), "container.lifecycle");
        assert_eq!(trait_refl.dependencies().len(), 1);
        assert_eq!(trait_refl.dependencies()[0], "container.exists");
    }

    #[test]
    fn test_system_reflection() {
        let source = r#"
system univrs.orchestrator @ 0.1.0 {
  requires container.lifecycle >= 0.0.2
  requires node.discovery >= 0.0.1
}

exegesis { Univrs orchestrator }
"#;

        let mut registry = SchemaRegistry::new();
        registry.load_schema(source).unwrap();

        let system = registry.get_system("univrs.orchestrator").unwrap();
        assert_eq!(system.version(), "0.1.0");
        assert_eq!(system.requirements().len(), 2);
    }

    #[test]
    fn test_performance_target() {
        let source = r#"
gen example.gen {
  example has field1: String
  example has field2: Int32
  example has field3: Bool
}

exegesis { Test }
"#;

        let mut registry = SchemaRegistry::new();
        let start = Instant::now();
        registry.load_schema(source).unwrap();
        let load_time = start.elapsed();

        let start = Instant::now();
        let _ = registry.get_gen("example.gen");
        let query_time = start.elapsed();

        // Query should be sub-millisecond
        assert!(query_time.as_micros() < 1000);
    }
}
