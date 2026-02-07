//! Integration tests for schema reflection API

use dol_reflect::schema_api::{GenReflection, SchemaRegistry, TraitReflection};
use metadol::CrdtStrategy;

#[test]
fn test_complex_gen_reflection() {
    let source = r#"
gen user.account {
  @crdt(immutable)
  account has id: String

  account has username: String = "anonymous"

  @crdt(lww)
  account has email: String

  @personal
  @crdt(lww)
  account has phone: Option<String>

  @crdt(pn_counter)
  account has login_count: Int32 = 0

  @crdt(or_set)
  account has roles: Set<String>
}

exegesis {
  A user account with CRDT-backed fields for distributed systems.
  The account supports eventual consistency through various CRDT strategies.
}
"#;

    let mut registry = SchemaRegistry::new();
    registry.load_schema(source).unwrap();

    let gen = registry.get_gen("user.account").unwrap();

    // Basic properties
    assert_eq!(gen.name(), "user.account");
    assert_eq!(gen.field_count(), 6);
    assert!(gen.exegesis().contains("CRDT-backed"));

    // Field lookups
    assert!(gen.get_field("id").is_some());
    assert!(gen.get_field("email").is_some());
    assert!(gen.get_field("nonexistent").is_none());

    // CRDT fields
    let crdt_fields = gen.crdt_fields();
    assert_eq!(crdt_fields.len(), 5);

    // Personal data fields
    let personal_fields = gen.personal_fields();
    assert_eq!(personal_fields.len(), 1);
    assert_eq!(personal_fields[0].name(), "phone");

    // Check specific field properties
    let id_field = gen.get_field("id").unwrap();
    assert_eq!(id_field.type_name(), "String");
    assert_eq!(id_field.crdt_strategy(), Some(CrdtStrategy::Immutable));
    assert!(!id_field.is_personal());

    let username_field = gen.get_field("username").unwrap();
    assert!(username_field.default_value().is_some());

    let login_field = gen.get_field("login_count").unwrap();
    assert_eq!(login_field.crdt_strategy(), Some(CrdtStrategy::PnCounter));
}

#[test]
fn test_multiple_gen_loading() {
    let source = r#"
gen container.exists {
  container has identity: String
  container has status: String
}

exegesis { Container exists }

gen container.runtime {
  runtime has pid: Int32
  runtime has memory: Int64
}

exegesis { Container runtime }

gen network.endpoint {
  endpoint has address: String
  endpoint has port: Int32
}

exegesis { Network endpoint }
"#;

    let mut registry = SchemaRegistry::new();
    registry.load_schema(source).unwrap();

    // Check all gens are loaded
    let gen_names = registry.gen_names();
    assert_eq!(gen_names.len(), 3);
    assert!(gen_names.contains(&"container.exists"));
    assert!(gen_names.contains(&"container.runtime"));
    assert!(gen_names.contains(&"network.endpoint"));

    // Query individual gens
    assert!(registry.get_gen("container.exists").is_some());
    assert!(registry.get_gen("container.runtime").is_some());
    assert!(registry.get_gen("network.endpoint").is_some());
}

#[test]
fn test_trait_with_dependencies() {
    let source = r#"
gen container.exists {
  container has identity: String
}

exegesis { Container }

gen identity.cryptographic {
  identity has public_key: String
}

exegesis { Identity }

trait container.lifecycle {
  uses container.exists
  uses identity.cryptographic

  container is created
  container is started
  container is stopped

  each transition emits event
}

exegesis {
  Container lifecycle trait with dependencies.
}
"#;

    let mut registry = SchemaRegistry::new();
    registry.load_schema(source).unwrap();

    let trait_refl = registry.get_trait("container.lifecycle").unwrap();

    assert_eq!(trait_refl.name(), "container.lifecycle");
    assert_eq!(trait_refl.dependencies().len(), 2);
    assert!(trait_refl.dependencies().contains(&"container.exists".to_string()));
    assert!(trait_refl
        .dependencies()
        .contains(&"identity.cryptographic".to_string()));
    assert!(trait_refl.exegesis().contains("dependencies"));
}

#[test]
fn test_system_with_requirements() {
    let source = r#"
system univrs.orchestrator @ 1.0.0 {
  requires container.lifecycle >= 0.0.2
  requires node.discovery >= 0.0.1
  requires identity.cryptographic >= 0.1.0

  nodes discover peers via gossip
  all operations are authenticated
}

exegesis {
  The Univrs orchestrator system with version requirements.
}
"#;

    let mut registry = SchemaRegistry::new();
    registry.load_schema(source).unwrap();

    let system = registry.get_system("univrs.orchestrator").unwrap();

    assert_eq!(system.name(), "univrs.orchestrator");
    assert_eq!(system.version(), "1.0.0");
    assert_eq!(system.requirements().len(), 3);

    // Check specific requirement
    let req = &system.requirements()[0];
    assert_eq!(req.0, "container.lifecycle");
    assert_eq!(req.1, ">=");
    assert_eq!(req.2, "0.0.2");
}

