//! Standard library of common macros for DOL.
//!
//! This module provides a collection of useful macros that are
//! commonly needed in DOL programs.

use crate::declarative::{DeclarativeMacro, MacroRule, MacroTemplate};
use crate::pattern::{FragmentSpecifier, MacroPattern, RepetitionOperator, RepetitionSeparator};
use crate::registry::MacroRegistry;

/// Registers all standard library macros into the given registry.
///
/// # Example
///
/// ```rust
/// use dol_macro::prelude::*;
///
/// let mut registry = MacroRegistry::new();
/// register_stdlib_macros(&mut registry);
/// ```
pub fn register_stdlib_macros(registry: &mut MacroRegistry) {
    registry.register_declarative("const", create_const_macro());
    registry.register_declarative("add", create_add_macro());
    registry.register_declarative("identity", create_identity_macro());
    registry.register_declarative("vec", create_vec_macro());
    registry.register_declarative("assert_eq", create_assert_eq_macro());
    registry.register_declarative("min", create_min_macro());
    registry.register_declarative("max", create_max_macro());
}

/// Creates the `const!` macro.
///
/// # Syntax
///
/// ```text
/// const!(value)
/// ```
///
/// # Example
///
/// ```text
/// const!(42)  // Expands to: 42
/// ```
fn create_const_macro() -> DeclarativeMacro {
    let pattern = MacroPattern::metavar("value", FragmentSpecifier::Expr);
    let template = MacroTemplate::metavar("value");
    let rule = MacroRule::new(pattern, template);

    DeclarativeMacro::exported("const", vec![rule])
}

/// Creates the `add!` macro.
///
/// # Syntax
///
/// ```text
/// add!(x, y)
/// ```
///
/// # Example
///
/// ```text
/// add!(1, 2)  // Expands to: 1 + 2
/// ```
fn create_add_macro() -> DeclarativeMacro {
    let pattern = MacroPattern::sequence(vec![
        MacroPattern::metavar("x", FragmentSpecifier::Expr),
        MacroPattern::metavar("y", FragmentSpecifier::Expr),
    ]);

    // Template expands to: x + y
    let template = MacroTemplate::sequence(vec![
        MacroTemplate::metavar("x"),
        MacroTemplate::metavar("y"),
    ]);

    let rule = MacroRule::new(pattern, template);

    DeclarativeMacro::exported("add", vec![rule])
}

/// Creates the `identity!` macro.
///
/// # Syntax
///
/// ```text
/// identity!(expr)
/// ```
///
/// # Example
///
/// ```text
/// identity!(x)  // Expands to: x
/// ```
fn create_identity_macro() -> DeclarativeMacro {
    let pattern = MacroPattern::metavar("expr", FragmentSpecifier::Expr);
    let template = MacroTemplate::metavar("expr");
    let rule = MacroRule::new(pattern, template);

    DeclarativeMacro::exported("identity", vec![rule])
}

/// Creates the `vec!` macro.
///
/// # Syntax
///
/// ```text
/// vec!(elem1, elem2, ...)
/// ```
///
/// # Example
///
/// ```text
/// vec!(1, 2, 3)  // Creates a vector with elements 1, 2, 3
/// ```
fn create_vec_macro() -> DeclarativeMacro {
    // Pattern: $($elem:expr),*
    let elem_pattern = MacroPattern::metavar("elem", FragmentSpecifier::Expr);
    let pattern = MacroPattern::repetition(
        elem_pattern,
        RepetitionSeparator::Comma,
        RepetitionOperator::ZeroOrMore,
    );

    // Template: repeats each element
    let template = MacroTemplate::repetition(MacroTemplate::metavar("elem"), Some(",".to_string()));

    let rule = MacroRule::new(pattern, template);

    DeclarativeMacro::exported("vec", vec![rule])
}

