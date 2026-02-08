//! CRDT introspection example
//!
//! This example demonstrates CRDT strategy analysis, validation,
//! and recommendations for DOL schemas.

use dol_reflect::crdt_introspection::{CrdtIntrospector, MergeSemantics};
use dol_reflect::schema_api::SchemaRegistry;
use metadol::CrdtStrategy;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("DOL CRDT Introspection Example");
    println!("===============================\n");

    // Schema with various CRDT annotations
    let schema = r#"
gen chat.message {
  @crdt(immutable)
  message has id: String

  @crdt(peritext)
  message has content: String

  @crdt(or_set)
  message has reactions: Set<String>

  @crdt(lww)
  message has edited_at: String
}

exegesis { Chat message with CRDT strategies }

gen counter.app {
  @crdt(immutable)
  app has id: String

  @crdt(pn_counter)
  app has click_count: Int32

  @crdt(lww)
  app has last_user: String
}

exegesis { Counter application }
"#;

    // Load schema
    let mut registry = SchemaRegistry::new();
    registry.load_schema(schema)?;

    // Create CRDT introspector
    let mut introspector = CrdtIntrospector::new();

    println!("=== CRDT Merge Semantics ===\n");

    // Demonstrate merge semantics for different strategies
    let strategies = vec![
        CrdtStrategy::Immutable,
        CrdtStrategy::Lww,
        CrdtStrategy::OrSet,
        CrdtStrategy::PnCounter,
        CrdtStrategy::Peritext,
    ];

    for strategy in strategies {
        let semantics = MergeSemantics::for_strategy(strategy);
        println!("Strategy: {:?}", strategy);
        println!("  Commutative: {}", semantics.is_commutative());
        println!("  Associative: {}", semantics.is_associative());
        println!("  Idempotent: {}", semantics.is_idempotent());
        println!("  SEC (Strong Eventual Consistency): {}", semantics.is_sec());
        println!(
            "  Conflict Resolution: {:?}",
            semantics.conflict_resolution()
        );
        println!();
    }

    println!("=== Field Analysis ===\n");

    // Analyze chat.message Gen
    if let Some(gen) = registry.get_gen("chat.message") {
        println!("Analyzing: {}\n", gen.name());

        for field in gen.crdt_fields() {
            let analysis = introspector.analyze_field(field)?;

            println!("Field: {}", analysis.field_name);
            println!("  Type: {}", analysis.field_type);
            println!("  Strategy: {:?}", analysis.strategy);
            println!("  Compatible: {}", analysis.compatible);
            println!("  SEC: {}", analysis.semantics.is_sec());

            if !analysis.issues.is_empty() {
                println!("  Issues:");
                for issue in &analysis.issues {
                    println!("    - {}", issue);
                }
            }
            println!();
        }
    }

    println!("=== Strategy Recommendations ===\n");

    // Demonstrate strategy recommendations for different types
    let types = vec![
        "String",
        "Int32",
        "Bool",
        "Set<String>",
        "Vec<Int32>",
        "List<String>",
    ];

    for type_name in types {
        if let Some(strategy) = introspector.recommend_strategy(type_name) {
            println!("{:20} -> {:?}", type_name, strategy);
        }
    }

    println!("\n=== Registry-Wide CRDT Analysis ===\n");

    // Analyze all CRDT fields in the registry
    let all_analyses = introspector.analyze_registry(&registry);

    for (gen_name, analyses) in &all_analyses {
        println!("Gen: {}", gen_name);
        println!("  CRDT Fields: {}", analyses.len());

        let compatible_count = analyses.iter().filter(|a| a.compatible).count();
        let issue_count = analyses.iter().filter(|a| !a.issues.is_empty()).count();

        println!("  Compatible: {}/{}", compatible_count, analyses.len());

        if issue_count > 0 {
            println!("  Issues: {}", issue_count);
            for analysis in analyses {
                if !analysis.issues.is_empty() {
                    println!("    Field '{}':", analysis.field_name);
                    for issue in &analysis.issues {
                        println!("      - {}", issue);
                    }
                }
            }
        }
        println!();
    }

    println!("=== CRDT Statistics ===\n");

    // Count CRDT usage by strategy
    use std::collections::HashMap;
    let mut strategy_counts: HashMap<CrdtStrategy, usize> = HashMap::new();

    for gen in registry.gens_with_crdt() {
        for field in gen.crdt_fields() {
            if let Some(strategy) = field.crdt_strategy() {
                *strategy_counts.entry(strategy).or_insert(0) += 1;
            }
        }
    }

    println!("CRDT Strategy Usage:");
    for (strategy, count) in strategy_counts {
        println!("  {:?}: {} fields", strategy, count);
    }

    println!("\nCRDT analysis completed!");

    Ok(())
}
