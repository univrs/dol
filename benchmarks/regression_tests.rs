//! Performance regression tests for CI
//!
//! These tests verify all performance budgets are met and fail if any are exceeded.
//! Run in CI to catch performance regressions early.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use vudo_p2p::P2PNode;
use vudo_state::{DocumentId, StateEngine};

/// Performance budgets (from task specification)
const WASM_SIZE_BUDGET_KB: usize = 100;
const MERGE_LATENCY_BUDGET_MS: u128 = 10;
const SYNC_THROUGHPUT_BUDGET_OPS_PER_SEC: f64 = 1000.0;
const MEMORY_BUDGET_MB: usize = 50;
const STARTUP_TIME_BUDGET_MS: u128 = 500;

#[test]
fn test_crdt_merge_latency_budget() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let state_a = Arc::new(StateEngine::new().await.unwrap());
        let state_b = Arc::new(StateEngine::new().await.unwrap());

        // Create two diverged documents
        let doc_id = DocumentId::new("docs", "test");
        let handle_a = state_a.create_document(doc_id.clone()).await.unwrap();
        let handle_b = state_b.create_document(doc_id.clone()).await.unwrap();

        // Add 5K operations to each (10K total)
        for i in 0..5000 {
            handle_a
                .update(|doc| {
                    doc.put(automerge::ROOT, format!("key_a_{}", i), i as i64)?;
                    Ok(())
                })
                .unwrap();

            handle_b
                .update(|doc| {
                    doc.put(automerge::ROOT, format!("key_b_{}", i), i as i64)?;
                    Ok(())
                })
                .unwrap();
        }

        // Measure merge time
        let start = Instant::now();

        // Get changes from both documents
        let changes_a = handle_a.read(|doc| Ok(doc.get_changes(&[]))).unwrap();
        let changes_b = handle_b.read(|doc| Ok(doc.get_changes(&[]))).unwrap();

        // Apply changes from B to A (merge)
        handle_a
            .update(|doc| {
                doc.apply_changes(changes_b)?;
                Ok(())
            })
            .unwrap();

        let elapsed = start.elapsed();

        // Assert budget
        assert!(
            elapsed.as_millis() < MERGE_LATENCY_BUDGET_MS,
            "CRDT merge latency {}ms exceeds {}ms budget for 10K operations",
            elapsed.as_millis(),
            MERGE_LATENCY_BUDGET_MS
        );

        println!(
            "✅ CRDT merge latency: {}ms (budget: {}ms)",
            elapsed.as_millis(),
            MERGE_LATENCY_BUDGET_MS
        );
    });
}

#[test]
fn test_sync_throughput_budget() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let state_a = Arc::new(StateEngine::new().await.unwrap());
        let state_b = Arc::new(StateEngine::new().await.unwrap());

        let node_a = P2PNode::new(state_a).await.unwrap();
        let node_b = P2PNode::new(state_b).await.unwrap();

        // Create document with 1000 operations
        let doc_id = DocumentId::new("docs", "test");
        let handle_a = node_a.state_engine.create_document(doc_id.clone()).await.unwrap();

        let start = Instant::now();

        for i in 0..1000 {
            handle_a
                .update(|doc| {
                    doc.put(automerge::ROOT, format!("key{}", i), i as i64)?;
                    Ok(())
                })
                .unwrap();
        }

        // Sync to peer
        node_a.sync_document(&doc_id, &node_b).await.unwrap();

        let elapsed = start.elapsed();
        let throughput = 1000.0 / elapsed.as_secs_f64();

        // Assert budget
        assert!(
            throughput > SYNC_THROUGHPUT_BUDGET_OPS_PER_SEC,
            "Sync throughput {:.2} ops/sec below {} ops/sec budget",
            throughput,
            SYNC_THROUGHPUT_BUDGET_OPS_PER_SEC
        );

        println!(
            "✅ Sync throughput: {:.2} ops/sec (budget: {} ops/sec)",
            throughput, SYNC_THROUGHPUT_BUDGET_OPS_PER_SEC
        );
    });
}

#[test]
#[cfg(target_os = "linux")]
fn test_memory_usage_budget() {
    use std::fs;

    fn get_memory_usage() -> usize {
        let status = fs::read_to_string("/proc/self/status").unwrap();
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return parts[1].parse::<usize>().unwrap() * 1024; // Convert KB to bytes
                }
            }
        }
        0
    }

    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let baseline_memory = get_memory_usage();

        let state_engine = Arc::new(StateEngine::new().await.unwrap());

        // Create 100K records
        for i in 0..100_000 {
            let doc_id = DocumentId::new("records", &format!("record{}", i));
            let handle = state_engine.create_document(doc_id).await.unwrap();

            handle
                .update(|doc| {
                    doc.put(automerge::ROOT, "id", i as i64)?;
                    doc.put(automerge::ROOT, "name", format!("Record {}", i))?;
                    doc.put(automerge::ROOT, "value", i as i64 * 2)?;
                    Ok(())
                })
                .unwrap();
        }

        let peak_memory = get_memory_usage();
        let delta_bytes = peak_memory.saturating_sub(baseline_memory);
        let delta_mb = delta_bytes / (1024 * 1024);

        // Assert budget
        assert!(
            delta_mb < MEMORY_BUDGET_MB,
            "Memory usage {}MB exceeds {}MB budget for 100K records",
            delta_mb,
            MEMORY_BUDGET_MB
        );

        println!(
            "✅ Memory usage: {}MB (budget: {}MB)",
            delta_mb, MEMORY_BUDGET_MB
        );
    });
}

