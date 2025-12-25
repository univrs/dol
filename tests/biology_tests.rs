//! Tests for DOL biology module

use metadol::ast::Declaration;
use metadol::error::ParseError;
use metadol::parser::Parser;

/// Helper to parse a string and return the declaration
fn parse(input: &str) -> Result<Declaration, ParseError> {
    let mut parser = Parser::new(input);
    parser.parse()
}

#[test]
fn test_parse_nutrient_gene() {
    let source = include_str!("../examples/stdlib/biology/types.dol");
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse types.dol: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_hyphal_trait() {
    let source = include_str!("../examples/stdlib/biology/hyphal.dol");
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse hyphal.dol: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_transport_trait() {
    let source = include_str!("../examples/stdlib/biology/transport.dol");
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse transport.dol: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_ecosystem_system() {
    let source = include_str!("../examples/stdlib/biology/ecosystem.dol");
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse ecosystem.dol: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_evolution() {
    let source = include_str!("../examples/stdlib/biology/evolution.dol");
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse evolution.dol: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_mycelium_network() {
    let source = include_str!("../examples/stdlib/biology/mycelium.dol");
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse mycelium.dol: {:?}",
        result.err()
    );
}

#[test]
fn test_nutrient_constraint() {
    let source = r#"
        gene TestNutrient {
            has carbon: Float64 = 70.0
            has nitrogen: Float64 = 10.0
            has phosphorus: Float64 = 1.0
            has water: Float64 = 20.0

            constraint stoichiometry {
                this.carbon / this.nitrogen >= 6.0 &&
                this.carbon / this.nitrogen <= 10.0
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok());
}

#[test]
fn test_ecosystem_constraints() {
    let source = r#"
        system TestEcosystem {
            state population: UInt64
            state carrying_capacity: UInt64

            constraint population_limit {
                this.population <= this.carrying_capacity
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok());
}
