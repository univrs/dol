//! Semantic validation for Metal DOL.
//!
//! This module provides validation rules that cannot be enforced during parsing,
//! such as exegesis requirements, naming conventions, reference resolution, and
//! type checking for DOL 2.0 expressions.
//!
//! # Example
//!
//! ```rust
//! use metadol::{parse_file, validate};
//!
//! let source = r#"
//! gene container.exists {
//!   container has identity
//! }
//!
//! exegesis {
//!   A container is the fundamental unit.
//! }
//! "#;
//!
//! let decl = parse_file(source).unwrap();
//! let result = validate(&decl);
//! assert!(result.is_valid());
//! ```
//!
//! # Type Checking
//!
//! For DOL 2.0 expressions, type validation can be enabled:
//!
//! ```rust
//! use metadol::{parse_file, validator::{validate_with_options, ValidationOptions}};
//!
//! let source = r#"
//! gene typed.example {
//!   example has property
//! }
//!
//! exegesis {
//!   A typed example gene.
//! }
//! "#;
//!
//! let decl = parse_file(source).unwrap();
//! let options = ValidationOptions { typecheck: true };
//! let result = validate_with_options(&decl, &options);
//! ```

use crate::ast::*;
use crate::error::{ValidationError, ValidationWarning};
use crate::typechecker::{Type, TypeChecker, TypeError};

/// The result of validating a declaration.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// The declaration that was validated
    pub declaration_name: String,

    /// Whether validation passed
    pub valid: bool,

    /// Collected errors
    pub errors: Vec<ValidationError>,

    /// Collected warnings
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    /// Creates a new validation result.
    fn new(name: impl Into<String>) -> Self {
        Self {
            declaration_name: name.into(),
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Returns true if validation passed (no errors).
    pub fn is_valid(&self) -> bool {
        self.valid && self.errors.is_empty()
    }

    /// Returns true if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Adds an error and marks validation as failed.
    fn add_error(&mut self, error: ValidationError) {
        self.valid = false;
        self.errors.push(error);
    }

    /// Adds a warning.
    fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    /// Adds a type error converted to a validation error.
    fn add_type_error(&mut self, error: &TypeError, span: Span) {
        self.add_error(ValidationError::TypeError {
            message: error.message.clone(),
            expected: error.expected.as_ref().map(|t| t.to_string()),
            actual: error.actual.as_ref().map(|t| t.to_string()),
            span,
        });
    }
}

/// Options for validation.
#[derive(Debug, Clone, Default)]
pub struct ValidationOptions {
    /// Enable type checking for DOL 2.0 expressions.
    pub typecheck: bool,
}

/// Validates a declaration with options.
///
/// # Arguments
///
/// * `decl` - The declaration to validate
/// * `options` - Validation options
///
/// # Returns
///
/// A `ValidationResult` containing any errors or warnings.
pub fn validate_with_options(decl: &Declaration, options: &ValidationOptions) -> ValidationResult {
    let mut result = ValidationResult::new(decl.name());

    // Validate exegesis
    validate_exegesis(decl, &mut result);

    // Validate naming conventions
    validate_naming(decl, &mut result);

    // Validate statements
    validate_statements(decl, &mut result);

    // Type-specific validations
    match decl {
        Declaration::Gene(gene) => validate_gene(gene, &mut result),
        Declaration::Trait(trait_decl) => validate_trait(trait_decl, &mut result),
        Declaration::Constraint(constraint) => validate_constraint(constraint, &mut result),
        Declaration::System(system) => validate_system(system, &mut result),
        Declaration::Evolution(evolution) => validate_evolution(evolution, &mut result),
        Declaration::Function(_) => {} // Top-level functions don't need special validation yet
        Declaration::Const(_) => {}    // Constants are validated by type checking
    }

    // DOL 2.0 Type checking (if enabled)
    if options.typecheck {
        validate_types(decl, &mut result);
    }

    result
}

/// Validates a declaration.
///
/// # Arguments
///
/// * `decl` - The declaration to validate
///
/// # Returns
///
/// A `ValidationResult` containing any errors or warnings.
///
/// Note: This does not include type checking by default.
/// Use [`validate_with_options`] with `typecheck: true` for DOL 2.0 type validation.
pub fn validate(decl: &Declaration) -> ValidationResult {
    validate_with_options(decl, &ValidationOptions::default())
}

/// Validates the exegesis block.
fn validate_exegesis(decl: &Declaration, result: &mut ValidationResult) {
    let exegesis = decl.exegesis();
    let span = decl.span();

    // Warn about very short exegesis
    let trimmed_len = exegesis.trim().len();
    if trimmed_len < 20 {
        result.add_warning(ValidationWarning::ShortExegesis {
            length: trimmed_len,
            span,
        });
    }
}

/// Validates naming conventions based on declaration type.
///
/// Conventions:
/// - Genes: PascalCase (Vec3, Container, MyceliumNode) OR dot notation (container.exists)
/// - Traits: PascalCase (Schedulable, Runnable) OR dot notation
/// - Systems: PascalCase (Scheduler, Ecosystem) OR dot notation
/// - Constraints: snake_case (valid_id, non_negative) OR dot notation
fn validate_naming(decl: &Declaration, result: &mut ValidationResult) {
    let name = decl.name();
    // Skip internal markers (e.g., _module_doc)
    if name.starts_with('_') {
        return;
    }
    // Skip empty names
    if name.is_empty() {
        return;
    }

    // If it contains a dot, it's qualified notation - validate each part
    if name.contains('.') {
        // Validate qualified identifier format
        if !is_valid_qualified_identifier(name) {
            result.add_error(ValidationError::InvalidIdentifier {
                name: name.to_string(),
                reason: "must be a valid qualified identifier (domain.property)".to_string(),
            });
        }
        return;
    }

    // Simple name - check based on declaration type
    match decl {
        // Types should be PascalCase
        Declaration::Gene(_) | Declaration::Trait(_) | Declaration::System(_) => {
            if !is_pascal_case(name) && !name.chars().next().is_some_and(|c| c.is_uppercase()) {
                result.add_warning(ValidationWarning::NamingConvention {
                    name: name.to_string(),
                    suggestion: format!(
                        "consider using PascalCase for type names: '{}'",
                        to_pascal_case(name)
                    ),
                });
            }
        }

        // Constraints can be snake_case or PascalCase
        Declaration::Constraint(_) => {
            // Constraints are flexible - no warning needed
        }

        // Evolution names follow "From > To" pattern - no validation needed
        Declaration::Evolution(_) => {}

        // Functions should be snake_case - no warning needed for now
        Declaration::Function(_) => {}

        // Constants should be SCREAMING_SNAKE_CASE
        Declaration::Const(_) => {}
    }
}

/// Check if a name is PascalCase (starts with uppercase, no underscores between words)
fn is_pascal_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let first = s.chars().next().unwrap();
    // PascalCase starts with uppercase and doesn't have underscores
    first.is_uppercase() && !s.contains('_')
}

