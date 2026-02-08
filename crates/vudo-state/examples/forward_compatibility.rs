//! Forward Compatibility Example
//!
//! Demonstrates how old peers can read documents with new schema versions
//! by ignoring unknown fields.

use automerge::transaction::Transactable;
use automerge::{Automerge, ROOT};
use serde::Deserialize;
use std::collections::HashSet;
use vudo_state::schema_evolution::ForwardCompatibleReader;

/// V1 schema: Only knows about username and age
#[derive(Debug, Deserialize)]
struct UserV1 {
    username: String,
    age: i64,
}

/// V2 schema: Added email field
#[derive(Debug, Deserialize)]
struct UserV2 {
    username: String,
    age: i64,
    email: String,
}

/// V3 schema: Added profile_photo and bio fields
#[derive(Debug, Deserialize)]
struct UserV3 {
    username: String,
    age: i64,
    email: String,
    profile_photo: String,
    bio: String,
}

fn main() -> vudo_state::Result<()> {
    println!("📚 Forward Compatibility Example\n");
    println!("Scenario: Old peer (v1) reads document from new peer (v3)\n");

    // Create a v3 document (with all fields)
    let mut doc = Automerge::new();
    {
        let mut tx = doc.transaction();
        tx.put(&ROOT, "username", "dave")?;
        tx.put(&ROOT, "age", 35i64)?;
        tx.put(&ROOT, "email", "dave@example.com")?;
        tx.put(&ROOT, "profile_photo", "https://example.com/photo.jpg")?;
        tx.put(&ROOT, "bio", "Rust enthusiast")?;
        tx.commit();
    }

    println!("✓ Created v3 document with all fields");
    println!("\n📄 V3 Document (new peer):");
    println!("  username: dave");
    println!("  age: 35");
    println!("  email: dave@example.com");
    println!("  profile_photo: https://example.com/photo.jpg");
    println!("  bio: Rust enthusiast");

    // V1 peer tries to read (only knows about username and age)
    println!("\n👤 V1 Peer: Reading document (forward-compatible)");

    let mut v1_fields = HashSet::new();
    v1_fields.insert("username".to_string());
    v1_fields.insert("age".to_string());

    let v1_reader = ForwardCompatibleReader::new(v1_fields);
    let user_v1: UserV1 = v1_reader.read_document(&doc)?;

    println!("\n📄 Document as read by V1 peer:");
    println!("  username: {}", user_v1.username);
    println!("  age: {}", user_v1.age);
    println!("  (email, profile_photo, bio ignored - unknown fields)");

    // V2 peer tries to read (knows about username, age, email)
    println!("\n👤 V2 Peer: Reading document (forward-compatible)");

    let mut v2_fields = HashSet::new();
    v2_fields.insert("username".to_string());
    v2_fields.insert("age".to_string());
    v2_fields.insert("email".to_string());

    let v2_reader = ForwardCompatibleReader::new(v2_fields);
    let user_v2: UserV2 = v2_reader.read_document(&doc)?;

    println!("\n📄 Document as read by V2 peer:");
    println!("  username: {}", user_v2.username);
    println!("  age: {}", user_v2.age);
    println!("  email: {}", user_v2.email);
    println!("  (profile_photo, bio ignored - unknown fields)");

    // V3 peer reads all fields
    println!("\n👤 V3 Peer: Reading document (all fields)");

    let mut v3_fields = HashSet::new();
    v3_fields.insert("username".to_string());
    v3_fields.insert("age".to_string());
    v3_fields.insert("email".to_string());
    v3_fields.insert("profile_photo".to_string());
    v3_fields.insert("bio".to_string());

    let v3_reader = ForwardCompatibleReader::new(v3_fields);
    let user_v3: UserV3 = v3_reader.read_document(&doc)?;

    println!("\n📄 Document as read by V3 peer:");
    println!("  username: {}", user_v3.username);
    println!("  age: {}", user_v3.age);
    println!("  email: {}", user_v3.email);
    println!("  profile_photo: {}", user_v3.profile_photo);
    println!("  bio: {}", user_v3.bio);

    println!("\n✅ Forward compatibility verified!");
    println!("📝 Key insights:");
    println!("   • V1 peer can read V3 documents (ignores unknown fields)");
    println!("   • V2 peer can read V3 documents (ignores newer fields)");
    println!("   • No errors - graceful degradation");
    println!("   • Allows gradual rollout of new schema versions");

    Ok(())
}
