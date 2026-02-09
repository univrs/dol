//! Reactive updates example demonstrating change subscriptions.

use automerge::{transaction::Transactable, ReadDoc, ROOT, ScalarValue};
use tokio::time::{sleep, Duration};
use vudo_state::*;

fn get_f64(doc: &impl ReadDoc, obj: automerge::ObjId, key: &str) -> Result<f64> {
    match doc.get(&obj, key).unwrap() {
        Some((automerge::Value::Scalar(s), _)) => {
            if let ScalarValue::F64(val) = s.as_ref() {
                Ok(*val)
            } else {
                panic!("Expected f64 value")
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

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize state engine
    println!("Initializing VUDO state engine...");
    let engine = StateEngine::new().await?;

    // Create a document
    let doc_id = DocumentId::new("sensors", "temperature");
    let handle = engine.create_document(doc_id.clone()).await?;

    // Subscribe to all changes on the document
    println!("Subscribing to document changes...");
    let filter = SubscriptionFilter::Document(doc_id.clone());
    let mut subscription = engine.subscribe(filter).await;

    // Spawn a task to receive change notifications
    let receiver_task = tokio::spawn(async move {
        println!("\n[Subscriber] Waiting for changes...");
        let mut count = 0;

        while let Some(event) = subscription.recv().await {
            count += 1;
            println!(
                "[Subscriber] Change #{} received for document {} at timestamp {}",
                count, event.document_id, event.timestamp
            );

            // Stop after receiving 5 changes
            if count >= 5 {
                break;
            }
        }

        println!("[Subscriber] Stopped listening.");
    });

    // Simulate sensor updates
    println!("\n[Producer] Simulating sensor readings...");
    for i in 1..=5 {
        sleep(Duration::from_millis(500)).await;

        let temperature = 20.0 + (i as f64 * 0.5);
        println!("[Producer] Update #{}: Temperature = {:.1}°C", i, temperature);

        handle.update_reactive(&engine.observable, move |doc| {
            doc.put(ROOT, "temperature", temperature)?;
            doc.put(ROOT, "reading_count", i as i64)?;
            doc.put(ROOT, "unit", "celsius")?;
            Ok(())
        })?;

        // Flush the batch immediately to ensure delivery
        engine.observable.flush_batch();
    }

    // Wait for the receiver to finish
    receiver_task.await.unwrap();

    // Read the final state
    println!("\nFinal sensor state:");
    handle.read(|doc| {
        let temp = get_f64(doc, ROOT, "temperature")?;
        let count = get_i64(doc, ROOT, "reading_count")?;
        let unit = get_string(doc, ROOT, "unit")?;

        println!("  Temperature: {:.1} {}", temp, unit);
        println!("  Total readings: {}", count);

        Ok(())
    })?;

    println!("\nDone!");
    Ok(())
}
