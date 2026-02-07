//! Declarative macro system for DOL.
//!
//! This module implements macro_rules!-style declarative macros with
//! pattern matching, hygienic expansion, and compile-time code generation.
//!
//! # Overview
//!
//! Declarative macros are defined using pattern-based rules that match
//! input and produce output through template expansion. Each macro consists
//! of one or more rules, each with a pattern and a template.
//!
//! # Syntax
//!
//! ```text
//! macro_rules! macro_name {
//!     (pattern1) => { template1 };
//!     (pattern2) => { template2 };
//!     // ... more rules
//! }
//! ```
//!
//! # Example
//!
//! ```text
//! // Define a macro
//! macro_rules! vec {
//!     ($($x:expr),*) => {
//!         {
//!             let mut temp = Vec::new();
//!             $(temp.push($x);)*
//!             temp
//!         }
//!     };
//! }
//!
//! // Use the macro
//! let v = vec!(1, 2, 3);
//! ```
//!
//! # Pattern Matching
//!
//! Patterns can include:
//! - Metavariables: `$name:fragment`
//! - Repetitions: `$(...)*`, `$(...)+`, `$(...)?`
//! - Literals: tokens that must match exactly
//! - Sequences: multiple patterns in order
//!
//! # Template Expansion
//!
//! Templates are expanded by substituting matched fragments for metavariables
//! and repeating template sections according to repetition patterns.

use crate::error::{MacroError, MacroResult};
use crate::hygiene::HygieneContext;
use crate::pattern::{MacroFragment, MacroPattern, PatternMatcher};
use metadol::ast::{Block, Declaration, Expr, Span, Stmt};
use std::collections::HashMap;

/// A macro rule with pattern and template.
///
/// Each rule specifies a pattern to match against input and a template
/// to generate output.
#[derive(Debug, Clone, PartialEq)]
pub struct MacroRule {
    /// The pattern to match
    pub pattern: MacroPattern,
    /// The template for expansion
    pub template: MacroTemplate,
    /// Optional guard condition
    pub guard: Option<MacroGuard>,
}

impl MacroRule {
    /// Creates a new macro rule.
    pub fn new(pattern: MacroPattern, template: MacroTemplate) -> Self {
        Self {
            pattern,
            template,
            guard: None,
        }
    }

    /// Creates a macro rule with a guard condition.
    pub fn with_guard(pattern: MacroPattern, template: MacroTemplate, guard: MacroGuard) -> Self {
        Self {
            pattern,
            template,
            guard: Some(guard),
        }
    }

    /// Attempts to match this rule against input.
    ///
    /// Returns the captured bindings on success, or None if matching fails.
    pub fn try_match(&self, input: &[Expr]) -> Option<HashMap<String, MacroFragment>> {
        let mut matcher = PatternMatcher::new();
        matcher.match_pattern(&self.pattern, input).ok()
    }

    /// Expands this rule with the given bindings.
    pub fn expand(
        &self,
        bindings: &HashMap<String, MacroFragment>,
        hygiene: &mut HygieneContext,
    ) -> MacroResult<Vec<Expr>> {
        self.template.expand(bindings, hygiene)
    }
}

/// Guard condition for macro rules.
///
/// Guards allow rules to be conditionally matched based on
/// properties of the captured bindings.
#[derive(Debug, Clone, PartialEq)]
pub enum MacroGuard {
    /// Check if a metavariable is a specific literal
    IsLiteral(String),
    /// Check if a metavariable is an identifier
    IsIdent(String),
    /// Check if a metavariable matches a specific value
    Equals(String, String),
    /// Custom guard predicate
    Custom(String),
}

