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
use std::collections::HashSet;

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
        Declaration::Const(_) | Declaration::SexVar(_) => {} // Constants and SexVars are validated by type checking
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

/// Validates a complete DOL file including module, uses, and declarations.
///
/// This performs file-level validation including:
/// - Module declaration format
/// - Use declaration validation (visibility, source resolution)
/// - Declaration validation
/// - Visibility rules enforcement
///
/// # Arguments
///
/// * `file` - The parsed DOL file to validate
///
/// # Returns
///
/// A `FileValidationResult` containing any errors or warnings.
pub fn validate_file(file: &DolFile) -> FileValidationResult {
    validate_file_with_options(file, &ValidationOptions::default())
}

/// Validates a complete DOL file with options.
pub fn validate_file_with_options(
    file: &DolFile,
    options: &ValidationOptions,
) -> FileValidationResult {
    let mut result = FileValidationResult::new();

    // Validate module declaration
    if let Some(ref module) = file.module {
        validate_module_decl(module, &mut result);
    }

    // Validate use declarations
    validate_use_declarations(&file.uses, &mut result);

    // Validate each declaration
    for decl in &file.declarations {
        let decl_result = validate_with_options(decl, options);
        result.declaration_results.push(decl_result);
    }

    // Cross-reference validation: check that uses reference valid declarations
    validate_use_references(file, &mut result);

    result
}

/// Result of validating a complete DOL file.
#[derive(Debug, Clone)]
pub struct FileValidationResult {
    /// Module-level errors
    pub module_errors: Vec<ValidationError>,
    /// Module-level warnings
    pub module_warnings: Vec<ValidationWarning>,
    /// Validation results for each declaration
    pub declaration_results: Vec<ValidationResult>,
}

impl FileValidationResult {
    /// Creates a new file validation result.
    fn new() -> Self {
        Self {
            module_errors: Vec::new(),
            module_warnings: Vec::new(),
            declaration_results: Vec::new(),
        }
    }

    /// Returns true if the file is valid (no errors).
    pub fn is_valid(&self) -> bool {
        self.module_errors.is_empty() && self.declaration_results.iter().all(|r| r.is_valid())
    }

    /// Returns true if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        !self.module_warnings.is_empty()
            || self.declaration_results.iter().any(|r| r.has_warnings())
    }

    /// Collects all errors from the file.
    pub fn all_errors(&self) -> Vec<&ValidationError> {
        let mut errors: Vec<&ValidationError> = self.module_errors.iter().collect();
        for result in &self.declaration_results {
            errors.extend(result.errors.iter());
        }
        errors
    }

    /// Collects all warnings from the file.
    pub fn all_warnings(&self) -> Vec<&ValidationWarning> {
        let mut warnings: Vec<&ValidationWarning> = self.module_warnings.iter().collect();
        for result in &self.declaration_results {
            warnings.extend(result.warnings.iter());
        }
        warnings
    }

    fn add_error(&mut self, error: ValidationError) {
        self.module_errors.push(error);
    }

    fn add_warning(&mut self, warning: ValidationWarning) {
        self.module_warnings.push(warning);
    }
}

/// Validates a module declaration.
fn validate_module_decl(module: &ModuleDecl, result: &mut FileValidationResult) {
    // Module path must not be empty
    if module.path.is_empty() {
        result.add_error(ValidationError::InvalidIdentifier {
            name: "module".to_string(),
            reason: "module path cannot be empty".to_string(),
        });
        return;
    }

    // Validate each path segment
    for segment in &module.path {
        if segment.is_empty() {
            result.add_error(ValidationError::InvalidIdentifier {
                name: module.path.join("."),
                reason: "module path segment cannot be empty".to_string(),
            });
        } else if !segment.chars().next().unwrap_or('_').is_alphabetic() {
            result.add_error(ValidationError::InvalidIdentifier {
                name: segment.clone(),
                reason: "module path segment must start with a letter".to_string(),
            });
        }
    }

    // If version is present, validate it
    if let Some(ref version) = module.version {
        if version.major == 0 && version.minor == 0 && version.patch == 0 {
            result.add_warning(ValidationWarning::NamingConvention {
                name: module.path.join("."),
                suggestion:
                    "module version 0.0.0 is typically reserved; consider starting at 0.0.1"
                        .to_string(),
            });
        }
    }
}