#[test]
fn test_evolution_tracking() {
    let source = r#"
evo container.lifecycle @ 0.0.2 > 0.0.1 {
  adds container is paused
  adds container is resumed

  deprecates container is hibernated

  removes "container is frozen"

  because "workload migration requires state preservation"
}

exegesis {
  Version 0.0.2 extends the lifecycle for migration support.
}
"#;

    let mut registry = SchemaRegistry::new();
    registry.load_schema(source).unwrap();

    let evo = registry.get_evo("container.lifecycle", "0.0.2").unwrap();

    assert_eq!(evo.name(), "container.lifecycle");
    assert_eq!(evo.version(), "0.0.2");
    assert_eq!(evo.parent_version(), "0.0.1");
    assert_eq!(evo.additions().len(), 2);
    assert_eq!(evo.deprecations().len(), 1);
    assert_eq!(evo.removals().len(), 1);
    assert!(evo.rationale().unwrap().contains("migration"));
}

#[test]
fn test_gen_inheritance() {
    let source = r#"
gen base.entity {
  entity has id: String
  entity has created_at: String
}

exegesis { Base entity }

gen user.profile extends base.entity {
  user has name: String
  user has email: String
}

exegesis { User profile extends entity }
"#;

    let mut registry = SchemaRegistry::new();
    registry.load_schema(source).unwrap();

    let base = registry.get_gen("base.entity").unwrap();
    assert!(base.extends().is_none());

    let user = registry.get_gen("user.profile").unwrap();
    assert_eq!(user.extends(), Some("base.entity"));
}

#[test]
fn test_empty_registry() {
    let registry = SchemaRegistry::new();

    assert!(registry.get_gen("nonexistent").is_none());
    assert!(registry.get_trait("nonexistent").is_none());
    assert!(registry.get_system("nonexistent").is_none());
    assert_eq!(registry.total_count(), 0);
}

#[test]
fn test_registry_clear() {
    let source = r#"
gen test.gen {
  test has field: String
}

exegesis { Test }
"#;

    let mut registry = SchemaRegistry::new();
    registry.load_schema(source).unwrap();
    assert_eq!(registry.total_count(), 1);

    registry.clear();
    assert_eq!(registry.total_count(), 0);
    assert!(registry.get_gen("test.gen").is_none());
}

#[test]
fn test_query_gens_with_crdt() {
    let source = r#"
gen normal.gen {
  normal has field: String
}

exegesis { Normal }

gen crdt.gen {
  @crdt(lww)
  crdt has field: String
}

exegesis { CRDT }
"#;

    let mut registry = SchemaRegistry::new();
    registry.load_schema(source).unwrap();

    let gens_with_crdt = registry.gens_with_crdt();
    assert_eq!(gens_with_crdt.len(), 1);
    assert_eq!(gens_with_crdt[0].name(), "crdt.gen");
}

#[test]
fn test_query_gens_with_personal_data() {
    let source = r#"
gen public.gen {
  public has field: String
}

exegesis { Public }

gen private.gen {
  @personal
  private has ssn: String

  private has name: String
}

exegesis { Private }
"#;

    let mut registry = SchemaRegistry::new();
    registry.load_schema(source).unwrap();

    let gens_with_personal = registry.gens_with_personal_data();
    assert_eq!(gens_with_personal.len(), 1);
    assert_eq!(gens_with_personal[0].name(), "private.gen");
}

#[test]
fn test_performance_large_schema() {
    use std::time::Instant;

    // Generate a large schema
    let mut source = String::new();
    for i in 0..100 {
        source.push_str(&format!(
            r#"
gen test.gen{} {{
  test has field1: String
  test has field2: Int32
  test has field3: Bool
}}

exegesis {{ Test gen {} }}

"#,
            i, i
        ));
    }

    let start = Instant::now();
    let mut registry = SchemaRegistry::new();
    registry.load_schema(&source).unwrap();
    let load_time = start.elapsed();

    println!("Load time for 100 gens: {:?}", load_time);

    // Test query performance
    let start = Instant::now();
    let _ = registry.get_gen("test.gen50");
    let query_time = start.elapsed();

    println!("Query time: {:?}", query_time);

    // Query should be sub-millisecond
    assert!(query_time.as_micros() < 1000, "Query took too long: {:?}", query_time);
}