impl MacroGuard {
    /// Evaluates the guard condition against bindings.
    pub fn evaluate(&self, bindings: &HashMap<String, MacroFragment>) -> bool {
        match self {
            Self::IsLiteral(name) => {
                if let Some(fragment) = bindings.get(name) {
                    matches!(fragment, MacroFragment::Literal(_))
                } else {
                    false
                }
            }

            Self::IsIdent(name) => {
                if let Some(fragment) = bindings.get(name) {
                    matches!(fragment, MacroFragment::Ident(_))
                } else {
                    false
                }
            }

            Self::Equals(name, value) => {
                if let Some(fragment) = bindings.get(name) {
                    if let MacroFragment::Ident(ident) = fragment {
                        ident == value
                    } else {
                        false
                    }
                } else {
                    false
                }
            }

            Self::Custom(_) => {
                // Custom guards would need evaluation context
                false
            }
        }
    }
}

/// Template for macro expansion.
///
/// Templates specify how to generate output from matched input,
/// using metavariable substitution and repetition.
#[derive(Debug, Clone, PartialEq)]
pub enum MacroTemplate {
    /// Empty template
    Empty,
    /// Literal expression
    Expr(Expr),
    /// Metavariable reference: `$name`
    Metavar(String),
    /// Sequence of templates
    Sequence(Vec<MacroTemplate>),
    /// Repeated template: `$(...)*`
    Repetition {
        /// The template to repeat
        template: Box<MacroTemplate>,
        /// Separator between repetitions
        separator: Option<String>,
    },
    /// Statement template
    Stmt(Stmt),
    /// Block template
    Block(Vec<MacroTemplate>),
    /// Declaration template
    Decl(Declaration),
}

impl MacroTemplate {
    /// Creates an expression template.
    pub fn expr(expr: Expr) -> Self {
        Self::Expr(expr)
    }

    /// Creates a metavariable reference.
    pub fn metavar(name: impl Into<String>) -> Self {
        Self::Metavar(name.into())
    }

    /// Creates a sequence template.
    pub fn sequence(templates: Vec<MacroTemplate>) -> Self {
        Self::Sequence(templates)
    }

    /// Creates a repetition template.
    pub fn repetition(template: MacroTemplate, separator: Option<String>) -> Self {
        Self::Repetition {
            template: Box::new(template),
            separator,
        }
    }

    /// Expands this template with the given bindings.
    pub fn expand(
        &self,
        bindings: &HashMap<String, MacroFragment>,
        hygiene: &mut HygieneContext,
    ) -> MacroResult<Vec<Expr>> {
        match self {
            Self::Empty => Ok(vec![]),

            Self::Expr(expr) => {
                let hygienic = hygiene.apply_hygiene_to_expr(expr);
                Ok(vec![hygienic])
            }

            Self::Metavar(name) => {
                if let Some(fragment) = bindings.get(name) {
                    Self::expand_fragment(fragment, hygiene)
                } else {
                    Err(MacroError::undefined(name))
                }
            }

            Self::Sequence(templates) => {
                let mut result = Vec::new();
                for template in templates {
                    result.extend(template.expand(bindings, hygiene)?);
                }
                Ok(result)
            }

            Self::Repetition {
                template,
                separator: _,
            } => {
                // Find the first metavariable in the template that has a repetition
                let metavars = self.find_metavars();
                if let Some(var_name) = metavars.first() {
                    if let Some(MacroFragment::Repetition(repetitions)) = bindings.get(var_name) {
                        let mut result = Vec::new();
                        for _rep in repetitions {
                            result.extend(template.expand(bindings, hygiene)?);
                        }
                        return Ok(result);
                    }
                }

                // If no repetition found, expand once
                template.expand(bindings, hygiene)
            }

            Self::Stmt(stmt) => {
                let hygienic = hygiene.apply_hygiene_to_stmt(stmt);
                // Convert statement to expression (if possible)
                match hygienic {
                    Stmt::Expr(expr) => Ok(vec![expr]),
                    _ => Ok(vec![]), // Statements don't convert to expressions
                }
            }

            Self::Block(templates) => {
                let mut exprs = Vec::new();
                for template in templates {
                    exprs.extend(template.expand(bindings, hygiene)?);
                }
                Ok(exprs)
            }

            Self::Decl(_) => {
                // Declarations don't expand to expressions
                Ok(vec![])
            }
        }
    }

