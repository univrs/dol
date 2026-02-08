//! Interactive comparison between eg-walker and Automerge

use eg_walker_dol::{
    EgWalkerText, AutomergeText, TextCrdt,
    correctness, benchmarks,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Eg-walker vs Automerge Comparison ===\n");

    // Run correctness tests
    println!("CORRECTNESS TESTS\n");
    correctness::run_all_tests::<EgWalkerText>("Eg-walker")?;
    println!();
    correctness::run_all_tests::<AutomergeText>("Automerge")?;

    // Run performance benchmarks
    println!("\n\nPERFORMANCE BENCHMARKS\n");
    benchmarks::run_comparison_suite::<EgWalkerText, AutomergeText>(
        "Eg-walker",
        "Automerge",
    )?;

    // Memory comparison
    println!("\n\nMEMORY FOOTPRINT COMPARISON\n");
    memory_comparison()?;

    Ok(())
}

fn memory_comparison() -> Result<(), Box<dyn std::error::Error>> {
    let sizes = [100, 1000, 5000, 10000];

    println!("Document size | Eg-walker (bytes) | Automerge (bytes) | Ratio");
    println!("-------------|-------------------|-------------------|-------");

    for size in sizes {
        // Eg-walker
        let mut eg = EgWalkerText::new("bench".to_string());
        for i in 0..size {
            eg.insert(eg.len(), "x")?;
        }
        let eg_mem = eg.memory_size();

        // Automerge
        let mut am = AutomergeText::new("bench".to_string());
        for i in 0..size {
            am.insert(am.len(), "x")?;
        }
        let am_mem = am.memory_size();

        let ratio = am_mem as f64 / eg_mem as f64;
        println!("{:12} | {:17} | {:17} | {:.2}x", size, eg_mem, am_mem, ratio);
    }

    println!("\nNote: Ratio > 1.0 means eg-walker uses less memory");

    Ok(())
}
