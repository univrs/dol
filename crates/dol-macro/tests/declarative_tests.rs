//! Integration tests for declarative macros.

use dol_macro::prelude::*;
use metadol::ast::{BinaryOp, Expr, Literal, Span};

#[test]
fn test_simple_constant_macro() {
    // Define: macro const42!() => 42
    let pattern = MacroPattern::Empty;
    let template = MacroTemplate::expr(Expr::Literal(Literal::Int(42)));
    let rule = MacroRule::new(pattern, template);
    let macro_def = DeclarativeMacro::new("const42", vec![rule]);

    let mut hygiene = HygieneContext::new();
    let result = macro_def.expand(&[], &mut hygiene);

    assert!(result.is_ok());
    let exprs = result.unwrap();
    assert_eq!(exprs.len(), 1);

    if let Expr::Literal(Literal::Int(n)) = exprs[0] {
        assert_eq!(n, 42);
    } else {
        panic!("Expected integer literal");
    }
}

#[test]
fn test_identity_macro() {
    // Define: macro identity!($x:expr) => $x
    use dol_macro::pattern::FragmentSpecifier;

    let pattern = MacroPattern::metavar("x", FragmentSpecifier::Expr);
    let template = MacroTemplate::metavar("x");
    let rule = MacroRule::new(pattern, template);
    let macro_def = DeclarativeMacro::new("identity", vec![rule]);

    let input = vec![Expr::Literal(Literal::Int(99))];
    let mut hygiene = HygieneContext::new();
    let result = macro_def.expand(&input, &mut hygiene);

    assert!(result.is_ok());
    let exprs = result.unwrap();
    assert_eq!(exprs.len(), 1);

    if let Expr::Literal(Literal::Int(n)) = exprs[0] {
        assert_eq!(n, 99);
    } else {
        panic!("Expected integer literal");
    }
}

#[test]
fn test_two_argument_macro() {
    // Define: macro pair!($x:expr, $y:expr) => ($x, $y)
    use dol_macro::pattern::FragmentSpecifier;

    let pattern = MacroPattern::sequence(vec![
        MacroPattern::metavar("x", FragmentSpecifier::Expr),
        MacroPattern::metavar("y", FragmentSpecifier::Expr),
    ]);

    let template = MacroTemplate::sequence(vec![
        MacroTemplate::metavar("x"),
        MacroTemplate::metavar("y"),
    ]);

    let rule = MacroRule::new(pattern, template);
    let macro_def = DeclarativeMacro::new("pair", vec![rule]);

    let input = vec![
        Expr::Literal(Literal::Int(1)),
        Expr::Literal(Literal::Int(2)),
    ];

    let mut hygiene = HygieneContext::new();
    let result = macro_def.expand(&input, &mut hygiene);

    assert!(result.is_ok());
    let exprs = result.unwrap();
    assert_eq!(exprs.len(), 2);
}

#[test]
fn test_macro_registry() {
    let mut registry = MacroRegistry::new();

    // Register a simple macro
    let pattern = MacroPattern::Empty;
    let template = MacroTemplate::expr(Expr::Literal(Literal::Int(42)));
    let rule = MacroRule::new(pattern, template);
    let macro_def = DeclarativeMacro::new("test", vec![rule]);

    registry.register_declarative("test", macro_def);

    assert!(registry.has_declarative("test"));
    assert_eq!(registry.len(), 1);

    let retrieved = registry.get_declarative("test");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name(), "test");
}

