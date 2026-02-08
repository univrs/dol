//! Offline queue example demonstrating operation persistence and replay.

use vudo_state::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Offline Operation Queue Demo ===\n");

    // Initialize state engine
    println!("1. Creating state engine and generating offline operations...");
    let engine = StateEngine::new().await?;

    // Simulate offline operations
    let ops = vec![
        (
            DocumentId::new("users", "alice"),
            "create-alice",
            OperationType::Create {
                document_id: DocumentId::new("users", "alice"),
            },
        ),
        (
            DocumentId::new("users", "bob"),
            "create-bob",
            OperationType::Create {
                document_id: DocumentId::new("users", "bob"),
            },
        ),
        (
            DocumentId::new("posts", "1"),
            "create-post-1",
            OperationType::Create {
                document_id: DocumentId::new("posts", "1"),
            },
        ),
    ];

    for (doc_id, key, op_type) in ops {
        let op = Operation::new_with_key(op_type, key.to_string());
        engine.queue.enqueue(op)?;
        println!("  Enqueued: {} (key: {})", doc_id, key);
    }

    println!("\n2. Queue status:");
    println!("  Operations in queue: {}", engine.queue.len());

    // Serialize the queue (simulate saving to disk)
    println!("\n3. Serializing queue to bytes (simulating offline persistence)...");
    let queue_bytes = engine.queue.serialize()?;
    println!("  Serialized size: {} bytes", queue_bytes.len());

    // Clear the current queue
    println!("\n4. Clearing current queue (simulating app restart)...");
    engine.queue.clear();
    println!("  Queue length after clear: {}", engine.queue.len());

    // Deserialize the queue (simulate loading from disk)
    println!("\n5. Deserializing queue from bytes (simulating restore)...");
    engine.queue.deserialize(&queue_bytes)?;
    println!("  Queue length after restore: {}", engine.queue.len());

    // Replay operations
    println!("\n6. Replaying operations from queue:");
    let mut success_count = 0;
    let mut error_count = 0;

    while let Some(op) = engine.queue.dequeue() {
        match &op.op_type {
            OperationType::Create { document_id } => {
                match engine.store.create(document_id.clone()) {
                    Ok(_) => {
                        println!("  ✓ Created document: {}", document_id);
                        success_count += 1;
                    }
                    Err(e) => {
                        println!("  ✗ Failed to create {}: {}", document_id, e);
                        error_count += 1;
                    }
                }
            }
            OperationType::Update { document_id, .. } => {
                println!("  ✓ Would apply update to: {}", document_id);
                success_count += 1;
            }
            OperationType::Delete { document_id } => {
                println!("  ✓ Would delete: {}", document_id);
                success_count += 1;
            }
        }
    }

    println!("\n7. Replay summary:");
    println!("  Successful operations: {}", success_count);
    println!("  Failed operations: {}", error_count);
    println!("  Queue length after replay: {}", engine.queue.len());

    // Demonstrate idempotency
    println!("\n8. Testing idempotency (duplicate operations)...");
    let op1 = Operation::new_with_key(
        OperationType::Create {
            document_id: DocumentId::new("users", "charlie"),
        },
        "create-charlie".to_string(),
    );

    let op2 = Operation::new_with_key(
        OperationType::Create {
            document_id: DocumentId::new("users", "charlie"),
        },
        "create-charlie".to_string(), // Same key
    );

    let id1 = engine.queue.enqueue(op1)?;
    let id2 = engine.queue.enqueue(op2)?;

    println!("  First enqueue ID: {:?}", id1);
    println!("  Second enqueue ID: {:?}", id2);
    println!("  IDs match (deduplicated): {}", id1 == id2);
    println!("  Queue length: {}", engine.queue.len());

    // Filter operations by document
    println!("\n9. Filtering operations by document:");
    let alice_ops = engine
        .queue
        .filter_by_document(&DocumentId::new("users", "charlie"));
    println!("  Operations for 'users/charlie': {}", alice_ops.len());

    println!("\n=== Demo Complete ===");
    Ok(())
}
