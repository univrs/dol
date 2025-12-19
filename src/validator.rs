//! Semantic validation for Metal DOL.
//!
//! This module provides validation rules that cannot be enforced during parsing,
//! such as exegesis requirements, naming conventions, and reference resolution.
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

use crate::ast::*;
use crate::error::{ValidationError, ValidationWarning};

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
pub fn validate(decl: &Declaration) -> ValidationResult {
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
    }

    result
}

/// Validates the exegesis block.
fn validate_exegesis(decl: &Declaration, result: &mut ValidationResult) {
    let exegesis = decl.exegesis();
    let span = decl.span();

    // Check for empty exegesis
    if exegesis.trim().is_empty() {
        result.add_error(ValidationError::EmptyExegesis { span });
        return;
    }

    // Warn about very short exegesis
    let trimmed_len = exegesis.trim().len();
    if trimmed_len < 20 {
        result.add_warning(ValidationWarning::ShortExegesis {
            length: trimmed_len,
            span,
        });
    }
}

/// Validates naming conventions.
fn validate_naming(decl: &Declaration, result: &mut ValidationResult) {
    let name = decl.name();

    // Check for valid qualified identifier format
    if !is_valid_qualified_identifier(name) {
        result.add_error(ValidationError::InvalidIdentifier {
            name: name.to_string(),
            reason: "must be a valid qualified identifier (domain.property)".to_string(),
        });
        return;
    }

    // Check naming convention based on declaration type
    let parts: Vec<&str> = name.split('.').collect();
    if parts.len() < 2 {
        result.add_warning(ValidationWarning::NamingConvention {
            name: name.to_string(),
            suggestion: "use qualified name like 'domain.property'".to_string(),
        });
    }

    // Check that parts are lowercase
    for part in &parts {
        if *part != part.to_lowercase() {
            result.add_warning(ValidationWarning::NamingConvention {
                name: name.to_string(),
                suggestion: format!("use lowercase: '{}'", name.to_lowercase()),
            });
            break;
        }
    }
}

/// Validates statements in a declaration.
fn validate_statements(decl: &Declaration, result: &mut ValidationResult) {
    let statements = match decl {
        Declaration::Gene(g) => &g.statements,
        Declaration::Trait(t) => &t.statements,
        Declaration::Constraint(c) => &c.statements,
        Declaration::System(s) => &s.statements,
        Declaration::Evolution(_) => return, // Evolution has different structure
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
        assert!(!result.is_valid());
        assert!(matches!(
            result.errors[0],
            ValidationError::EmptyExegesis { .. }
        ));
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
}