/// Validates use declarations.
fn validate_use_declarations(uses: &[UseDecl], result: &mut FileValidationResult) {
    let mut seen_imports: HashSet<String> = HashSet::new();

    for use_decl in uses {
        // Check for duplicate imports
        let import_key = format_import_key(use_decl);
        if seen_imports.contains(&import_key) {
            result.add_warning(ValidationWarning::NamingConvention {
                name: import_key.clone(),
                suggestion: "duplicate import; this import was already declared".to_string(),
            });
        } else {
            seen_imports.insert(import_key);
        }

        // Validate import path
        validate_import_path(use_decl, result);

        // Validate visibility rules
        validate_use_visibility(use_decl, result);
    }
}

/// Formats an import key for duplicate detection.
fn format_import_key(use_decl: &UseDecl) -> String {
    let source_prefix = match &use_decl.source {
        ImportSource::Local => "local:".to_string(),
        ImportSource::Registry { org, package, .. } => format!("@{}/{}:", org, package),
        ImportSource::Git { url, .. } => format!("git:{}:", url),
        ImportSource::Https { url, .. } => format!("https:{}:", url),
    };
    format!("{}{}", source_prefix, use_decl.path.join("."))
}

/// Validates the import path in a use declaration.
fn validate_import_path(use_decl: &UseDecl, result: &mut FileValidationResult) {
    // Registry imports must have valid org/package
    if let ImportSource::Registry { org, package, .. } = &use_decl.source {
        if org.is_empty() {
            result.add_error(ValidationError::InvalidIdentifier {
                name: "registry import".to_string(),
                reason: "organization name cannot be empty".to_string(),
            });
        }
        if package.is_empty() {
            result.add_error(ValidationError::InvalidIdentifier {
                name: "registry import".to_string(),
                reason: "package name cannot be empty".to_string(),
            });
        }
    }

    // Git imports must have a URL
    if let ImportSource::Git { url, .. } = &use_decl.source {
        if url.is_empty() {
            result.add_error(ValidationError::InvalidIdentifier {
                name: "git import".to_string(),
                reason: "git URL cannot be empty".to_string(),
            });
        }
    }

    // HTTPS imports must have a valid URL
    if let ImportSource::Https { url, .. } = &use_decl.source {
        if url.is_empty() {
            result.add_error(ValidationError::InvalidIdentifier {
                name: "https import".to_string(),
                reason: "URL cannot be empty".to_string(),
            });
        }
        if !url.starts_with("https://") {
            result.add_error(ValidationError::InvalidIdentifier {
                name: url.clone(),
                reason: "HTTPS import URL must start with https://".to_string(),
            });
        }
    }

    // Validate named items if present
    if let UseItems::Named(items) = &use_decl.items {
        let mut seen_names: HashSet<String> = HashSet::new();
        for item in items {
            if seen_names.contains(&item.name) {
                result.add_warning(ValidationWarning::NamingConvention {
                    name: item.name.clone(),
                    suggestion: "duplicate item in import list".to_string(),
                });
            } else {
                seen_names.insert(item.name.clone());
            }
        }
    }
}

/// Validates visibility rules for use declarations.
fn validate_use_visibility(use_decl: &UseDecl, result: &mut FileValidationResult) {
    match use_decl.visibility {
        Visibility::Public => {
            // Public re-exports are allowed for all import types
        }
        Visibility::PubSpirit => {
            // pub(spirit) re-exports are only meaningful for local imports
            // For external imports, warn that pub(spirit) has limited utility
            if !matches!(use_decl.source, ImportSource::Local) {
                result.add_warning(ValidationWarning::NamingConvention {
                    name: format_import_key(use_decl),
                    suggestion: "pub(spirit) visibility on external imports has limited utility; consider using 'pub' instead".to_string(),
                });
            }
        }
        Visibility::PubParent => {
            // pub(parent) re-exports make an import visible only to the parent module
            // This is a valid use case for intermediate re-exports
        }
        Visibility::Private => {
            // Private imports are the default and always valid
        }
    }
}