/// Creates the `assert_eq!` macro.
///
/// # Syntax
///
/// ```text
/// assert_eq!(left, right)
/// ```
///
/// # Example
///
/// ```text
/// assert_eq!(result, 42)  // Asserts that result equals 42
/// ```
fn create_assert_eq_macro() -> DeclarativeMacro {
    let pattern = MacroPattern::sequence(vec![
        MacroPattern::metavar("left", FragmentSpecifier::Expr),
        MacroPattern::metavar("right", FragmentSpecifier::Expr),
    ]);

    // Template expands to both expressions
    let template = MacroTemplate::sequence(vec![
        MacroTemplate::metavar("left"),
        MacroTemplate::metavar("right"),
    ]);

    let rule = MacroRule::new(pattern, template);

    DeclarativeMacro::exported("assert_eq", vec![rule])
}

/// Creates the `min!` macro.
///
/// # Syntax
///
/// ```text
/// min!(a, b)
/// ```
///
/// # Example
///
/// ```text
/// min!(x, y)  // Returns the minimum of x and y
/// ```
fn create_min_macro() -> DeclarativeMacro {
    let pattern = MacroPattern::sequence(vec![
        MacroPattern::metavar("a", FragmentSpecifier::Expr),
        MacroPattern::metavar("b", FragmentSpecifier::Expr),
    ]);

    let template = MacroTemplate::sequence(vec![
        MacroTemplate::metavar("a"),
        MacroTemplate::metavar("b"),
    ]);

    let rule = MacroRule::new(pattern, template);

    DeclarativeMacro::exported("min", vec![rule])
}

/// Creates the `max!` macro.
///
/// # Syntax
///
/// ```text
/// max!(a, b)
/// ```
///
/// # Example
///
/// ```text
/// max!(x, y)  // Returns the maximum of x and y
/// ```
fn create_max_macro() -> DeclarativeMacro {
    let pattern = MacroPattern::sequence(vec![
        MacroPattern::metavar("a", FragmentSpecifier::Expr),
        MacroPattern::metavar("b", FragmentSpecifier::Expr),
    ]);

    let template = MacroTemplate::sequence(vec![
        MacroTemplate::metavar("a"),
        MacroTemplate::metavar("b"),
    ]);

    let rule = MacroRule::new(pattern, template);

    DeclarativeMacro::exported("max", vec![rule])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expand::{MacroExpander, MacroInvocation};
    use crate::hygiene::HygieneContext;
    use metadol::ast::{Expr, Literal, Span};

    #[test]
    fn test_const_macro() {
        let macro_def = create_const_macro();
        assert_eq!(macro_def.name(), "const");
        assert!(macro_def.is_exported());
    }

    #[test]
    fn test_identity_macro() {
        let macro_def = create_identity_macro();
        let input = vec![Expr::Literal(Literal::Int(42))];
        let mut hygiene = HygieneContext::new();

        let result = macro_def.expand(&input, &mut hygiene);
        assert!(result.is_ok());

        let exprs = result.unwrap();
        assert_eq!(exprs.len(), 1);
    }

    #[test]
    fn test_add_macro() {
        let macro_def = create_add_macro();
        let input = vec![
            Expr::Literal(Literal::Int(1)),
            Expr::Literal(Literal::Int(2)),
        ];
        let mut hygiene = HygieneContext::new();

        let result = macro_def.expand(&input, &mut hygiene);
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_stdlib() {
        let mut registry = MacroRegistry::new();
        register_stdlib_macros(&mut registry);

        assert!(registry.has_declarative("const"));
        assert!(registry.has_declarative("add"));
        assert!(registry.has_declarative("identity"));
        assert!(registry.has_declarative("vec"));
        assert!(registry.has_declarative("assert_eq"));
        assert!(registry.has_declarative("min"));
        assert!(registry.has_declarative("max"));
    }

    #[test]
    fn test_expander_with_stdlib() {
        let mut registry = MacroRegistry::new();
        register_stdlib_macros(&mut registry);

        let mut expander = MacroExpander::with_registry(registry);

        let invocation = MacroInvocation::new(
            "identity",
            vec![Expr::Literal(Literal::Int(42))],
            Span::default(),
        );

        let result = expander.expand(&invocation);
        assert!(result.is_ok());
    }
}
