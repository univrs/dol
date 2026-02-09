//! Simple state management example demonstrating basic CRUD operations.

use automerge::{transaction::Transactable, ReadDoc, ROOT, ScalarValue};
use vudo_state::*;

fn get_string(doc: &impl ReadDoc, obj: automerge::ObjId, key: &str) -> Result<String> {
    match doc.get(&obj, key).unwrap() {
        Some((automerge::Value::Scalar(s), _)) => {
            if let ScalarValue::Str(smol_str) = s.as_ref() {
                Ok(smol_str.to_string())
            } else {
                panic!("Expected string value")
            }
        }
        _ => panic!("Value not found"),
    }
}

fn get_i64(doc: &impl ReadDoc, obj: automerge::ObjId, key: &str) -> Result<i64> {
    match doc.get(&obj, key).unwrap() {
        Some((automerge::Value::Scalar(s), _)) => {
            if let ScalarValue::Int(val) = s.as_ref() {
                Ok(*val)
            } else {
                panic!("Expected int value")
            }
        }
        _ => panic!("Value not found"),
    }
}

fn get_bool(doc: &impl ReadDoc, obj: automerge::ObjId, key: &str) -> Result<bool> {
    match doc.get(&obj, key).unwrap() {
        Some((automerge::Value::Scalar(s), _)) => {
            if let ScalarValue::Boolean(val) = s.as_ref() {
                Ok(*val)
            } else {
                panic!("Expected bool value")
            }
        }
        _ => panic!("Value not found"),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize state engine
    println!("Initializing VUDO state engine...");
    let engine = StateEngine::new().await?;

    // Create a document
    println!("\nCreating document 'users/alice'...");
    let doc_id = DocumentId::new("users", "alice");
    let handle = engine.create_document(doc_id.clone()).await?;

    // Update the document
    println!("Updating document with user data...");
    handle.update(|doc| {
        doc.put(ROOT, "name", "Alice")?;
        doc.put(ROOT, "email", "alice@example.com")?;
        doc.put(ROOT, "age", 30i64)?;
        doc.put(ROOT, "active", true)?;
        Ok(())
    })?;

    // Read from the document
    println!("\nReading document data...");
    handle.read(|doc| {
        let name = get_string(doc, ROOT, "name")?;
        let email = get_string(doc, ROOT, "email")?;
        let age = get_i64(doc, ROOT, "age")?;
        let active = get_bool(doc, ROOT, "active")?;

        println!("  Name: {}", name);
        println!("  Email: {}", email);
        println!("  Age: {}", age);
        println!("  Active: {}", active);

        Ok(())
    })?;

    // Get document metadata
    println!("\nDocument metadata:");
    let metadata = handle.metadata();
    println!("  ID: {}", metadata.id);
    println!("  Created at: {}", metadata.created_at);
    println!("  Last modified: {}", metadata.last_modified);
    println!("  Size: {} bytes", metadata.size);
    println!("  Version: {}", metadata.version);

    // Create another document
    println!("\nCreating document 'users/bob'...");
    let bob_id = DocumentId::new("users", "bob");
    let bob_handle = engine.create_document(bob_id.clone()).await?;

    bob_handle.update(|doc| {
        doc.put(ROOT, "name", "Bob")?;
        doc.put(ROOT, "email", "bob@example.com")?;
        doc.put(ROOT, "age", 25i64)?;
        Ok(())
    })?;

    // List all documents in the 'users' namespace
    println!("\nListing all users:");
    let users = engine.store.list_namespace("users");
    for user_id in users {
        let user_handle = engine.get_document(&user_id).await?;
        user_handle.read(|doc| {
            let name = get_string(doc, ROOT, "name")?;
            println!("  - {} ({})", name, user_id);
            Ok(())
        })?;
    }

    // Show engine statistics
    println!("\nEngine statistics:");
    let stats = engine.stats();
    println!("  Total documents: {}", stats.document_count);
    println!("  Total size: {} bytes", stats.total_document_size);
    println!("  Queue length: {}", stats.queue_length);

    // Delete a document
    println!("\nDeleting 'users/bob'...");
    engine.delete_document(&bob_id).await?;

    println!("\nFinal document count: {}", engine.stats().document_count);

    println!("\nDone!");
    Ok(())
}
