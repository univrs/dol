//! Basic usage example for eg-walker DOL integration

use eg_walker_dol::{EgWalkerText, TextCrdt};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Eg-walker Basic Usage ===\n");

    // Create a new document
    let mut doc = EgWalkerText::new("user1".to_string());
    println!("Created document for user1");

    // Insert some text
    doc.insert(0, "Hello")?;
    println!("After insert 'Hello': {}", doc.get_text());

    // Append more text
    doc.insert(5, " World")?;
    println!("After insert ' World': {}", doc.get_text());

    // Insert in middle
    doc.insert(5, " Beautiful")?;
    println!("After insert ' Beautiful': {}", doc.get_text());

    // Delete some text
    doc.delete(6, 10)?; // Delete "Beautiful "
    println!("After delete: {}", doc.get_text());

    // Get document info
    println!("\nDocument info:");
    println!("  Length: {} characters", doc.len());
    println!("  Memory: {} bytes", doc.memory_size());
    println!("  Operations: {}", doc.operation_count());

    // Serialize document
    let bytes = doc.to_bytes()?;
    println!("\nSerialized size: {} bytes", bytes.len());

    // Deserialize
    let restored = EgWalkerText::from_bytes(&bytes)?;
    println!("Restored text: {}", restored.get_text());

    Ok(())
}