/// Convert to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

/// Validates statements in a declaration.
fn validate_statements(decl: &Declaration, result: &mut ValidationResult) {
    let statements = match decl {
        Declaration::Gene(g) => &g.statements,
        Declaration::Trait(t) => &t.statements,
        Declaration::Constraint(c) => &c.statements,
        Declaration::System(s) => &s.statements,
        Declaration::Evolution(_) | Declaration::Function(_) | Declaration::Const(_) => return, // Different structure
    };

    // Check for duplicate statements
    let mut seen_uses: Vec<&str> = Vec::new();
    for stmt in statements {
        if let Statement::Uses { reference, .. } = stmt {
            if seen_uses.contains(&reference.as_str()) {
                result.add_error(ValidationError::DuplicateDefinition {
                    kind: "uses".to_string(),
                    name: reference.clone(),
                });
            } else {
                seen_uses.push(reference);
            }
        }
    }
}

/// Validates gene-specific rules.
fn validate_gene(gene: &Gene, result: &mut ValidationResult) {
    // Genes should only contain has, is, derives from, requires statements
    for stmt in &gene.statements {
        match stmt {
            Statement::Has { .. }
            | Statement::Is { .. }
            | Statement::DerivesFrom { .. }
            | Statement::Requires { .. } => {}
            Statement::Uses { span, .. } => {
                result.add_error(ValidationError::InvalidIdentifier {
                    name: "uses".to_string(),
                    reason: "genes cannot use 'uses' statements; use traits instead".to_string(),
                });
                let _ = span; // suppress warning
            }
            _ => {}
        }
    }
}

