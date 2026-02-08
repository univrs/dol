//! Pattern matching for declarative macros.
//!
//! This module provides the pattern matching system for macro_rules!-style
//! declarative macros, including fragment specifiers and repetition patterns.

use crate::error::{MacroError, MacroResult};
use metadol::ast::{Block, Declaration, Expr, Literal, Stmt, TypeExpr};
use std::collections::HashMap;

/// Fragment specifier for macro patterns.
///
/// Fragment specifiers indicate what kind of AST node a metavariable
/// can match in a macro pattern.
///
/// # Examples
///
/// ```text
/// $name:ident     // Matches an identifier
/// $expr:expr      // Matches an expression
/// $ty:type        // Matches a type
/// $stmt:stmt      // Matches a statement
/// $block:block    // Matches a block
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FragmentSpecifier {
    /// Identifier: `$name:ident`
    Ident,
    /// Expression: `$expr:expr`
    Expr,
    /// Statement: `$stmt:stmt`
    Stmt,
    /// Type: `$ty:type`
    Type,
    /// Block: `$block:block`
    Block,
    /// Declaration: `$decl:decl`
    Decl,
    /// Literal: `$lit:literal`
    Literal,
    /// Pattern: `$pat:pat`
    Pat,
    /// Path: `$path:path`
    Path,
    /// Token tree: `$tt:tt` (matches any single token)
    Tt,
    /// Visibility: `$vis:vis`
    Vis,
}

impl FragmentSpecifier {
    /// Parses a fragment specifier from a string.
    pub fn from_str(s: &str) -> MacroResult<Self> {
        match s {
            "ident" => Ok(Self::Ident),
            "expr" => Ok(Self::Expr),
            "stmt" => Ok(Self::Stmt),
            "type" => Ok(Self::Type),
            "block" => Ok(Self::Block),
            "decl" => Ok(Self::Decl),
            "literal" | "lit" => Ok(Self::Literal),
            "pat" => Ok(Self::Pat),
            "path" => Ok(Self::Path),
            "tt" => Ok(Self::Tt),
            "vis" => Ok(Self::Vis),
            _ => Err(MacroError::invalid_fragment(s)),
        }
    }

    /// Returns the name of this fragment specifier.
    pub fn name(&self) -> &str {
        match self {
            Self::Ident => "ident",
            Self::Expr => "expr",
            Self::Stmt => "stmt",
            Self::Type => "type",
            Self::Block => "block",
            Self::Decl => "decl",
            Self::Literal => "literal",
            Self::Pat => "pat",
            Self::Path => "path",
            Self::Tt => "tt",
            Self::Vis => "vis",
        }
    }
}

/// A fragment captured by a macro pattern.
///
/// Fragments are the actual AST nodes captured by metavariables
/// during pattern matching.
#[derive(Debug, Clone, PartialEq)]
pub enum MacroFragment {
    /// Identifier fragment
    Ident(String),
    /// Expression fragment
    Expr(Expr),
    /// Statement fragment
    Stmt(Stmt),
    /// Type fragment
    Type(TypeExpr),
    /// Block fragment
    Block(Block),
    /// Declaration fragment
    Decl(Declaration),
    /// Literal fragment
    Literal(Literal),
    /// Path fragment (dotted identifier)
    Path(Vec<String>),
    /// Token tree fragment (for tt specifier)
    TokenTree(String),
    /// Visibility fragment
    Vis(metadol::ast::Visibility),
    /// Repetition of fragments
    Repetition(Vec<MacroFragment>),
}

