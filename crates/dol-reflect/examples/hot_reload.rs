//! Hot-reload example
//!
//! This example demonstrates dynamic schema loading with hot-reload support.
//! It watches a directory for schema changes and automatically reloads them.

use dol_reflect::dynamic_load::{process_schema_event, SchemaLoader};
use std::path::Path;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("DOL Hot-Reload Example");
    println!("======================\n");

    // Create a temporary directory for our schemas
    let temp_dir = tempfile::tempdir()?;
    let schema_dir = temp_dir.path();

    println!("Schema directory: {}", schema_dir.display());
    println!();

    // Create initial schema file
    let schema_file = schema_dir.join("user.dol");
    tokio::fs::write(
        &schema_file,
        r#"
gen user.profile {
  user has id: String
  user has name: String
}

exegesis { User profile v1 }
"#,
    )
    .await?;

    // Create the loader
    let mut loader = SchemaLoader::new();

    // Load initial schemas
    println!("Loading initial schemas...");
    loader.load_directory(schema_dir).await?;

    // Print loaded schemas
    {
        let registry = loader.registry();
        let guard = registry.read().await;
        println!("Loaded {} declarations", guard.total_count());
        for name in guard.gen_names() {
            if let Some(gen) = guard.get_gen(name) {
                println!("  - {} ({} fields)", gen.name(), gen.field_count());
            }
        }
    }
    println!();

    // Set up file watcher for hot-reload
    println!("Starting hot-reload watcher...");
    let (watcher, mut rx) = loader.watch_directory(schema_dir).await?;

    // Spawn a task to handle schema events
    let registry = loader.registry();
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            println!("\n[EVENT] Schema change detected!");
            if let Err(e) = process_schema_event(event).await {
                eprintln!("Error processing schema event: {}", e);
            }
        }
    });

    // Simulate some file modifications
    println!("Simulating schema modifications...\n");

    // Wait a bit
    sleep(Duration::from_secs(1)).await;

    // Modify the schema
    println!("Modifying schema (adding email field)...");
    tokio::fs::write(
        &schema_file,
        r#"
gen user.profile {
  user has id: String
  user has name: String
  user has email: String
}

exegesis { User profile v2 - added email }
"#,
    )
    .await?;

    // Wait for the event to be processed
    sleep(Duration::from_secs(2)).await;

    // Reload manually to see changes
    loader.reload_file(&schema_file).await?;

    // Print updated schema
    {
        let guard = registry.read().await;
        if let Some(gen) = guard.get_gen("user.profile") {
            println!("\nUpdated Gen:");
            println!("  Name: {}", gen.name());
            println!("  Fields: {}", gen.field_count());
            println!("  Exegesis: {}", gen.exegesis());
        }
    }

    // Create a new schema file
    println!("\nCreating new schema file...");
    let new_schema = schema_dir.join("message.dol");
    tokio::fs::write(
        &new_schema,
        r#"
gen chat.message {
  @crdt(immutable)
  message has id: String

  @crdt(peritext)
  message has content: String
}

exegesis { Chat message }
"#,
    )
    .await?;

    // Wait for creation event
    sleep(Duration::from_secs(2)).await;

    // Load the new file
    loader.load_file(&new_schema).await?;

    // Print all schemas
    {
        let guard = registry.read().await;
        println!("\nAll Gens:");
        for name in guard.gen_names() {
            if let Some(gen) = guard.get_gen(name) {
                println!("  - {}: {} fields", gen.name(), gen.field_count());
            }
        }
    }

    println!("\nHot-reload example completed!");
    println!("In a real application, the watcher would continue running.");

    // Keep watcher alive
    drop(watcher);

    Ok(())
}