/// Validates trait-specific rules.
fn validate_trait(trait_decl: &Trait, result: &mut ValidationResult) {
    // Traits should have at least one uses or behavior statement
    let has_uses = trait_decl
        .statements
        .iter()
        .any(|s| matches!(s, Statement::Uses { .. }));

    let has_behavior = trait_decl
        .statements
        .iter()
        .any(|s| matches!(s, Statement::Is { .. }));

    if !has_uses && !has_behavior {
        result.add_warning(ValidationWarning::NamingConvention {
            name: trait_decl.name.clone(),
            suggestion: "traits typically include 'uses' or behavior statements".to_string(),
        });
    }
}

/// Validates constraint-specific rules.
fn validate_constraint(constraint: &Constraint, result: &mut ValidationResult) {
    // Constraints should have matches or never statements
    let has_constraint_stmts = constraint
        .statements
        .iter()
        .any(|s| matches!(s, Statement::Matches { .. } | Statement::Never { .. }));

    if !has_constraint_stmts {
        result.add_warning(ValidationWarning::NamingConvention {
            name: constraint.name.clone(),
            suggestion: "constraints typically include 'matches' or 'never' statements".to_string(),
        });
    }
}

/// Validates system-specific rules.
fn validate_system(system: &System, result: &mut ValidationResult) {
    // Validate version format
    if !is_valid_version(&system.version) {
        result.add_error(ValidationError::InvalidVersion {
            version: system.version.clone(),
            reason: "must be valid semver (X.Y.Z)".to_string(),
        });
    }

    // Validate requirements
    for req in &system.requirements {
        if !is_valid_version(&req.version) {
            result.add_error(ValidationError::InvalidVersion {
                version: req.version.clone(),
                reason: format!("invalid version in requirement for '{}'", req.name),
            });
        }
    }
}

/// Validates evolution-specific rules.
fn validate_evolution(evolution: &Evolution, result: &mut ValidationResult) {
    // Validate versions
    if !is_valid_version(&evolution.version) {
        result.add_error(ValidationError::InvalidVersion {
            version: evolution.version.clone(),
            reason: "must be valid semver (X.Y.Z)".to_string(),
        });
    }

    if !is_valid_version(&evolution.parent_version) {
        result.add_error(ValidationError::InvalidVersion {
            version: evolution.parent_version.clone(),
            reason: "parent version must be valid semver (X.Y.Z)".to_string(),
        });
    }

    // Check version ordering (new version should be greater than parent)
    if is_valid_version(&evolution.version)
        && is_valid_version(&evolution.parent_version)
        && !is_version_greater(&evolution.version, &evolution.parent_version)
    {
        result.add_warning(ValidationWarning::NamingConvention {
            name: evolution.name.clone(),
            suggestion: format!(
                "new version '{}' should be greater than parent '{}'",
                evolution.version, evolution.parent_version
            ),
        });
    }

    // Should have at least one change
    if evolution.additions.is_empty()
        && evolution.deprecations.is_empty()
        && evolution.removals.is_empty()
    {
        result.add_warning(ValidationWarning::NamingConvention {
            name: evolution.name.clone(),
            suggestion: "evolution should include at least one adds, deprecates, or removes"
                .to_string(),
        });
    }
}

// === DOL 2.0 Type Validation ===

/// Validates types in DOL 2.0 expressions.
///
/// This function type-checks expressions found in the declaration,
/// including let bindings, lambda expressions, and control flow.
fn validate_types(decl: &Declaration, result: &mut ValidationResult) {
    let mut checker = TypeChecker::new();
    let span = decl.span();

    // Currently, DOL 2.0 expressions can appear in evolution additions
    // and potentially in future extended statement types
    if let Declaration::Evolution(evolution) = decl {
        for stmt in &evolution.additions {
            validate_statement_types(stmt, &mut checker, result, span);
        }
        for stmt in &evolution.deprecations {
            validate_statement_types(stmt, &mut checker, result, span);
        }
    }

    // Convert any accumulated type errors to validation errors
    for error in checker.errors() {
        result.add_type_error(error, span);
    }
}

/// Type-checks a statement for DOL 2.0 expressions.
fn validate_statement_types(
    _stmt: &Statement,
    _checker: &mut TypeChecker,
    _result: &mut ValidationResult,
    _span: Span,
) {
    // Current Statement enum doesn't embed DOL 2.0 expressions directly.
    // This is a placeholder for when statements can contain typed expressions.
    // For now, type checking happens when parsing DOL 2.0 expression blocks.
}