    /// Expands a captured fragment.
    fn expand_fragment(
        fragment: &MacroFragment,
        hygiene: &mut HygieneContext,
    ) -> MacroResult<Vec<Expr>> {
        match fragment {
            MacroFragment::Expr(expr) => {
                let hygienic = hygiene.apply_hygiene_to_expr(expr);
                Ok(vec![hygienic])
            }

            MacroFragment::Ident(name) => {
                let hygienic = hygiene.make_hygienic(name);
                Ok(vec![Expr::Ident(hygienic)])
            }

            MacroFragment::Literal(lit) => Ok(vec![Expr::Literal(lit.clone())]),

            MacroFragment::Path(parts) => {
                let path = parts.join(".");
                Ok(vec![Expr::Ident(path)])
            }

            MacroFragment::Repetition(fragments) => {
                let mut result = Vec::new();
                for frag in fragments {
                    result.extend(Self::expand_fragment(frag, hygiene)?);
                }
                Ok(result)
            }

            _ => Ok(vec![]),
        }
    }

    /// Finds all metavariable references in this template.
    fn find_metavars(&self) -> Vec<String> {
        let mut vars = Vec::new();
        self.collect_metavars(&mut vars);
        vars
    }

    fn collect_metavars(&self, vars: &mut Vec<String>) {
        match self {
            Self::Metavar(name) => vars.push(name.clone()),
            Self::Sequence(templates) => {
                for t in templates {
                    t.collect_metavars(vars);
                }
            }
            Self::Repetition { template, .. } => {
                template.collect_metavars(vars);
            }
            Self::Block(templates) => {
                for t in templates {
                    t.collect_metavars(vars);
                }
            }
            _ => {}
        }
    }
}

/// A declarative macro definition.
///
/// Declarative macros consist of a name and one or more rules.
/// When invoked, the macro tries each rule in order until one matches.
#[derive(Debug, Clone)]
pub struct DeclarativeMacro {
    /// Macro name
    pub name: String,
    /// List of macro rules
    pub rules: Vec<MacroRule>,
    /// Whether this macro is exported
    pub exported: bool,
}

impl DeclarativeMacro {
    /// Creates a new declarative macro.
    pub fn new(name: impl Into<String>, rules: Vec<MacroRule>) -> Self {
        Self {
            name: name.into(),
            rules,
            exported: false,
        }
    }

    /// Creates an exported declarative macro.
    pub fn exported(name: impl Into<String>, rules: Vec<MacroRule>) -> Self {
        Self {
            name: name.into(),
            rules,
            exported: true,
        }
    }

    /// Expands the macro with the given input.
    ///
    /// Tries each rule in order until one matches, then expands using that rule.
    pub fn expand(
        &self,
        input: &[Expr],
        hygiene: &mut HygieneContext,
    ) -> MacroResult<Vec<Expr>> {
        hygiene.enter_expansion();

        for rule in &self.rules {
            if let Some(bindings) = rule.try_match(input) {
                // Check guard if present
                if let Some(guard) = &rule.guard {
                    if !guard.evaluate(&bindings) {
                        continue;
                    }
                }

                let result = rule.expand(&bindings, hygiene);
                hygiene.exit_expansion();
                return result;
            }
        }

        hygiene.exit_expansion();
        Err(MacroError::pattern_mismatch(
            "any rule",
            &format!("{} argument(s)", input.len()),
        ))
    }

    /// Returns the name of this macro.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns true if this macro is exported.
    pub fn is_exported(&self) -> bool {
        self.exported
    }

    /// Returns the number of rules in this macro.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::{FragmentSpecifier, MacroPattern};
    use metadol::ast::Literal;

