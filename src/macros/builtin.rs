//! Built-in macros for Metal DOL 2.0.
//!
//! This module provides the standard macro library that ships with DOL,
//! including commonly used macros for code generation, string manipulation,
//! and conditional compilation.
//!
//! # Available Macros
//!
//! | Macro | Description | Example |
//! |-------|-------------|---------|
//! | `#derive` | Generate trait implementations | `#[derive(Debug, Clone)]` |
//! | `#stringify` | Convert expression to string | `#stringify(foo.bar)` → `"foo.bar"` |
//! | `#concat` | Concatenate strings | `#concat("a", "b")` → `"ab"` |
//! | `#env` | Access environment variable | `#env("HOME")` |
//! | `#cfg` | Conditional compilation | `#cfg(debug)` |
//! | `#file` | Current file path | `#file()` |
//! | `#line` | Current line number | `#line()` |
//! | `#column` | Current column number | `#column()` |
//! | `#include_str` | Include file contents | `#include_str("path/to/file")` |
//! | `#debug_assert` | Debug-only assertion | `#debug_assert(x > 0)` |
//! | `#todo` | Mark unimplemented code | `#todo("implement later")` |
//! | `#unreachable` | Mark unreachable code | `#unreachable()` |
//! | `#assert` | Assert condition | `#assert(x > 0)` |
//! | `#assert_eq` | Assert equality | `#assert_eq(result, expected)` |
//! | `#assert_ne` | Assert inequality | `#assert_ne(status, "error")` |
//! | `#format` | String formatting | `#format("Hello, {}!", name)` |
//! | `#dbg` | Debug print expression | `#dbg(calculate())` |
//! | `#compile_error` | Compile-time error | `#compile_error("not supported")` |
//! | `#vec` | Create vector literal | `#vec(1, 2, 3)` |
//! | `#option_env` | Optional env access | `#option_env("VAR")` |

use super::{Macro, MacroContext, MacroError, MacroInput, MacroOutput};
use crate::ast::{Expr, Literal};
use std::collections::HashMap;
use std::sync::Arc;

/// Registry of built-in macros.
///
/// Provides access to all standard macros that ship with DOL.
pub struct BuiltinMacros {
    macros: HashMap<String, Arc<dyn Macro>>,
}

impl BuiltinMacros {
    /// Creates a new registry with all built-in macros.
    pub fn new() -> Self {
        let mut macros: HashMap<String, Arc<dyn Macro>> = HashMap::new();

        // Register all built-in macros
        macros.insert("derive".to_string(), Arc::new(DeriveMacro));
        macros.insert("stringify".to_string(), Arc::new(StringifyMacro));
        macros.insert("concat".to_string(), Arc::new(ConcatMacro));
        macros.insert("env".to_string(), Arc::new(EnvMacro));
        macros.insert("cfg".to_string(), Arc::new(CfgMacro));
        macros.insert("file".to_string(), Arc::new(FileMacro));
        macros.insert("line".to_string(), Arc::new(LineMacro));
        macros.insert("column".to_string(), Arc::new(ColumnMacro));
        macros.insert("option_env".to_string(), Arc::new(OptionEnvMacro));
        macros.insert("include_str".to_string(), Arc::new(IncludeStrMacro));
        macros.insert("debug_assert".to_string(), Arc::new(DebugAssertMacro));
        macros.insert("todo".to_string(), Arc::new(TodoMacro));
        macros.insert("unreachable".to_string(), Arc::new(UnreachableMacro));

        // Assertion macros
        macros.insert("assert".to_string(), Arc::new(AssertMacro));
        macros.insert("assert_eq".to_string(), Arc::new(AssertEqMacro));
        macros.insert("assert_ne".to_string(), Arc::new(AssertNeMacro));

        // Utility macros
        macros.insert("format".to_string(), Arc::new(FormatMacro));
        macros.insert("dbg".to_string(), Arc::new(DbgMacro));
        macros.insert("compile_error".to_string(), Arc::new(CompileErrorMacro));
        macros.insert("vec".to_string(), Arc::new(VecMacro));

        Self { macros }
    }

    /// Gets a macro by name.
    pub fn get(&self, name: &str) -> Option<Arc<dyn Macro>> {
        self.macros.get(name).cloned()
    }

    /// Returns an iterator over all macro names.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.macros.keys().map(|s| s.as_str())
    }

    /// Returns the number of registered macros.
    pub fn len(&self) -> usize {
        self.macros.len()
    }

    /// Returns true if there are no registered macros.
    pub fn is_empty(&self) -> bool {
        self.macros.is_empty()
    }
}

impl Default for BuiltinMacros {
    fn default() -> Self {
        Self::new()
    }
}

// === Built-in Macro Implementations ===

/// `#derive` - Generate trait implementations.
///
/// Used as an attribute macro to derive standard trait implementations.
///
/// # Syntax
///
/// ```dol
/// #[derive(Debug, Clone)]
/// gene container.exists { ... }
/// ```
///
/// # Supported Derives
///
/// - `Debug`: Generate debug formatting
/// - `Clone`: Generate cloning capability
/// - `PartialEq`: Generate equality comparison
/// - `Eq`: Generate total equality
/// - `Hash`: Generate hashing
/// - `Default`: Generate default value
pub struct DeriveMacro;

impl Macro for DeriveMacro {
    fn name(&self) -> &str {
        "derive"
    }

    fn expand(&self, input: MacroInput, _ctx: &MacroContext) -> Result<MacroOutput, MacroError> {
        // For now, #derive is primarily an attribute macro
        // that marks declarations for trait generation
        match input {
            MacroInput::IdentList(_traits) => {
                // Return the declaration unchanged for now
                // Actual implementation would generate trait impls
                Ok(MacroOutput::none())
            }
            MacroInput::Declaration(decl) => {
                // Pass through the declaration
                Ok(MacroOutput::Declaration(decl))
            }
            _ => Err(MacroError::invalid_argument(
                "derive expects a list of trait names",
            )),
        }
    }

    fn description(&self) -> &str {
        "Generate trait implementations for a declaration"
    }

    fn is_attribute_macro(&self) -> bool {
        true
    }

    fn is_expr_macro(&self) -> bool {
        false
    }

    fn min_args(&self) -> usize {
        1
    }
}

/// `#stringify` - Convert an expression to a string literal.
///
/// Takes any expression and returns its source code as a string.
///
/// # Syntax
///
/// ```dol
/// let name = #stringify(container.exists);
/// // name == "container.exists"
/// ```
pub struct StringifyMacro;

impl Macro for StringifyMacro {
    fn name(&self) -> &str {
        "stringify"
    }

    fn expand(&self, input: MacroInput, _ctx: &MacroContext) -> Result<MacroOutput, MacroError> {
        match input {
            MacroInput::Expr(expr) => {
                let text = stringify_expr(&expr);
                Ok(MacroOutput::expr(Expr::Literal(Literal::String(text))))
            }
            MacroInput::Ident(name) => Ok(MacroOutput::expr(Expr::Literal(Literal::String(name)))),
            MacroInput::IdentList(names) => {
                let text = names.join(", ");
                Ok(MacroOutput::expr(Expr::Literal(Literal::String(text))))
            }
            MacroInput::ExprList(exprs) => {
                let text: Vec<String> = exprs.iter().map(stringify_expr).collect();
                Ok(MacroOutput::expr(Expr::Literal(Literal::String(
                    text.join(", "),
                ))))
            }
            MacroInput::Empty => Ok(MacroOutput::expr(Expr::Literal(Literal::String(
                String::new(),
            )))),
            _ => Err(MacroError::invalid_argument(
                "stringify expects an expression or identifier",
            )),
        }
    }