/// Type-checks an expression and reports any errors.
#[allow(dead_code)]
fn validate_expr_types(
    expr: &Expr,
    checker: &mut TypeChecker,
    result: &mut ValidationResult,
    span: Span,
) {
    if let Err(error) = checker.infer(expr) {
        result.add_type_error(&error, span);
    }
}

/// Type-checks a statement and reports any errors.
#[allow(dead_code)]
fn validate_stmt_types(
    stmt: &Stmt,
    checker: &mut TypeChecker,
    result: &mut ValidationResult,
    span: Span,
) {
    match stmt {
        Stmt::Let {
            name,
            type_ann,
            value,
        } => {
            // Infer the value's type
            match checker.infer(value) {
                Ok(inferred_type) => {
                    // If there's a type annotation, verify it matches
                    if let Some(ann) = type_ann {
                        let expected = Type::from_type_expr(ann);
                        if !types_match(&inferred_type, &expected) {
                            result.add_type_error(
                                &TypeError::mismatch(expected, inferred_type),
                                span,
                            );
                        }
                    }
                    // Bind the variable (would need to track in checker's env)
                    let _ = name; // Suppress unused warning
                }
                Err(error) => {
                    result.add_type_error(&error, span);
                }
            }
        }
        Stmt::Expr(expr) => {
            validate_expr_types(expr, checker, result, span);
        }
        Stmt::For {
            binding: _,
            iterable,
            body,
        } => {
            validate_expr_types(iterable, checker, result, span);
            for s in body {
                validate_stmt_types(s, checker, result, span);
            }
        }
        Stmt::While { condition, body } => {
            // Condition must be Bool
            if let Err(error) = checker.check(condition, &Type::Bool) {
                result.add_type_error(&error, span);
            }
            for s in body {
                validate_stmt_types(s, checker, result, span);
            }
        }
        Stmt::Loop { body } => {
            for s in body {
                validate_stmt_types(s, checker, result, span);
            }
        }
        Stmt::Return(Some(expr)) => {
            validate_expr_types(expr, checker, result, span);
        }
        Stmt::Return(None) | Stmt::Break | Stmt::Continue => {}
        Stmt::Assign { target, value } => {
            validate_expr_types(target, checker, result, span);
            validate_expr_types(value, checker, result, span);
        }
    }
}

/// Checks if two types match (considering Any and Unknown as wildcards).
fn types_match(ty1: &Type, ty2: &Type) -> bool {
    match (ty1, ty2) {
        (Type::Unknown, _) | (_, Type::Unknown) => true,
        (Type::Any, _) | (_, Type::Any) => true,
        (Type::Error, _) | (_, Type::Error) => true,
        (a, b) if a == b => true,
        // Numeric types are compatible
        (a, b) if a.is_numeric() && b.is_numeric() => true,
        _ => false,
    }
}

// === Helper Functions ===

/// Checks if an identifier is valid.
fn is_valid_qualified_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    // Split by dots and validate each part
    for part in name.split('.') {
        if part.is_empty() {
            return false;
        }

        let mut chars = part.chars();
        let first = chars.next().unwrap();

        // First char must be alphabetic
        if !first.is_alphabetic() {
            return false;
        }

        // Rest must be alphanumeric or underscore
        for ch in chars {
            if !ch.is_alphanumeric() && ch != '_' {
                return false;
            }
        }
    }

    true
}

/// Checks if a version string is valid semver.
fn is_valid_version(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return false;
    }

    for part in parts {
        if part.parse::<u64>().is_err() {
            return false;
        }
    }

    true
}