/// Validates that use references are consistent within the file.
fn validate_use_references(file: &DolFile, _result: &mut FileValidationResult) {
    // Collect all declared names in this file
    let declared_names: HashSet<String> = file
        .declarations
        .iter()
        .map(|d| d.name().to_string())
        .collect();

    // Check that local re-exports actually exist
    for use_decl in &file.uses {
        // For local re-exports, the path should reference something
        // This is a soft check since the referenced module might be in another file
        if matches!(
            use_decl.visibility,
            Visibility::Public | Visibility::PubSpirit | Visibility::PubParent
        ) && matches!(use_decl.source, ImportSource::Local)
        {
            let full_path = use_decl.path.join(".");
            if !full_path.is_empty() && !declared_names.contains(&full_path) {
                // This is just informational - the import might reference another module
                // _result.add_warning(ValidationWarning::NamingConvention {
                //     name: full_path,
                //     suggestion: "re-exported item not found in this file".to_string(),
                // });
            }
        }
    }
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

        // SexVars should be SCREAMING_SNAKE_CASE like constants
        Declaration::SexVar(_) => {}
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
        Declaration::Evolution(_)
        | Declaration::Function(_)
        | Declaration::Const(_)
        | Declaration::SexVar(_) => return, // Different structure
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
fn validate_gene(gene: &Gen, result: &mut ValidationResult) {
    // Genes should only contain has, is, derives from, requires statements
    for stmt in &gene.statements {
        match stmt {
            Statement::Has { .. }
            | Statement::Is { .. }
            | Statement::DerivesFrom { .. }
            | Statement::Requires { .. } => {}
            Statement::HasField(field) => {
                // Validate CRDT annotations if present
                if let Some(ref crdt) = field.crdt_annotation {
                    validate_crdt_type_compatibility(field, crdt, result);
                }
            }
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

    // Validate CRDT constraints
    validate_crdt_constraints(gene, result);
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
fn validate_constraint(constraint: &Rule, result: &mut ValidationResult) {
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
fn validate_evolution(evolution: &Evo, result: &mut ValidationResult) {
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

// === CRDT Validation (RFC-001) ===

/// Validates CRDT type-strategy compatibility based on RFC-001 Table 4.1.
///
/// This function checks whether a CRDT strategy can be applied to a given field type.
/// The compatibility matrix is defined in RFC-001 Section 4.
fn validate_crdt_type_compatibility(
    field: &HasField,
    crdt: &CrdtAnnotation,
    result: &mut ValidationResult,
) {
    let is_compatible = match (&field.type_, &crdt.strategy) {
        // String strategies (both String and string)
        (TypeExpr::Named(name), CrdtStrategy::Immutable) if is_string_type(name) => true,
        (TypeExpr::Named(name), CrdtStrategy::Lww) if is_string_type(name) => true,
        (TypeExpr::Named(name), CrdtStrategy::Peritext) if is_string_type(name) => true,
        (TypeExpr::Named(name), CrdtStrategy::MvRegister) if is_string_type(name) => true,

        // Integer strategies (i8, i16, i32, i64, i128, u8, u16, u32, u64, u128)
        (TypeExpr::Named(name), CrdtStrategy::Immutable) if is_integer_type(name) => true,
        (TypeExpr::Named(name), CrdtStrategy::Lww) if is_integer_type(name) => true,
        (TypeExpr::Named(name), CrdtStrategy::PnCounter) if is_integer_type(name) => true,
        (TypeExpr::Named(name), CrdtStrategy::MvRegister) if is_integer_type(name) => true,

        // Float strategies (f32, f64, Float32, Float64)
        (TypeExpr::Named(name), CrdtStrategy::Immutable) if is_float_type(name) => true,
        (TypeExpr::Named(name), CrdtStrategy::Lww) if is_float_type(name) => true,
        (TypeExpr::Named(name), CrdtStrategy::MvRegister) if is_float_type(name) => true,

        // Bool strategies
        (TypeExpr::Named(name), CrdtStrategy::Immutable) if is_bool_type(name) => true,
        (TypeExpr::Named(name), CrdtStrategy::Lww) if is_bool_type(name) => true,
        (TypeExpr::Named(name), CrdtStrategy::MvRegister) if is_bool_type(name) => true,

        // Set strategies
        (TypeExpr::Generic { name, .. }, CrdtStrategy::Immutable) if name == "Set" => true,
        (TypeExpr::Generic { name, .. }, CrdtStrategy::OrSet) if name == "Set" => true,
        (TypeExpr::Generic { name, .. }, CrdtStrategy::MvRegister) if name == "Set" => true,

        // Vec/List strategies
        (TypeExpr::Generic { name, .. }, CrdtStrategy::Immutable)
            if name == "Vec" || name == "List" =>
        {
            true
        }
        (TypeExpr::Generic { name, .. }, CrdtStrategy::Lww) if name == "Vec" || name == "List" => {
            true
        }
        (TypeExpr::Generic { name, .. }, CrdtStrategy::Rga) if name == "Vec" || name == "List" => {
            true
        }
        (TypeExpr::Generic { name, .. }, CrdtStrategy::MvRegister)
            if name == "Vec" || name == "List" =>
        {
            true
        }

        // Option/Result strategies - all strategies can wrap these
        (TypeExpr::Generic { name, .. }, _) if name == "Option" || name == "Result" => true,

        // Map strategies
        (TypeExpr::Generic { name, .. }, CrdtStrategy::Immutable) if name == "Map" => true,
        (TypeExpr::Generic { name, .. }, CrdtStrategy::Lww) if name == "Map" => true,
        (TypeExpr::Generic { name, .. }, CrdtStrategy::MvRegister) if name == "Map" => true,

        // Tuple types can use immutable, lww, or mv_register
        (TypeExpr::Tuple(_), CrdtStrategy::Immutable) => true,
        (TypeExpr::Tuple(_), CrdtStrategy::Lww) => true,
        (TypeExpr::Tuple(_), CrdtStrategy::MvRegister) => true,

        // Custom/unknown named types are allowed with these "universal" strategies
        // (for extensibility with user-defined types)
        (TypeExpr::Named(_), CrdtStrategy::Immutable) => true,
        (TypeExpr::Named(_), CrdtStrategy::Lww) => true,
        (TypeExpr::Named(_), CrdtStrategy::MvRegister) => true,

        // All other combinations are incompatible
        _ => false,
    };

    if !is_compatible {
        let type_str = format_type_expr(&field.type_);
        let suggestion = suggest_valid_strategies(&field.type_);

        result.add_error(ValidationError::IncompatibleCrdtStrategy {
            field: field.name.clone(),
            type_: type_str,
            strategy: format!("{:?}", crdt.strategy),
            suggestion,
            span: field.span,
        });
    }
}

/// Checks if a type name represents a string type.
fn is_string_type(name: &str) -> bool {
    matches!(name, "String" | "string")
}

/// Checks if a type name represents a boolean type.
fn is_bool_type(name: &str) -> bool {
    matches!(name, "Bool" | "bool")
}

/// Checks if a type name represents an integer type.
fn is_integer_type(name: &str) -> bool {
    matches!(
        name,
        "i8" | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "Int8"
            | "Int16"
            | "Int32"
            | "Int64"
            | "Int128"
            | "UInt8"
            | "UInt16"
            | "UInt32"
            | "UInt64"
            | "UInt128"
            | "Int"
            | "int"
    )
}

/// Checks if a type name represents a floating-point type.
fn is_float_type(name: &str) -> bool {
    matches!(
        name,
        "f32" | "f64" | "Float32" | "Float64" | "Float" | "float"
    )
}

/// Formats a TypeExpr for display in error messages.
fn format_type_expr(type_expr: &TypeExpr) -> String {
    match type_expr {
        TypeExpr::Named(name) => name.clone(),
        TypeExpr::Generic { name, args } => {
            let args_str = args
                .iter()
                .map(format_type_expr)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}<{}>", name, args_str)
        }
        TypeExpr::Function {
            params,
            return_type,
        } => {
            let params_str = params
                .iter()
                .map(format_type_expr)
                .collect::<Vec<_>>()
                .join(", ");
            format!("({}) -> {}", params_str, format_type_expr(return_type))
        }
        TypeExpr::Tuple(types) => {
            let types_str = types
                .iter()
                .map(format_type_expr)
                .collect::<Vec<_>>()
                .join(", ");
            format!("({})", types_str)
        }
        TypeExpr::Never => "!".to_string(),
        TypeExpr::Enum { .. } => "enum { ... }".to_string(),
    }
}

/// Suggests valid CRDT strategies for a given type.
fn suggest_valid_strategies(type_expr: &TypeExpr) -> String {
    match type_expr {
        TypeExpr::Named(name) if is_string_type(name) => {
            "Valid strategies for String: immutable, lww, peritext, mv_register".to_string()
        }
        TypeExpr::Named(name) if is_integer_type(name) => {
            "Valid strategies for integers: immutable, lww, pn_counter, mv_register".to_string()
        }
        TypeExpr::Named(name) if is_float_type(name) => {
            "Valid strategies for floats: immutable, lww, mv_register".to_string()
        }
        TypeExpr::Named(name) if is_bool_type(name) => {
            "Valid strategies for Bool: immutable, lww, mv_register".to_string()
        }
        TypeExpr::Generic { name, .. } if name == "Set" => {
            "Valid strategies for Set: immutable, or_set, mv_register".to_string()
        }
        TypeExpr::Generic { name, .. } if name == "Vec" || name == "List" => {
            "Valid strategies for Vec/List: immutable, lww, rga, mv_register".to_string()
        }
        TypeExpr::Generic { name, .. } if name == "Map" => {
            "Valid strategies for Map: immutable, lww, mv_register".to_string()
        }
        _ => "Valid strategies depend on the specific type".to_string(),
    }
}

/// Categorizes a constraint based on its compatibility with CRDT semantics.
///
/// This implements the three-category framework from RFC-001 Section 5:
/// - CrdtSafe: Constraint is enforced by the CRDT strategy itself
/// - EventuallyConsistent: Constraint may temporarily violate during partition
/// - StrongConsistency: Constraint requires coordination (escrow/BFT)
#[derive(Debug, PartialEq, Eq)]
enum ConstraintCategory {
    /// Category A: Constraint is CRDT-safe (enforced by merge strategy)
    CrdtSafe,
    /// Category B: Constraint is eventually consistent
    EventuallyConsistent,
    /// Category C: Constraint requires strong consistency
    StrongConsistency,
}

/// Categorizes a constraint based on its semantic properties.
///
/// This is a simplified heuristic that analyzes constraint patterns.
/// A full implementation would require constraint expression parsing.
fn categorize_constraint(constraint_name: &str, _field_name: &str) -> ConstraintCategory {
    // Heuristics based on constraint naming and common patterns
    let lower_name = constraint_name.to_lowercase();

    // Immutability and append-only constraints are CRDT-safe
    if lower_name.contains("immutable")
        || lower_name.contains("append")
        || lower_name.contains("monotonic")
    {
        return ConstraintCategory::CrdtSafe;
    }

    // Uniqueness and escrow constraints require strong consistency
    if lower_name.contains("unique")
        || lower_name.contains("escrow")
        || lower_name.contains("balance")
        || lower_name.contains("capacity")
        || lower_name.contains("quota")
    {
        return ConstraintCategory::StrongConsistency;
    }

    // Bounds and cardinality constraints are eventually consistent
    if lower_name.contains("bound")
        || lower_name.contains("limit")
        || lower_name.contains("count")
        || lower_name.contains("size")
    {
        return ConstraintCategory::EventuallyConsistent;
    }

    // Default to eventually consistent for safety
    ConstraintCategory::EventuallyConsistent
}

/// Validates constraint-CRDT compatibility for genes with CRDT annotations.
///
/// This checks that constraints on CRDT-annotated fields are compatible
/// with the distributed merge semantics of the chosen CRDT strategy.
fn validate_crdt_constraints(gene: &Gen, result: &mut ValidationResult) {
    // Collect all constraints (we would need to pass these from the broader context)
    // For now, we'll look for constraints within the gene's statements
    for stmt in &gene.statements {
        if let Statement::HasField(field) = stmt {
            if let Some(ref _crdt) = field.crdt_annotation {
                if let Some(ref constraint_expr) = field.constraint {
                    // Analyze the constraint expression
                    let constraint_name = format!("constraint on {}", field.name);
                    let category = categorize_constraint(&constraint_name, &field.name);

                    match category {
                        ConstraintCategory::CrdtSafe => {
                            // No warning - constraint is safe
                        }
                        ConstraintCategory::EventuallyConsistent => {
                            result.add_warning(ValidationWarning::EventuallyConsistent {
                                constraint: constraint_name,
                                field: field.name.clone(),
                                message: format!(
                                    "Constraint may temporarily violate during network partition. Expression: {:?}",
                                    constraint_expr
                                ),
                                span: field.span,
                            });
                        }
                        ConstraintCategory::StrongConsistency => {
                            result.add_warning(ValidationWarning::RequiresCoordination {
                                constraint: constraint_name,
                                field: field.name.clone(),
                                message: "Constraint requires coordination to maintain in distributed system".to_string(),
                                suggestion: "Consider using escrow pattern from RFC-001 Section 5.4 or removing CRDT annotation".to_string(),
                                span: field.span,
                            });
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_gene(name: &str, exegesis: &str) -> Declaration {
        Declaration::Gene(Gen {
            visibility: Visibility::default(),
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

    // === File Validation Tests ===

    fn make_use_decl(visibility: Visibility, source: ImportSource, path: Vec<&str>) -> UseDecl {
        UseDecl {
            visibility,
            source,
            path: path.into_iter().map(|s| s.to_string()).collect(),
            items: UseItems::Single,
            alias: None,
            span: Span::default(),
        }
    }

    #[test]
    fn test_validate_file_empty() {
        let file = DolFile {
            module: None,
            uses: vec![],
            declarations: vec![],
        };
        let result = validate_file(&file);
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_file_with_module() {
        let file = DolFile {
            module: Some(ModuleDecl {
                path: vec!["container".to_string()],
                version: None,
                span: Span::default(),
            }),
            uses: vec![],
            declarations: vec![],
        };
        let result = validate_file(&file);
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_file_with_versioned_module() {
        let file = DolFile {
            module: Some(ModuleDecl {
                path: vec!["container".to_string(), "lib".to_string()],
                version: Some(Version {
                    major: 1,
                    minor: 0,
                    patch: 0,
                    suffix: None,
                }),
                span: Span::default(),
            }),
            uses: vec![],
            declarations: vec![],
        };
        let result = validate_file(&file);
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_file_with_zero_version_warning() {
        let file = DolFile {
            module: Some(ModuleDecl {
                path: vec!["test".to_string()],
                version: Some(Version {
                    major: 0,
                    minor: 0,
                    patch: 0,
                    suffix: None,
                }),
                span: Span::default(),
            }),
            uses: vec![],
            declarations: vec![],
        };
        let result = validate_file(&file);
        assert!(result.is_valid()); // Still valid, just warning
        assert!(result.has_warnings());
    }

    #[test]
    fn test_validate_local_use() {
        let file = DolFile {
            module: None,
            uses: vec![make_use_decl(
                Visibility::Private,
                ImportSource::Local,
                vec!["container", "state"],
            )],
            declarations: vec![],
        };
        let result = validate_file(&file);
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_pub_use() {
        let file = DolFile {
            module: None,
            uses: vec![make_use_decl(
                Visibility::Public,
                ImportSource::Local,
                vec!["container", "Container"],
            )],
            declarations: vec![],
        };
        let result = validate_file(&file);
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_registry_use() {
        let file = DolFile {
            module: None,
            uses: vec![make_use_decl(
                Visibility::Private,
                ImportSource::Registry {
                    org: "univrs".to_string(),
                    package: "std".to_string(),
                    version: None,
                },
                vec!["io"],
            )],
            declarations: vec![],
        };
        let result = validate_file(&file);
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_duplicate_use_warning() {
        let file = DolFile {
            module: None,
            uses: vec![
                make_use_decl(Visibility::Private, ImportSource::Local, vec!["container"]),
                make_use_decl(Visibility::Private, ImportSource::Local, vec!["container"]),
            ],
            declarations: vec![],
        };
        let result = validate_file(&file);
        assert!(result.is_valid()); // Still valid, just warning
        assert!(result.has_warnings());
    }

    #[test]
    fn test_validate_empty_registry_org_error() {
        let file = DolFile {
            module: None,
            uses: vec![make_use_decl(
                Visibility::Private,
                ImportSource::Registry {
                    org: "".to_string(),
                    package: "std".to_string(),
                    version: None,
                },
                vec![],
            )],
            declarations: vec![],
        };
        let result = validate_file(&file);
        assert!(!result.is_valid());
    }

    #[test]
    fn test_validate_https_url_format() {
        // Valid HTTPS URL
        let file = DolFile {
            module: None,
            uses: vec![make_use_decl(
                Visibility::Private,
                ImportSource::Https {
                    url: "https://example.com/module.dol".to_string(),
                    sha256: None,
                },
                vec![],
            )],
            declarations: vec![],
        };
        let result = validate_file(&file);
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_https_missing_protocol_error() {
        let file = DolFile {
            module: None,
            uses: vec![make_use_decl(
                Visibility::Private,
                ImportSource::Https {
                    url: "example.com/module.dol".to_string(), // Missing https://
                    sha256: None,
                },
                vec![],
            )],
            declarations: vec![],
        };
        let result = validate_file(&file);
        assert!(!result.is_valid());
    }

    #[test]
    fn test_file_validation_result_all_errors() {
        let mut result = FileValidationResult::new();
        result.add_error(ValidationError::InvalidIdentifier {
            name: "test".to_string(),
            reason: "test error".to_string(),
        });
        let errors = result.all_errors();
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_file_validation_result_all_warnings() {
        let mut result = FileValidationResult::new();
        result.add_warning(ValidationWarning::ShortExegesis {
            length: 5,
            span: Span::default(),
        });
        let warnings = result.all_warnings();
        assert_eq!(warnings.len(), 1);
    }

    // === CRDT Validation Tests ===

    #[test]
    fn test_crdt_string_immutable_compatible() {
        let field = HasField {
            name: "id".to_string(),
            type_: TypeExpr::Named("String".to_string()),
            default: None,
            constraint: None,
            crdt_annotation: Some(CrdtAnnotation {
                strategy: CrdtStrategy::Immutable,
                options: vec![],
                span: Span::default(),
            }),
            personal: false,
            span: Span::default(),
        };

        let mut result = ValidationResult::new("test");
        if let Some(ref crdt) = field.crdt_annotation {
            validate_crdt_type_compatibility(&field, crdt, &mut result);
        }

        assert!(result.is_valid());
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_crdt_string_peritext_compatible() {
        let field = HasField {
            name: "content".to_string(),
            type_: TypeExpr::Named("String".to_string()),
            default: None,
            constraint: None,
            crdt_annotation: Some(CrdtAnnotation {
                strategy: CrdtStrategy::Peritext,
                options: vec![],
                span: Span::default(),
            }),
            personal: false,
            span: Span::default(),
        };

        let mut result = ValidationResult::new("test");
        if let Some(ref crdt) = field.crdt_annotation {
            validate_crdt_type_compatibility(&field, crdt, &mut result);
        }

        assert!(result.is_valid());
    }

    #[test]
    fn test_crdt_integer_pn_counter_compatible() {
        let field = HasField {
            name: "count".to_string(),
            type_: TypeExpr::Named("i32".to_string()),
            default: None,
            constraint: None,
            crdt_annotation: Some(CrdtAnnotation {
                strategy: CrdtStrategy::PnCounter,
                options: vec![],
                span: Span::default(),
            }),
            personal: false,
            span: Span::default(),
        };

        let mut result = ValidationResult::new("test");
        if let Some(ref crdt) = field.crdt_annotation {
            validate_crdt_type_compatibility(&field, crdt, &mut result);
        }

        assert!(result.is_valid());
    }

    #[test]
    fn test_crdt_set_or_set_compatible() {
        let field = HasField {
            name: "tags".to_string(),
            type_: TypeExpr::Generic {
                name: "Set".to_string(),
                args: vec![TypeExpr::Named("String".to_string())],
            },
            default: None,
            constraint: None,
            crdt_annotation: Some(CrdtAnnotation {
                strategy: CrdtStrategy::OrSet,
                options: vec![],
                span: Span::default(),
            }),
            personal: false,
            span: Span::default(),
        };

        let mut result = ValidationResult::new("test");
        if let Some(ref crdt) = field.crdt_annotation {
            validate_crdt_type_compatibility(&field, crdt, &mut result);
        }

        assert!(result.is_valid());
    }

    #[test]
    fn test_crdt_vec_rga_compatible() {
        let field = HasField {
            name: "items".to_string(),
            type_: TypeExpr::Generic {
                name: "Vec".to_string(),
                args: vec![TypeExpr::Named("String".to_string())],
            },
            default: None,
            constraint: None,
            crdt_annotation: Some(CrdtAnnotation {
                strategy: CrdtStrategy::Rga,
                options: vec![],
                span: Span::default(),
            }),
            personal: false,
            span: Span::default(),
        };

        let mut result = ValidationResult::new("test");
        if let Some(ref crdt) = field.crdt_annotation {
            validate_crdt_type_compatibility(&field, crdt, &mut result);
        }

        assert!(result.is_valid());
    }

    #[test]
    fn test_crdt_incompatible_strategy() {
        // OR-Set cannot be used with String
        let field = HasField {
            name: "name".to_string(),
            type_: TypeExpr::Named("String".to_string()),
            default: None,
            constraint: None,
            crdt_annotation: Some(CrdtAnnotation {
                strategy: CrdtStrategy::OrSet,
                options: vec![],
                span: Span::default(),
            }),
            personal: false,
            span: Span::default(),
        };

        let mut result = ValidationResult::new("test");
        if let Some(ref crdt) = field.crdt_annotation {
            validate_crdt_type_compatibility(&field, crdt, &mut result);
        }

        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
        match &result.errors[0] {
            ValidationError::IncompatibleCrdtStrategy { field, .. } => {
                assert_eq!(field, "name");
            }
            _ => panic!("Expected IncompatibleCrdtStrategy error"),
        }
    }

    #[test]
    fn test_crdt_incompatible_pn_counter_on_string() {
        // PN-Counter cannot be used with String
        let field = HasField {
            name: "text".to_string(),
            type_: TypeExpr::Named("String".to_string()),
            default: None,
            constraint: None,
            crdt_annotation: Some(CrdtAnnotation {
                strategy: CrdtStrategy::PnCounter,
                options: vec![],
                span: Span::default(),
            }),
            personal: false,
            span: Span::default(),
        };

        let mut result = ValidationResult::new("test");
        if let Some(ref crdt) = field.crdt_annotation {
            validate_crdt_type_compatibility(&field, crdt, &mut result);
        }

        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_categorize_constraint_immutable() {
        let category = categorize_constraint("immutable_id", "id");
        assert_eq!(category, ConstraintCategory::CrdtSafe);
    }

    #[test]
    fn test_categorize_constraint_unique() {
        let category = categorize_constraint("unique_username", "username");
        assert_eq!(category, ConstraintCategory::StrongConsistency);
    }

    #[test]
    fn test_categorize_constraint_escrow() {
        let category = categorize_constraint("escrow_balance", "balance");
        assert_eq!(category, ConstraintCategory::StrongConsistency);
    }

    #[test]
    fn test_categorize_constraint_bounded() {
        let category = categorize_constraint("bounded_count", "count");
        assert_eq!(category, ConstraintCategory::EventuallyConsistent);
    }

    #[test]
    fn test_is_integer_type() {
        assert!(is_integer_type("i32"));
        assert!(is_integer_type("u64"));
        assert!(is_integer_type("Int32"));
        assert!(is_integer_type("UInt64"));
        assert!(!is_integer_type("String"));
        assert!(!is_integer_type("f32"));
    }

    #[test]
    fn test_is_float_type() {
        assert!(is_float_type("f32"));
        assert!(is_float_type("f64"));
        assert!(is_float_type("Float32"));
        assert!(is_float_type("Float64"));
        assert!(!is_float_type("i32"));
        assert!(!is_float_type("String"));
    }

    #[test]
    fn test_format_type_expr_simple() {
        let type_expr = TypeExpr::Named("String".to_string());
        assert_eq!(format_type_expr(&type_expr), "String");
    }

    #[test]
    fn test_format_type_expr_generic() {
        let type_expr = TypeExpr::Generic {
            name: "Vec".to_string(),
            args: vec![TypeExpr::Named("i32".to_string())],
        };
        assert_eq!(format_type_expr(&type_expr), "Vec<i32>");
    }

    #[test]
    fn test_suggest_valid_strategies_string() {
        let type_expr = TypeExpr::Named("String".to_string());
        let suggestion = suggest_valid_strategies(&type_expr);
        assert!(suggestion.contains("immutable"));
        assert!(suggestion.contains("lww"));
        assert!(suggestion.contains("peritext"));
    }

    #[test]
    fn test_suggest_valid_strategies_integer() {
        let type_expr = TypeExpr::Named("i32".to_string());
        let suggestion = suggest_valid_strategies(&type_expr);
        assert!(suggestion.contains("pn_counter"));
    }

    #[test]
    fn test_suggest_valid_strategies_set() {
        let type_expr = TypeExpr::Generic {
            name: "Set".to_string(),
            args: vec![],
        };
        let suggestion = suggest_valid_strategies(&type_expr);
        assert!(suggestion.contains("or_set"));
    }
}