    fn description(&self) -> &str {
        "Convert an expression to its string representation"
    }

    fn min_args(&self) -> usize {
        1
    }

    fn max_args(&self) -> Option<usize> {
        Some(1)
    }
}

/// `#concat` - Concatenate string literals.
///
/// Takes multiple string arguments and joins them into one.
///
/// # Syntax
///
/// ```dol
/// let name = #concat("prefix_", "middle_", "suffix");
/// // name == "prefix_middle_suffix"
/// ```
pub struct ConcatMacro;

impl Macro for ConcatMacro {
    fn name(&self) -> &str {
        "concat"
    }

    fn expand(&self, input: MacroInput, _ctx: &MacroContext) -> Result<MacroOutput, MacroError> {
        match input {
            MacroInput::ExprList(exprs) => {
                let mut result = String::new();
                for expr in exprs {
                    match expr {
                        Expr::Literal(Literal::String(s)) => result.push_str(&s),
                        Expr::Literal(Literal::Int(n)) => result.push_str(&n.to_string()),
                        Expr::Literal(Literal::Float(f)) => result.push_str(&f.to_string()),
                        Expr::Literal(Literal::Bool(b)) => result.push_str(&b.to_string()),
                        Expr::Identifier(name) => result.push_str(&name),
                        _ => {
                            return Err(MacroError::type_error(
                                "string literal",
                                "complex expression",
                            ))
                        }
                    }
                }
                Ok(MacroOutput::expr(Expr::Literal(Literal::String(result))))
            }
            MacroInput::IdentList(names) => {
                let result = names.join("");
                Ok(MacroOutput::expr(Expr::Literal(Literal::String(result))))
            }
            MacroInput::Expr(expr) => match *expr {
                Expr::Literal(Literal::String(s)) => {
                    Ok(MacroOutput::expr(Expr::Literal(Literal::String(s))))
                }
                _ => Err(MacroError::type_error("string literal", "expression")),
            },
            MacroInput::Empty => Ok(MacroOutput::expr(Expr::Literal(Literal::String(
                String::new(),
            )))),
            _ => Err(MacroError::invalid_argument(
                "concat expects string literals or identifiers",
            )),
        }
    }

    fn description(&self) -> &str {
        "Concatenate strings at compile time"
    }

    fn min_args(&self) -> usize {
        0
    }
}

/// `#env` - Access an environment variable at compile time.
///
/// Returns the value of an environment variable as a string literal.
/// Fails compilation if the variable is not set.
///
/// # Syntax
///
/// ```dol
/// let home = #env("HOME");
/// ```
pub struct EnvMacro;

impl Macro for EnvMacro {
    fn name(&self) -> &str {
        "env"
    }

    fn expand(&self, input: MacroInput, ctx: &MacroContext) -> Result<MacroOutput, MacroError> {
        let var_name = match input {
            MacroInput::Expr(expr) => match *expr {
                Expr::Literal(Literal::String(s)) => s,
                Expr::Identifier(name) => name,
                _ => {
                    return Err(MacroError::type_error(
                        "string literal or identifier",
                        "expression",
                    ))
                }
            },
            MacroInput::Ident(name) => name,
            MacroInput::ExprList(exprs) if exprs.len() == 1 => match &exprs[0] {
                Expr::Literal(Literal::String(s)) => s.clone(),
                Expr::Identifier(name) => name.clone(),
                _ => return Err(MacroError::type_error("string literal", "expression")),
            },
            _ => {
                return Err(MacroError::invalid_argument(
                    "env expects a single string argument",
                ))
            }
        };

        match ctx.get_env(&var_name) {
            Some(value) => Ok(MacroOutput::expr(Expr::Literal(Literal::String(
                value.to_string(),
            )))),
            None => Err(MacroError::new(format!(
                "environment variable '{}' not defined",
                var_name
            ))),
        }
    }

    fn description(&self) -> &str {
        "Access environment variable at compile time (fails if not set)"
    }

    fn min_args(&self) -> usize {
        1
    }

    fn max_args(&self) -> Option<usize> {
        Some(1)
    }
}

/// `#option_env` - Optionally access an environment variable.
///
/// Returns the value of an environment variable as an Option.
/// Returns None if the variable is not set (doesn't fail).
///
/// # Syntax
///
/// ```dol
/// let maybe_home = #option_env("HOME");
/// ```
pub struct OptionEnvMacro;

impl Macro for OptionEnvMacro {
    fn name(&self) -> &str {
        "option_env"
    }

    fn expand(&self, input: MacroInput, ctx: &MacroContext) -> Result<MacroOutput, MacroError> {
        let var_name = match input {
            MacroInput::Expr(expr) => match *expr {
                Expr::Literal(Literal::String(s)) => s,
                Expr::Identifier(name) => name,
                _ => return Err(MacroError::type_error("string literal", "expression")),
            },
            MacroInput::Ident(name) => name,
            MacroInput::ExprList(exprs) if exprs.len() == 1 => match &exprs[0] {
                Expr::Literal(Literal::String(s)) => s.clone(),
                _ => return Err(MacroError::type_error("string literal", "expression")),
            },
            _ => {
                return Err(MacroError::invalid_argument(
                    "option_env expects a single string argument",
                ))
            }
        };

        match ctx.get_env(&var_name) {
            Some(value) => {
                // Return Some(value) represented as a call
                Ok(MacroOutput::expr(Expr::Call {
                    callee: Box::new(Expr::Identifier("Some".to_string())),
                    args: vec![Expr::Literal(Literal::String(value.to_string()))],
                }))
            }
            None => {
                // Return None
                Ok(MacroOutput::expr(Expr::Identifier("None".to_string())))
            }
        }
    }

    fn description(&self) -> &str {
        "Access environment variable at compile time (returns None if not set)"
    }

    fn min_args(&self) -> usize {
        1
    }

    fn max_args(&self) -> Option<usize> {
        Some(1)
    }
}

/// `#cfg` - Conditional compilation based on configuration flags.
///
/// Evaluates a condition based on compilation flags and feature flags.
///
/// # Syntax
///
/// ```dol
/// #cfg(debug)       // True in debug mode
/// #cfg(feature = "async")  // True if feature enabled
/// #cfg(not(test))   // Negation
/// #cfg(all(unix, feature = "io"))  // Conjunction
/// #cfg(any(windows, macos))  // Disjunction
/// ```
pub struct CfgMacro;

impl Macro for CfgMacro {
    fn name(&self) -> &str {
        "cfg"
    }