    #[test]
    fn test_macro_rule_creation() {
        let pattern = MacroPattern::Empty;
        let template = MacroTemplate::expr(Expr::Literal(Literal::Int(42)));
        let rule = MacroRule::new(pattern, template);

        assert!(rule.guard.is_none());
    }

    #[test]
    fn test_macro_rule_matching() {
        let pattern = MacroPattern::Empty;
        let template = MacroTemplate::expr(Expr::Literal(Literal::Int(42)));
        let rule = MacroRule::new(pattern, template);

        let input: Vec<Expr> = vec![];
        let bindings = rule.try_match(&input);
        assert!(bindings.is_some());
    }

    #[test]
    fn test_macro_rule_expansion() {
        let pattern = MacroPattern::Empty;
        let template = MacroTemplate::expr(Expr::Literal(Literal::Int(42)));
        let rule = MacroRule::new(pattern, template);

        let bindings = HashMap::new();
        let mut hygiene = HygieneContext::new();
        let result = rule.expand(&bindings, &mut hygiene);

        assert!(result.is_ok());
        let exprs = result.unwrap();
        assert_eq!(exprs.len(), 1);
    }

    #[test]
    fn test_template_metavar() {
        let template = MacroTemplate::metavar("x");
        let mut bindings = HashMap::new();
        bindings.insert(
            "x".to_string(),
            MacroFragment::Ident("foo".to_string()),
        );

        let mut hygiene = HygieneContext::new();
        let result = template.expand(&bindings, &mut hygiene);

        assert!(result.is_ok());
        let exprs = result.unwrap();
        assert_eq!(exprs.len(), 1);
    }

    #[test]
    fn test_template_sequence() {
        let template = MacroTemplate::sequence(vec![
            MacroTemplate::expr(Expr::Literal(Literal::Int(1))),
            MacroTemplate::expr(Expr::Literal(Literal::Int(2))),
        ]);

        let bindings = HashMap::new();
        let mut hygiene = HygieneContext::new();
        let result = template.expand(&bindings, &mut hygiene);

        assert!(result.is_ok());
        let exprs = result.unwrap();
        assert_eq!(exprs.len(), 2);
    }

    #[test]
    fn test_declarative_macro() {
        let pattern = MacroPattern::Empty;
        let template = MacroTemplate::expr(Expr::Literal(Literal::Int(42)));
        let rule = MacroRule::new(pattern, template);

        let macro_def = DeclarativeMacro::new("test", vec![rule]);
        assert_eq!(macro_def.name(), "test");
        assert_eq!(macro_def.rule_count(), 1);
        assert!(!macro_def.is_exported());
    }

    #[test]
    fn test_declarative_macro_expansion() {
        let pattern = MacroPattern::Empty;
        let template = MacroTemplate::expr(Expr::Literal(Literal::Int(42)));
        let rule = MacroRule::new(pattern, template);

        let macro_def = DeclarativeMacro::new("test", vec![rule]);
        let mut hygiene = HygieneContext::new();

        let input: Vec<Expr> = vec![];
        let result = macro_def.expand(&input, &mut hygiene);

        assert!(result.is_ok());
        let exprs = result.unwrap();
        assert_eq!(exprs.len(), 1);
    }

    #[test]
    fn test_macro_guard() {
        let guard = MacroGuard::IsIdent("x".to_string());
        let mut bindings = HashMap::new();
        bindings.insert(
            "x".to_string(),
            MacroFragment::Ident("foo".to_string()),
        );

        assert!(guard.evaluate(&bindings));
    }

    #[test]
    fn test_macro_guard_equals() {
        let guard = MacroGuard::Equals("x".to_string(), "foo".to_string());
        let mut bindings = HashMap::new();
        bindings.insert(
            "x".to_string(),
            MacroFragment::Ident("foo".to_string()),
        );

        assert!(guard.evaluate(&bindings));

        bindings.insert(
            "x".to_string(),
            MacroFragment::Ident("bar".to_string()),
        );
        assert!(!guard.evaluate(&bindings));
    }
}