impl MacroFragment {
    /// Returns the fragment as an identifier, if it is one.
    pub fn as_ident(&self) -> Option<&str> {
        match self {
            Self::Ident(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the fragment as an expression, if it is one.
    pub fn as_expr(&self) -> Option<&Expr> {
        match self {
            Self::Expr(e) => Some(e),
            _ => None,
        }
    }

    /// Returns the fragment as a statement, if it is one.
    pub fn as_stmt(&self) -> Option<&Stmt> {
        match self {
            Self::Stmt(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the fragment as a type, if it is one.
    pub fn as_type(&self) -> Option<&TypeExpr> {
        match self {
            Self::Type(t) => Some(t),
            _ => None,
        }
    }

    /// Returns the fragment as a repetition, if it is one.
    pub fn as_repetition(&self) -> Option<&[MacroFragment]> {
        match self {
            Self::Repetition(r) => Some(r),
            _ => None,
        }
    }
}

/// Repetition separator in macro patterns.
///
/// # Examples
///
/// ```text
/// $($item:expr),*    // Comma separator
/// $($stmt:stmt);*    // Semicolon separator
/// $($decl:decl)*     // No separator
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepetitionSeparator {
    /// Comma separator
    Comma,
    /// Semicolon separator
    Semicolon,
    /// No separator
    None,
    /// Custom separator
    Custom(String),
}

/// Repetition operator in macro patterns.
///
/// # Examples
///
/// ```text
/// $(...)*    // Zero or more
/// $(...)+    // One or more
/// $(...)?    // Zero or one
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepetitionOperator {
    /// Zero or more: `*`
    ZeroOrMore,
    /// One or more: `+`
    OneOrMore,
    /// Zero or one: `?`
    Optional,
}

/// A pattern for matching macro invocations.
///
/// Patterns specify the structure of input that a macro rule can match.
///
/// # Examples
///
/// ```text
/// // Empty pattern
/// () => { ... }
///
/// // Single identifier
/// ($name:ident) => { ... }
///
/// // Multiple patterns
/// ($x:expr, $y:expr) => { ... }
///
/// // Repetition
/// ($($item:expr),*) => { ... }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum MacroPattern {
    /// Empty pattern: `()`
    Empty,
    /// Metavariable: `$name:fragment`
    Metavar {
        /// Variable name
        name: String,
        /// Fragment specifier
        fragment: FragmentSpecifier,
    },
    /// Literal token
    Token(String),
    /// Sequence of patterns
    Sequence(Vec<MacroPattern>),
    /// Repeated pattern: `$(...) sep op`
    Repetition {
        /// The pattern to repeat
        pattern: Box<MacroPattern>,
        /// Separator between repetitions
        separator: RepetitionSeparator,
        /// Repetition operator
        operator: RepetitionOperator,
    },
}

impl MacroPattern {
    /// Creates a new metavariable pattern.
    pub fn metavar(name: impl Into<String>, fragment: FragmentSpecifier) -> Self {
        Self::Metavar {
            name: name.into(),
            fragment,
        }
    }

    /// Creates a new token pattern.
    pub fn token(text: impl Into<String>) -> Self {
        Self::Token(text.into())
    }

    /// Creates a new sequence pattern.
    pub fn sequence(patterns: Vec<MacroPattern>) -> Self {
        Self::Sequence(patterns)
    }

    /// Creates a new repetition pattern.
    pub fn repetition(
        pattern: MacroPattern,
        separator: RepetitionSeparator,
        operator: RepetitionOperator,
    ) -> Self {
        Self::Repetition {
            pattern: Box::new(pattern),
            separator,
            operator,
        }
    }

    /// Returns true if this pattern is empty.
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Extracts all metavariable names from this pattern.
    pub fn metavariables(&self) -> Vec<&str> {
        let mut vars = Vec::new();
        self.collect_metavars(&mut vars);
        vars
    }

    fn collect_metavars<'a>(&'a self, vars: &mut Vec<&'a str>) {
        match self {
            Self::Metavar { name, .. } => vars.push(name),
            Self::Sequence(patterns) => {
                for p in patterns {
                    p.collect_metavars(vars);
                }
            }
            Self::Repetition { pattern, .. } => {
                pattern.collect_metavars(vars);
            }
            _ => {}
        }
    }
}

/// Pattern matcher for macro invocations.
///
/// Performs pattern matching between macro patterns and input,
/// capturing fragments into bindings.
pub struct PatternMatcher {
    /// Captured bindings
    bindings: HashMap<String, MacroFragment>,
}

impl PatternMatcher {
    /// Creates a new pattern matcher.
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    /// Matches a pattern against input expressions.
    ///
    /// # Returns
    ///
    /// A map of metavariable names to captured fragments on success,
    /// or an error if matching fails.
    pub fn match_pattern(
        &mut self,
        pattern: &MacroPattern,
        input: &[Expr],
    ) -> MacroResult<HashMap<String, MacroFragment>> {
        self.bindings.clear();
        self.match_pattern_impl(pattern, input, 0)?;
        Ok(self.bindings.clone())
    }

    fn match_pattern_impl(
        &mut self,
        pattern: &MacroPattern,
        input: &[Expr],
        pos: usize,
    ) -> MacroResult<usize> {
        match pattern {
            MacroPattern::Empty => {
                if input.is_empty() {
                    Ok(0)
                } else {
                    Err(MacroError::pattern_mismatch("empty", "non-empty"))
                }
            }

            MacroPattern::Metavar { name, fragment } => {
                if pos >= input.len() {
                    return Err(MacroError::pattern_mismatch(
                        fragment.name(),
                        "end of input",
                    ));
                }

                let captured = self.capture_fragment(fragment, &input[pos])?;
                self.bindings.insert(name.clone(), captured);
                Ok(pos + 1)
            }

            MacroPattern::Token(expected) => {
                if pos >= input.len() {
                    return Err(MacroError::pattern_mismatch(expected, "end of input"));
                }

                // For simplicity, we check if the expression stringifies to the token
                // A more sophisticated implementation would tokenize the expression
                let actual = format!("{:?}", input[pos]);
                if actual.contains(expected) {
                    Ok(pos + 1)
                } else {
                    Err(MacroError::pattern_mismatch(expected, &actual))
                }
            }

            MacroPattern::Sequence(patterns) => {
                let mut current_pos = pos;
                for p in patterns {
                    current_pos = self.match_pattern_impl(p, input, current_pos)?;
                }
                Ok(current_pos)
            }

            MacroPattern::Repetition {
                pattern,
                separator: _,
                operator,
            } => {
                let mut current_pos = pos;
                let mut repetitions = Vec::new();

                loop {
                    match self.match_pattern_impl(pattern, input, current_pos) {
                        Ok(new_pos) => {
                            // Capture the matched fragment
                            if let Some(last_binding) = self.bindings.values().last() {
                                repetitions.push(last_binding.clone());
                            }
                            current_pos = new_pos;

                            if current_pos >= input.len() {
                                break;
                            }
                        }
                        Err(_) => {
                            // No more matches
                            break;
                        }
                    }
                }

                // Validate repetition count
                match operator {
                    RepetitionOperator::ZeroOrMore => {
                        // Any count is valid
                    }
                    RepetitionOperator::OneOrMore => {
                        if repetitions.is_empty() {
                            return Err(MacroError::pattern_mismatch(
                                "one or more",
                                "zero matches",
                            ));
                        }
                    }
                    RepetitionOperator::Optional => {
                        if repetitions.len() > 1 {
                            return Err(MacroError::pattern_mismatch(
                                "zero or one",
                                "multiple matches",
                            ));
                        }
                    }
                }

                Ok(current_pos)
            }
        }
    }

    fn capture_fragment(
        &self,
        fragment: &FragmentSpecifier,
        expr: &Expr,
    ) -> MacroResult<MacroFragment> {
        match fragment {
            FragmentSpecifier::Expr => Ok(MacroFragment::Expr(expr.clone())),

            FragmentSpecifier::Ident => {
                if let Expr::Identifier(name) = expr {
                    Ok(MacroFragment::Ident(name.clone()))
                } else {
                    Err(MacroError::type_mismatch("identifier", "expression"))
                }
            }

            FragmentSpecifier::Literal => {
                if let Expr::Literal(lit) = expr {
                    Ok(MacroFragment::Literal(lit.clone()))
                } else {
                    Err(MacroError::type_mismatch("literal", "expression"))
                }
            }

            FragmentSpecifier::Path => {
                // Extract dotted path from identifier
                if let Expr::Identifier(name) = expr {
                    let parts: Vec<String> = name.split('.').map(String::from).collect();
                    Ok(MacroFragment::Path(parts))
                } else {
                    Err(MacroError::type_mismatch("path", "expression"))
                }
            }

            FragmentSpecifier::Tt => {
                // Token tree: just stringify the expression
                Ok(MacroFragment::TokenTree(format!("{:?}", expr)))
            }

            _ => {
                // Other fragment types would require more context
                // For now, we'll wrap the expression
                Ok(MacroFragment::Expr(expr.clone()))
            }
        }
    }

    /// Returns the captured bindings.
    pub fn bindings(&self) -> &HashMap<String, MacroFragment> {
        &self.bindings
    }
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fragment_specifier_parsing() {
        assert_eq!(
            FragmentSpecifier::from_str("ident").unwrap(),
            FragmentSpecifier::Ident
        );
        assert_eq!(
            FragmentSpecifier::from_str("expr").unwrap(),
            FragmentSpecifier::Expr
        );
        assert!(FragmentSpecifier::from_str("invalid").is_err());
    }

    #[test]
    fn test_pattern_metavariables() {
        let pattern = MacroPattern::sequence(vec![
            MacroPattern::metavar("x", FragmentSpecifier::Ident),
            MacroPattern::metavar("y", FragmentSpecifier::Expr),
        ]);

        let vars = pattern.metavariables();
        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&"x"));
        assert!(vars.contains(&"y"));
    }

    #[test]
    fn test_empty_pattern_matching() {
        let mut matcher = PatternMatcher::new();
        let pattern = MacroPattern::Empty;
        let input: Vec<Expr> = vec![];

        let result = matcher.match_pattern(&pattern, &input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_metavar_pattern_matching() {
        let mut matcher = PatternMatcher::new();
        let pattern = MacroPattern::metavar("x", FragmentSpecifier::Ident);
        let input = vec![Expr::Identifier("foo".to_string())];

        let result = matcher.match_pattern(&pattern, &input);
        assert!(result.is_ok());

        let bindings = result.unwrap();
        assert!(bindings.contains_key("x"));
        assert_eq!(bindings["x"].as_ident(), Some("foo"));
    }

    #[test]
    fn test_sequence_pattern_matching() {
        let mut matcher = PatternMatcher::new();
        let pattern = MacroPattern::sequence(vec![
            MacroPattern::metavar("x", FragmentSpecifier::Ident),
            MacroPattern::metavar("y", FragmentSpecifier::Ident),
        ]);
        let input = vec![
            Expr::Identifier("foo".to_string()),
            Expr::Identifier("bar".to_string()),
        ];

        let result = matcher.match_pattern(&pattern, &input);
        assert!(result.is_ok());

        let bindings = result.unwrap();
        assert_eq!(bindings.len(), 2);
    }

    #[test]
    fn test_pattern_mismatch() {
        let mut matcher = PatternMatcher::new();
        let pattern = MacroPattern::metavar("x", FragmentSpecifier::Ident);
        let input = vec![Expr::Literal(Literal::Int(42))];

        let result = matcher.match_pattern(&pattern, &input);
        assert!(result.is_err());
    }
}
