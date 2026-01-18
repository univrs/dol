use metadol::repl::{EvalResult, SpiritRepl};

fn main() {
    let mut repl = SpiritRepl::new();

    println!("=== DOL REPL Expression Evaluation Demo ===\n");

    // Test simple integer literal
    println!(">>> 42");
    match repl.eval("42") {
        Ok(EvalResult::Expression { value, .. }) => println!("=> {}\n", value),
        other => println!("Error: {:?}\n", other),
    }

    // Test float literal
    println!(">>> 3.14159");
    match repl.eval("3.14159") {
        Ok(EvalResult::Expression { value, .. }) => println!("=> {}\n", value),
        other => println!("Error: {:?}\n", other),
    }

    // Test arithmetic
    println!(">>> 10 + 20 * 2");
    match repl.eval("10 + 20 * 2") {
        Ok(EvalResult::Expression { value, .. }) => println!("=> {}\n", value),
        other => println!("Error: {:?}\n", other),
    }

    // Test subtraction
    println!(">>> 100 - 37");
    match repl.eval("100 - 37") {
        Ok(EvalResult::Expression { value, .. }) => println!("=> {}\n", value),
        other => println!("Error: {:?}\n", other),
    }

    // Define a function
    println!(">>> pub fun square(x: i64) -> i64 {{ x * x }}");
    match repl.eval("pub fun square(x: i64) -> i64 { x * x }") {
        Ok(EvalResult::Defined { name, kind, .. }) => println!("Defined {} '{}'\n", kind, name),
        other => println!("Error: {:?}\n", other),
    }

    // Call the function
    println!(">>> square(7)");
    match repl.eval("square(7)") {
        Ok(EvalResult::Expression { value, .. }) => println!("=> {}\n", value),
        other => println!("Error: {:?}\n", other),
    }

    // Define a gene
    println!(">>> gen Point {{ has x: i64\\n has y: i64 }}");
    match repl.eval("gen Point { has x: i64\n has y: i64 }") {
        Ok(EvalResult::Defined { name, kind, .. }) => println!("Defined {} '{}'\n", kind, name),
        other => println!("Error: {:?}\n", other),
    }

    // Define a function using the gene
    println!(">>> pub fun getX() -> i64 {{ let p = Point {{ x: 42, y: 100 }}\\n p.x }}");
    match repl.eval("pub fun getX() -> i64 { let p = Point { x: 42, y: 100 }\n p.x }") {
        Ok(EvalResult::Defined { name, kind, .. }) => println!("Defined {} '{}'\n", kind, name),
        other => println!("Error: {:?}\n", other),
    }

    // Call getX
    println!(">>> getX()");
    match repl.eval("getX()") {
        Ok(EvalResult::Expression { value, .. }) => println!("=> {}\n", value),
        other => println!("Error: {:?}\n", other),
    }

    // Float arithmetic
    println!(">>> 1.5 + 2.5");
    match repl.eval("1.5 + 2.5") {
        Ok(EvalResult::Expression { value, .. }) => println!("=> {}\n", value),
        other => println!("Error: {:?}\n", other),
    }

    // Define and call a float function
    println!(">>> pub fun area(r: f64) -> f64 {{ 3.14159 * r * r }}");
    match repl.eval("pub fun area(r: f64) -> f64 { 3.14159 * r * r }") {
        Ok(EvalResult::Defined { name, kind, .. }) => println!("Defined {} '{}'\n", kind, name),
        other => println!("Error: {:?}\n", other),
    }

    println!(">>> area(10.0)");
    match repl.eval("area(10.0)") {
        Ok(EvalResult::Expression { value, .. }) => println!("=> {}\n", value),
        other => println!("Error: {:?}\n", other),
    }

    println!("=== Demo Complete ===");
}
