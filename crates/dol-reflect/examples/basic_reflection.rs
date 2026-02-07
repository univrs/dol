//! Basic schema reflection example
//!
//! This example demonstrates how to use the schema reflection API
//! to query Gen structures and fields at runtime.

use dol_reflect::schema_api::SchemaRegistry;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // DOL schema source
    let schema = r#"
gen user.profile {
  @crdt(immutable)
  user has id: String

  user has username: String = "anonymous"

  @personal
  @crdt(lww)
  user has email: String

  @crdt(pn_counter)
  user has login_count: Int32 = 0
}

exegesis {
  A user profile with CRDT-backed fields for distributed systems.
}

trait user.authentication {
  uses user.profile

  user is authenticated
  user is authorized
}

exegesis {
  User authentication trait.
}
"#;

    // Create a registry and load the schema
    let mut registry = SchemaRegistry::new();
    registry.load_schema(schema)?;

    // Query the Gen
    println!("=== Gen Reflection ===");
    if let Some(gen) = registry.get_gen("user.profile") {
        println!("Gen: {}", gen.name());
        println!("Fields: {}", gen.field_count());
        println!("Exegesis: {}", gen.exegesis());
        println!();

        // List all fields
        println!("Fields:");
        for field in gen.fields() {
            println!("  - {}: {}", field.name(), field.type_name());
            if let Some(default) = field.default_value() {
                println!("    Default: {}", default);
            }
            if let Some(strategy) = field.crdt_strategy() {
                println!("    CRDT: {:?}", strategy);
            }
            if field.is_personal() {
                println!("    Personal data: yes");
            }
        }
        println!();

        // Query CRDT fields
        println!("CRDT Fields:");
        for field in gen.crdt_fields() {
            println!(
                "  - {}: {} (strategy: {:?})",
                field.name(),
                field.type_name(),
                field.crdt_strategy().unwrap()
            );
        }
        println!();

        // Query personal data fields
        println!("Personal Data Fields:");
        for field in gen.personal_fields() {
            println!("  - {}: {}", field.name(), field.type_name());
        }
    }

    // Query the Trait
    println!("\n=== Trait Reflection ===");
    if let Some(trait_refl) = registry.get_trait("user.authentication") {
        println!("Trait: {}", trait_refl.name());
        println!("Dependencies:");
        for dep in trait_refl.dependencies() {
            println!("  - uses {}", dep);
        }
    }

    // Show registry statistics
    println!("\n=== Registry Statistics ===");
    println!("Total declarations: {}", registry.total_count());
    println!("Gens: {}", registry.gen_names().len());
    println!("Traits: {}", registry.trait_names().len());
    println!("Systems: {}", registry.system_names().len());
    println!("Evolutions: {}", registry.evo_names().len());

    Ok(())
}