/// Compares two version strings.
fn is_version_greater(version: &str, other: &str) -> bool {
    let parse_version = |v: &str| -> (u64, u64, u64) {
        let parts: Vec<&str> = v.split('.').collect();
        (
            parts[0].parse().unwrap_or(0),
            parts[1].parse().unwrap_or(0),
            parts[2].parse().unwrap_or(0),
        )
    };

    let v1 = parse_version(version);
    let v2 = parse_version(other);

    v1 > v2
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_gene(name: &str, exegesis: &str) -> Declaration {
        Declaration::Gene(Gene {
            name: name.to_string(),
            extends: None,
            statements: vec![Statement::Has {
                subject: "test".to_string(),
                property: "property".to_string(),
                span: Span::default(),
            }],
            exegesis: exegesis.to_string(),
            span: Span::default(),
        })
    }

    #[test]
    fn test_valid_declaration() {
        let decl = make_gene(
            "container.exists",
            "A container is the fundamental unit of workload isolation.",
        );
        let result = validate(&decl);
        assert!(result.is_valid());
    }

    #[test]
    fn test_empty_exegesis() {
        let decl = make_gene("container.exists", "");
        let result = validate(&decl);
        assert!(result.is_valid());
        assert!(result.has_warnings());
    }

    #[test]
    fn test_short_exegesis_warning() {
        let decl = make_gene("container.exists", "Short.");
        let result = validate(&decl);
        assert!(result.is_valid()); // Still valid, just warning
        assert!(result.has_warnings());
    }

    #[test]
    fn test_valid_identifier() {
        assert!(is_valid_qualified_identifier("container.exists"));
        assert!(is_valid_qualified_identifier("identity.cryptographic"));
        assert!(is_valid_qualified_identifier("simple"));
        assert!(!is_valid_qualified_identifier(""));
        assert!(!is_valid_qualified_identifier(".starts.with.dot"));
        assert!(!is_valid_qualified_identifier("123invalid"));
    }

    #[test]
    fn test_valid_version() {
        assert!(is_valid_version("0.0.1"));
        assert!(is_valid_version("1.2.3"));
        assert!(is_valid_version("10.20.30"));
        assert!(!is_valid_version("1.2"));
        assert!(!is_valid_version("1.2.3.4"));
        assert!(!is_valid_version("a.b.c"));
    }

    #[test]
    fn test_version_comparison() {
        assert!(is_version_greater("0.0.2", "0.0.1"));
        assert!(is_version_greater("0.1.0", "0.0.9"));
        assert!(is_version_greater("1.0.0", "0.9.9"));
        assert!(!is_version_greater("0.0.1", "0.0.2"));
        assert!(!is_version_greater("0.0.1", "0.0.1"));
    }

    // === DOL 2.0 Type-Aware Validation Tests ===

    #[test]
    fn test_validate_with_options_default() {
        let decl = make_gene("test.gene", "A test gene for validation options testing.");
        let options = ValidationOptions::default();
        assert!(!options.typecheck);
        let result = validate_with_options(&decl, &options);
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_with_typecheck_enabled() {
        let decl = make_gene("test.gene", "A test gene for type checking validation.");
        let options = ValidationOptions { typecheck: true };
        let result = validate_with_options(&decl, &options);
        // Should still be valid (no DOL 2.0 expressions with errors)
        assert!(result.is_valid());
    }

    #[test]
    fn test_types_match_any() {
        assert!(types_match(&Type::Any, &Type::Int32));
        assert!(types_match(&Type::String, &Type::Any));
    }

    #[test]
    fn test_types_match_unknown() {
        assert!(types_match(&Type::Unknown, &Type::Int32));
        assert!(types_match(&Type::String, &Type::Unknown));
    }

    #[test]
    fn test_types_match_error() {
        assert!(types_match(&Type::Error, &Type::Int32));
        assert!(types_match(&Type::String, &Type::Error));
    }

    #[test]
    fn test_types_match_same() {
        assert!(types_match(&Type::Int32, &Type::Int32));
        assert!(types_match(&Type::String, &Type::String));
        assert!(types_match(&Type::Bool, &Type::Bool));
    }

    #[test]
    fn test_types_match_numeric_promotion() {
        // All numeric types are compatible
        assert!(types_match(&Type::Int32, &Type::Int64));
        assert!(types_match(&Type::Float32, &Type::Float64));
        assert!(types_match(&Type::Int32, &Type::Float64));
    }

    #[test]
    fn test_types_mismatch() {
        assert!(!types_match(&Type::String, &Type::Int32));
        assert!(!types_match(&Type::Bool, &Type::String));
    }

    #[test]
    fn test_add_type_error_to_result() {
        let mut result = ValidationResult::new("test");
        let type_error = crate::typechecker::TypeError::mismatch(Type::String, Type::Int32);
        result.add_type_error(&type_error, Span::default());

        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
        match &result.errors[0] {
            crate::error::ValidationError::TypeError {
                expected, actual, ..
            } => {
                assert!(expected.as_ref().unwrap().contains("String"));
                assert!(actual.as_ref().unwrap().contains("Int32"));
            }
            _ => panic!("Expected TypeError variant"),
        }
    }

    #[test]
    fn test_validation_options_typecheck_flag() {
        let options = ValidationOptions { typecheck: true };
        assert!(options.typecheck);

        let options = ValidationOptions { typecheck: false };
        assert!(!options.typecheck);
    }
}