    fn expand(&self, input: MacroInput, ctx: &MacroContext) -> Result<MacroOutput, MacroError> {
        let result = match input {
            MacroInput::Ident(name) => {
                // Simple flag or feature check: #cfg(debug) or #cfg(async)
                ctx.get_cfg(&name) || ctx.has_feature(&name)
            }
            MacroInput::Expr(expr) => {
                // Expression-based cfg
                evaluate_cfg_expr(&expr, ctx)?
            }
            MacroInput::ExprList(exprs) if exprs.len() == 1 => evaluate_cfg_expr(&exprs[0], ctx)?,
            MacroInput::Config(config) => {
                // Key-value style: feature = "name"
                if let Some(value) = config.get("feature") {
                    if let Some(feature_name) = value.as_string() {
                        ctx.has_feature(feature_name)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => {
                return Err(MacroError::invalid_argument(
                    "cfg expects a configuration predicate",
                ))
            }
        };

        Ok(MacroOutput::expr(Expr::Literal(Literal::Bool(result))))
    }

    fn description(&self) -> &str {
        "Conditional compilation based on configuration flags"
    }

    fn min_args(&self) -> usize {
        1
    }
}

/// `#file` - Current file path at compile time.
///
/// Returns the path of the current source file.
///
/// # Syntax
///
/// ```dol
/// let path = #file();
/// ```
pub struct FileMacro;

impl Macro for FileMacro {
    fn name(&self) -> &str {
        "file"
    }

    fn expand(&self, _input: MacroInput, ctx: &MacroContext) -> Result<MacroOutput, MacroError> {
        let path = ctx
            .file_path
            .clone()
            .unwrap_or_else(|| "<unknown>".to_string());
        Ok(MacroOutput::expr(Expr::Literal(Literal::String(path))))
    }

    fn description(&self) -> &str {
        "Get the current source file path"
    }

    fn max_args(&self) -> Option<usize> {
        Some(0)
    }
}

/// `#line` - Current line number at compile time.
///
/// Returns the line number where the macro is invoked.
///
/// # Syntax
///
/// ```dol
/// let ln = #line();
/// ```
pub struct LineMacro;

impl Macro for LineMacro {
    fn name(&self) -> &str {
        "line"
    }

    fn expand(&self, _input: MacroInput, ctx: &MacroContext) -> Result<MacroOutput, MacroError> {
        Ok(MacroOutput::expr(Expr::Literal(Literal::Int(
            ctx.line as i64,
        ))))
    }

    fn description(&self) -> &str {
        "Get the current source line number"
    }

    fn max_args(&self) -> Option<usize> {
        Some(0)
    }
}

/// `#column` - Current column number at compile time.
///
/// Returns the column number where the macro is invoked.
///
/// # Syntax
///
/// ```dol
/// let col = #column();
/// ```
pub struct ColumnMacro;

impl Macro for ColumnMacro {
    fn name(&self) -> &str {
        "column"
    }

    fn expand(&self, _input: MacroInput, ctx: &MacroContext) -> Result<MacroOutput, MacroError> {
        Ok(MacroOutput::expr(Expr::Literal(Literal::Int(
            ctx.column as i64,
        ))))
    }

    fn description(&self) -> &str {
        "Get the current source column number"
    }

    fn max_args(&self) -> Option<usize> {
        Some(0)
    }
}

/// `#include_str` - Include file contents as a string.
///
/// Reads a file at compile time and includes its contents as a string literal.
///
/// # Syntax
///
/// ```dol
/// let license = #include_str("LICENSE");
/// ```
pub struct IncludeStrMacro;

impl Macro for IncludeStrMacro {
    fn name(&self) -> &str {
        "include_str"
    }

    fn expand(&self, input: MacroInput, _ctx: &MacroContext) -> Result<MacroOutput, MacroError> {
        let path = match input {
            MacroInput::Expr(expr) => match *expr {
                Expr::Literal(Literal::String(s)) => s,
                _ => return Err(MacroError::type_error("string literal", "expression")),
            },
            MacroInput::ExprList(exprs) if exprs.len() == 1 => match &exprs[0] {
                Expr::Literal(Literal::String(s)) => s.clone(),
                _ => return Err(MacroError::type_error("string literal", "expression")),
            },
            _ => {
                return Err(MacroError::invalid_argument(
                    "include_str expects a file path string",
                ))
            }
        };

        match std::fs::read_to_string(&path) {
            Ok(contents) => Ok(MacroOutput::expr(Expr::Literal(Literal::String(contents)))),
            Err(e) => Err(MacroError::new(format!(
                "failed to read file '{}': {}",
                path, e
            ))),
        }
    }

    fn description(&self) -> &str {
        "Include file contents as a string at compile time"
    }

    fn min_args(&self) -> usize {
        1
    }

    fn max_args(&self) -> Option<usize> {
        Some(1)
    }
}

/// `#debug_assert` - Assert that is only checked in debug mode.
///
/// Expands to an assertion in debug builds, nothing in release.
///
/// # Syntax
///
/// ```dol
/// #debug_assert(x > 0);
/// #debug_assert(valid, "must be valid");
/// ```
pub struct DebugAssertMacro;

impl Macro for DebugAssertMacro {
    fn name(&self) -> &str {
        "debug_assert"
    }

    fn expand(&self, input: MacroInput, ctx: &MacroContext) -> Result<MacroOutput, MacroError> {
        // In release mode, expand to nothing
        if !ctx.get_cfg("debug") {
            return Ok(MacroOutput::none());
        }

        // In debug mode, expand to an if check
        match input {
            MacroInput::Expr(condition) => {
                // Generate: if !condition { panic("assertion failed") }
                Ok(MacroOutput::expr(Expr::If {
                    condition: Box::new(Expr::Unary {
                        op: crate::ast::UnaryOp::Not,
                        operand: condition,
                    }),
                    then_branch: Box::new(Expr::Call {
                        callee: Box::new(Expr::Identifier("panic".to_string())),
                        args: vec![Expr::Literal(Literal::String(
                            "assertion failed".to_string(),
                        ))],
                    }),
                    else_branch: None,
                }))
            }
            MacroInput::ExprList(exprs) if !exprs.is_empty() => {
                let condition = exprs[0].clone();
                let message = if exprs.len() > 1 {
                    match &exprs[1] {
                        Expr::Literal(Literal::String(s)) => s.clone(),
                        _ => "assertion failed".to_string(),
                    }
                } else {
                    "assertion failed".to_string()
                };

                Ok(MacroOutput::expr(Expr::If {
                    condition: Box::new(Expr::Unary {
                        op: crate::ast::UnaryOp::Not,
                        operand: Box::new(condition),
                    }),
                    then_branch: Box::new(Expr::Call {
                        callee: Box::new(Expr::Identifier("panic".to_string())),
                        args: vec![Expr::Literal(Literal::String(message))],
                    }),
                    else_branch: None,
                }))
            }
            _ => Err(MacroError::invalid_argument(
                "debug_assert expects a condition expression",
            )),
        }
    }

    fn description(&self) -> &str {
        "Assert condition in debug builds only"
    }

    fn min_args(&self) -> usize {
        1
    }
}

/// `#todo` - Mark unimplemented code.
///
/// Expands to a panic with a "not yet implemented" message.
///
/// # Syntax
///
/// ```dol
/// #todo()
/// #todo("implement login logic")
/// ```
pub struct TodoMacro;

impl Macro for TodoMacro {
    fn name(&self) -> &str {
        "todo"
    }

    fn expand(&self, input: MacroInput, _ctx: &MacroContext) -> Result<MacroOutput, MacroError> {
        let message = match input {
            MacroInput::Empty => "not yet implemented".to_string(),
            MacroInput::Expr(expr) => match *expr {
                Expr::Literal(Literal::String(s)) => format!("not yet implemented: {}", s),
                _ => "not yet implemented".to_string(),
            },
            MacroInput::ExprList(exprs) if !exprs.is_empty() => match &exprs[0] {
                Expr::Literal(Literal::String(s)) => format!("not yet implemented: {}", s),
                _ => "not yet implemented".to_string(),
            },
            _ => "not yet implemented".to_string(),
        };

        Ok(MacroOutput::expr(Expr::Call {
            callee: Box::new(Expr::Identifier("panic".to_string())),
            args: vec![Expr::Literal(Literal::String(message))],
        }))
    }

    fn description(&self) -> &str {
        "Mark code as not yet implemented"
    }
}

/// `#unreachable` - Mark code as unreachable.
///
/// Indicates that a code path should never be executed.
///
/// # Syntax
///
/// ```dol
/// #unreachable()
/// #unreachable("this case is impossible")
/// ```
pub struct UnreachableMacro;

impl Macro for UnreachableMacro {
    fn name(&self) -> &str {
        "unreachable"
    }

    fn expand(&self, input: MacroInput, _ctx: &MacroContext) -> Result<MacroOutput, MacroError> {
        let message = match input {
            MacroInput::Empty => "entered unreachable code".to_string(),
            MacroInput::Expr(expr) => match *expr {
                Expr::Literal(Literal::String(s)) => {
                    format!("entered unreachable code: {}", s)
                }
                _ => "entered unreachable code".to_string(),
            },
            MacroInput::ExprList(exprs) if !exprs.is_empty() => match &exprs[0] {
                Expr::Literal(Literal::String(s)) => {
                    format!("entered unreachable code: {}", s)
                }
                _ => "entered unreachable code".to_string(),
            },
            _ => "entered unreachable code".to_string(),
        };

        Ok(MacroOutput::expr(Expr::Call {
            callee: Box::new(Expr::Identifier("panic".to_string())),
            args: vec![Expr::Literal(Literal::String(message))],
        }))
    }

    fn description(&self) -> &str {
        "Mark code as unreachable"
    }
}

/// `#assert` - Assert a condition is true.
///
/// Always checks the condition, unlike `#debug_assert`.
///
/// # Syntax
///
/// ```dol
/// #assert(x > 0);
/// #assert(valid, "must be valid");
/// ```
pub struct AssertMacro;

impl Macro for AssertMacro {
    fn name(&self) -> &str {
        "assert"
    }

    fn expand(&self, input: MacroInput, _ctx: &MacroContext) -> Result<MacroOutput, MacroError> {
        match input {
            MacroInput::Expr(condition) => {
                let condition_str = stringify_expr(&condition);
                let message = format!("assertion failed: {}", condition_str);
                Ok(MacroOutput::expr(Expr::If {
                    condition: Box::new(Expr::Unary {
                        op: crate::ast::UnaryOp::Not,
                        operand: condition,
                    }),
                    then_branch: Box::new(Expr::Call {
                        callee: Box::new(Expr::Identifier("panic".to_string())),
                        args: vec![Expr::Literal(Literal::String(message))],
                    }),
                    else_branch: None,
                }))
            }
            MacroInput::ExprList(exprs) if !exprs.is_empty() => {
                let condition = exprs[0].clone();
                let condition_str = stringify_expr(&condition);
                let message = if exprs.len() > 1 {
                    match &exprs[1] {
                        Expr::Literal(Literal::String(s)) => s.clone(),
                        _ => format!("assertion failed: {}", condition_str),
                    }
                } else {
                    format!("assertion failed: {}", condition_str)
                };

                Ok(MacroOutput::expr(Expr::If {
                    condition: Box::new(Expr::Unary {
                        op: crate::ast::UnaryOp::Not,
                        operand: Box::new(condition),
                    }),
                    then_branch: Box::new(Expr::Call {
                        callee: Box::new(Expr::Identifier("panic".to_string())),
                        args: vec![Expr::Literal(Literal::String(message))],
                    }),
                    else_branch: None,
                }))
            }
            _ => Err(MacroError::invalid_argument(
                "assert expects a condition expression",
            )),
        }
    }

    fn description(&self) -> &str {
        "Assert condition is true (always checked)"
    }

    fn min_args(&self) -> usize {
        1
    }
}

/// `#assert_eq` - Assert two values are equal.
///
/// Compares two expressions and panics if they are not equal.
///
/// # Syntax
///
/// ```dol
/// #assert_eq(result, expected);
/// #assert_eq(x, 42, "x should be 42");
/// ```
pub struct AssertEqMacro;

impl Macro for AssertEqMacro {
    fn name(&self) -> &str {
        "assert_eq"
    }

    fn expand(&self, input: MacroInput, _ctx: &MacroContext) -> Result<MacroOutput, MacroError> {
        match input {
            MacroInput::ExprList(exprs) if exprs.len() >= 2 => {
                let left = exprs[0].clone();
                let right = exprs[1].clone();
                let left_str = stringify_expr(&left);
                let right_str = stringify_expr(&right);

                let message = if exprs.len() > 2 {
                    match &exprs[2] {
                        Expr::Literal(Literal::String(s)) => s.clone(),
                        _ => format!(
                            "assertion failed: `(left == right)`\n  left: `{}`\n right: `{}`",
                            left_str, right_str
                        ),
                    }
                } else {
                    format!(
                        "assertion failed: `(left == right)`\n  left: `{}`\n right: `{}`",
                        left_str, right_str
                    )
                };

                // Generate: if left != right { panic(message) }
                Ok(MacroOutput::expr(Expr::If {
                    condition: Box::new(Expr::Binary {
                        left: Box::new(left),
                        op: crate::ast::BinaryOp::Ne,
                        right: Box::new(right),
                    }),
                    then_branch: Box::new(Expr::Call {
                        callee: Box::new(Expr::Identifier("panic".to_string())),
                        args: vec![Expr::Literal(Literal::String(message))],
                    }),
                    else_branch: None,
                }))
            }
            _ => Err(MacroError::invalid_argument(
                "assert_eq expects two expressions to compare",
            )),
        }
    }

    fn description(&self) -> &str {
        "Assert two values are equal"
    }

    fn min_args(&self) -> usize {
        2
    }
}

/// `#assert_ne` - Assert two values are not equal.
///
/// Compares two expressions and panics if they are equal.
///
/// # Syntax
///
/// ```dol
/// #assert_ne(result, 0);
/// #assert_ne(status, "error", "should not be error");
/// ```
pub struct AssertNeMacro;

impl Macro for AssertNeMacro {
    fn name(&self) -> &str {
        "assert_ne"
    }

    fn expand(&self, input: MacroInput, _ctx: &MacroContext) -> Result<MacroOutput, MacroError> {
        match input {
            MacroInput::ExprList(exprs) if exprs.len() >= 2 => {
                let left = exprs[0].clone();
                let right = exprs[1].clone();
                let left_str = stringify_expr(&left);
                let right_str = stringify_expr(&right);

                let message = if exprs.len() > 2 {
                    match &exprs[2] {
                        Expr::Literal(Literal::String(s)) => s.clone(),
                        _ => format!(
                            "assertion failed: `(left != right)`\n  left: `{}`\n right: `{}`",
                            left_str, right_str
                        ),
                    }
                } else {
                    format!(
                        "assertion failed: `(left != right)`\n  left: `{}`\n right: `{}`",
                        left_str, right_str
                    )
                };

                // Generate: if left == right { panic(message) }
                Ok(MacroOutput::expr(Expr::If {
                    condition: Box::new(Expr::Binary {
                        left: Box::new(left),
                        op: crate::ast::BinaryOp::Eq,
                        right: Box::new(right),
                    }),
                    then_branch: Box::new(Expr::Call {
                        callee: Box::new(Expr::Identifier("panic".to_string())),
                        args: vec![Expr::Literal(Literal::String(message))],
                    }),
                    else_branch: None,
                }))
            }
            _ => Err(MacroError::invalid_argument(
                "assert_ne expects two expressions to compare",
            )),
        }
    }

    fn description(&self) -> &str {
        "Assert two values are not equal"
    }

    fn min_args(&self) -> usize {
        2
    }
}

/// `#format` - Format a string with placeholders.
///
/// Similar to Rust's `format!` macro, substitutes `{}` placeholders with values.
///
/// # Syntax
///
/// ```dol
/// let msg = #format("Hello, {}!", name);
/// let info = #format("x={}, y={}", x, y);
/// ```
pub struct FormatMacro;

impl Macro for FormatMacro {
    fn name(&self) -> &str {
        "format"
    }

    fn expand(&self, input: MacroInput, _ctx: &MacroContext) -> Result<MacroOutput, MacroError> {
        match input {
            MacroInput::ExprList(exprs) if !exprs.is_empty() => {
                let format_string = match &exprs[0] {
                    Expr::Literal(Literal::String(s)) => s.clone(),
                    _ => {
                        return Err(MacroError::type_error(
                            "string literal",
                            "expression",
                        ))
                    }
                };

                let args = &exprs[1..];

                // Count placeholders in format string
                let placeholder_count = format_string.matches("{}").count();
                if placeholder_count != args.len() {
                    return Err(MacroError::invalid_argument(&format!(
                        "format string has {} placeholders but {} arguments were provided",
                        placeholder_count,
                        args.len()
                    )));
                }

                // Build concatenation expression
                let parts: Vec<&str> = format_string.split("{}").collect();
                let mut result_exprs: Vec<Expr> = Vec::new();

                for (i, part) in parts.iter().enumerate() {
                    if !part.is_empty() {
                        result_exprs.push(Expr::Literal(Literal::String(part.to_string())));
                    }
                    if i < args.len() {
                        // Wrap arg in a to_string call for runtime formatting
                        result_exprs.push(Expr::Call {
                            callee: Box::new(Expr::Member {
                                object: Box::new(args[i].clone()),
                                field: "to_string".to_string(),
                            }),
                            args: vec![],
                        });
                    }
                }

                // Generate concat call with all parts
                Ok(MacroOutput::expr(Expr::Call {
                    callee: Box::new(Expr::Identifier("concat".to_string())),
                    args: result_exprs,
                }))
            }
            MacroInput::Expr(expr) => match *expr {
                Expr::Literal(Literal::String(s)) => {
                    // No placeholders, just return the string
                    Ok(MacroOutput::expr(Expr::Literal(Literal::String(s))))
                }
                _ => Err(MacroError::type_error("string literal", "expression")),
            },
            _ => Err(MacroError::invalid_argument(
                "format expects a format string and arguments",
            )),
        }
    }

    fn description(&self) -> &str {
        "Format a string with placeholders"
    }

    fn min_args(&self) -> usize {
        1
    }
}

/// `#dbg` - Debug output macro.
///
/// Prints the expression and its value, returning the value.
/// Useful for quick debugging.
///
/// # Syntax
///
/// ```dol
/// let x = #dbg(calculate());  // prints: [file:line] calculate() = <value>
/// ```
pub struct DbgMacro;

impl Macro for DbgMacro {
    fn name(&self) -> &str {
        "dbg"
    }

    fn expand(&self, input: MacroInput, ctx: &MacroContext) -> Result<MacroOutput, MacroError> {
        match input {
            MacroInput::Expr(expr) => {
                let expr_str = stringify_expr(&expr);
                let location = format!(
                    "[{}:{}]",
                    ctx.file_path.as_deref().unwrap_or("<unknown>"),
                    ctx.line
                );

                // Generate: { let __dbg_tmp = expr; print(location, expr_str, "=", __dbg_tmp); __dbg_tmp }
                Ok(MacroOutput::expr(Expr::Block {
                    statements: vec![
                        crate::ast::Stmt::Let {
                            name: "__dbg_tmp".to_string(),
                            type_ann: None,
                            value: *expr,
                        },
                        crate::ast::Stmt::Expr(Expr::Call {
                            callee: Box::new(Expr::Identifier("eprintln".to_string())),
                            args: vec![
                                Expr::Literal(Literal::String(format!(
                                    "{} {} = {{:?}}",
                                    location, expr_str
                                ))),
                                Expr::Identifier("__dbg_tmp".to_string()),
                            ],
                        }),
                    ],
                    final_expr: Some(Box::new(Expr::Identifier("__dbg_tmp".to_string()))),
                }))
            }
            MacroInput::ExprList(exprs) if exprs.len() == 1 => {
                let expr = exprs[0].clone();
                let expr_str = stringify_expr(&expr);
                let location = format!(
                    "[{}:{}]",
                    ctx.file_path.as_deref().unwrap_or("<unknown>"),
                    ctx.line
                );

                Ok(MacroOutput::expr(Expr::Block {
                    statements: vec![
                        crate::ast::Stmt::Let {
                            name: "__dbg_tmp".to_string(),
                            type_ann: None,
                            value: expr,
                        },
                        crate::ast::Stmt::Expr(Expr::Call {
                            callee: Box::new(Expr::Identifier("eprintln".to_string())),
                            args: vec![
                                Expr::Literal(Literal::String(format!(
                                    "{} {} = {{:?}}",
                                    location, expr_str
                                ))),
                                Expr::Identifier("__dbg_tmp".to_string()),
                            ],
                        }),
                    ],
                    final_expr: Some(Box::new(Expr::Identifier("__dbg_tmp".to_string()))),
                }))
            }
            _ => Err(MacroError::invalid_argument("dbg expects an expression")),
        }
    }

    fn description(&self) -> &str {
        "Debug print an expression and its value"
    }

    fn min_args(&self) -> usize {
        1
    }

    fn max_args(&self) -> Option<usize> {
        Some(1)
    }
}

/// `#compile_error` - Emit a compile-time error.
///
/// Causes compilation to fail with the specified message.
///
/// # Syntax
///
/// ```dol
/// #compile_error("This feature is not yet implemented");
/// ```
pub struct CompileErrorMacro;

impl Macro for CompileErrorMacro {
    fn name(&self) -> &str {
        "compile_error"
    }

    fn expand(&self, input: MacroInput, _ctx: &MacroContext) -> Result<MacroOutput, MacroError> {
        let message = match input {
            MacroInput::Expr(expr) => match *expr {
                Expr::Literal(Literal::String(s)) => s,
                _ => return Err(MacroError::type_error("string literal", "expression")),
            },
            MacroInput::ExprList(exprs) if exprs.len() == 1 => match &exprs[0] {
                Expr::Literal(Literal::String(s)) => s.clone(),
                _ => return Err(MacroError::type_error("string literal", "expression")),
            },
            _ => {
                return Err(MacroError::invalid_argument(
                    "compile_error expects a string message",
                ))
            }
        };

        Err(MacroError::new(format!("compile error: {}", message)))
    }

    fn description(&self) -> &str {
        "Emit a compile-time error"
    }

    fn min_args(&self) -> usize {
        1
    }

    fn max_args(&self) -> Option<usize> {
        Some(1)
    }
}

/// `#vec` - Create a vector literal.
///
/// Creates a vector containing the specified elements.
///
/// # Syntax
///
/// ```dol
/// let numbers = #vec(1, 2, 3, 4, 5);
/// let names = #vec("alice", "bob", "charlie");
/// ```
pub struct VecMacro;

impl Macro for VecMacro {
    fn name(&self) -> &str {
        "vec"
    }

    fn expand(&self, input: MacroInput, _ctx: &MacroContext) -> Result<MacroOutput, MacroError> {
        let elements = match input {
            MacroInput::ExprList(exprs) => exprs,
            MacroInput::Expr(expr) => vec![*expr],
            MacroInput::Empty => vec![],
            _ => {
                return Err(MacroError::invalid_argument(
                    "vec expects a list of expressions",
                ))
            }
        };

        // Generate: Vec::from([elem1, elem2, ...])
        Ok(MacroOutput::expr(Expr::Call {
            callee: Box::new(Expr::Member {
                object: Box::new(Expr::Identifier("Vec".to_string())),
                field: "from".to_string(),
            }),
            args: vec![Expr::Call {
                callee: Box::new(Expr::Identifier("__array__".to_string())),
                args: elements,
            }],
        }))
    }

    fn description(&self) -> &str {
        "Create a vector from elements"
    }
}

// === Helper Functions ===

/// Converts an expression to its string representation.
fn stringify_expr(expr: &Expr) -> String {
    match expr {
        Expr::Literal(lit) => match lit {
            Literal::Int(n) => n.to_string(),
            Literal::Float(f) => f.to_string(),
            Literal::String(s) => format!("\"{}\"", s),
            Literal::Bool(b) => b.to_string(),
        },
        Expr::Identifier(name) => name.clone(),
        Expr::Binary { left, op, right } => {
            let op_str = match op {
                crate::ast::BinaryOp::Add => "+",
                crate::ast::BinaryOp::Sub => "-",
                crate::ast::BinaryOp::Mul => "*",
                crate::ast::BinaryOp::Div => "/",
                crate::ast::BinaryOp::Mod => "%",
                crate::ast::BinaryOp::Pow => "^",
                crate::ast::BinaryOp::Eq => "==",
                crate::ast::BinaryOp::Ne => "!=",
                crate::ast::BinaryOp::Lt => "<",
                crate::ast::BinaryOp::Le => "<=",
                crate::ast::BinaryOp::Gt => ">",
                crate::ast::BinaryOp::Ge => ">=",
                crate::ast::BinaryOp::And => "&&",
                crate::ast::BinaryOp::Or => "||",
                crate::ast::BinaryOp::Pipe => "|>",
                crate::ast::BinaryOp::Compose => ">>",
                crate::ast::BinaryOp::Apply => "@",
                crate::ast::BinaryOp::Bind => ":=",
                crate::ast::BinaryOp::Member => ".",
            };
            format!(
                "({} {} {})",
                stringify_expr(left),
                op_str,
                stringify_expr(right)
            )
        }
        Expr::Unary { op, operand } => {
            let op_str = match op {
                crate::ast::UnaryOp::Neg => "-",
                crate::ast::UnaryOp::Not => "!",
                crate::ast::UnaryOp::Quote => "'",
                crate::ast::UnaryOp::Reflect => "?",
            };
            format!("{}{}", op_str, stringify_expr(operand))
        }
        Expr::Call { callee, args } => {
            let args_str: Vec<String> = args.iter().map(stringify_expr).collect();
            format!("{}({})", stringify_expr(callee), args_str.join(", "))
        }
        Expr::Member { object, field } => {
            format!("{}.{}", stringify_expr(object), field)
        }
        Expr::Lambda { params, body, .. } => {
            let params_str: Vec<String> = params.iter().map(|(name, _)| name.clone()).collect();
            format!("|{}| {}", params_str.join(", "), stringify_expr(body))
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let else_str = else_branch
                .as_ref()
                .map(|e| format!(" else {}", stringify_expr(e)))
                .unwrap_or_default();
            format!(
                "if {} {{ {} }}{}",
                stringify_expr(condition),
                stringify_expr(then_branch),
                else_str
            )
        }
        Expr::Quote(inner) => format!("'{}", stringify_expr(inner)),
        Expr::Unquote(inner) => format!(",{}", stringify_expr(inner)),
        Expr::QuasiQuote(inner) => format!("''{}", stringify_expr(inner)),
        Expr::Eval(inner) => format!("!{{{}}}", stringify_expr(inner)),
        Expr::Match { scrutinee, .. } => format!("match {} {{ ... }}", stringify_expr(scrutinee)),
        Expr::Block { final_expr, .. } => {
            if let Some(expr) = final_expr {
                format!("{{ {} }}", stringify_expr(expr))
            } else {
                "{ }".to_string()
            }
        }
        Expr::Reflect(type_expr) => format!("?{:?}", type_expr),
        Expr::IdiomBracket { func, args } => {
            let args_str = args.iter().map(stringify_expr).collect::<Vec<_>>().join(" ");
            format!("[| {} {} |]", stringify_expr(func), args_str)
        }
    }
}

/// Evaluates a cfg expression.
fn evaluate_cfg_expr(expr: &Expr, ctx: &MacroContext) -> Result<bool, MacroError> {
    match expr {
        Expr::Identifier(name) => Ok(ctx.get_cfg(name) || ctx.has_feature(name)),
        Expr::Call { callee, args } => {
            if let Expr::Identifier(func) = callee.as_ref() {
                match func.as_str() {
                    "not" => {
                        if args.len() != 1 {
                            return Err(MacroError::invalid_argument("not expects 1 argument"));
                        }
                        Ok(!evaluate_cfg_expr(&args[0], ctx)?)
                    }
                    "all" => {
                        for arg in args {
                            if !evaluate_cfg_expr(arg, ctx)? {
                                return Ok(false);
                            }
                        }
                        Ok(true)
                    }
                    "any" => {
                        for arg in args {
                            if evaluate_cfg_expr(arg, ctx)? {
                                return Ok(true);
                            }
                        }
                        Ok(false)
                    }
                    "feature" => {
                        if args.len() != 1 {
                            return Err(MacroError::invalid_argument("feature expects 1 argument"));
                        }
                        if let Expr::Literal(Literal::String(s)) = &args[0] {
                            Ok(ctx.has_feature(s))
                        } else {
                            Err(MacroError::type_error("string literal", "expression"))
                        }
                    }
                    _ => Err(MacroError::invalid_argument(&format!(
                        "unknown cfg function: {}",
                        func
                    ))),
                }
            } else {
                Err(MacroError::invalid_argument("invalid cfg expression"))
            }
        }
        Expr::Binary { left, op, right } => {
            match op {
                crate::ast::BinaryOp::Eq => {
                    // Handle feature = "name" style
                    if let (Expr::Identifier(key), Expr::Literal(Literal::String(value))) =
                        (left.as_ref(), right.as_ref())
                    {
                        if key == "feature" {
                            return Ok(ctx.has_feature(value));
                        }
                    }
                    Err(MacroError::invalid_argument("invalid cfg comparison"))
                }
                _ => Err(MacroError::invalid_argument("invalid cfg operator")),
            }
        }
        _ => Err(MacroError::invalid_argument("invalid cfg expression")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_macros_registry() {
        let builtins = BuiltinMacros::new();
        assert!(!builtins.is_empty());
        assert!(builtins.get("stringify").is_some());
        assert!(builtins.get("concat").is_some());
        assert!(builtins.get("env").is_some());
        assert!(builtins.get("cfg").is_some());
        assert!(builtins.get("derive").is_some());
    }

    #[test]
    fn test_stringify_macro() {
        let macro_impl = StringifyMacro;
        let ctx = MacroContext::new();

        let input = MacroInput::ident("container.exists");
        let output = macro_impl.expand(input, &ctx).unwrap();

        if let MacroOutput::Expr(expr) = output {
            if let Expr::Literal(Literal::String(s)) = *expr {
                assert_eq!(s, "container.exists");
            } else {
                panic!("Expected string literal");
            }
        } else {
            panic!("Expected expression output");
        }
    }

    #[test]
    fn test_concat_macro() {
        let macro_impl = ConcatMacro;
        let ctx = MacroContext::new();

        let input = MacroInput::expr_list(vec![
            Expr::Literal(Literal::String("hello".to_string())),
            Expr::Literal(Literal::String(" ".to_string())),
            Expr::Literal(Literal::String("world".to_string())),
        ]);

        let output = macro_impl.expand(input, &ctx).unwrap();

        if let MacroOutput::Expr(expr) = output {
            if let Expr::Literal(Literal::String(s)) = *expr {
                assert_eq!(s, "hello world");
            } else {
                panic!("Expected string literal");
            }
        } else {
            panic!("Expected expression output");
        }
    }

    #[test]
    fn test_env_macro() {
        let macro_impl = EnvMacro;
        let mut ctx = MacroContext::new();
        ctx.env_vars
            .insert("TEST_VAR".to_string(), "test_value".to_string());

        let input = MacroInput::expr(Expr::Literal(Literal::String("TEST_VAR".to_string())));
        let output = macro_impl.expand(input, &ctx).unwrap();

        if let MacroOutput::Expr(expr) = output {
            if let Expr::Literal(Literal::String(s)) = *expr {
                assert_eq!(s, "test_value");
            } else {
                panic!("Expected string literal");
            }
        } else {
            panic!("Expected expression output");
        }
    }

    #[test]
    fn test_env_macro_missing() {
        let macro_impl = EnvMacro;
        let mut ctx = MacroContext::new();
        ctx.env_vars.clear();

        let input = MacroInput::expr(Expr::Literal(Literal::String(
            "NONEXISTENT_VAR".to_string(),
        )));
        let result = macro_impl.expand(input, &ctx);

        assert!(result.is_err());
    }

    #[test]
    fn test_cfg_macro() {
        let macro_impl = CfgMacro;
        let mut ctx = MacroContext::new();
        ctx.set_cfg("debug", true);
        ctx.add_feature("async");

        // Test simple flag
        let input = MacroInput::ident("debug");
        let output = macro_impl.expand(input, &ctx).unwrap();

        if let MacroOutput::Expr(expr) = output {
            if let Expr::Literal(Literal::Bool(b)) = *expr {
                assert!(b);
            } else {
                panic!("Expected bool literal");
            }
        } else {
            panic!("Expected expression output");
        }

        // Test feature
        let input = MacroInput::ident("async");
        let output = macro_impl.expand(input, &ctx).unwrap();

        if let MacroOutput::Expr(expr) = output {
            if let Expr::Literal(Literal::Bool(b)) = *expr {
                assert!(b);
            } else {
                panic!("Expected bool literal");
            }
        } else {
            panic!("Expected expression output");
        }
    }

    #[test]
    fn test_line_macro() {
        let macro_impl = LineMacro;
        let ctx = MacroContext::with_location(Some("test.dol".to_string()), 42, 10);

        let output = macro_impl.expand(MacroInput::empty(), &ctx).unwrap();

        if let MacroOutput::Expr(expr) = output {
            if let Expr::Literal(Literal::Int(n)) = *expr {
                assert_eq!(n, 42);
            } else {
                panic!("Expected int literal");
            }
        } else {
            panic!("Expected expression output");
        }
    }

    #[test]
    fn test_file_macro() {
        let macro_impl = FileMacro;
        let ctx = MacroContext::with_location(Some("test.dol".to_string()), 1, 1);

        let output = macro_impl.expand(MacroInput::empty(), &ctx).unwrap();

        if let MacroOutput::Expr(expr) = output {
            if let Expr::Literal(Literal::String(s)) = *expr {
                assert_eq!(s, "test.dol");
            } else {
                panic!("Expected string literal");
            }
        } else {
            panic!("Expected expression output");
        }
    }

    #[test]
    fn test_todo_macro() {
        let macro_impl = TodoMacro;
        let ctx = MacroContext::new();

        let output = macro_impl.expand(MacroInput::empty(), &ctx).unwrap();

        if let MacroOutput::Expr(expr) = output {
            if let Expr::Call { callee, args } = *expr {
                if let Expr::Identifier(name) = *callee {
                    assert_eq!(name, "panic");
                    assert_eq!(args.len(), 1);
                } else {
                    panic!("Expected identifier callee");
                }
            } else {
                panic!("Expected call expression");
            }
        } else {
            panic!("Expected expression output");
        }
    }

    #[test]
    fn test_derive_macro_is_attribute() {
        let macro_impl = DeriveMacro;
        assert!(macro_impl.is_attribute_macro());
        assert!(!macro_impl.is_expr_macro());
    }

    #[test]
    fn test_assert_macro() {
        let macro_impl = AssertMacro;
        let ctx = MacroContext::new();

        // Test with simple condition
        let input = MacroInput::expr(Expr::Literal(Literal::Bool(true)));
        let output = macro_impl.expand(input, &ctx).unwrap();

        if let MacroOutput::Expr(expr) = output {
            if let Expr::If { condition, .. } = *expr {
                // The condition should be !true (negated)
                if let Expr::Unary { op, .. } = *condition {
                    assert!(matches!(op, crate::ast::UnaryOp::Not));
                } else {
                    panic!("Expected unary negation");
                }
            } else {
                panic!("Expected if expression");
            }
        } else {
            panic!("Expected expression output");
        }
    }

    #[test]
    fn test_assert_eq_macro() {
        let macro_impl = AssertEqMacro;
        let ctx = MacroContext::new();

        let input = MacroInput::expr_list(vec![
            Expr::Literal(Literal::Int(1)),
            Expr::Literal(Literal::Int(1)),
        ]);

        let output = macro_impl.expand(input, &ctx).unwrap();

        if let MacroOutput::Expr(expr) = output {
            if let Expr::If {
                condition,
                then_branch,
                ..
            } = *expr
            {
                // Condition should be left != right
                if let Expr::Binary { op, .. } = *condition {
                    assert!(matches!(op, crate::ast::BinaryOp::Ne));
                } else {
                    panic!("Expected binary comparison");
                }
                // Then branch should be panic call
                if let Expr::Call { callee, .. } = *then_branch {
                    if let Expr::Identifier(name) = *callee {
                        assert_eq!(name, "panic");
                    } else {
                        panic!("Expected panic identifier");
                    }
                } else {
                    panic!("Expected call expression");
                }
            } else {
                panic!("Expected if expression");
            }
        } else {
            panic!("Expected expression output");
        }
    }

    #[test]
    fn test_assert_ne_macro() {
        let macro_impl = AssertNeMacro;
        let ctx = MacroContext::new();

        let input = MacroInput::expr_list(vec![
            Expr::Literal(Literal::Int(1)),
            Expr::Literal(Literal::Int(2)),
        ]);

        let output = macro_impl.expand(input, &ctx).unwrap();

        if let MacroOutput::Expr(expr) = output {
            if let Expr::If { condition, .. } = *expr {
                // Condition should be left == right
                if let Expr::Binary { op, .. } = *condition {
                    assert!(matches!(op, crate::ast::BinaryOp::Eq));
                } else {
                    panic!("Expected binary comparison");
                }
            } else {
                panic!("Expected if expression");
            }
        } else {
            panic!("Expected expression output");
        }
    }

    #[test]
    fn test_format_macro() {
        let macro_impl = FormatMacro;
        let ctx = MacroContext::new();

        // Test with no placeholders
        let input = MacroInput::expr(Expr::Literal(Literal::String("hello".to_string())));
        let output = macro_impl.expand(input, &ctx).unwrap();

        if let MacroOutput::Expr(expr) = output {
            if let Expr::Literal(Literal::String(s)) = *expr {
                assert_eq!(s, "hello");
            } else {
                panic!("Expected string literal");
            }
        } else {
            panic!("Expected expression output");
        }

        // Test with placeholder
        let input = MacroInput::expr_list(vec![
            Expr::Literal(Literal::String("Hello, {}!".to_string())),
            Expr::Identifier("name".to_string()),
        ]);
        let output = macro_impl.expand(input, &ctx).unwrap();

        if let MacroOutput::Expr(expr) = output {
            if let Expr::Call { callee, args } = *expr {
                if let Expr::Identifier(name) = *callee {
                    assert_eq!(name, "concat");
                    assert_eq!(args.len(), 3); // "Hello, ", name.to_string(), "!"
                } else {
                    panic!("Expected concat identifier");
                }
            } else {
                panic!("Expected call expression");
            }
        } else {
            panic!("Expected expression output");
        }
    }

    #[test]
    fn test_format_macro_placeholder_mismatch() {
        let macro_impl = FormatMacro;
        let ctx = MacroContext::new();

        // Too few arguments
        let input = MacroInput::expr_list(vec![
            Expr::Literal(Literal::String("Hello, {} and {}!".to_string())),
            Expr::Identifier("name".to_string()),
        ]);
        let result = macro_impl.expand(input, &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_dbg_macro() {
        let macro_impl = DbgMacro;
        let ctx = MacroContext::with_location(Some("test.dol".to_string()), 10, 5);

        let input = MacroInput::expr(Expr::Identifier("x".to_string()));
        let output = macro_impl.expand(input, &ctx).unwrap();

        if let MacroOutput::Expr(expr) = output {
            if let Expr::Block {
                statements,
                final_expr,
            } = *expr
            {
                assert_eq!(statements.len(), 2); // let + eprintln
                assert!(final_expr.is_some()); // returns __dbg_tmp
            } else {
                panic!("Expected block expression");
            }
        } else {
            panic!("Expected expression output");
        }
    }

    #[test]
    fn test_compile_error_macro() {
        let macro_impl = CompileErrorMacro;
        let ctx = MacroContext::new();

        let input = MacroInput::expr(Expr::Literal(Literal::String(
            "intentional error".to_string(),
        )));
        let result = macro_impl.expand(input, &ctx);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("intentional error"));
    }

    #[test]
    fn test_vec_macro() {
        let macro_impl = VecMacro;
        let ctx = MacroContext::new();

        // Test with elements
        let input = MacroInput::expr_list(vec![
            Expr::Literal(Literal::Int(1)),
            Expr::Literal(Literal::Int(2)),
            Expr::Literal(Literal::Int(3)),
        ]);

        let output = macro_impl.expand(input, &ctx).unwrap();

        if let MacroOutput::Expr(expr) = output {
            if let Expr::Call { callee, args } = *expr {
                // Should be Vec.from(...)
                if let Expr::Member { object, field } = *callee {
                    if let Expr::Identifier(name) = *object {
                        assert_eq!(name, "Vec");
                        assert_eq!(field, "from");
                    } else {
                        panic!("Expected Vec identifier");
                    }
                } else {
                    panic!("Expected member expression");
                }
                // Should have one argument (the array)
                assert_eq!(args.len(), 1);
            } else {
                panic!("Expected call expression");
            }
        } else {
            panic!("Expected expression output");
        }
    }

    #[test]
    fn test_vec_macro_empty() {
        let macro_impl = VecMacro;
        let ctx = MacroContext::new();

        let input = MacroInput::empty();
        let output = macro_impl.expand(input, &ctx).unwrap();

        if let MacroOutput::Expr(expr) = output {
            if let Expr::Call { args, .. } = *expr {
                // Inner array call should have no elements
                if let Expr::Call { args: inner_args, .. } = &args[0] {
                    assert!(inner_args.is_empty());
                } else {
                    panic!("Expected inner call");
                }
            } else {
                panic!("Expected call expression");
            }
        } else {
            panic!("Expected expression output");
        }
    }

    #[test]
    fn test_registry_includes_new_macros() {
        let builtins = BuiltinMacros::new();

        // Test new macros are registered
        assert!(builtins.get("assert").is_some());
        assert!(builtins.get("assert_eq").is_some());
        assert!(builtins.get("assert_ne").is_some());
        assert!(builtins.get("format").is_some());
        assert!(builtins.get("dbg").is_some());
        assert!(builtins.get("compile_error").is_some());
        assert!(builtins.get("vec").is_some());

        // Should now have 20 macros (13 original + 7 new)
        assert_eq!(builtins.len(), 20);
    }
}
