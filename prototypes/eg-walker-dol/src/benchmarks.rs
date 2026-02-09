//! Benchmark utilities for comparing text CRDT implementations

use crate::{TextCrdt, Result};
use std::time::{Duration, Instant};

/// Benchmark result for a single operation
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub duration: Duration,
    pub operations: usize,
    pub memory_bytes: usize,
}

impl BenchmarkResult {
    pub fn ops_per_sec(&self) -> f64 {
        self.operations as f64 / self.duration.as_secs_f64()
    }

    pub fn bytes_per_op(&self) -> f64 {
        self.memory_bytes as f64 / self.operations as f64
    }
}

/// Benchmark insert operations
pub fn benchmark_inserts<T: TextCrdt>(
    num_operations: usize,
    text_per_op: &str,
) -> Result<BenchmarkResult> {
    let mut doc = T::new("bench".to_string());

    let start = Instant::now();

    for _ in 0..num_operations {
        let pos = doc.len();
        doc.insert(pos, text_per_op)?;
    }

    let duration = start.elapsed();
    let memory_bytes = doc.memory_size();

    Ok(BenchmarkResult {
        name: "insert".to_string(),
        duration,
        operations: num_operations,
        memory_bytes,
    })
}

/// Benchmark delete operations
pub fn benchmark_deletes<T: TextCrdt>(
    num_operations: usize,
) -> Result<BenchmarkResult> {
    // First, create a document with text
    let mut doc = T::new("bench".to_string());
    for _ in 0..num_operations {
        doc.insert(doc.len(), "x")?;
    }

    let start = Instant::now();

    for _ in 0..num_operations {
        if doc.len() > 0 {
            doc.delete(0, 1)?;
        }
    }

    let duration = start.elapsed();
    let memory_bytes = doc.memory_size();

    Ok(BenchmarkResult {
        name: "delete".to_string(),
        duration,
        operations: num_operations,
        memory_bytes,
    })
}

/// Benchmark merge operations
pub fn benchmark_merges<T: TextCrdt>(
    num_replicas: usize,
    ops_per_replica: usize,
) -> Result<BenchmarkResult> {
    // Create replicas with different operations
    let mut replicas: Vec<T> = (0..num_replicas)
        .map(|i| {
            let mut doc = T::new(format!("replica-{}", i));
            for j in 0..ops_per_replica {
                let _ = doc.insert(doc.len(), &format!("R{}-{} ", i, j));
            }
            doc
        })
        .collect();

    let start = Instant::now();

    // Merge all replicas into the first one
    let mut result = replicas[0].clone();
    for i in 1..num_replicas {
        result.merge(&replicas[i])?;
    }
    replicas[0] = result;

    let duration = start.elapsed();
    let memory_bytes = replicas[0].memory_size();
    let total_ops = num_replicas * ops_per_replica;

    Ok(BenchmarkResult {
        name: "merge".to_string(),
        duration,
        operations: total_ops,
        memory_bytes,
    })
}

/// Benchmark serialization
pub fn benchmark_serialization<T: TextCrdt>(
    num_operations: usize,
) -> Result<BenchmarkResult> {
    // Create a document with operations
    let mut doc = T::new("bench".to_string());
    for i in 0..num_operations {
        doc.insert(doc.len(), &format!("Op {} ", i))?;
    }

    let start = Instant::now();
    let bytes = doc.to_bytes()?;
    let duration = start.elapsed();

    Ok(BenchmarkResult {
        name: "serialize".to_string(),
        duration,
        operations: 1,
        memory_bytes: bytes.len(),
    })
}

/// Benchmark deserialization
pub fn benchmark_deserialization<T: TextCrdt>(
    num_operations: usize,
) -> Result<BenchmarkResult> {
    // Create a document with operations
    let mut doc = T::new("bench".to_string());
    for i in 0..num_operations {
        doc.insert(doc.len(), &format!("Op {} ", i))?;
    }
    let bytes = doc.to_bytes()?;

    let start = Instant::now();
    let _restored = T::from_bytes(&bytes)?;
    let duration = start.elapsed();

    Ok(BenchmarkResult {
        name: "deserialize".to_string(),
        duration,
        operations: 1,
        memory_bytes: bytes.len(),
    })
}

/// Run all benchmarks and compare
pub fn run_comparison_suite<T1: TextCrdt, T2: TextCrdt>(
    name1: &str,
    name2: &str,
) -> Result<()> {
    println!("\n=== Text CRDT Benchmark Comparison ===\n");

    // Insert benchmark
    println!("Insert Operations (1000 ops, 5 chars each):");
    let insert1 = benchmark_inserts::<T1>(1000, "Hello")?;
    let insert2 = benchmark_inserts::<T2>(1000, "Hello")?;
    print_comparison(name1, name2, &insert1, &insert2);

    // Delete benchmark
    println!("\nDelete Operations (1000 ops):");
    let delete1 = benchmark_deletes::<T1>(1000)?;
    let delete2 = benchmark_deletes::<T2>(1000)?;
    print_comparison(name1, name2, &delete1, &delete2);

    // Merge benchmark
    println!("\nMerge Operations (10 replicas, 100 ops each):");
    let merge1 = benchmark_merges::<T1>(10, 100)?;
    let merge2 = benchmark_merges::<T2>(10, 100)?;
    print_comparison(name1, name2, &merge1, &merge2);

    // Serialization benchmark
    println!("\nSerialization (1000 ops):");
    let ser1 = benchmark_serialization::<T1>(1000)?;
    let ser2 = benchmark_serialization::<T2>(1000)?;
    print_comparison(name1, name2, &ser1, &ser2);

    // Deserialization benchmark
    println!("\nDeserialization (1000 ops):");
    let deser1 = benchmark_deserialization::<T1>(1000)?;
    let deser2 = benchmark_deserialization::<T2>(1000)?;
    print_comparison(name1, name2, &deser1, &deser2);

    Ok(())
}

fn print_comparison(name1: &str, name2: &str, result1: &BenchmarkResult, result2: &BenchmarkResult) {
    println!("  {}:", name1);
    println!("    Time: {:?}", result1.duration);
    println!("    Memory: {} bytes", result1.memory_bytes);
    println!("    Ops/sec: {:.0}", result1.ops_per_sec());
    if result1.operations > 1 {
        println!("    Bytes/op: {:.2}", result1.bytes_per_op());
    }

    println!("  {}:", name2);
    println!("    Time: {:?}", result2.duration);
    println!("    Memory: {} bytes", result2.memory_bytes);
    println!("    Ops/sec: {:.0}", result2.ops_per_sec());
    if result2.operations > 1 {
        println!("    Bytes/op: {:.2}", result2.bytes_per_op());
    }

    // Calculate speedup
    let time_ratio = result2.duration.as_secs_f64() / result1.duration.as_secs_f64();
    let mem_ratio = result2.memory_bytes as f64 / result1.memory_bytes as f64;

    println!("  Comparison:");
    if time_ratio > 1.0 {
        println!("    {} is {:.2}x faster", name1, time_ratio);
    } else {
        println!("    {} is {:.2}x faster", name2, 1.0 / time_ratio);
    }

    if mem_ratio > 1.0 {
        println!("    {} uses {:.2}x less memory", name1, mem_ratio);
    } else {
        println!("    {} uses {:.2}x less memory", name2, 1.0 / mem_ratio);
    }
}