#[test]
fn test_startup_time_budget() {
    let start = Instant::now();

    // Initialize runtime
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        // Initialize VUDO state engine
        let state_engine = Arc::new(StateEngine::new().await.unwrap());

        // Initialize P2P network
        let _p2p_node = P2PNode::new(state_engine).await.unwrap();
    });

    let elapsed = start.elapsed();

    // Assert budget
    assert!(
        elapsed.as_millis() < STARTUP_TIME_BUDGET_MS,
        "Startup time {}ms exceeds {}ms budget",
        elapsed.as_millis(),
        STARTUP_TIME_BUDGET_MS
    );

    println!(
        "✅ Startup time: {}ms (budget: {}ms)",
        elapsed.as_millis(),
        STARTUP_TIME_BUDGET_MS
    );
}

#[test]
fn test_state_engine_operations_performance() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let state_engine = Arc::new(StateEngine::new().await.unwrap());

        // Document create should be fast (< 1ms)
        let start = Instant::now();
        let doc_id = DocumentId::new("users", "alice");
        let handle = state_engine.create_document(doc_id.clone()).await.unwrap();
        let create_elapsed = start.elapsed();

        assert!(
            create_elapsed < Duration::from_millis(1),
            "Document create took {:?}, exceeds 1ms",
            create_elapsed
        );

        // Document write should be fast (< 100μs)
        let start = Instant::now();
        handle
            .update(|doc| {
                doc.put(automerge::ROOT, "name", "Alice")?;
                Ok(())
            })
            .unwrap();
        let write_elapsed = start.elapsed();

        assert!(
            write_elapsed < Duration::from_micros(100),
            "Document write took {:?}, exceeds 100μs",
            write_elapsed
        );

        // Document read should be fast (< 50μs)
        let start = Instant::now();
        handle
            .read(|doc| {
                let _: Option<(automerge::Value, automerge::ObjId)> =
                    doc.get(automerge::ROOT, "name")?;
                Ok(())
            })
            .unwrap();
        let read_elapsed = start.elapsed();

        assert!(
            read_elapsed < Duration::from_micros(50),
            "Document read took {:?}, exceeds 50μs",
            read_elapsed
        );

        println!("✅ State engine operations:");
        println!("   - Create: {:?}", create_elapsed);
        println!("   - Write: {:?}", write_elapsed);
        println!("   - Read: {:?}", read_elapsed);
    });
}

#[test]
fn test_batch_operations_performance() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let state_engine = Arc::new(StateEngine::new().await.unwrap());
        let doc_id = DocumentId::new("docs", "test");
        let handle = state_engine.create_document(doc_id).await.unwrap();

        // Batch 1000 operations should be fast (< 100ms)
        let start = Instant::now();

        for i in 0..1000 {
            handle
                .update(|doc| {
                    doc.put(automerge::ROOT, format!("key{}", i), i as i64)?;
                    Ok(())
                })
                .unwrap();
        }

        let elapsed = start.elapsed();

        assert!(
            elapsed < Duration::from_millis(100),
            "Batch 1000 operations took {:?}, exceeds 100ms",
            elapsed
        );

        let throughput = 1000.0 / elapsed.as_secs_f64();

        println!(
            "✅ Batch operations: {} ops in {:?} ({:.2} ops/sec)",
            1000, elapsed, throughput
        );
    });
}

#[test]
fn test_concurrent_operations_performance() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let state_engine = Arc::new(StateEngine::new().await.unwrap());

        // Create 10 documents concurrently
        let start = Instant::now();

        let mut handles = Vec::new();
        for i in 0..10 {
            let state_engine = Arc::clone(&state_engine);
            let handle = tokio::spawn(async move {
                let doc_id = DocumentId::new("docs", &format!("doc{}", i));
                let handle = state_engine.create_document(doc_id).await.unwrap();

                // Add 100 operations
                for j in 0..100 {
                    handle
                        .update(|doc| {
                            doc.put(automerge::ROOT, format!("key{}", j), j as i64)?;
                            Ok(())
                        })
                        .unwrap();
                }
            });
            handles.push(handle);
        }

        futures::future::join_all(handles).await;

        let elapsed = start.elapsed();

        // 10 documents * 100 ops = 1000 ops should be fast with concurrency (< 200ms)
        assert!(
            elapsed < Duration::from_millis(200),
            "Concurrent 1000 operations took {:?}, exceeds 200ms",
            elapsed
        );

        let throughput = 1000.0 / elapsed.as_secs_f64();

        println!(
            "✅ Concurrent operations: 1000 ops in {:?} ({:.2} ops/sec)",
            elapsed, throughput
        );
    });
}

#[test]
fn test_all_budgets() {
    println!("\n=== Performance Budget Verification ===\n");

    test_crdt_merge_latency_budget();
    test_sync_throughput_budget();

    #[cfg(target_os = "linux")]
    test_memory_usage_budget();

    test_startup_time_budget();
    test_state_engine_operations_performance();
    test_batch_operations_performance();
    test_concurrent_operations_performance();

    println!("\n=== All Performance Budgets Met ✅ ===\n");
}