#[test]
fn test_macro_expander() {
    let mut expander = MacroExpander::new();

    // Register a macro
    let pattern = MacroPattern::Empty;
    let template = MacroTemplate::expr(Expr::Literal(Literal::String("hello".to_string())));
    let rule = MacroRule::new(pattern, template);
    let macro_def = DeclarativeMacro::new("hello", vec![rule]);

    expander.registry_mut().register_declarative("hello", macro_def);

    // Expand the macro
    let invocation = dol_macro::expand::MacroInvocation::simple("hello", Span::default());
    let result = expander.expand(&invocation);

    assert!(result.is_ok());
    let exprs = result.unwrap();
    assert_eq!(exprs.len(), 1);

    if let Expr::Literal(Literal::String(s)) = &exprs[0] {
        assert_eq!(s, "hello");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn test_pattern_matching_failure() {
    use dol_macro::pattern::FragmentSpecifier;

    // Define a macro that expects an identifier
    let pattern = MacroPattern::metavar("x", FragmentSpecifier::Ident);
    let template = MacroTemplate::metavar("x");
    let rule = MacroRule::new(pattern, template);
    let macro_def = DeclarativeMacro::new("ident_only", vec![rule]);

    // Try to pass a literal instead
    let input = vec![Expr::Literal(Literal::Int(42))];
    let mut hygiene = HygieneContext::new();
    let result = macro_def.expand(&input, &mut hygiene);

    // Should fail
    assert!(result.is_err());
}

#[test]
fn test_multiple_rules() {
    use dol_macro::pattern::FragmentSpecifier;

    // Define a macro with multiple rules:
    // macro add!() => 0
    // macro add!($x:expr) => $x
    let rule1 = MacroRule::new(
        MacroPattern::Empty,
        MacroTemplate::expr(Expr::Literal(Literal::Int(0))),
    );

    let rule2 = MacroRule::new(
        MacroPattern::metavar("x", FragmentSpecifier::Expr),
        MacroTemplate::metavar("x"),
    );

    let macro_def = DeclarativeMacro::new("add", vec![rule1, rule2]);

    // Test first rule
    let mut hygiene = HygieneContext::new();
    let result = macro_def.expand(&[], &mut hygiene);
    assert!(result.is_ok());
    let exprs = result.unwrap();
    if let Expr::Literal(Literal::Int(n)) = exprs[0] {
        assert_eq!(n, 0);
    }

    // Test second rule
    let input = vec![Expr::Literal(Literal::Int(5))];
    let mut hygiene = HygieneContext::new();
    let result = macro_def.expand(&input, &mut hygiene);
    assert!(result.is_ok());
    let exprs = result.unwrap();
    if let Expr::Literal(Literal::Int(n)) = exprs[0] {
        assert_eq!(n, 5);
    }
}

#[test]
fn test_hygiene() {
    use dol_macro::pattern::FragmentSpecifier;

    // Test that identifiers are made hygienic
    let pattern = MacroPattern::metavar("x", FragmentSpecifier::Ident);
    let template = MacroTemplate::metavar("x");
    let rule = MacroRule::new(pattern, template);
    let macro_def = DeclarativeMacro::new("hygienic", vec![rule]);

    let input = vec![Expr::Ident("temp".to_string())];
    let mut hygiene = HygieneContext::new();
    let result = macro_def.expand(&input, &mut hygiene);

    assert!(result.is_ok());
    let exprs = result.unwrap();
    assert_eq!(exprs.len(), 1);

    // The identifier should be hygienic (modified from original)
    if let Expr::Ident(name) = &exprs[0] {
        assert_ne!(name, "temp");
        assert!(name.contains("temp"));
    } else {
        panic!("Expected identifier");
    }
}

#[test]
fn test_stdlib_macros() {
    let mut registry = MacroRegistry::new();
    dol_macro::stdlib::register_stdlib_macros(&mut registry);

    // Check that all stdlib macros are registered
    assert!(registry.has_declarative("const"));
    assert!(registry.has_declarative("add"));
    assert!(registry.has_declarative("identity"));
    assert!(registry.has_declarative("vec"));
    assert!(registry.has_declarative("assert_eq"));
    assert!(registry.has_declarative("min"));
    assert!(registry.has_declarative("max"));

    // Test that they're all exported
    let exported = registry.export();
    assert_eq!(exported.len(), registry.len());
}

#[test]
fn test_expander_recursion_limit() {
    let mut expander = MacroExpander::new();
    expander.set_max_depth(5);

    // Manually set depth to test limit
    let invocation = dol_macro::expand::MacroInvocation::simple("test", Span::default());

    // This should fail if we're at the limit
    for _ in 0..5 {
        let _ = expander.expand(&invocation);
    }

    // Depth should be back to 0 after each expansion
    assert_eq!(expander.depth(), 0);
}

#[test]
fn test_macro_export() {
    let mut registry = MacroRegistry::new();

    let rule = MacroRule::new(MacroPattern::Empty, MacroTemplate::Empty);
    let exported = DeclarativeMacro::exported("exported", vec![rule.clone()]);
    let private = DeclarativeMacro::new("private", vec![rule]);

    registry.register_declarative("exported", exported);
    registry.register_declarative("private", private);

    let exported_registry = registry.export();
    assert_eq!(exported_registry.len(), 1);
    assert!(exported_registry.has_declarative("exported"));
    assert!(!exported_registry.has_declarative("private"));
}

#[test]
fn test_registry_merge() {
    let mut registry1 = MacroRegistry::new();
    let mut registry2 = MacroRegistry::new();

    let rule = MacroRule::new(MacroPattern::Empty, MacroTemplate::Empty);
    let macro1 = DeclarativeMacro::new("macro1", vec![rule.clone()]);
    let macro2 = DeclarativeMacro::new("macro2", vec![rule]);

    registry1.register_declarative("macro1", macro1);
    registry2.register_declarative("macro2", macro2);

    registry1.merge(registry2);

    assert_eq!(registry1.len(), 2);
    assert!(registry1.has_declarative("macro1"));
    assert!(registry1.has_declarative("macro2"));
}
